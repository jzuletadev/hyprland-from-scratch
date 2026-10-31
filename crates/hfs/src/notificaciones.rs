// ~/hyprland-from-scratch/crates/hfs/src/notificaciones.rs
//
// El historial de notificaciones, en la esquina de arriba a la derecha.
//
// No es un daemon propio: habla con dunst por su interfaz `org.dunstproject.cmd0`.
// Escribir un daemon significa quedarse con `org.freedesktop.Notifications`, que es
// un nombre unico en el bus, asi que seria sustituir a dunst de golpe y quedarse sin
// avisos emergentes el dia que algo falle. Mientras dunst siga ahi, esto es una vista
// sobre su historial, y el dia que haga falta un daemon propio lo unico que cambia es
// de donde salen estas filas.
//
// Tocar una entrada la devuelve a la pantalla (`NotificationPopHistory`), y eso es lo
// que recupera **sus acciones**: una notificacion en el historial ya no las tiene, y
// no hay forma de invocarlas desde fuera del daemon. Es la respuesta honesta al
// «Anotar en satty» del boceto.

use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::{sys, Palette};
use hfs_ui::widgets;
use hfs_ui::{dbus, EdgeSurface, Handle, Place, Side, Spec};

/// Cuantas se enseñan. El historial de dunst puede tener cientos y esto es un
/// vistazo, no un archivo.
/// Cuantas se guardan en la lista. Antes eran 6 porque el panel crecia sin tope y
/// mas no cabian; ahora hace scroll, asi que caben todas las que dunst recuerde.
const MAXIMO: usize = 40;

const SERVICIO: &str = "org.freedesktop.Notifications";
const RUTA: &str = "/org/freedesktop/Notifications";
const IFACE: &str = "org.dunstproject.cmd0";

pub struct Notificaciones {
    surface: EdgeSurface,
    pub handle: Handle,
}

struct Nota {
    id: u32,
    app: String,
    resumen: String,
    cuerpo: String,
    icono: String,
    /// Microsegundos desde el arranque, que es como los marca dunst.
    sello_us: i64,
    urgente: bool,
}

