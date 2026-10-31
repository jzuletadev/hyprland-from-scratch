// ~/hyprland-from-scratch/crates/hfs/src/espacios.rs
//
// El rail de espacios del canto izquierdo: el icono de arch arriba y debajo un punto
// por espacio, el activo mas largo y del color del acento.
//
// Es la primera superficie reactiva del escritorio: no pregunta nada cada x
// milisegundos. Hyprland avisa por su socket de eventos y el rail se rehace.
//
// El rail enseña los espacios que existen EN ESTA PANTALLA, ni mas ni menos. En
// Hyprland un espacio vive en un monitor concreto, asi que una fila fija del 1 al 10
// seria mentira en cuanto hay dos pantallas. Como la cuenta cambia, el panel se mide
// y se encoge: uno de altura fija con dos puntos dentro se ve como un fallo. Para ir
// a un espacio que todavia no existe esta la rueda del raton, que hace `workspace
// e+1` y lo crea.

use gtk4 as gtk;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::hyprland::{self, State};
use hfs_core::Palette;
use hfs_ui::{EdgeSurface, Handle, Place, Side, Spec};

/// Altura de partida; en cuanto llega el primer estado se ajusta sola.
const ALTO_INICIAL: i32 = 180;
/// Lado del icono de arch.
const ARCH_PX: i32 = 18;

/// El logo de arch, dibujado y no sacado del tema de iconos.
///
/// Los temas de iconos de Arch traen el logo en PNG y en azul, asi que no sigue al
/// tema; y no hay version «symbolic» que se pueda teñir. Son seis lineas de Cairo y
/// asi sale siempre del color del acento.
///
/// Es el mismo trazado de la maqueta: el triangulo de fuera menos el de dentro, con
/// regla de relleno par-impar para que el de dentro sea un hueco y no una mancha.
fn arch_icon(color: hfs_core::Rgb, lado: i32) -> gtk::DrawingArea {
    let a = gtk::DrawingArea::new();
    a.set_content_width(lado);
    a.set_content_height(lado);
    a.set_draw_func(move |_, ctx, w, h| {
        // El trazado esta en el lienzo de 24x24 del boceto.
        let k = (w as f64 / 24.0).min(h as f64 / 24.0);
        ctx.save().ok();
        ctx.translate(
            (w as f64 - 24.0 * k) / 2.0,
            (h as f64 - 24.0 * k) / 2.0,
        );
        ctx.scale(k, k);
        ctx.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
        ctx.move_to(12.0, 2.0);
        ctx.line_to(3.0, 21.0);
        ctx.line_to(21.0, 21.0);
        ctx.close_path();
        ctx.move_to(12.0, 9.0);
        ctx.line_to(16.0, 21.0);
        ctx.line_to(8.0, 21.0);
        ctx.close_path();
        let (r, g, b) = color.as_f64();
        ctx.set_source_rgb(r, g, b);
        ctx.fill().ok();
        ctx.restore().ok();
    });
    a
}

pub struct Espacios {
    surface: EdgeSurface,
    puntos: gtk::Box,
    connector: String,
    pub handle: Handle,
}

impl Espacios {
    pub fn new(
        app: &gtk::Application,
        palette: &Palette,
        monitor: &gtk::gdk::Monitor,
        connector: &str,
    ) -> Self {
        let m = hfs_ui::medidas::Medidas::para(&monitor.geometry());

        let mut spec = Spec::new("hfs-espacios", Side::Left, m.rail, ALTO_INICIAL);
        spec.place = Place::Center(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("rail");

        // --- el icono de arch ---
        let arch = gtk::Button::new();
        arch.add_css_class("arch");
        arch.set_has_frame(false);
        arch.set_halign(gtk::Align::Center);
        arch.set_child(Some(&arch_icon(palette.accent, ARCH_PX)));
        {
            // La ficha del sistema. Es una ventana normal y no una superficie: se
            // consulta de un vistazo y se cierra, asi que se puede mover y no pide el
            // teclado. La `windowrule` de `dev.hfs.shell` la deja flotante y centrada.
            let (app, palette) = (app.clone(), palette.clone());
            arch.connect_clicked(move |_| crate::sistema::abrir(&app, &palette));
        }

        let sep = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sep.add_css_class("rail-sep");

        let puntos = gtk::Box::new(gtk::Orientation::Vertical, 0);
        puntos.add_css_class("ws");

        surface.body.append(&arch);
        surface.body.append(&sep);
        surface.body.append(&puntos);

        // La rueda cambia de espacio, y `e+1` crea el siguiente si no existe: es la
        // unica forma de llegar a un espacio que todavia no esta en el rail.
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.connect_scroll(|_, _, dy| {
            let destino = if dy > 0.0 { "e+1" } else { "e-1" };
            if let Err(e) = hyprland::dispatch(&format!("workspace {destino}")) {
                eprintln!("hfs: {e}");
            }
            glib::Propagation::Stop
        });
        surface.body.add_controller(scroll);

        Self {
            surface,
            puntos,
            connector: connector.to_string(),
            handle,
        }
    }

    /// Rehace los puntos con el estado nuevo y vuelve a medir el panel.
    ///
    /// Se tiran y se vuelven a crear en vez de reciclarlos: son tres o cuatro
    /// botones, y asi no hay forma de que se quede uno con el estado de otro espacio.
    pub fn update(&self, state: &State) {
        while let Some(hijo) = self.puntos.first_child() {
            self.puntos.remove(&hijo);
        }

        let activo = state.active_on(&self.connector);
        for ws in state.on_monitor(&self.connector) {
            let b = gtk::Button::new();
            b.add_css_class("ws-dot");
            b.set_has_frame(false);
            b.set_halign(gtk::Align::Center);
            if ws.windows > 0 {
                b.add_css_class("has");
            }
            if Some(ws.id) == activo {
                b.add_css_class("on");
            }
            // Sin tooltip a proposito: GTK los dibuja en una ventana aparte y sobre
            // una superficie de layer-shell no llegan a aparecer (probado: cuatro
            // segundos encima de un punto y nada). Los puntos van en orden de id, y
            // los numeros los tienes en SUPER+1..9.
            let id = ws.id;
            b.connect_clicked(move |_| {
                if let Err(e) = hyprland::dispatch(&format!("workspace {id}")) {
                    eprintln!("hfs: {e}");
                }
            });
            self.puntos.append(&b);
        }

        self.surface.fit_length();
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }

    pub fn present(&self) {
        self.surface.present();
    }
}
