// ~/hyprland-from-scratch/crates/hfs/src/shell.rs
//
// El escritorio: UN proceso con todas las superficies dentro.
//
// No son procesos separados a proposito. Cada proceso de GTK4 se trae su copia del
// toolkit, sus temas de iconos y su conexion a Wayland: ocho superficies serian ocho
// veces eso para enseñar un reloj y unos puntos. Asi la paleta se lee una vez, el CSS
// se carga una vez y el socket de eventos de Hyprland se escucha una vez para todas.
//
// Y UN JUEGO DE SUPERFICIES POR PANTALLA. Una superficie de layer-shell vive clavada
// en el monitor donde el compositor la pone al mapearla, y ahi se queda: con una sola
// copia, al pasarte a la otra pantalla el panel sencillamente no esta. Era la duda
// que quedaba apuntada en V2-PLAN.md y la respuesta es esta.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::css;

use crate::{
    bandeja, cabecera, ctrl, energia, espacios, hypr, ipc, notificaciones, portapapeles, reloj,
    rueda, sni,
};

const APP_ID: &str = "dev.hfs.shell";
/// Cuanto se asoma el dock cuando se toca una tecla multimedia.
const ASOMO_MS: u32 = 1500;
/// Cuanto se quedan todas al pedir verlas de golpe. Mas tiempo, porque aqui la idea
/// es mirarlas, no comprobar un valor de reojo.
const TODO_MS: u32 = 4000;

/// Todo lo que vive en una pantalla.
struct Pantalla {
    connector: String,
    ctrl: ctrl::Ctrl,
    espacios: espacios::Espacios,
    reloj: reloj::Reloj,
    cabecera: cabecera::Cabecera,
    energia: energia::Energia,
    notifs: notificaciones::Notificaciones,
    clip: portapapeles::Portapapeles,
    bandeja: bandeja::Bandeja,
    /// La rueda se construye la primera vez que se pide, no al arrancar.
    ///
    /// Lo caro no es construirla (medido: ~2 MB de widgets) sino MAPEARLA: ocupa el
    /// monitor entero y son ~125 MB de buffers que no se devuelven al cerrarla. Eso
    /// se paga igual, pero solo en la pantalla donde de verdad se abra.
    ///
    /// Lo que si se ahorra de verdad es que cada rueda se trae su copia de la lista
    /// de aplicaciones y su vigilante de cambios: con cuatro monitores eran cuatro
    /// de cada una, para una ventana que solo puede estar abierta en un sitio.
    rueda: RefCell<Option<rueda::Rueda>>,
    /// Lo que hace falta para construirla cuando toque.
    app: gtk::Application,
    palette: Palette,
    monitor: gtk::gdk::Monitor,
}

impl Pantalla {
    fn destroy(&self) {
        self.ctrl.destroy();
        self.espacios.destroy();
        self.reloj.destroy();
        self.cabecera.destroy();
        self.energia.destroy();
        self.notifs.destroy();
        self.clip.destroy();
        self.bandeja.destroy();
        if let Some(r) = self.rueda.borrow().as_ref() {
            r.destroy();
        }
    }

    fn alternar_rueda(&self) {
        let mut slot = self.rueda.borrow_mut();
        let r = slot
            .get_or_insert_with(|| rueda::new(&self.app, &self.palette, &self.monitor));
        r.toggle();
    }

    fn abrir_panel(&self, nombre: &str) {
        let mut slot = self.rueda.borrow_mut();
        let r = slot.get_or_insert_with(|| rueda::new(&self.app, &self.palette, &self.monitor));
        r.abrir_panel(nombre);
    }

    /// Cerrar no la construye: si no existe, ya esta cerrada.
    fn cerrar_rueda(&self) {
        if let Some(r) = self.rueda.borrow().as_ref() {
            r.close();
        }
    }
}

pub fn run() -> std::process::ExitCode {
    let app = gtk::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build);
    app.connect_shutdown(|_| ipc::clear_socket());
    let code = app.run_with_args::<&str>(&["hfs"]);
    if code != glib::ExitCode::SUCCESS {
        return std::process::ExitCode::FAILURE;
    }
    std::process::ExitCode::SUCCESS
}

