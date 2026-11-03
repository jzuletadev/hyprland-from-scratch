// ~/hyprland-from-scratch/crates/hfs/src/accesos.rs
//
// Capturas y accesos rapidos, en la esquina de abajo a la derecha.
//
// Sustituye al reloj, que se mudo al dashboard. El cambio tiene sentido: el reloj y
// el calendario son el mismo dato a dos escalas y estaban separados, mientras que lo
// que de verdad se quiere a mano en una esquina es lo que se usa a medias de otra
// cosa — una captura, silenciar los avisos, no dejar que la pantalla se apague.
//
// Tres filas, como el boceto: captura, grabacion y accesos. Las dos primeras acaban
// en el icono de su carpeta, porque despues de la decima captura lo que hace falta no
// es otra captura sino encontrar las anteriores.

use std::cell::Cell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::{widgets, EdgeSurface, Handle, Place, Side, Spec};

const CAPTURAS: &str = "Pictures/screenshots";
const GRABACIONES: &str = "Videos/recordings";

pub struct Accesos {
    surface: EdgeSurface,
    pub handle: Handle,
    refrescar: Rc<dyn Fn()>,
}

impl Accesos {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let m = hfs_ui::medidas::Medidas::para(&monitor.geometry());

        let mut spec = Spec::new("hfs-accesos", Side::Bottom, m.esquina, m.esquina_alto);
        spec.place = Place::End(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("accesos");
        surface.body.set_orientation(gtk::Orientation::Vertical);
        surface.body.set_spacing(6);

        // Cerrar antes de disparar: `slurp` y `grim` no pueden trabajar con un panel
        // encima, y aunque pudieran saldria el panel en la foto.
        let cerrar: Rc<dyn Fn()> = {
            let h = handle.clone();
            Rc::new(move || h.close_now())
        };

        // --- captura ---
        //
        // Satty es un CONMUTADOR, no un tercer boton: anotar no es otra forma de
        // capturar, es que ademas la abras para pintarla. Como boton obligaba a
        // elegir entre «region» y «region anotada», que es la misma accion.
        let (f_cap, cajas_cap) = fila("Screenshot", "camera-photo-symbolic");
        let g_cap = grupo();
        let satty = Rc::new(Cell::new(false));
        let b_satty = conmutador("Satty", "Open it in Satty to annotate");
        {
            let satty = satty.clone();
            b_satty.connect_toggled(move |b| satty.set(b.is_active()));
        }
        for (txt, region) in [("Screen", false), ("Region", true)] {
            let b = boton(txt);
            let (cerrar, satty) = (cerrar.clone(), satty.clone());
            b.connect_clicked(move |_| {
                // `annotate` del script solo sabe de region; para la pantalla entera
                // se captura y se abre satty a mano.
                let arg = match (region, satty.get()) {
                    (true, true) => "annotate",
                    (true, false) => "region",
                    (false, _) => "screen",
                };
                let abrir = !region && satty.get();
                cerrar();
                let s = hfs_core::paths::repo().join("scripts/screenshot.sh");
                let mut cmd = format!("sleep 0.25; {} {arg}", s.display());
                if abrir {
                    let d = hfs_core::paths::home().join(CAPTURAS);
                    cmd.push_str(&format!(
                        "; f=$(ls -t '{}'/*.png 2>/dev/null | head -1); [ -n \"$f\" ] && exec satty -f \"$f\"",
                        d.display()
                    ));
                }
                crate::proc::sh(&cmd);
            });
            g_cap.append(&b);
        }
        g_cap.append(&b_satty);
        cajas_cap.append(&g_cap);
        cajas_cap.append(&carpeta(CAPTURAS, cerrar.clone()));

        // --- grabacion ---
        let (f_rec, cajas_rec) = fila("Recording", "media-record-symbolic");
        let g_rec = grupo();
        let audio = Rc::new(Cell::new(false));
        let b_audio = conmutador("Audio", "Record sound as well");
        {
            let audio = audio.clone();
            b_audio.connect_toggled(move |b| audio.set(b.is_active()));
        }
        let b_grabar = boton("Screen");
        let b_region = boton("Region");
        for (b, region) in [(&b_grabar, false), (&b_region, true)] {
            let (cerrar, audio) = (cerrar.clone(), audio.clone());
            b.connect_clicked(move |_| {
                if crate::captura::grabando() {
                    crate::captura::parar();
                    return;
                }
                let a = audio.get();
                cerrar();
                crate::captura::arrancar(region, a);
            });
        }
        g_rec.append(&b_grabar);
        g_rec.append(&b_region);
        g_rec.append(&b_audio);
        cajas_rec.append(&g_rec);
        cajas_rec.append(&carpeta(GRABACIONES, cerrar.clone()));

        // --- accesos rapidos ---
        //
        // Aqui viven tambien CAPS, NUM y la distribucion. Estaban en la bandeja, que
        // es el sitio de lo que publican OTROS programas; el estado del teclado es
        // nuestro y se consulta cuando se consulta, no de reojo.
        let (f_acc, cajas_acc) = fila("Quick actions", "preferences-system-symbolic");
        let caps = pastilla("CAPS");
        let num = pastilla("NUM");
        let dist = gtk::Button::new();
        dist.add_css_class("tray-pill");
        dist.set_has_frame(false);
        let dist_txt = gtk::Label::new(Some("··"));
        dist.set_child(Some(&dist_txt));
        dist.set_tooltip_text(Some("Keyboard layout · click to switch"));
        let estado_teclas = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        estado_teclas.append(&caps);
        estado_teclas.append(&num);
        estado_teclas.append(&dist);

        let b_ajustes = icono("preferences-system-symbolic", "Settings (the wheel)");
        {
            let cerrar = cerrar.clone();
            b_ajustes.connect_clicked(move |_| {
                cerrar();
                crate::ipc::nudge(crate::ipc::Aviso::Rueda);
            });
        }
        let b_despierto = gtk::ToggleButton::new();
        b_despierto.add_css_class("acc-ic");
        b_despierto.set_has_frame(false);
        {
            let img = gtk::Image::from_icon_name("preferences-desktop-screensaver-symbolic");
            img.set_pixel_size(15);
            b_despierto.set_child(Some(&img));
        }
        b_despierto.set_tooltip_text(Some("Keep the screen awake"));
        let b_silencio = gtk::ToggleButton::new();
        b_silencio.add_css_class("acc-ic");
        b_silencio.set_has_frame(false);
        {
            let img = gtk::Image::from_icon_name("notifications-disabled-symbolic");
            img.set_pixel_size(15);
            b_silencio.set_child(Some(&img));
        }
        b_silencio.set_tooltip_text(Some("Mute notifications"));

        estado_teclas.set_hexpand(true);
        cajas_acc.append(&estado_teclas);
        cajas_acc.append(&b_ajustes);
        cajas_acc.append(&b_despierto);
        cajas_acc.append(&b_silencio);

        surface.body.append(&f_cap);
        surface.body.append(&f_rec);
        surface.body.append(&f_acc);

        // Los dos conmutadores reflejan el estado de VERDAD, no lo que se pulso: el
        // demonio puede haber cambiado por fuera, y un boton que miente es peor que
        // no tenerlo.
        let refrescar: Rc<dyn Fn()> = {
            let (b_despierto, b_silencio, b_grabar) =
                (b_despierto.clone(), b_silencio.clone(), b_grabar.clone());
            let (caps, num, dist_txt) = (caps.clone(), num.clone(), dist_txt.clone());
            Rc::new(move || {
                match hfs_core::teclado::estado() {
                    Ok(e) => {
                        caps.set_css_classes(if e.caps { &["tray-pill", "on"] } else { &["tray-pill"] });
                        num.set_css_classes(if e.num { &["tray-pill", "on"] } else { &["tray-pill"] });
                        dist_txt.set_text(if e.distribucion.is_empty() { "··" } else { &e.distribucion });
                    }
                    Err(e) => {
                        eprintln!("hfs: no pude leer el teclado: {e}");
                        dist_txt.set_text("··");
                    }
                }
                b_despierto.set_active(!crate::proc::vivo("hypridle"));
                b_silencio.set_active(dunst_en_pausa());
                b_grabar.set_label(if crate::captura::grabando() {
                    "Stop"
                } else {
                    "Screen"
                });
            })
        };
        refrescar();

        {
            let refrescar_t = refrescar.clone();
            dist.connect_clicked(move |_| {
                if let Err(e) = hfs_core::teclado::siguiente_distribucion() {
                    eprintln!("hfs: no pude cambiar la distribución: {e}");
                }
                refrescar_t();
            });
        }

        b_despierto.connect_toggled(|b| {
            // `hypridle` es un `exec-once`, no un servicio: se para matandolo y se
            // repone lanzandolo otra vez.
            if b.is_active() {
                crate::proc::sh("pkill -x hypridle");
            } else if !crate::proc::vivo("hypridle") {
                crate::proc::sh("exec hypridle");
            }
        });
        b_silencio.connect_toggled(|b| {
            crate::proc::sh(&format!(
                "dunstctl set-paused {}",
                if b.is_active() { "true" } else { "false" }
            ));
        });

        {
            let refrescar = refrescar.clone();
            handle.connect_toggled(move |abierto| {
                if abierto {
                    refrescar();
                }
            });
        }

        Self {
            surface,
            handle,
            refrescar,
        }
    }

