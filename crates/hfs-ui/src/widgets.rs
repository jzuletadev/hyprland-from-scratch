// ~/hyprland-from-scratch/crates/hfs-ui/src/widgets.rs
//
// Las piezas que se repiten en varias superficies: el anillo, la barra, la cabecera
// de panel, el boton con icono y la baldosa.
//
// Existe por lo que paso en la v1: el marco de ventana estaba copiado byte a byte en
// 11 de 11 ficheros, `safe_addstr` once veces, y habia tres formulas distintas de
// centrar el pie. Aqui cada una vive una vez, y el que toque el color de un anillo lo
// cambia en todos.

use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

use gtk4::prelude::*;

use hfs_core::{Palette, Rgb};

/// El color que le toca a un porcentaje: tranquilo, atento, mal.
pub fn umbral(p: &Palette, pct: u8) -> Rgb {
    match pct {
        0..=59 => p.ok,
        60..=84 => p.warn,
        _ => p.bad,
    }
}

// ---------------------------------------------------------------- anillo

/// Un donut con un numero dentro. Es como se enseña un porcentaje en todo el
/// escritorio: ocupa poco, se lee de lejos y no miente sobre la escala.
#[derive(Clone)]
pub struct Ring {
    pub root: gtk4::Overlay,
    area: gtk4::DrawingArea,
    label: gtk4::Label,
    valor: Rc<Cell<f64>>,
    color: Rc<Cell<(f64, f64, f64)>>,
}

impl Ring {
    pub fn new(palette: &Palette, lado: i32) -> Self {
        let valor = Rc::new(Cell::new(0.0f64));
        let color = Rc::new(Cell::new(palette.accent.as_f64()));

        let area = gtk4::DrawingArea::new();
        area.set_content_width(lado);
        area.set_content_height(lado);
        {
            let (valor, color) = (valor.clone(), color.clone());
            let pista = palette.fg;
            area.set_draw_func(move |_, ctx, w, h| {
                let (w, h) = (w as f64, h as f64);
                let grosor = (w.min(h) / 8.0).max(2.0);
                let r = (w.min(h) - grosor) / 2.0;
                let (cx, cy) = (w / 2.0, h / 2.0);

                let (pr, pg, pb) = pista.as_f64();
                ctx.set_line_width(grosor);
                ctx.set_source_rgba(pr, pg, pb, 0.14);
                ctx.arc(cx, cy, r, 0.0, 2.0 * PI);
                ctx.stroke().ok();

                let v = valor.get().clamp(0.0, 1.0);
                if v <= 0.0 {
                    return;
                }
                let (cr, cg, cb) = color.get();
                ctx.set_source_rgb(cr, cg, cb);
                ctx.set_line_cap(gtk4::cairo::LineCap::Round);
                // Desde arriba y en el sentido de las agujas del reloj.
                ctx.arc(cx, cy, r, -PI / 2.0, -PI / 2.0 + 2.0 * PI * v);
                ctx.stroke().ok();
            });
        }

        let label = gtk4::Label::new(None);
        label.add_css_class("ring-val");
        label.set_halign(gtk4::Align::Center);
        label.set_valign(gtk4::Align::Center);

        let root = gtk4::Overlay::new();
        root.set_child(Some(&area));
        root.add_overlay(&label);
        root.set_halign(gtk4::Align::Center);
        root.set_valign(gtk4::Align::Center);

        Self {
            root,
            area,
            label,
            valor,
            color,
        }
    }

    pub fn set(&self, pct: u8, texto: &str, color: Rgb) {
        self.valor.set(pct as f64 / 100.0);
        self.color.set(color.as_f64());
        self.label.set_text(texto);
        self.area.queue_draw();
    }
}

// ---------------------------------------------------------------- barra

/// Una barra fina. Para lo que no merece un anillo: el progreso de una cancion, la
/// carga de la bateria dentro de una baldosa.
#[derive(Clone)]
pub struct Bar {
    pub root: gtk4::DrawingArea,
    valor: Rc<Cell<f64>>,
    color: Rc<Cell<(f64, f64, f64)>>,
}

