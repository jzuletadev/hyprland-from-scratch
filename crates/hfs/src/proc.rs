// ~/hyprland-from-scratch/crates/hfs/src/proc.rs
//
// Lanzar procesos desde el escritorio, y recogerlos.
//
//   HAY QUE RECOGERLOS, Y ESE ES TODO EL MODULO. Un hijo que termina y cuyo padre no
//   llama a `wait()` se queda de **zombi**: ya no corre, pero su entrada en la tabla de
//   procesos sigue ahi, con su PID y su `/proc/<pid>/comm` intactos. En un programa
//   corto da igual, porque al salir el padre los zombis pasan a init y init los
//   recoge. `hfs shell` no sale nunca.
//
// Esto no era teorico. El panel de Captura se quedo diciendo «grabando» con la
// grabacion ya parada: `wf-recorder` habia terminado, pero su zombi seguia
// respondiendo al `/proc` que miraba `grabando()`, asi que parecia vivo para siempre y
// el boton de parar no hacia nada visible. `pgrep` tambien lo veia (como `<defunct>`),
// o sea que la v1 tenia el mismo fallo latente y se salvaba solo porque sus TUIs eran
// procesos cortos.
//
// La recogida va por `glib::child_watch_add`, que es la forma correcta dentro de un
// bucle de GLib: no hace falta hilo ni manejador de SIGCHLD propio.

use hfs_ui::gtk::glib;

/// Le pasa a GLib la tarea de recoger un hijo ya lanzado. Para los casos en que hubo
/// que quedarse con el `Child` por algo (escribirle por stdin, guardar su pid).
pub fn adoptar(h: std::process::Child) {
    let pid = glib::Pid(h.id() as i32);
    // `h` se suelta sin esperar a proposito: a partir de aqui el hijo es de GLib, y
    // dejar vivo el `Child` seria tener dos duenos del mismo PID.
    std::mem::forget(h);
    glib::child_watch_add(pid, |_, _| {});
}

fn vigilar(hijo: std::io::Result<std::process::Child>, que: &str) {
    match hijo {
        Ok(h) => adoptar(h),
        Err(e) => eprintln!("hfs: no pude lanzar «{que}»: {e}"),
    }
}

/// Una orden de shell. Va por `sh -c` porque varias encadenan dos cosas o necesitan
/// sustitucion de ordenes.
///
/// Devuelve el PID. Con `exec` dentro, ese PID acaba siendo el del programa de verdad,
/// que es lo que permite seguirlo despues sin rebuscar en /proc.
pub fn sh(cmd: &str) -> Option<u32> {
    match std::process::Command::new("sh").arg("-c").arg(cmd).spawn() {
        Ok(h) => {
            let pid = h.id();
            adoptar(h);
            Some(pid)
        }
        Err(e) => {
            eprintln!("hfs: no pude lanzar «{cmd}»: {e}");
            None
        }
    }
}

/// Un programa con sus argumentos, sin shell de por medio.
pub fn exec<I, S>(programa: &str, args: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    vigilar(
        std::process::Command::new(programa).args(args).spawn(),
        programa,
    );
}

/// Si un PID concreto esta vivo y se llama asi. Es una lectura de dos ficheros.
fn es(pid: u32, nombre: &str) -> bool {
    let p = format!("/proc/{pid}");
    let coincide = std::fs::read_to_string(format!("{p}/comm"))
        .map(|c| c.trim() == nombre)
        .unwrap_or(false);
    if !coincide {
        return false;
    }
    no_zombi(std::path::Path::new(&p))
}

/// Un zombi conserva su nombre, asi que hay que mirar el estado.
fn no_zombi(p: &std::path::Path) -> bool {
    let Ok(stat) = std::fs::read_to_string(p.join("stat")) else {
        return false;
    };
    // El segundo campo es el nombre entre parentesis y puede llevar espacios, asi que
    // se corta por el ultimo `)`; el siguiente es el estado.
    stat.rsplit_once(')')
        .and_then(|(_, r)| r.split_whitespace().next())
        .map(|e| e != "Z")
        .unwrap_or(false)
}

/// Sigue la pista a un programa por su nombre **sin barrer /proc cada vez**.
///
/// Barrer /proc entero son ~400 ficheros abiertos; a 1 Hz eso medido son **1,57 % de
/// CPU en reposo**, mas que el `waybar-autohide.sh` de la v1 (1,3 %) que esta misma
/// v2 se jacta de haber jubilado. Asi que: si ya se cual es el PID, se mira ese y
/// nada mas, y el barrido completo solo se hace de tarde en tarde para enterarse de
/// uno que haya arrancado por fuera.
pub struct Vigia {
    nombre: &'static str,
    pid: std::cell::Cell<Option<u32>>,
    /// Ticks desde el ultimo barrido completo.
    desde_barrido: std::cell::Cell<u8>,
}

/// Cada cuantas consultas sin PID conocido se barre /proc. A 1 Hz son 8 s. Solo hace
/// falta para enterarse de una grabacion lanzada por fuera: la que lanza el propio
/// panel ceba el PID y se ve al instante.
const CADA: u8 = 8;

impl Vigia {
    pub const fn new(nombre: &'static str) -> Self {
        Self {
            nombre,
            pid: std::cell::Cell::new(None),
            desde_barrido: std::cell::Cell::new(CADA),
        }
    }

    /// Apunta un PID recien lanzado. Asi la primera consulta ya va por el camino
    /// barato y la marca sale al instante, sin esperar al barrido.
    pub fn cebar(&self, pid: u32) {
        self.pid.set(Some(pid));
        self.desde_barrido.set(0);
    }

    pub fn vivo(&self) -> bool {
        if let Some(pid) = self.pid.get() {
            if es(pid, self.nombre) {
                return true;
            }
            self.pid.set(None);
        }
        let n = self.desde_barrido.get().saturating_add(1);
        if n < CADA {
            self.desde_barrido.set(n);
            return false;
        }
        self.desde_barrido.set(0);
        match buscar(self.nombre) {
            Some(pid) => {
                self.pid.set(Some(pid));
                true
            }
            None => false,
        }
    }
}

/// El barrido caro. Devuelve el PID del primer proceso vivo con ese nombre.
pub fn buscar(nombre: &str) -> Option<u32> {
    let d = std::fs::read_dir("/proc").ok()?;
    for e in d.flatten() {
        let p = e.path();
        let coincide = std::fs::read_to_string(p.join("comm"))
            .map(|c| c.trim() == nombre)
            .unwrap_or(false);
        if !coincide || !no_zombi(&p) {
            continue;
        }
        return p
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.parse().ok());
    }
    None
}

/// Si hay algun proceso **vivo** con ese nombre. Barre /proc entero: para algo que se
/// consulta de vez en cuando. Lo que se consulta a menudo usa `Vigia`.
pub fn vivo(nombre: &str) -> bool {
    buscar(nombre).is_some()
}
