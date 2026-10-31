// ~/hyprland-from-scratch/crates/hfs/src/ctrl.rs
//
// El dock de controles del canto derecho: volumen arriba, brillo abajo.
//
// Fue la primera superficie del escritorio v2 a proposito. Es la que mas se usa, la
// unica que no depende de D-Bus, y la que obligo a resolver de una vez las tres cosas
// que todas las demas reutilizan: la superficie de layer-shell sin zona exclusiva, la
// forma que nace del borde y el CSS que sale de la paleta.
//
// Apilados en vertical y no en paralelo: el pulgar sube y baja por el mismo canto en
// vez de cruzar de una columna a otra, y el dock queda estrecho.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::{audio, backlight::Backlight, Palette, Rgb};
use hfs_ui::{EdgeSurface, Handle, Side, Spec};


// Proporciones sacadas de la maqueta (design/), en tanto por ciento de la pantalla.
const ALTO_PCT: f64 = 0.27;
const PISTA_PX: i32 = 22;
const PASO_RUEDA: i16 = 5;
/// Cada cuanto se relee el valor MIENTRAS el dock esta abierto. Cerrado no se
/// pregunta nada: un panel que no se ve no debe costar CPU. Es justo el error del
/// waybar-autohide de la v1, que gasta un 1,3 % constante.
const SONDEO_MS: u64 = 600;

pub struct Ctrl {
    surface: EdgeSurface,
    pub handle: Handle,
    refresh: Rc<dyn Fn()>,
}

impl Ctrl {
    /// Vuelve a leer volumen y brillo. Lo llama el aviso de las teclas multimedia.
    pub fn refresh(&self) {
        (self.refresh)();
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }

    pub fn present(&self) {
        self.surface.present();
    }

    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);
        let mut spec = Spec::new(
            "hfs-ctrl",
            Side::Right,
            m.rail,
            (g.height() as f64 * ALTO_PCT).round() as i32,
        );
        // Proporcion sacada de la maqueta: la muesca es poco mas de un cuarto del
        // ancho del dock. Mas pequeña no se lee como que el panel nace del canto.
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();

        // El brillo se detecta una vez: el dispositivo no cambia en caliente.
        let backlight = match Backlight::detect() {
            Ok(b) => Some(b),
            Err(e) => {
                eprintln!("hfs: sin control de brillo ({e})");
                None
            }
        };

        let vol = Meter::new(palette, Kind::Volume);
        let bri = Meter::new(palette, Kind::Brightness);

        // --- lo que hace cada gesto ---
        {
            let m = vol.clone();
            vol.on_set(move |v| {
                if let Err(e) = audio::set(v) {
                    eprintln!("hfs: {e}");
                }
                m.refresh_audio();
            });
            let m = vol.clone();
            vol.on_secondary(move || match audio::toggle_mute() {
                Ok(v) => m.apply(v.percent, v.muted),
                Err(e) => eprintln!("hfs: {e}"),
            });
        }
        if let Some(bl) = backlight.clone() {
            let m = bri.clone();
            bri.on_set(move |v| {
                if let Err(e) = bl.set(v) {
                    eprintln!("hfs: {e}");
                }
                if let Ok(p) = bl.percent() {
                    m.apply(p, false);
                }
            });
        } else {
            bri.root.set_sensitive(false);
        }

        let sep = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        sep.add_css_class("dock-sep");

        surface.body.append(&vol.root);
        surface.body.append(&sep);
        surface.body.append(&bri.root);

        // --- refrescar ---
        let refresh: Rc<dyn Fn()> = {
            let vol = vol.clone();
            let bri = bri.clone();
            let backlight = backlight.clone();
            Rc::new(move || {
                vol.refresh_audio();
                if let Some(bl) = &backlight {
                    match bl.percent() {
                        Ok(p) => bri.apply(p, false),
                        Err(e) => eprintln!("hfs: {e}"),
                    }
                }
            })
        };
        refresh();

        // Sondear solo mientras se ve.
        {
            let refresh = refresh.clone();
            let timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
            handle.connect_toggled(move |abierto| {
                let mut t = timer.borrow_mut();
                if let Some(id) = t.take() {
                    id.remove();
                }
                if abierto {
                    refresh();
                    let refresh = refresh.clone();
                    *t = Some(glib::timeout_add_local(
                        std::time::Duration::from_millis(SONDEO_MS),
                        move || {
                            refresh();
                            glib::ControlFlow::Continue
                        },
                    ));
                }
            });
        }

        Self {
            surface,
            handle,
            refresh,
        }
    }
}

