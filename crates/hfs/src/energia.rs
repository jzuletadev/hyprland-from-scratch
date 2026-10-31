// ~/hyprland-from-scratch/crates/hfs/src/energia.rs
//
// Energia, en la esquina de arriba a la izquierda: la bateria, las seis acciones de
// sesion y los perfiles de rendimiento.
//
// Dos cosas que la v1 hacia mal y aqui no:
//
//   APAGAR PIDE CONFIRMACION. En el TUI de la v1, `p` apagaba la maquina al instante
//   y `q` —salir— esta en la tecla de al lado. Aqui reiniciar y apagar se convierten
//   en «¿seguro?» dentro del propio panel, sin ventana nueva y sin mover el raton.
//
//   EL PERFIL SE VE, NO SE ADIVINA. PowerProfiles dice cual esta activo y, cuando el
//   sistema lo esta limitando, por que (`PerformanceDegraded`). La v1 cambiaba de
//   perfil a ciegas y no comprobaba si habia funcionado.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::sys;
use hfs_core::Palette;
use hfs_ui::widgets::{self, Ring};
use hfs_ui::{dbus, EdgeSurface, Handle, Place, Side, Spec};

const LATIDO_S: u64 = 10;

const PP_SERVICIO: &str = "net.hadess.PowerProfiles";
const PP_RUTA: &str = "/net/hadess/PowerProfiles";
const PP_IFACE: &str = "net.hadess.PowerProfiles";

/// Lo que hace cada boton. `peligro` es lo que no se puede deshacer.
struct Accion {
    titulo: &'static str,
    icono: &'static str,
    /// `sh -c` porque dos de ellas son «bloquea y luego duerme», y el orden importa:
    /// si se suspende antes de que hyprlock haya pintado, la pantalla se despierta
    /// enseñando lo que habia.
    cmd: &'static str,
    peligro: bool,
}

const ACCIONES: &[Accion] = &[
    Accion {
        titulo: "Bloquear",
        icono: "system-lock-screen-symbolic",
        cmd: "hyprlock",
        peligro: false,
    },
    Accion {
        titulo: "Suspender",
        icono: "weather-clear-night-symbolic",
        cmd: "hyprlock & sleep 0.5 && systemctl suspend",
        peligro: false,
    },
    Accion {
        titulo: "Hibernar",
        icono: "media-playback-pause-symbolic",
        cmd: "hyprlock & sleep 0.5 && systemctl hibernate",
        peligro: false,
    },
    Accion {
        titulo: "Sesión",
        icono: "system-log-out-symbolic",
        cmd: "hyprctl dispatch exit",
        peligro: true,
    },
    Accion {
        titulo: "Reiniciar",
        icono: "system-reboot-symbolic",
        cmd: "systemctl reboot",
        peligro: true,
    },
    Accion {
        titulo: "Apagar",
        icono: "system-shutdown-symbolic",
        cmd: "systemctl poweroff",
        peligro: true,
    },
];

/// Como se llaman en castellano los perfiles de PowerProfiles.
fn nombre_perfil(id: &str) -> &str {
    match id {
        "power-saver" => "Ahorro",
        "balanced" => "Equilibrado",
        "performance" => "Máximo",
        otro => otro,
    }
}

pub struct Energia {
    surface: EdgeSurface,
    pub handle: Handle,
}