impl Notificaciones {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);
        let mut spec = Spec::new(
            "hfs-notificaciones",
            Side::Top,
            m.esquina,
            m.esquina_alto,
        );
        spec.place = Place::End(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("notifs");
        surface.body.set_orientation(gtk::Orientation::Vertical);
        surface.body.set_spacing(8);

        let (cab, cuenta) = widgets::header("Notificaciones");
        let limpiar = gtk::Button::new();
        limpiar.add_css_class("bic");
        limpiar.set_has_frame(false);
        let img = gtk::Image::from_icon_name("user-trash-symbolic");
        img.set_pixel_size(14);
        limpiar.set_child(Some(&img));
        cab.append(&limpiar);

        let lista = gtk::Box::new(gtk::Orientation::Vertical, 6);

        // La lista va dentro de un scroll con tope de alto: el panel crece con las
        // notificaciones que haya hasta ese tope y a partir de ahi se desplaza, en
        // vez de estirarse hasta ocupar media pantalla.
        let rollo = gtk::ScrolledWindow::new();
        rollo.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        // Llena lo que quede del panel, que ahora tiene alto fijo. Un tope propio
        // sobra: el tope es el panel.
        rollo.set_vexpand(true);
        rollo.set_child(Some(&lista));

        surface.body.append(&cab);
        surface.body.append(&rollo);

        let proxy = dbus::session(SERVICIO, RUTA, IFACE).ok();
        if proxy.is_none() {
            eprintln!("hfs: sin dunst; no habra historial de notificaciones");
        }

        let refrescar: Rc<dyn Fn()> = {
            let (lista, cuenta, proxy, handle, palette) = (
                lista.clone(),
                cuenta.clone(),
                proxy.clone(),
                handle.clone(),
                palette.clone(),
            );
            Rc::new(move || {
                let notas = leer_historial(proxy.as_ref());
                cuenta.set_text(&match notas.len() {
                    0 => "vacío".to_string(),
                    1 => "1 guardada".to_string(),
                    n => format!("{n} guardadas"),
                });
                pintar(&lista, &notas, proxy.as_ref(), &handle, &palette);
            })
        };
        refrescar();

        {
            let (proxy, refrescar) = (proxy.clone(), refrescar.clone());
            limpiar.connect_clicked(move |_| {
                if let Some(p) = &proxy
                    && let Err(e) = dbus::call(p, "NotificationClearHistory", None)
                {
                    eprintln!("hfs: no pude vaciar el historial: {e}");
                }
                refrescar();
            });
        }

        // Reactivo: dunst emite un cambio de `historyLength` cada vez que algo entra
        // o sale del historial, asi que no hace falta preguntar cada x segundos.
        if let Some(p) = &proxy {
            // `connect_local` y no `connect_g_properties_changed`: el segundo exige
            // que el cierre sea Send + Sync, y aqui dentro hay Rc y widgets de GTK,
            // que no lo son ni lo pueden ser. La señal llega igual, y llega en el
            // hilo del bucle principal, que es donde hace falta.
            let refrescar = refrescar.clone();
            p.connect_local("g-properties-changed", false, move |_| {
                refrescar();
                None
            });
        }
        // Y al abrir, por si el panel llevaba cerrado desde antes de arrancar dunst.
        {
            let refrescar = refrescar.clone();
            handle.connect_toggled(move |abierto| {
                if abierto {
                    refrescar();
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

fn leer_historial(proxy: Option<&gio::DBusProxy>) -> Vec<Nota> {
    let Some(p) = proxy.filter(|p| dbus::vivo(p)) else {
        return Vec::new();
    };
    let Ok(r) = dbus::call(p, "NotificationListHistory", None) else {
        return Vec::new();
    };
    // La respuesta es `(aa{sv})`: el hijo 0 ES el array. Pedirle que lo desenvuelva
    // como un `v` hace saltar una asercion de GLib y devuelve nada, que es por lo que
    // el historial salia vacio teniendo dunst entradas.
    if r.n_children() == 0 {
        return Vec::new();
    }
    let lista = r.child_value(0);

    let mut v = Vec::new();
    for i in 0..lista.n_children() {
        let d = glib::VariantDict::new(Some(&lista.child_value(i)));
        let s = |k: &str| {
            d.lookup_value(k, None)
                .and_then(|x| x.get::<String>())
                .unwrap_or_default()
        };
        v.push(Nota {
            id: d
                .lookup_value("id", None)
                .and_then(|x| x.get::<i32>())
                .unwrap_or(0)
                .max(0) as u32,
            app: s("appname"),
            resumen: s("summary"),
            cuerpo: s("body"),
            icono: s("icon_path"),
            sello_us: d
                .lookup_value("timestamp", None)
                .and_then(|x| x.get::<i64>())
                .unwrap_or(0),
            urgente: s("urgency") == "CRITICAL",
        });
        if v.len() >= MAXIMO {
            break;
        }
    }
    v
}

fn pintar(
    lista: &gtk::Box,
    notas: &[Nota],
    proxy: Option<&gio::DBusProxy>,
    handle: &Handle,
    palette: &Palette,
) {
    widgets::clear(lista);
    if notas.is_empty() {
        let l = gtk::Label::new(Some("nada pendiente"));
        l.add_css_class("sempty");
        l.set_xalign(0.0);
        lista.append(&l);
        return;
    }
    let _ = palette;

    for n in notas {
        let fila = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        fila.add_css_class("note");
        if n.urgente {
            fila.add_css_class("urgente");
        }

        let img = gtk::Image::new();
        img.add_css_class("thumb");
        img.set_pixel_size(22);
        // El icono puede venir como ruta a un fichero o como nombre del tema; dunst
        // mezcla las dos y hay aplicaciones que mandan un .bmp.
        if !n.icono.is_empty() && std::path::Path::new(&n.icono).is_file() {
            img.set_from_file(Some(&n.icono));
        } else if !n.icono.is_empty() {
            img.set_icon_name(Some(&n.icono));
        } else {
            img.set_icon_name(Some("dialog-information-symbolic"));
        }
        img.set_valign(gtk::Align::Start);

        let cabeza = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let app = widgets::ellipsized(Some(&n.app), "src");
        let hora = gtk::Label::new(Some(&hora_de(n.sello_us)));
        hora.add_css_class("k");
        hora.set_hexpand(true);
        hora.set_xalign(1.0);
        cabeza.append(&app);
        cabeza.append(&hora);

        let cuerpo = gtk::Box::new(gtk::Orientation::Vertical, 2);
        cuerpo.set_hexpand(true);
        cuerpo.append(&cabeza);
        cuerpo.append(&widgets::ellipsized(Some(&n.resumen), "ti"));
        if !n.cuerpo.is_empty() {
            cuerpo.append(&widgets::ellipsized(Some(&n.cuerpo), "dt"));
        }

        fila.append(&img);
        fila.append(&cuerpo);

        // Tocarla la devuelve a la pantalla con sus acciones vivas.
        let click = gtk::GestureClick::new();
        {
            let (proxy, handle, id) = (proxy.cloned(), handle.clone(), n.id);
            click.connect_released(move |_, _, _, _| {
                if let Some(p) = &proxy
                    && let Err(e) =
                        dbus::call(p, "NotificationPopHistory", Some(&(id,).to_variant()))
                {
                    eprintln!("hfs: no pude recuperar la notificacion {id}: {e}");
                }
                handle.close_now();
            });
        }
        fila.add_controller(click);
        lista.append(&fila);
    }
}

/// dunst marca las notificaciones con microsegundos desde el arranque, no con la
/// hora. Para enseñar «23:31» hay que restarle a ahora lo que lleva encendida.
fn hora_de(sello_us: i64) -> String {
    let Ok(arriba) = sys::uptime() else {
        return String::new();
    };
    let edad = arriba.as_secs_f64() - sello_us as f64 / 1e6;
    if !(0.0..=60.0 * 60.0 * 24.0 * 30.0).contains(&edad) {
        return String::new();
    }
    glib::DateTime::now_local()
        .and_then(|d| d.add_seconds(-edad))
        .and_then(|d| d.format("%H:%M"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}
