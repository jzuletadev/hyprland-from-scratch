// ~/hyprland-from-scratch/crates/hfs/src/hypr.rs
//
// El puente entre el socket de eventos de Hyprland y el bucle de glib.
//
// Esta es la primera superficie del escritorio que es REACTIVA de verdad: no hay
// sondeo de ningun tipo. Hyprland manda una linea por evento y aqui se reparte.
//
// Dos decisiones:
//
//   El evento solo dispara, no informa. Al llegar uno se vuelve a pedir el estado
//   entero por el otro socket. Mantener un estado incremental a base de deltas
//   («openwindow suma uno al contador de ese espacio») es donde viven los fallos,
//   y la consulta es una ida y vuelta por un socket local.
//
//   Los eventos se agrupan. Una sola accion del usuario (abrir una ventana en otro
//   espacio) suelta tres o cuatro lineas seguidas; sin agrupar serian tres o cuatro
//   consultas y tres o cuatro repintados para un unico cambio.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::hyprland::{self, Event, State};

/// Cuanto se espera antes de consultar, para que una rafaga de eventos sea una sola
/// consulta. Por debajo del umbral de percepcion, asi que no se nota.
const AGRUPAR_MS: u64 = 20;

type Sub = Rc<dyn Fn(&State)>;
type OnMonitors = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

#[derive(Clone)]
pub struct Watch {
    subs: Rc<RefCell<Vec<Sub>>>,
    on_monitors: OnMonitors,
    /// El ultimo estado conocido. Lo leen las teclas multimedia para saber en que
    /// pantalla sacar el aviso sin tener que preguntar otra vez.
    last: Rc<RefCell<State>>,
    pendiente: Rc<Cell<bool>>,
    /// La conexion tiene que seguir viva: su flujo de entrada no la sujeta.
    conn: Rc<RefCell<Option<gio::SocketConnection>>>,
}

impl Default for Watch {
    fn default() -> Self {
        Self::new()
    }
}

impl Watch {
    pub fn new() -> Self {
        Self {
            subs: Rc::new(RefCell::new(Vec::new())),
            on_monitors: Rc::new(RefCell::new(None)),
            last: Rc::new(RefCell::new(State::default())),
            pendiente: Rc::new(Cell::new(false)),
            conn: Rc::new(RefCell::new(None)),
        }
    }

    /// Se llama con el estado nuevo cada vez que algo cambia.
    pub fn subscribe(&self, f: impl Fn(&State) + 'static) {
        self.subs.borrow_mut().push(Rc::new(f));
    }

    /// Se llama cuando se enchufa o se quita un monitor: hay que rehacer las
    /// superficies, porque cada una vive clavada en una pantalla.
    pub fn on_monitors(&self, f: impl Fn() + 'static) {
        *self.on_monitors.borrow_mut() = Some(Rc::new(f));
    }

    /// Olvida a los suscriptores. Lo usa el rehacer: las superficies viejas se van.
    pub fn clear_subs(&self) {
        self.subs.borrow_mut().clear();
    }

    pub fn state(&self) -> State {
        self.last.borrow().clone()
    }

    /// Consulta y reparte, ahora mismo.
    pub fn refresh(&self) {
        match State::query() {
            Ok(s) => {
                *self.last.borrow_mut() = s.clone();
                let subs = self.subs.borrow().clone();
                for f in subs {
                    f(&s);
                }
            }
            Err(e) => eprintln!("hfs: no pude leer el estado de Hyprland: {e}"),
        }
    }

    /// Consulta y reparte, pero agrupando lo que llegue en los proximos milisegundos.
    fn refresh_soon(&self) {
        if self.pendiente.replace(true) {
            return;
        }
        let me = self.clone();
        glib::timeout_add_local_once(Duration::from_millis(AGRUPAR_MS), move || {
            me.pendiente.set(false);
            me.refresh();
        });
    }

    /// Se engancha al flujo de eventos y se queda leyendo lineas dentro del bucle de
    /// glib. Si falla, lo dice y sigue: el escritorio tiene que arrancar igual, solo
    /// que los espacios se quedaran como estaban al abrir.
    pub fn start(&self) {
        let path = match hyprland::event_socket() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("hfs: {e}");
                return;
            }
        };
        let client = gio::SocketClient::new();
        let addr = gio::UnixSocketAddress::new(&path);
        let conn = match gio::prelude::SocketClientExt::connect(&client, &addr, gio::Cancellable::NONE) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "hfs: sin eventos de Hyprland ({e}); los espacios no se actualizaran solos"
                );
                return;
            }
        };
        let input = gio::DataInputStream::new(&conn.input_stream());
        *self.conn.borrow_mut() = Some(conn);
        self.read_next(input);
    }

    fn read_next(&self, input: gio::DataInputStream) {
        let me = self.clone();
        let siguiente = input.clone();
        input.read_line_utf8_async(glib::Priority::DEFAULT, gio::Cancellable::NONE, move |r| {
            match r {
                Ok(Some(line)) => {
                    match hyprland::classify(&line) {
                        Event::Workspaces => me.refresh_soon(),
                        Event::Monitors => {
                            let f = me.on_monitors.borrow().clone();
                            if let Some(f) = f {
                                f();
                            }
                        }
                        Event::Other => {}
                    }
                    me.read_next(siguiente);
                }
                // Si Hyprland cierra el socket es que se esta yendo, y nosotros
                // detras. No hay nada que reintentar.
                Ok(None) => {}
                Err(e) => eprintln!("hfs: se corto el flujo de eventos de Hyprland: {e}"),
            }
        });
    }
}