    pub fn refresh(&self) {
        (self.refrescar)();
    }
    pub fn destroy(&self) {
        self.surface.destroy();
    }
    pub fn present(&self) {
        self.surface.present();
    }
}

/// Una pastilla de estado: encendida o apagada, nunca ausente. Que desaparezca
/// cuando esta apagada obligaria a recordar que existia.
fn pastilla(texto: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(texto));
    l.add_css_class("tray-pill");
    l
}

/// Una fila: su titulo arriba y los botones debajo.
fn fila(titulo: &str, icono: &str) -> (gtk::Box, gtk::Box) {
    let caja = widgets::tile();
    let th = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    th.add_css_class("th");
    let img = gtk::Image::from_icon_name(icono);
    img.set_pixel_size(13);
    th.append(&img);
    th.append(&gtk::Label::new(Some(titulo)));
    let botones = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    caja.append(&th);
    caja.append(&botones);
    (caja, botones)
}

/// El grupo de acciones de una fila: todas del mismo ancho.
///
/// Es un grupo aparte y no la fila entera porque el icono de la carpeta NO debe medir
/// lo que «Pantalla»: metiendolo dentro, cuatro piezas homogeneas no cabian en los
/// 391 px del panel y la fila se salia por la derecha.
fn grupo() -> gtk::Box {
    let g = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    g.set_homogeneous(true);
    g.set_hexpand(true);
    g
}