fn build(app: &gtk::Application) {
    let (inicial, err) = Palette::load();
    if let Some(e) = err {
        eprintln!("hfs: uso la paleta de reserva porque {e}");
    }
    let provider = css::apply(&inicial);
    // La paleta puede cambiar en caliente, asi que vive en una celda y no en una
    // variable: al cambiar de tema hay que rehacer las superficies, porque los
    // colores estan dentro de los cierres de dibujo de Cairo, no solo en el CSS.
    let palette = Rc::new(RefCell::new(inicial));

    let watch = hypr::Watch::new();
    let pantallas: Rc<RefCell<Vec<Pantalla>>> = Rc::new(RefCell::new(Vec::new()));

    // El registro de la bandeja es del bus, no de la pantalla: uno por proceso, y
    // sobrevive a que se rehagan las superficies. Si no arranca, la bandeja se queda
    // con la grabacion y sin iconos, que sigue siendo util.
    let sni = sni::Bandeja::new();
    if sni.is_none() {
        eprintln!("hfs: sin bus de sesión; la bandeja va sin iconos");
    }

    let rehacer: Rc<dyn Fn()> = {
        let (app, palette, watch, pantallas, sni) = (
            app.clone(),
            palette.clone(),
            watch.clone(),
            pantallas.clone(),
            sni.clone(),
        );
        Rc::new(move || {
            let palette = palette.borrow().clone();
            for p in pantallas.borrow().iter() {
                p.destroy();
            }
            pantallas.borrow_mut().clear();
            watch.clear_subs();
            // Las bandejas viejas ya no existen: que el registro deje de llamarlas.
            if let Some(s) = sni.as_ref() {
                s.olvidar_suscriptores();
            }

            for monitor in monitores() {
                let Some(connector) = monitor.connector() else {
                    eprintln!("hfs: un monitor sin nombre de conector; lo salto");
                    continue;
                };
                let connector = connector.to_string();

                let p = Pantalla {
                    ctrl: ctrl::Ctrl::new(&app, &palette, &monitor),
                    espacios: espacios::Espacios::new(&app, &palette, &monitor, &connector),
                    reloj: reloj::Reloj::new(&app, &palette, &monitor),
                    cabecera: cabecera::Cabecera::new(&app, &palette, &monitor),
                    energia: energia::Energia::new(&app, &palette, &monitor),
                    notifs: notificaciones::Notificaciones::new(&app, &palette, &monitor),
                    clip: portapapeles::Portapapeles::new(&app, &palette, &monitor),
                    bandeja: bandeja::Bandeja::new(&app, &palette, &monitor, sni.as_ref()),
                    rueda: RefCell::new(None),
                    app: app.clone(),
                    palette: palette.clone(),
                    monitor,
                    connector,
                };
                p.ctrl.present();
                p.espacios.present();
                p.reloj.present();
                p.cabecera.present();
                p.energia.present();
                p.notifs.present();
                p.clip.present();
                p.bandeja.present();
                pantallas.borrow_mut().push(p);
            }

            // Cada rail se queda escuchando el estado para redibujar sus puntos.
            let listas = pantallas.clone();
            watch.subscribe(move |estado| {
                for p in listas.borrow().iter() {
                    p.espacios.update(estado);
                }
            });
            watch.refresh();
        })
    };

    rehacer();
    seguir_el_tema(&palette, &provider, &rehacer);
    vigilar_la_grabacion(&pantallas);
    vigilar_el_portapapeles();

    // Enchufar o quitar una pantalla: hay que rehacerlo todo, porque una superficie
    // no se puede mudar de monitor. Va aplazado a un «idle» para no hacerlo desde
    // dentro del aviso que lo dispara.
    {
        let rehacer = rehacer.clone();
        watch.on_monitors(move || {
            let rehacer = rehacer.clone();
            glib::idle_add_local_once(move || rehacer());
        });
    }
    if let Some(display) = gtk::gdk::Display::default() {
        let rehacer = rehacer.clone();
        display.monitors().connect_items_changed(move |_, _, _, _| {
            let rehacer = rehacer.clone();
            glib::idle_add_local_once(move || rehacer());
        });
    }

    watch.start();
    escuchar_avisos(&watch, &pantallas);
}