impl Energia {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);
        let mut spec = Spec::new(
            "hfs-energia",
            Side::Top,
            m.esquina,
            m.esquina_alto,
        );
        // Arriba a la izquierda: nace de la esquina.
        spec.place = Place::Start(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("energia");
        surface.body.set_orientation(gtk::Orientation::Vertical);
        surface.body.set_spacing(10);

        // --- cabecera con el anillo de la bateria ---
        let (cab, restante) = widgets::header("Energía");
        let anillo = Ring::new(palette, 38);
        cab.append(&anillo.root);

        // --- las seis acciones ---
        let rejilla = gtk::Grid::new();
        rejilla.add_css_class("grid3");
        rejilla.set_row_spacing(6);
        rejilla.set_column_spacing(6);
        rejilla.set_column_homogeneous(true);

        // La confirmacion vive en el propio panel: una barra que sustituye a la
        // rejilla. Abrir una ventana para preguntar seria perder el sitio y el foco.
        let confirmar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        confirmar.add_css_class("confirm");
        let pregunta = gtk::Label::new(None);
        pregunta.set_hexpand(true);
        pregunta.set_xalign(0.0);
        let si = gtk::Button::with_label("Sí");
        si.add_css_class("confirm-si");
        let no = gtk::Button::with_label("Cancelar");
        confirmar.append(&pregunta);
        confirmar.append(&no);
        confirmar.append(&si);
        confirmar.set_visible(false);

        let pendiente: Rc<RefCell<Option<&'static str>>> = Rc::new(RefCell::new(None));

        for (i, a) in ACCIONES.iter().enumerate() {
            let b = widgets::key(a.icono, a.titulo);
            if a.peligro {
                b.add_css_class("danger");
            }
            let (rejilla_, confirmar_, pregunta_, pendiente_, handle_) = (
                rejilla.clone(),
                confirmar.clone(),
                pregunta.clone(),
                pendiente.clone(),
                handle.clone(),
            );
            b.connect_clicked(move |_| {
                if a.peligro {
                    *pendiente_.borrow_mut() = Some(a.cmd);
                    pregunta_.set_text(&format!("¿{}?", a.titulo));
                    rejilla_.set_visible(false);
                    confirmar_.set_visible(true);
                } else {
                    handle_.close_now();
                    ejecutar(a.cmd);
                }
            });
            rejilla.attach(&b, (i % 3) as i32, (i / 3) as i32, 1, 1);
        }

        let volver: Rc<dyn Fn()> = {
            let (rejilla, confirmar, pendiente) =
                (rejilla.clone(), confirmar.clone(), pendiente.clone());
            Rc::new(move || {
                *pendiente.borrow_mut() = None;
                confirmar.set_visible(false);
                rejilla.set_visible(true);
            })
        };
        {
            let volver = volver.clone();
            no.connect_clicked(move |_| volver());
        }
        {
            let (volver, pendiente, handle) = (volver.clone(), pendiente.clone(), handle.clone());
            si.connect_clicked(move |_| {
                let cmd = pendiente.borrow_mut().take();
                volver();
                if let Some(cmd) = cmd {
                    handle.close_now();
                    ejecutar(cmd);
                }
            });
        }
        // Al cerrarse el panel, la pregunta no se queda esperando a la proxima vez.
        {
            let volver = volver.clone();
            handle.connect_toggled(move |abierto| {
                if !abierto {
                    volver();
                }
            });
        }

        // --- perfiles ---
        let perfiles = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        perfiles.add_css_class("seg");
        perfiles.set_homogeneous(true);
        let degradado = gtk::Label::new(None);
        degradado.add_css_class("sub");
        degradado.set_xalign(0.0);

        let pp = dbus::system(PP_SERVICIO, PP_RUTA, PP_IFACE).ok();
        let botones_perfil: Rc<RefCell<Vec<(String, gtk::Button)>>> =
            Rc::new(RefCell::new(Vec::new()));

        match pp.as_ref().filter(|p| dbus::vivo(p)) {
            Some(p) => {
                // La lista sale del servicio, no de una constante: una maquina sin
                // modo «performance» no debe enseñar un boton que no hace nada.
                let ids: Vec<String> = perfiles_desde_variant(p).unwrap_or_else(|| {
                    vec!["power-saver".into(), "balanced".into(), "performance".into()]
                });
                for id in ids {
                    let b = gtk::Button::with_label(nombre_perfil(&id));
                    b.add_css_class("seg-item");
                    b.set_has_frame(false);
                    let (p, id2) = (p.clone(), id.clone());
                    b.connect_clicked(move |_| {
                        if let Err(e) = dbus::set_prop(
                            &p,
                            PP_IFACE,
                            "ActiveProfile",
                            &id2.to_variant(),
                        ) {
                            eprintln!("hfs: no pude poner el perfil «{id2}»: {e}");
                        }
                    });
                    perfiles.append(&b);
                    botones_perfil.borrow_mut().push((id, b));
                }
            }
            None => {
                eprintln!("hfs: sin power-profiles-daemon; no habra perfiles");
                let l = gtk::Label::new(Some("sin perfiles de energía"));
                l.add_css_class("sub");
                perfiles.append(&l);
            }
        }

        surface.body.append(&cab);
        surface.body.append(&rejilla);
        surface.body.append(&confirmar);
        surface.body.append(&perfiles);
        surface.body.append(&degradado);

        // --- refrescar ---
        let refrescar: Rc<dyn Fn()> = {
            let (anillo, restante, degradado) =
                (anillo.clone(), restante.clone(), degradado.clone());
            let (pp, botones_perfil, palette) =
                (pp.clone(), botones_perfil.clone(), palette.clone());
            Rc::new(move || {
                match sys::battery() {
                    Some(b) => {
                        let color = if b.charging() || b.full() {
                            palette.ok
                        } else {
                            widgets::umbral(&palette, 100 - b.percent)
                        };
                        anillo.set(b.percent, &format!("{}", b.percent), color);
                        restante.set_text(&match (b.time_left, b.status.as_str()) {
                            (_, "Full") => "llena".to_string(),
                            (Some(t), "Charging") => format!("{} para llenar", sys::human_duration(t)),
                            (Some(t), _) => sys::human_duration(t),
                            (None, s) => s.to_string(),
                        });
                    }
                    None => {
                        anillo.set(0, "—", palette.muted);
                        restante.set_text("sin batería");
                    }
                }

                if let Some(p) = pp.as_ref().filter(|p| dbus::vivo(p)) {
                    let activo = dbus::prop::<String>(p, "ActiveProfile").unwrap_or_default();
                    for (id, b) in botones_perfil.borrow().iter() {
                        if *id == activo {
                            b.add_css_class("on");
                        } else {
                            b.remove_css_class("on");
                        }
                    }
                    let motivo = dbus::prop::<String>(p, "PerformanceDegraded").unwrap_or_default();
                    degradado.set_visible(!motivo.is_empty());
                    degradado.set_text(&format!("rendimiento limitado: {motivo}"));
                } else {
                    degradado.set_visible(false);
                }
            })
        };
        refrescar();

        {
            let refrescar = refrescar.clone();
            let timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
            handle.connect_toggled(move |abierto| {
                let mut t = timer.borrow_mut();
                if let Some(id) = t.take() {
                    id.remove();
                }
                if abierto {
                    refrescar();
                    let refrescar = refrescar.clone();
                    *t = Some(glib::timeout_add_local(
                        Duration::from_secs(LATIDO_S),
                        move || {
                            refrescar();
                            glib::ControlFlow::Continue
                        },
                    ));
                }
            });
        }

        Self { surface, handle }
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }
    pub fn present(&self) {
        self.surface.present();
    }
}

/// `Profiles` es un `aa{sv}`; de cada entrada solo interesa su clave `Profile`.
fn perfiles_desde_variant(p: &gio::DBusProxy) -> Option<Vec<String>> {
    let v = p.cached_property("Profiles")?;
    let mut ids = Vec::new();
    for i in 0..v.n_children() {
        let d = glib::VariantDict::new(Some(&v.child_value(i)));
        if let Some(id) = d.lookup_value("Profile", None).and_then(|x| x.get::<String>()) {
            ids.push(id);
        }
    }
    (!ids.is_empty()).then_some(ids)
}

/// Lanza una accion de sesion. Va por `sh -c` porque dos de ellas encadenan dos
/// ordenes, y nada de esto se traga el fallo: si systemctl se queja, se ve.
fn ejecutar(cmd: &'static str) {
    crate::proc::sh(cmd);
}