fn boton(txt: &str) -> gtk::Button {
    let b = gtk::Button::with_label(txt);
    b.add_css_class("key");
    b.set_has_frame(false);
    b.set_hexpand(true);
    b
}

/// Un conmutador con el mismo aspecto que un boton de accion. Antes «Satty» era un
/// boton y «Audio» un conmutador, y se veia: dos piezas distintas en la misma fila
/// para dos cosas que se eligen igual.
fn conmutador(txt: &str, titulo: &str) -> gtk::ToggleButton {
    let b = gtk::ToggleButton::with_label(txt);
    b.add_css_class("key");
    b.set_has_frame(false);
    b.set_hexpand(true);
    b.set_tooltip_text(Some(titulo));
    b
}

fn icono(nombre: &str, titulo: &str) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("acc-ic");
    b.set_has_frame(false);
    let img = gtk::Image::from_icon_name(nombre);
    img.set_pixel_size(15);
    b.set_child(Some(&img));
    b.set_tooltip_text(Some(titulo));
    b
}

/// El icono que abre la carpeta donde acaba lo que se captura.
fn carpeta(rel: &'static str, cerrar: Rc<dyn Fn()>) -> gtk::Button {
    let b = icono("folder-symbolic", "Open the folder");
    b.connect_clicked(move |_| {
        cerrar();
        let d = hfs_core::paths::home().join(rel);
        // `xdg-open` NO vale aqui: en esta maquina el handler de `inode/directory`
        // es `kitty-open.desktop`, asi que abria una terminal. Se llama al gestor de
        // ficheros por su nombre, con xdg-open detras por si no estuviera.
        crate::proc::sh(&format!(
            "mkdir -p '{0}'; command -v thunar >/dev/null && exec thunar '{0}' || exec xdg-open '{0}'",
            d.display()
        ));
    });
    b
}

/// Si dunst tiene las notificaciones en pausa. Se pregunta al demonio en vez de
/// recordar lo que se pulso: `dunstctl` tambien se usa desde fuera.
fn dunst_en_pausa() -> bool {
    hfs_core::cmd::run("dunstctl", &["is-paused"])
        .map(|s| s.trim() == "true")
        .unwrap_or(false)
}
