// ~/hyprland-from-scratch/crates/hfs/src/ipc.rs
//
// Lo minimo para que un `hfs vol up` haga que el escritorio ya levantado reaccione.
//
// Es un socket de dominio Unix y una palabra por linea. Empece con un pidfile y una
// señal, que es lo que hace el repo con Waybar, pero glib 0.22 ya no trae el enganche
// de señales Unix al bucle principal. El socket sale mejor igualmente: no hay pid que
// se quede viejo, no hay carrera al reiniciar el escritorio, llega sin retardo (un
// GFileMonitor habria metido hasta 800 ms, que para un OSD se nota) y, a diferencia
// de una señal, puede decir QUE ha pasado.

use hfs_core::paths;
use std::fs;
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// Lo que se le puede decir al escritorio desde fuera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aviso {
    /// El volumen o el brillo han cambiado: asomar el dock de controles.
    Ctrl,
    /// Enseñar todas las superficies a la vez. Es la unica forma de verlas, porque
    /// los bordes no llevan ninguna marca.
    Todo,
    /// Abrir o cerrar la rueda del centro.
    Rueda,
    /// Abrir la rueda directamente en uno de sus paneles, saltandose los radios.
    /// Son las teclas que la v1 tenia para sus TUIs: SUPER+/ para los atajos,
    /// SUPER+A para el tema, SUPER+W para el fondo y SUPER+R para la captura. La
    /// memoria muscular se conserva aunque la herramienta de debajo haya cambiado.
    Panel(&'static str),
    /// Abrir la ficha de la maquina. Normalmente se toca el icono de arch del rail,
    /// pero tenerlo aqui la hace invocable desde fuera, que es como se prueba.
    Sistema,
}

impl Aviso {
    fn palabra(self) -> &'static str {
        match self {
            Aviso::Ctrl => "ctrl",
            Aviso::Todo => "todo",
            Aviso::Rueda => "rueda",
            Aviso::Panel(p) => p,
            Aviso::Sistema => "sistema",
        }
    }

    pub fn leer(linea: &str) -> Option<Self> {
        match linea.trim() {
            "ctrl" => Some(Aviso::Ctrl),
            "todo" => Some(Aviso::Todo),
            "rueda" => Some(Aviso::Rueda),
            "sistema" => Some(Aviso::Sistema),
            p @ ("atajos" | "tema" | "fondo" | "captura" | "sonido") => Some(Aviso::Panel(
                // Se reescribe a un `&'static str` para que `Aviso` siga siendo Copy.
                match p {
                    "atajos" => "atajos",
                    "tema" => "tema",
                    "fondo" => "fondo",
                    "sonido" => "sonido",
                    _ => "captura",
                },
            )),
            _ => None,
        }
    }
}

pub fn socket_path() -> PathBuf {
    paths::runtime().join("shell.sock")
}

/// Deja el sitio libre para poder escuchar. Un socket que quedo de una sesion
/// anterior impide volver a atarlo.
pub fn clear_socket() {
    let _ = fs::remove_file(socket_path());
}

pub fn prepare_socket_dir() {
    if let Some(dir) = socket_path().parent() {
        let _ = fs::create_dir_all(dir);
    }
    clear_socket();
}

/// Avisa al escritorio si esta vivo. Si no lo esta, no pasa nada: cambiar el volumen
/// sin escritorio tiene que seguir funcionando igual.
pub fn nudge(a: Aviso) {
    if let Ok(mut s) = UnixStream::connect(socket_path()) {
        let _ = writeln!(s, "{}", a.palabra());
    }
}
