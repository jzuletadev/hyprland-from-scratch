// ~/hyprland-from-scratch/crates/hfs/src/portapapeles.rs
//
// El historial del portapapeles, en la esquina de abajo a la izquierda.
//
// El historial no existia en la v1: `wl-copy` guarda lo ultimo y punto. Aqui lo
// alimenta `wl-paste --watch`, que el escritorio lanza al arrancar y que llama a
// `hfs clip-add` cada vez que cambia la seleccion.
//
// Tocar una entrada la devuelve al portapapeles (`wl-copy`); pegar sigue siendo cosa
// tuya, con tu Ctrl+V de siempre. Meterse a sintetizar la pulsacion seria adivinar en
// que ventana quieres pegar.
//
// NO tiene filtro todavia, y el boceto lo pide. El motivo es que escribir necesita el
// teclado, y un panel de borde no lo pide (`KeyboardMode::None`): si lo pidiera «a
// demanda», al cerrarse solo despues de que el raton se vaya podria quedarse con el
// foco del teclado. Esta apuntado en V2-PLAN.md.

use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;

use hfs_core::clipboard::{self, Clase};
use hfs_core::{sys, Palette};
use hfs_ui::widgets;
use hfs_ui::{EdgeSurface, Handle, Place, Side, Spec};

/// Cuantas entradas se pintan. Antes eran 8 porque el panel crecia sin tope; ahora
/// hace scroll y se pueden ver todas las que haya.
const VISIBLES: usize = 60;

pub struct Portapapeles {
    surface: EdgeSurface,
    pub handle: Handle,
}

impl Portapapeles {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);
        let mut spec = Spec::new(
            "hfs-portapapeles",
            Side::Bottom,
            m.esquina,
            m.esquina_alto,
        );
        // Abajo a la izquierda: nace de la esquina.
        spec.place = Place::Start(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("clip");
        surface.body.set_orientation(gtk::Orientation::Vertical);
        surface.body.set_spacing(6);

        let (cab, cuenta) = widgets::header("Portapapeles");
        let vaciar = gtk::Button::new();
        vaciar.add_css_class("bic");
        vaciar.set_has_frame(false);
        let img = gtk::Image::from_icon_name("user-trash-symbolic");
        img.set_pixel_size(14);
        vaciar.set_child(Some(&img));
        cab.append(&vaciar);

        let lista = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let pie = gtk::Label::new(Some("tocar copia · Ctrl+V pega"));
        pie.add_css_class("foot");
        pie.set_xalign(0.0);

        let rollo = gtk::ScrolledWindow::new();
        rollo.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        // Llena lo que quede del panel, que ahora tiene alto fijo. Un tope propio
        // sobra: el tope es el panel.
        rollo.set_vexpand(true);
        rollo.set_child(Some(&lista));

        surface.body.append(&cab);
        surface.body.append(&rollo);
        surface.body.append(&pie);

        let refrescar: Rc<dyn Fn()> = {
            let (lista, cuenta, handle) = (lista.clone(), cuenta.clone(), handle.clone());
            Rc::new(move || {
                let entradas = clipboard::list(VISIBLES);
                let total = clipboard::list(usize::MAX).len();
                cuenta.set_text(&match total {
                    0 => "vacío".to_string(),
                    1 => "1 entrada".to_string(),
                    n => format!("{n} entradas"),
                });
                pintar(&lista, &entradas, &handle);
            })
        };
        refrescar();

        {
            let refrescar = refrescar.clone();
            vaciar.connect_clicked(move |_| {
                if let Err(e) = clipboard::vaciar() {
                    eprintln!("hfs: {e}");
                }
                refrescar();
            });
        }
        // Se relee al abrir. Vigilar el directorio seria mas fino, pero el historial
        // solo cambia cuando copias algo, y entonces el panel esta cerrado.
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

fn icono_de(c: Clase) -> &'static str {
    match c {
        Clase::Url => "insert-link-symbolic",
        Clase::Ruta => "folder-symbolic",
        Clase::Codigo => "text-x-generic-symbolic",
        Clase::Texto => "edit-paste-symbolic",
    }
}

fn pintar(lista: &gtk::Box, entradas: &[clipboard::Entrada], handle: &Handle) {
    widgets::clear(lista);
    if entradas.is_empty() {
        let l = gtk::Label::new(Some("nada copiado todavía"));
        l.add_css_class("sempty");
        l.set_xalign(0.0);
        lista.append(&l);
        return;
    }
    for e in entradas {
        let fila = widgets::row();
        let img = gtk::Image::from_icon_name(icono_de(e.clase));
        img.set_pixel_size(14);
        let texto = widgets::ellipsized(Some(&e.vista), "");
        texto.set_hexpand(true);
        let edad = gtk::Label::new(Some(&hace(e.edad)));
        edad.add_css_class("k");
        fila.append(&img);
        fila.append(&texto);
        fila.append(&edad);

        let click = gtk::GestureClick::new();
        {
            let (id, handle) = (e.id.clone(), handle.clone());
            click.connect_released(move |_, _, _, _| {
                match clipboard::leer(&id) {
                    Ok(bytes) => copiar(&bytes),
                    Err(e) => eprintln!("hfs: {e}"),
                }
                handle.close_now();
            });
        }
        fila.add_controller(click);
        lista.append(&fila);
    }
}

/// Devuelve el contenido al portapapeles. Por `wl-copy` y no por la API de GTK
/// porque el portapapeles de GTK muere con el widget que lo ofrece: en cuanto el
/// panel se cierra, lo copiado dejaria de estar disponible.
fn copiar(bytes: &[u8]) {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let hijo = Command::new("wl-copy")
        .arg("--type")
        .arg("text/plain;charset=utf-8")
        .stdin(Stdio::piped())
        .spawn();
    match hijo {
        Ok(mut h) => {
            if let Some(mut e) = h.stdin.take()
                && let Err(err) = e.write_all(bytes)
            {
                eprintln!("hfs: no pude pasarle el texto a wl-copy: {err}");
            }
            // Y que lo recoja GLib: una copia por cada vez que se toca una entrada,
            // y el escritorio no sale nunca.
            crate::proc::adoptar(h);
        }
        Err(e) => eprintln!("hfs: no pude ejecutar wl-copy: {e}"),
    }
}

fn hace(d: std::time::Duration) -> String {
    let s = d.as_secs();
    if s < 60 {
        "ahora".into()
    } else if s < 3600 * 24 {
        sys::human_duration(d)
    } else {
        format!("{} d", s / 86400)
    }
}
