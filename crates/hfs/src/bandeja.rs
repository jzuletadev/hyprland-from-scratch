// ~/hyprland-from-scratch/crates/hfs/src/bandeja.rs
//
// La bandeja: los programas que corren sin ventana. Canto de abajo, al centro.
//
//   ESTO SE HABIA DESCARTADO, Y ERA UN ERROR. El plan decia que el protocolo es viejo
//   y que con las notificaciones y la cabecera delante ya no quedaba casi nada que
//   solo la bandeja resolviera. En esta maquina si queda: `udiskie` **solo** se maneja
//   por su icono —es como se expulsa un USB— y Steam igual cuando cierras su ventana.
//   Al quitar Waybar se fueron los dos sin sustituto.
//
//   Y LLEVA LA GRABACION DENTRO. Es lo que pidio el usuario y tiene sentido: una
//   grabacion es exactamente eso, algo corriendo sin ventana. Tocar su icono la para.
//
//   PERO UNA BANDEJA ESCONDIDA NO AVISA DE NADA. Si el motivo de meter la grabacion
//   aqui es no olvidarse de pararla, un panel que hay que destapar no sirve: si te
//   olvidaste de la grabacion, te olvidas de mirar la bandeja. Por eso mientras se
//   graba el canto lleva una **marca** (`Handle::set_mark`): una pastilla roja de 34
//   px asomando por abajo. Es la unica excepcion a «los cantos no llevan marca», y
//   solo existe mientras dura el motivo.

use std::rc::Rc;

use gtk4 as gtk;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::widgets;
use hfs_ui::{EdgeSurface, Handle, Place, Side, Spec};

const ALTO: i32 = 58;
/// Lado del icono de cada programa, en px.
const ICONO: i32 = 22;

pub struct Bandeja {
    surface: EdgeSurface,
    pub handle: Handle,
    /// Repinta la lista de iconos. Lo llama el sondeo de `shell.rs` cuando la
    /// grabacion empieza o termina.
    refrescar: Rc<dyn Fn()>,
    /// El color del aviso, sacado de la paleta de esta superficie.
    rojo: hfs_core::Rgb,
}

impl Bandeja {
    pub fn new(
        app: &gtk::Application,
        palette: &Palette,
        monitor: &gtk::gdk::Monitor,
        sni: Option<&Rc<crate::sni::Bandeja>>,
    ) -> Self {
        let m = hfs_ui::medidas::Medidas::para(&monitor.geometry());
        let mut spec = Spec::new("hfs-bandeja", Side::Bottom, m.bandeja, ALTO);
        spec.place = Place::Center(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("bandeja");
        surface.body.set_orientation(gtk::Orientation::Horizontal);
        surface.body.set_spacing(4);
        surface.body.set_halign(gtk::Align::Center);

        let fila = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        fila.set_halign(gtk::Align::Center);
        let vacio = gtk::Label::new(Some("nada en segundo plano"));
        vacio.add_css_class("foot");

        surface.body.append(&fila);
        surface.body.append(&vacio);

        let refrescar: Rc<dyn Fn()> = {
            let (fila, vacio, handle) = (fila.clone(), vacio.clone(), handle.clone());
            let sni = sni.cloned();
            let palette = palette.clone();
            Rc::new(move || {
                widgets::clear(&fila);
                let mut cuantos = 0;

                // La grabacion va primero: es lo unico de aqui que urge.
                if crate::captura::grabando() {
                    fila.append(&boton_grabacion(&palette));
                    cuantos += 1;
                }

                if let Some(s) = sni.as_ref() {
                    for it in s.items() {
                        // `Passive` quiere decir «no me pintes ahora».
                        if it.estado == "Passive" {
                            continue;
                        }
                        fila.append(&boton_item(s, &it));
                        cuantos += 1;
                    }
                }

                vacio.set_visible(cuantos == 0);
                fila.set_visible(cuantos > 0);
                // Las DOS medidas, y en este orden. Ajustar solo el ancho dejaba el
                // alto clavado al del panel vacio —que es como nace, antes de que
                // ningun programa se haya registrado— y despues el icono ya no cabia:
                // GTK lo avisaba con «needs at least 60» y el icono salia cortado.
                handle.fit_depth();
                handle.fit_length();
            })
        };
        refrescar();

        if let Some(s) = sni {
            s.cuando_cambie(refrescar.clone());
        }
        {
            let refrescar = refrescar.clone();
            handle.connect_toggled(move |abierto| {
                if abierto {
                    refrescar();
                }
            });
        }

        surface.fit_length();
        surface.fit_depth();
        Self {
            surface,
            handle,
            refrescar,
            rojo: palette.bad,
        }
    }

    /// Enciende o apaga el aviso de grabacion: la marca del canto siempre, y el icono
    /// dentro del panel. `Handle::set_mark` ya se calla si el color no cambia, asi que
    /// esto se puede llamar cada segundo sin pedir un frame de mas.
    pub fn grabacion(&self, activa: bool, cambio: bool) {
        self.handle.set_mark(if activa { Some(self.rojo) } else { None });
        if cambio {
            (self.refrescar)();
        }
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }
    pub fn present(&self) {
        self.surface.present();
    }
}

/// El icono de la grabacion. Tocarlo la para, que es para lo que esta.
fn boton_grabacion(palette: &Palette) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("tray");
    b.add_css_class("rec");
    b.set_has_frame(false);
    let punto = gtk::DrawingArea::new();
    punto.set_content_width(ICONO);
    punto.set_content_height(ICONO);
    let (r, g, bl) = palette.bad.as_f64();
    punto.set_draw_func(move |_, ctx, w, h| {
        ctx.set_source_rgb(r, g, bl);
        ctx.arc(w as f64 / 2.0, h as f64 / 2.0, 6.0, 0.0, std::f64::consts::TAU);
        ctx.fill().ok();
    });
    b.set_child(Some(&punto));
    b.set_tooltip_text(Some("Grabando · tocar para parar"));
    b.connect_clicked(|_| crate::captura::parar());
    b
}