impl Bar {
    pub fn new(palette: &Palette, alto: i32) -> Self {
        let valor = Rc::new(Cell::new(0.0f64));
        let color = Rc::new(Cell::new(palette.accent.as_f64()));
        let root = gtk4::DrawingArea::new();
        root.set_content_height(alto);
        root.set_hexpand(true);
        root.set_valign(gtk4::Align::Center);
        {
            let (valor, color) = (valor.clone(), color.clone());
            let pista = palette.fg;
            root.set_draw_func(move |_, ctx, w, h| {
                let (w, h) = (w as f64, h as f64);
                let r = h / 2.0;
                let (pr, pg, pb) = pista.as_f64();
                redondeado(ctx, 0.0, 0.0, w, h, r);
                ctx.set_source_rgba(pr, pg, pb, 0.13);
                ctx.fill().ok();

                let v = valor.get().clamp(0.0, 1.0);
                if v <= 0.0 {
                    return;
                }
                // Nunca mas corta que su propio alto: un relleno de dos pixeles no se
                // lee como «poco», se lee como un fallo de dibujo.
                let ancho = (w * v).max(h);
                let (cr, cg, cb) = color.get();
                redondeado(ctx, 0.0, 0.0, ancho, h, r);
                ctx.set_source_rgb(cr, cg, cb);
                ctx.fill().ok();
            });
        }
        Self { root, valor, color }
    }

    pub fn set(&self, frac: f64, color: Rgb) {
        self.valor.set(frac);
        self.color.set(color.as_f64());
        self.root.queue_draw();
    }
}

/// Un rectangulo con las cuatro esquinas redondeadas.
pub fn redondeado(ctx: &gtk4::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::FRAC_PI_2;
    let r = r.min(w / 2.0).min(h / 2.0);
    ctx.new_path();
    ctx.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    ctx.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    ctx.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    ctx.arc(x + r, y + r, r, PI, 1.5 * PI);
    ctx.close_path();
}

// ---------------------------------------------------------------- piezas sueltas

/// La cabecera de un panel: titulo a la izquierda y un dato suelto a la derecha.
/// Devuelve la caja y la etiqueta de la derecha, que es la que cambia.
pub fn header(titulo: &str) -> (gtk4::Box, gtk4::Label) {
    let caja = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
    caja.add_css_class("ph");
    let t = gtk4::Label::new(Some(titulo));
    t.set_xalign(0.0);
    let extra = gtk4::Label::new(None);
    extra.add_css_class("sp");
    extra.set_xalign(1.0);
    extra.set_hexpand(true);
    extra.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    caja.append(&t);
    caja.append(&extra);
    (caja, extra)
}

/// Un boton con icono encima y texto debajo, como los de Energia.
pub fn key(icono: &str, texto: &str) -> gtk4::Button {
    let b = gtk4::Button::new();
    b.add_css_class("key");
    b.set_has_frame(false);
    let caja = gtk4::Box::new(gtk4::Orientation::Vertical, 5);
    caja.set_halign(gtk4::Align::Center);
    let img = gtk4::Image::from_icon_name(icono);
    img.set_pixel_size(18);
    let l = gtk4::Label::new(Some(texto));
    l.add_css_class("key-tag");
    caja.append(&img);
    caja.append(&l);
    b.set_child(Some(&caja));
    b
}

/// Una baldosa: el recuadro con fondo propio que agrupa un dato y su contexto.
pub fn tile() -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Vertical, 3);
    b.add_css_class("tile");
    b
}

/// Una fila de lista: icono, texto que crece, y un dato a la derecha.
pub fn row() -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
    b.add_css_class("row");
    b
}

pub fn sep() -> gtk4::Box {
    let b = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    b.add_css_class("dock-sep");
    b
}

/// Una etiqueta que no empuja el panel a lo ancho por mucho que crezca su texto.
pub fn ellipsized(texto: Option<&str>, clase: &str) -> gtk4::Label {
    let l = gtk4::Label::new(texto);
    if !clase.is_empty() {
        l.add_css_class(clase);
    }
    l.set_xalign(0.0);
    l.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    // `ellipsize` NO basta. Elidir es lo que se hace al PINTAR; al MEDIR, la etiqueta
    // sigue pidiendo el ancho natural de su texto entero. Sin este tope, una
    // notificacion con una ruta larga o una entrada del portapapeles con un JSON
    // dentro ensanchaban su panel y rompian la medida comun del escritorio.
    l.set_max_width_chars(1);
    l
}

/// Vacia un contenedor. Rehacer una lista entera sale mas barato en lineas y en
/// fallos que reciclar sus hijos, y son siempre listas cortas.
pub fn clear(caja: &impl IsA<gtk4::Widget>) {
    let caja = caja.as_ref();
    while let Some(h) = caja.first_child() {
        if let Some(b) = caja.downcast_ref::<gtk4::Box>() {
            b.remove(&h);
        } else {
            h.unparent();
        }
    }
}

/// Un contador de cosas pendientes, para el aviso ambiental.
pub type Contador = Rc<RefCell<usize>>;