/// Levanta `wl-paste --watch`, que es lo que alimenta el historial del portapapeles.
///
/// El proceso sobrevive al nuestro, asi que se apunta su pid: al arrancar se mata el
/// que quedara de la sesion anterior. Sin eso, cada reinicio del escritorio dejaria
/// un vigilante mas, y todos escribiendo en el mismo sitio.
fn vigilar_el_portapapeles() {
    let pidfile = hfs_core::paths::runtime().join("clip-watch.pid");
    if let Ok(txt) = std::fs::read_to_string(&pidfile)
        && let Ok(pid) = txt.trim().parse::<i32>()
    {
        // Solo si de verdad sigue siendo un wl-paste: un pid se reutiliza.
        let suyo = std::fs::read_to_string(format!("/proc/{pid}/cmdline")).unwrap_or_default();
        if suyo.contains("wl-paste") {
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
        }
    }

    let Ok(exe) = std::env::current_exe() else {
        eprintln!("hfs: no se donde estoy; no habra historial de portapapeles");
        return;
    };
    // `--type text`: copiar una imagen no debe dejar diez megas en el historial.
    match std::process::Command::new("wl-paste")
        .args(["--type", "text", "--watch"])
        .arg(&exe)
        .arg("clip-add")
        .spawn()
    {
        Ok(h) => {
            if let Some(dir) = pidfile.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(&pidfile, h.id().to_string());
            // Vive toda la sesion, pero si se muere no debe quedarse de zombi.
            crate::proc::adoptar(h);
        }
        Err(e) => eprintln!("hfs: sin historial de portapapeles ({e})"),
    }
}

/// Vigila el fichero del tema y rehace el escritorio cuando cambia.
///
/// Sin esto habia que reiniciar `hfs shell` despues de cambiar de tema, porque la
/// paleta se leia una sola vez al arrancar. Se vigila el DIRECTORIO y no el fichero:
/// quien lo escribe puede reemplazarlo en vez de modificarlo, y entonces el vigilante
/// del fichero se queda mirando un inodo que ya no existe.
/// El unico sondeo permanente del escritorio.
///
/// Existe por una sola razon: la marca de grabacion del canto tiene que aparecer sin
/// que nadie roce nada, y `wf-recorder` no avisa de nada por D-Bus ni por fichero.
/// Es UNO para todo el proceso y no uno por pantalla: con dos monitores, dos
/// temporizadores haciendo el mismo trabajo salian medidos a 0,70 % de CPU.
/// `captura::grabando()` por dentro sigue un PID cacheado, asi que un tick normal son
/// dos lecturas de /proc, no las ~800 de barrerlo entero.
fn vigilar_la_grabacion(pantallas: &Rc<RefCell<Vec<Pantalla>>>) {
    let pantallas = pantallas.clone();
    let antes = std::cell::Cell::new(false);
    glib::timeout_add_seconds_local(1, move || {
        let ahora = crate::captura::grabando();
        let cambio = ahora != antes.get();
        antes.set(ahora);
        for p in pantallas.borrow().iter() {
            p.bandeja.grabacion(ahora, cambio);
        }
        glib::ControlFlow::Continue
    });
}

fn seguir_el_tema(
    palette: &Rc<RefCell<Palette>>,
    provider: &gtk::CssProvider,
    rehacer: &Rc<dyn Fn()>,
) {
    let fichero = hfs_core::paths::current_theme_file();
    let Some(dir) = fichero.parent() else { return };
    let monitor = match gio::File::for_path(dir).monitor_directory(
        gio::FileMonitorFlags::NONE,
        gio::Cancellable::NONE,
    ) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("hfs: no podre seguir los cambios de tema ({e})");
            return;
        }
    };

    let nombre = fichero.file_name().map(|n| n.to_owned());
    let (palette, provider, rehacer) = (palette.clone(), provider.clone(), rehacer.clone());
    let pendiente = Rc::new(std::cell::Cell::new(false));
    monitor.connect_changed(move |_, f, _, ev| {
        if !matches!(
            ev,
            gio::FileMonitorEvent::ChangesDoneHint | gio::FileMonitorEvent::Created
        ) {
            return;
        }
        if f.basename().map(|b| b.into_os_string()) != nombre {
            return;
        }
        // Escribir un fichero dispara varios avisos; se agrupan.
        if pendiente.replace(true) {
            return;
        }
        let (palette, provider, rehacer, pendiente) = (
            palette.clone(),
            provider.clone(),
            rehacer.clone(),
            pendiente.clone(),
        );
        glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
            pendiente.set(false);
            let (nueva, err) = Palette::load();
            if let Some(e) = err {
                eprintln!("hfs: uso la paleta de reserva porque {e}");
            }
            provider.load_from_string(&css::stylesheet(&nueva));
            *palette.borrow_mut() = nueva;
            rehacer();
        });
    });
    // El vigilante tiene que seguir vivo despues de salir de aqui.
    std::mem::forget(monitor);
}