// ---------------------------------------------------------------- el medidor

/// Que hacer cuando el usuario mueve el medidor, y que hacer con el boton derecho.
/// Van en Rc<RefCell<..>> porque se asignan despues de construir el medidor: quien
/// lo construye no sabe todavia que hay detras.
type SetCb = Rc<RefCell<Option<Rc<dyn Fn(u8)>>>>;
type ActionCb = Rc<RefCell<Option<Rc<dyn Fn()>>>>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Volume,
    Brightness,
}

/// Un medidor vertical: numero arriba, pista larga, icono dentro de la pista abajo.
///
/// El icono va DENTRO y no encima en un boton porque asi el medidor es todo pista y
/// queda alargado. Un boton con una pista corta debajo se ve como una gota.
#[derive(Clone)]
struct Meter {
    root: gtk::Box,
    area: gtk::DrawingArea,
    icon: gtk::Image,
    label: gtk::Label,
    kind: Kind,
    value: Rc<Cell<u8>>,
    muted: Rc<Cell<bool>>,
    on_set: SetCb,
    on_secondary: ActionCb,
}

impl Meter {
    fn new(palette: &Palette, kind: Kind) -> Self {
        let value = Rc::new(Cell::new(0u8));
        let muted = Rc::new(Cell::new(false));

        let label = gtk::Label::new(Some("--"));
        label.add_css_class("meter-value");

        let area = gtk::DrawingArea::new();
        area.set_size_request(PISTA_PX, -1);
        area.set_vexpand(true);
        area.set_halign(gtk::Align::Center);
        {
            let (value, muted) = (value.clone(), muted.clone());
            let track = palette.fg;
            let accent = palette.accent;
            let bad = palette.bad;
            area.set_draw_func(move |_, ctx, w, h| {
                let (w, h) = (w as f64, h as f64);
                let r = w / 2.0;
                let (tr, tg, tb) = track.as_f64();
                rounded(ctx, 0.0, 0.0, w, h, r);
                ctx.set_source_rgba(tr, tg, tb, 0.13);
                ctx.fill().ok();

                let v = value.get() as f64 / 100.0;
                if v <= 0.0 {
                    return;
                }
                // Nunca mas corto que su propio ancho: un relleno de 2 px no se lee
                // como «bajo», se lee como un fallo de dibujo.
                let fh = (h * v).max(w);
                ctx.save().ok();
                rounded(ctx, 0.0, 0.0, w, h, r);
                ctx.clip();
                rounded(ctx, 0.0, h - fh, w, fh, r);
                let c: Rgb = if muted.get() { bad } else { accent };
                let (cr, cg, cb) = c.as_f64();
                ctx.set_source_rgb(cr, cg, cb);
                ctx.fill().ok();
                ctx.restore().ok();
            });
        }

        let icon = gtk::Image::from_icon_name(icon_name(kind, 0, false));
        icon.add_css_class("meter-icon");
        icon.set_pixel_size(14);
        icon.set_halign(gtk::Align::Center);
        icon.set_valign(gtk::Align::End);
        // Que no se coma los clics: la pista es quien los recibe.
        icon.set_can_target(false);

        let overlay = gtk::Overlay::new();
        overlay.set_child(Some(&area));
        overlay.add_overlay(&icon);
        overlay.set_vexpand(true);

        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("meter");
        root.set_vexpand(true);
        root.append(&label);
        root.append(&overlay);

        let me = Self {
            root,
            area,
            icon,
            label,
            kind,
            value,
            muted,
            on_set: Rc::new(RefCell::new(None)),
            on_secondary: Rc::new(RefCell::new(None)),
        };
        me.wire_gestures();
        me
    }

    fn on_set(&self, f: impl Fn(u8) + 'static) {
        *self.on_set.borrow_mut() = Some(Rc::new(f));
    }

