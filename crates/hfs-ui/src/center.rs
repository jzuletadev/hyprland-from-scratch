// ~/hyprland-from-scratch/crates/hfs-ui/src/center.rs
//
// Una superficie que ocupa el monitor entero y se abre en el centro: la rueda.
//
// No es un panel de borde y no comparte casi nada con ellos. Tres diferencias:
//
//   TECLADO. Es la primera superficie del escritorio que lo necesita, y por eso pide
//   `KeyboardMode::Exclusive`. Los atajos de Hyprland siguen funcionando igualmente
//   —el compositor los resuelve antes de reenviar la tecla—, asi que volver a tocar
//   SUPER cierra la rueda aunque el teclado sea suyo. Lo que NO se puede es dejarla
//   mapeada y escondida: con el teclado en exclusiva se tragaria todo lo que
//   escribieras en cualquier sitio.
//
//   SE MAPEA Y SE DESMAPEA. Por eso mismo. Un panel de borde vive mapeado siempre y
//   se esconde con la region de entrada; esta se oculta de verdad (`set_visible`), y
//   asi mientras no se ve no recibe ni raton ni teclas. De paso se ahorra el truco
//   del rectangulo transparente, que solo hace falta cuando la superficie tiene que
//   mantener su tamaño estando plegada.
//
//   ZONA EXCLUSIVA -1. Los paneles de borde van a 0 para no reservar sitio. Aqui es
//   -1, que significa «no me recortes por las zonas que hayan reservado otros»: sin
//   eso, la barra de la v1 le quitaria su franja al velo y la rueda no saldria
//   centrada en la pantalla, sino en lo que sobra.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use hfs_core::Palette;

/// Lo que tarda el velo en aparecer y en irse.
const FUNDIDO_MS: u32 = 170;

pub struct CenterSurface {
    pub window: gtk4::ApplicationWindow,
    /// Donde va el contenido. Ya viene centrado en la pantalla.
    pub body: gtk4::Box,
    revealer: gtk4::Revealer,
    open: Rc<Cell<bool>>,
}

impl CenterSurface {
    pub fn new(app: &gtk4::Application, namespace: &str, palette: &Palette) -> Self {
        let window = gtk4::ApplicationWindow::new(app);
        window.add_css_class("hfs");
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace(Some(namespace));
        for e in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            window.set_anchor(e, true);
        }
        window.set_exclusive_zone(-1);
        window.set_keyboard_mode(KeyboardMode::Exclusive);

        // El velo. Se dibuja con Cairo y no con CSS porque el color sale de la
        // paleta: un velo gris fijo sobre un tema claro se ve como suciedad.
        let scrim = gtk4::DrawingArea::new();
        {
            let (r, g, b) = palette.bg.as_f64();
            scrim.set_draw_func(move |_, ctx, w, h| {
                ctx.set_source_rgba(r, g, b, 0.45);
                ctx.rectangle(0.0, 0.0, w as f64, h as f64);
                ctx.fill().ok();
            });
        }

        let body = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        body.set_halign(gtk4::Align::Center);
        body.set_valign(gtk4::Align::Center);

        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&scrim));
        overlay.add_overlay(&body);

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(gtk4::RevealerTransitionType::Crossfade);
        revealer.set_transition_duration(FUNDIDO_MS);
        revealer.set_child(Some(&overlay));
        revealer.set_reveal_child(false);
        revealer.set_hexpand(true);
        revealer.set_vexpand(true);
        window.set_child(Some(&revealer));

        let me = Self {
            window,
            body,
            revealer,
            open: Rc::new(Cell::new(false)),
        };

        // Tocar fuera cierra.
        let click = gtk4::GestureClick::new();
        {
            let this = me.handle();
            click.connect_released(move |_, _, _, _| this.close());
        }
        scrim.add_controller(click);

        me
    }

    pub fn set_monitor(&self, monitor: &gtk4::gdk::Monitor) {
        self.window.set_monitor(Some(monitor));
    }

    pub fn handle(&self) -> CenterHandle {
        CenterHandle {
            window: self.window.clone(),
            revealer: self.revealer.clone(),
            open: self.open.clone(),
        }
    }

    /// Las teclas, antes de que las vea nadie mas. En fase de captura a proposito:
    /// con el buscador abierto hay un GtkEntry dentro que se quedaria con todo, y
    /// Escape o las flechas no llegarian nunca.
    pub fn connect_keys<F>(&self, f: F)
    where
        F: Fn(gtk4::gdk::Key, gtk4::gdk::ModifierType) -> glib::Propagation + 'static,
    {
        let key = gtk4::EventControllerKey::new();
        key.set_propagation_phase(gtk4::PropagationPhase::Capture);
        key.connect_key_pressed(move |_, keyval, _, state| f(keyval, state));
        self.window.add_controller(key);
    }

    pub fn destroy(&self) {
        self.window.destroy();
    }
}

#[derive(Clone)]
pub struct CenterHandle {
    window: gtk4::ApplicationWindow,
    revealer: gtk4::Revealer,
    open: Rc<Cell<bool>>,
}

impl CenterHandle {
    pub fn is_open(&self) -> bool {
        self.open.get()
    }

    pub fn open(&self) {
        if self.open.replace(true) {
            return;
        }
        self.window.present();
        // El fundido se lanza en el siguiente ciclo: si se pide en el mismo, GTK ya
        // ha compuesto el primer fotograma con el velo entero y no hay transicion.
        let revealer = self.revealer.clone();
        glib::idle_add_local_once(move || revealer.set_reveal_child(true));
    }

    pub fn close(&self) {
        if !self.open.replace(false) {
            return;
        }
        self.revealer.set_reveal_child(false);
        // Desmapear solo cuando el fundido ha terminado; si no, desaparece de golpe.
        let (window, open) = (self.window.clone(), self.open.clone());
        glib::timeout_add_local_once(Duration::from_millis(FUNDIDO_MS as u64 + 20), move || {
            if !open.get() {
                window.set_visible(false);
            }
        });
    }

    pub fn toggle(&self) {
        if self.is_open() {
            self.close();
        } else {
            self.open();
        }
    }
}