/// Los monitores de la sesion, en orden.
fn monitores() -> Vec<gtk::gdk::Monitor> {
    let Some(display) = gtk::gdk::Display::default() else {
        eprintln!("hfs: sin pantalla; no hay donde dibujar");
        return Vec::new();
    };
    let lista = display.monitors();
    (0..lista.n_items())
        .filter_map(|i| lista.item(i))
        .filter_map(|o| o.downcast::<gtk::gdk::Monitor>().ok())
        .collect()
}

/// Las teclas multimedia llaman a `hfs vol up`, que se conecta a este socket: el dock
/// se asoma un segundo y hace de OSD, asi que no hay que escribir un OSD aparte.
///
/// Se asoma solo en la pantalla que tiene el foco. Con dos monitores, un aviso en el
/// que no estas mirando es un aviso que no existe.
fn escuchar_avisos(watch: &hypr::Watch, pantallas: &Rc<RefCell<Vec<Pantalla>>>) {
    ipc::prepare_socket_dir();
    let service = gio::SocketService::new();
    let addr = gio::UnixSocketAddress::new(&ipc::socket_path());
    if let Err(e) = service.add_address(
        &addr,
        gio::SocketType::Stream,
        gio::SocketProtocol::Default,
        None::<&glib::Object>,
    ) {
        eprintln!("hfs: las teclas multimedia no podran avisar al dock ({e})");
        return;
    }

    let (watch, pantallas) = (watch.clone(), pantallas.clone());
    service.connect_incoming(move |_, conn, _| {
        let (watch, pantallas) = (watch.clone(), pantallas.clone());
        // La conexion se mete en el cierre para que siga viva mientras se lee.
        let input = gio::DataInputStream::new(&conn.input_stream());
        let conn = conn.clone();
        input.read_line_utf8_async(
            glib::Priority::DEFAULT,
            gio::Cancellable::NONE,
            move |r| {
                let _ = &conn;
                let Ok(Some(linea)) = r else { return };
                let Some(aviso) = ipc::Aviso::leer(&linea) else {
                    eprintln!("hfs: no entiendo el aviso «{}»", linea.trim());
                    return;
                };
                atender(aviso, &watch, &pantallas);
            },
        );
        true
    });
    service.start();
    // El servicio tiene que seguir vivo despues de salir de aqui.
    std::mem::forget(service);
}

/// Hace lo que pide el aviso, solo en la pantalla que tiene el foco. Con dos
/// monitores, un aviso en el que no estas mirando es un aviso que no existe.
fn atender(aviso: ipc::Aviso, watch: &hypr::Watch, pantallas: &Rc<RefCell<Vec<Pantalla>>>) {
    let estado = watch.state();
    let foco = estado.focused_monitor();

    // La rueda es modal: no puede quedarse una abierta en la otra pantalla.
    if aviso == ipc::Aviso::Rueda {
        for p in pantallas.borrow().iter() {
            if !foco.is_none_or(|m| m == p.connector) {
                p.cerrar_rueda();
            }
        }
    }

    for p in pantallas.borrow().iter() {
        // Sin saber cual tiene el foco, se asoman todas: mejor de mas que ninguna.
        if !foco.is_none_or(|m| m == p.connector) {
            continue;
        }
        match aviso {
            ipc::Aviso::Ctrl => {
                p.ctrl.refresh();
                p.ctrl.handle.peek(ASOMO_MS);
            }
            ipc::Aviso::Todo => {
                p.ctrl.refresh();
                for h in [
                    &p.ctrl.handle,
                    &p.espacios.handle,
                    &p.reloj.handle,
                    &p.cabecera.handle,
                    &p.energia.handle,
                    &p.notifs.handle,
                    &p.clip.handle,
                    &p.bandeja.handle,
                ] {
                    h.peek(TODO_MS);
                }
            }
            ipc::Aviso::Rueda => p.alternar_rueda(),
            ipc::Aviso::Panel(n) => p.abrir_panel(n),
            ipc::Aviso::Sistema => crate::sistema::abrir(&p.app, &p.palette),
        }
    }
}