    fn on_secondary(&self, f: impl Fn() + 'static) {
        *self.on_secondary.borrow_mut() = Some(Rc::new(f));
    }

    fn fire_set(&self, v: u8) {
        let cb = self.on_set.borrow().clone();
        if let Some(cb) = cb {
            cb(v);
        }
    }

    fn wire_gestures(&self) {
        // Arrastrar y pulsar son el mismo gesto: `drag_begin` ya salta al pulsar.
        let inicio = Rc::new(Cell::new(0.0f64));
        let drag = gtk::GestureDrag::new();
        {
            let (me, inicio) = (self.clone(), inicio.clone());
            drag.connect_drag_begin(move |_, _, y| {
                inicio.set(y);
                me.fire_set(me.value_at(y));
            });
        }
        {
            let (me, inicio) = (self.clone(), inicio.clone());
            drag.connect_drag_update(move |_, _, dy| {
                me.fire_set(me.value_at(inicio.get() + dy));
            });
        }
        self.area.add_controller(drag);

        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        {
            let me = self.clone();
            scroll.connect_scroll(move |_, _, dy| {
                let paso = if dy > 0.0 { -PASO_RUEDA } else { PASO_RUEDA };
                let v = (me.value.get() as i16 + paso).clamp(0, 100) as u8;
                me.fire_set(v);
                glib::Propagation::Stop
            });
        }
        self.area.add_controller(scroll);

        // Boton derecho: silenciar. Solo el volumen lo usa.
        let click = gtk::GestureClick::new();
        click.set_button(gtk::gdk::BUTTON_SECONDARY);
        {
            let me = self.clone();
            click.connect_pressed(move |_, _, _, _| {
                let cb = me.on_secondary.borrow().clone();
                if let Some(cb) = cb {
                    cb();
                }
            });
        }
        self.area.add_controller(click);
    }

    /// De una coordenada vertical al valor. Arriba es 100.
    fn value_at(&self, y: f64) -> u8 {
        let h = self.area.height() as f64;
        if h <= 0.0 {
            return self.value.get();
        }
        (((1.0 - y / h) * 100.0).round()).clamp(0.0, 100.0) as u8
    }

    fn refresh_audio(&self) {
        match audio::get() {
            Ok(v) => self.apply(v.percent, v.muted),
            Err(e) => {
                eprintln!("hfs: {e}");
                self.label.set_text("--");
            }
        }
    }

    fn apply(&self, percent: u8, muted: bool) {
        self.value.set(percent);
        self.muted.set(muted);
        self.label.set_text(&percent.to_string());
        self.icon
            .set_icon_name(Some(icon_name(self.kind, percent, muted)));

        // El icono vive al fondo de la pista. Si el relleno no llega hasta el, queda
        // sobre el fondo oscuro y hay que devolverle el color del texto.
        let h = self.area.height() as f64;
        let alcanza = h > 0.0 && (h * percent as f64 / 100.0) >= 26.0;
        self.icon.remove_css_class("on-track");
        self.icon.remove_css_class("muted");
        if muted {
            self.icon.add_css_class("muted");
        } else if !alcanza {
            self.icon.add_css_class("on-track");
        }
        self.area.queue_draw();
    }
}

fn icon_name(kind: Kind, percent: u8, muted: bool) -> &'static str {
    match kind {
        Kind::Brightness => "display-brightness-symbolic",
        Kind::Volume if muted || percent == 0 => "audio-volume-muted-symbolic",
        Kind::Volume if percent < 34 => "audio-volume-low-symbolic",
        Kind::Volume if percent < 67 => "audio-volume-medium-symbolic",
        Kind::Volume => "audio-volume-high-symbolic",
    }
}

/// Rectangulo con las cuatro esquinas redondeadas.
fn rounded(ctx: &gtk::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::{FRAC_PI_2, PI};
    let r = r.min(w / 2.0).min(h / 2.0);
    ctx.new_path();
    ctx.arc(x + w - r, y + r, r, -FRAC_PI_2, 0.0);
    ctx.arc(x + w - r, y + h - r, r, 0.0, FRAC_PI_2);
    ctx.arc(x + r, y + h - r, r, FRAC_PI_2, PI);
    ctx.arc(x + r, y + r, r, PI, 1.5 * PI);
    ctx.close_path();
}