fn boton_item(sni: &Rc<crate::sni::Bandeja>, it: &crate::sni::Item) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("tray");
    b.set_has_frame(false);
    if it.estado == "NeedsAttention" {
        b.add_css_class("atencion");
    }
    b.set_child(Some(&imagen_de(it)));
    b.set_tooltip_text(Some(it.etiqueta()));

    {
        let (sni, it) = (sni.clone(), it.clone());
        b.connect_clicked(move |_| sni.activar(&it));
    }
    // Clic derecho: que el programa saque su propio menu.
    {
        let (sni, it) = (sni.clone(), it.clone());
        let der = gtk::GestureClick::new();
        der.set_button(3);
        der.connect_pressed(move |_, _, _, _| sni.menu(&it));
        b.add_controller(der);
    }
    b
}

/// El icono del programa. Primero por nombre —asi sigue al tema de iconos—, y si no
/// hay, el mapa de bits que publique el propio programa.
fn imagen_de(it: &crate::sni::Item) -> gtk::Image {
    if !it.icono.is_empty() {
        // Un tema propio hay que anadirlo a la busqueda o el nombre no resuelve.
        if let Some(dir) = it.tema_propio.as_ref()
            && let Some(d) = gtk::gdk::Display::default()
        {
            gtk::IconTheme::for_display(&d).add_search_path(dir);
        }
        let img = gtk::Image::from_icon_name(&it.icono);
        img.set_pixel_size(ICONO);
        return img;
    }
    if let Some(tex) = textura(it) {
        let img = gtk::Image::from_paintable(Some(&tex));
        img.set_pixel_size(ICONO);
        return img;
    }
    let img = gtk::Image::from_icon_name("application-x-executable-symbolic");
    img.set_pixel_size(ICONO);
    img
}

/// Convierte el `IconPixmap` del protocolo en algo pintable.
///
/// Viene en ARGB32 **big-endian**, que no es el orden de esta maquina, asi que hay que
/// darle la vuelta a cada pixel. Y GDK quiere RGBA sin premultiplicar.
fn textura(it: &crate::sni::Item) -> Option<gtk::gdk::Texture> {
    let (w, h, datos) = it.pixmap.as_ref()?;
    let (w, h) = (*w, *h);
    let mut rgba = Vec::with_capacity(datos.len());
    for p in datos.as_chunks::<4>().0 {
        let (a, r, g, b) = (p[0], p[1], p[2], p[3]);
        rgba.extend_from_slice(&[r, g, b, a]);
    }
    let bytes = hfs_ui::gtk::glib::Bytes::from_owned(rgba);
    Some(gtk::gdk::MemoryTexture::new(
        w,
        h,
        gtk::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (w * 4) as usize,
    ).upcast())
}
