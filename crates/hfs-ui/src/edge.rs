// ~/hyprland-from-scratch/crates/hfs-ui/src/edge.rs
//
// Una superficie de borde: la ventana de layer-shell, la forma con sus muescas y la
// logica de aparecer al rozar el canto.
//
// Dos decisiones que son el escritorio entero:
//
//   zona exclusiva = 0  ->  el compositor NO reserva sitio, asi que nada de lo que
//                           hay debajo se encoge. Es la diferencia entre esto y una
//                           barra. Waybar reserva; esto cae encima.
//
//   region de entrada   ->  la superficie mide lo que mide el panel abierto, pero
//                           mientras esta cerrada solo recibe raton una franja de
//                           unos pocos pixeles pegada al canto. Por eso el escritorio
//                           no lleva ninguna marca y aun asi se puede invocar.
//
// Al entrar en la franja se ensancha la region a toda la superficie ANTES de empezar
// la espera. Si no, mover el raton hacia dentro saca el puntero de la franja, el
// compositor manda un «leave» y el panel no llegaria a abrirse nunca.
//
// Un panel puede nacer de un canto (centrado, `Place::Center`) o de una ESQUINA
// (`Place::Start` / `Place::End`), y entonces se ancla a los dos cantos a la vez. Lo
// que cambia no es solo donde cae: el extremo que se pega a la esquina pierde su
// muesca y su radio, porque ahi ya no hay pantalla de la que nacer.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::cairo;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use hfs_core::Palette;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    fn edge(self) -> Edge {
        match self {
            Side::Top => Edge::Top,
            Side::Bottom => Edge::Bottom,
            Side::Left => Edge::Left,
            Side::Right => Edge::Right,
        }
    }
    fn vertical_edge(self) -> bool {
        matches!(self, Side::Left | Side::Right)
    }
    fn transition(self) -> gtk4::RevealerTransitionType {
        match self {
            Side::Right => gtk4::RevealerTransitionType::SlideLeft,
            Side::Left => gtk4::RevealerTransitionType::SlideRight,
            Side::Top => gtk4::RevealerTransitionType::SlideDown,
            Side::Bottom => gtk4::RevealerTransitionType::SlideUp,
        }
    }
}

/// Donde cae el panel a lo largo de su canto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// Centrado sobre el canto. El numero lo corre desde el centro.
    Center(i32),
    /// Anclado a la esquina donde el canto empieza: arriba en los paneles
    /// verticales, izquierda en los horizontales. El numero lo separa del vertice;
    /// con 0 nace de el, que es lo normal en este escritorio.
    Start(i32),
    /// Lo mismo en la esquina donde el canto acaba.
    End(i32),
}

#[derive(Debug, Clone)]
pub struct Spec {
    pub namespace: String,
    pub side: Side,
    /// Tamaño del cuerpo del panel, sin contar las muescas.
    pub width: i32,
    pub height: i32,
    /// Radio interior y lado de la muesca concava.
    pub radius: f64,
    pub notch: f64,
    /// Grosor de la franja que recibe el raton con el panel cerrado.
    pub hover: i32,
    /// Donde cae sobre el canto, y si es una esquina.
    pub place: Place,
    /// Cuanto hay que posarse en el canto antes de que se abra, y cuanto se espera
    /// antes de cerrarlo al salir.
    pub dwell_ms: u32,
    pub grace_ms: u32,
    pub anim_ms: u32,
    /// Opacidad del fondo del panel.
    pub alpha: f64,
}

impl Spec {
    pub fn new(namespace: &str, side: Side, width: i32, height: i32) -> Self {
        Self {
            namespace: namespace.to_string(),
            side,
            width,
            height,
            radius: 14.0,
            notch: 12.0,
            hover: 6,
            place: Place::Center(0),
            dwell_ms: 140,
            grace_ms: 260,
            anim_ms: 240,
            alpha: 0.93,
        }
    }

    /// Que extremos llevan muesca, en el orden de la pantalla (arriba/izquierda
    /// primero).
    ///
    /// Un panel pegado al vertice deja sin muesca el extremo que se le pega: no hay
    /// nada de lo que separarse. Si se le diera separacion dejaria de tocar la
    /// esquina y llevaria las dos, igual que uno centrado.
    fn notches(&self) -> crate::shape::Notches {
        match self.place {
            Place::Center(_) => (true, true),
            Place::Start(0) => (false, true),
            Place::End(0) => (true, false),
            Place::Start(_) | Place::End(_) => (true, true),
        }
    }

    /// Tamaño de la superficie: el cuerpo mas el sitio de las muescas que haya.
    fn surface_size(&self) -> (i32, i32) {
        let n = self.notch.ceil() as i32;
        let (a, b) = self.notches();
        let extra = n * (a as i32 + b as i32);
        if self.side.vertical_edge() {
            (self.width, self.height + extra)
        } else {
            (self.width + extra, self.height)
        }
    }

    /// Los margenes que dejan libre el sitio de cada muesca.
    fn insets(&self) -> (i32, i32) {
        let n = self.notch.ceil() as i32;
        let (a, b) = self.notches();
        (if a { n } else { 0 }, if b { n } else { 0 })
    }
}

struct Timers {
    dwell: Option<glib::SourceId>,
    grace: Option<glib::SourceId>,
    peek: Option<glib::SourceId>,
}

impl Timers {
    fn cancel(slot: &mut Option<glib::SourceId>) {
        if let Some(id) = slot.take() {
            id.remove();
        }
    }
}

pub struct EdgeSurface {
    pub window: gtk4::ApplicationWindow,
    /// Donde se mete el contenido del panel.
    pub body: gtk4::Box,
    revealer: gtk4::Revealer,
    /// Las piezas que reservan el tamaño; cambiarlo es cambiarselo a las tres.
    spacer: gtk4::DrawingArea,
    root: gtk4::Overlay,
    overlay: gtk4::Overlay,
    spec: Spec,
    size: Rc<Cell<(i32, i32)>>,
    /// El color de la marca del canto, o `None` si no hay nada que avisar.
    marca: Rc<Cell<Option<(f64, f64, f64)>>>,
    timers: Rc<RefCell<Timers>>,
    open: Rc<RefCell<bool>>,
    pointer_in: Rc<RefCell<bool>>,
}

impl EdgeSurface {
    pub fn new(app: &gtk4::Application, spec: Spec, palette: &Palette) -> Self {
        let (sw, sh) = spec.surface_size();

        let window = gtk4::ApplicationWindow::new(app);
        window.add_css_class("hfs");
        // Sin esto layer-shell no sabe que tamaño pedirle al compositor y la
        // superficie sale con el 200x200 por defecto de GtkWindow: el tamaño natural
        // del hijo no le llega porque con el panel plegado el revealer mide cero.
        window.set_default_size(sw, sh);
        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_namespace(Some(spec.namespace.as_str()));
        window.set_anchor(spec.side.edge(), true);
        // Lo que hace que esto no sea una barra.
        window.set_exclusive_zone(0);
        window.set_keyboard_mode(KeyboardMode::None);
        // Sin anclar nada mas, layer-shell centra la superficie sobre el canto. Para
        // una esquina se ancla tambien el canto perpendicular: eso, y no un margen,
        // es lo que hace que el panel nazca de la esquina y siga ahi a cualquier
        // resolucion.
        let (inicio, fin) = if spec.side.vertical_edge() {
            (Edge::Top, Edge::Bottom)
        } else {
            (Edge::Left, Edge::Right)
        };
        match spec.place {
            Place::Center(0) => {}
            Place::Center(off) => window.set_margin(inicio, off),
            Place::Start(sep) => {
                window.set_anchor(inicio, true);
                window.set_margin(inicio, sep);
            }
            Place::End(sep) => {
                window.set_anchor(fin, true);
                window.set_margin(fin, sep);
            }
        }

        // El fondo: la forma de una pieza, con sus muescas.
        let bg = gtk4::DrawingArea::new();
        bg.add_css_class("surface-bg");
        {
            let (side, radius, notch, alpha) = (spec.side, spec.radius, spec.notch, spec.alpha);
            let notches = spec.notches();
            let (r, g, b) = palette.surface.as_f64();
            let (br, bg_, bb) = palette.bg.as_f64();
            bg.set_draw_func(move |_, ctx, w, h| {
                crate::shape::path(ctx, side, w as f64, h as f64, radius, notch, notches);
                // Un borde finisimo del color del fondo despega el panel de la
                // pantalla sin dibujar una linea clara que delate el rectangulo.
                ctx.set_source_rgba(r, g, b, alpha);
                ctx.fill_preserve().ok();
                ctx.set_source_rgba(br, bg_, bb, 0.55);
                ctx.set_line_width(1.0);
                ctx.stroke().ok();
            });
        }

        let body = gtk4::Box::new(
            if spec.side.vertical_edge() {
                gtk4::Orientation::Vertical
            } else {
                gtk4::Orientation::Horizontal
            },
            0,
        );
        body.add_css_class("panel");
        let (ia, ib) = spec.insets();
        if spec.side.vertical_edge() {
            body.set_margin_top(ia);
            body.set_margin_bottom(ib);
        } else {
            body.set_margin_start(ia);
            body.set_margin_end(ib);
        }

        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&bg));
        overlay.add_overlay(&body);
        overlay.set_measure_overlay(&body, true);
        overlay.set_size_request(sw, sh);

        let revealer = gtk4::Revealer::new();
        revealer.set_transition_type(spec.side.transition());
        revealer.set_transition_duration(spec.anim_ms);
        revealer.set_child(Some(&overlay));
        revealer.set_reveal_child(false);
        match spec.side {
            Side::Right => {
                revealer.set_halign(gtk4::Align::End);
                revealer.set_valign(gtk4::Align::Fill);
            }
            Side::Left => {
                revealer.set_halign(gtk4::Align::Start);
                revealer.set_valign(gtk4::Align::Fill);
            }
            Side::Top => {
                revealer.set_valign(gtk4::Align::Start);
                revealer.set_halign(gtk4::Align::Fill);
            }
            Side::Bottom => {
                revealer.set_valign(gtk4::Align::End);
                revealer.set_halign(gtk4::Align::Fill);
            }
        }

        // La raiz reserva SIEMPRE el tamaño completo, abierto o cerrado, y ademas
        // SIEMPRE pinta algo.
        //
        // Lo segundo no es un detalle: con el panel plegado no hay nada que dibujar,
        // GTK no entrega ningun buffer y el compositor se queda con el tamaño inicial
        // (200x200 de GtkWindow). Medido: GTK decia 71x371 y Hyprland seguia diciendo
        // 200x200, asi que la franja de raton caia a 130 px del canto y el panel no se
        // abria nunca. Un rectangulo transparente de la medida justa lo arregla.
        let spacer = gtk4::DrawingArea::new();
        spacer.set_size_request(sw, sh);
        // Y es ademas el unico sitio donde se puede pintar con el panel plegado, que
        // es lo que hace posible la MARCA: un aviso minimo asomando por el canto
        // cuando hay algo que no puede esperar a que el usuario roce el borde. Los
        // cantos no llevan marca a proposito; esta es la excepcion, y solo existe
        // mientras dura el motivo.
        let marca: Rc<Cell<Option<(f64, f64, f64)>>> = Rc::new(Cell::new(None));
        {
            let (marca, side) = (marca.clone(), spec.side);
            spacer.set_draw_func(move |_, ctx, w, h| {
                ctx.set_source_rgba(0.0, 0.0, 0.0, 0.0);
                ctx.rectangle(0.0, 0.0, w as f64, h as f64);
                ctx.fill().ok();

                let Some((r, g, b)) = marca.get() else { return };
                let (w, h) = (w as f64, h as f64);
                // Una pastilla pegada al canto, larga a lo ancho del canto y fina
                // hacia dentro: se ve de reojo y no tapa nada.
                let (largo, grueso) = (34.0, 3.0);
                let (x, y, pw, ph) = match side {
                    Side::Bottom => ((w - largo) / 2.0, h - grueso, largo, grueso),
                    Side::Top => ((w - largo) / 2.0, 0.0, largo, grueso),
                    Side::Left => (0.0, (h - largo) / 2.0, grueso, largo),
                    Side::Right => (w - grueso, (h - largo) / 2.0, grueso, largo),
                };
                crate::widgets::redondeado(ctx, x, y, pw, ph, grueso / 2.0);
                ctx.set_source_rgb(r, g, b);
                ctx.fill().ok();
            });
        }

        let root = gtk4::Overlay::new();
        root.add_css_class("root");
        root.set_size_request(sw, sh);
        root.set_child(Some(&spacer));
        root.add_overlay(&revealer);
        revealer.set_hexpand(true);
        revealer.set_vexpand(true);
        window.set_child(Some(&root));

        let me = Self {
            window,
            body,
            revealer,
            spacer,
            root,
            overlay,
            size: Rc::new(Cell::new((sw, sh))),
            marca,
            spec,
            timers: Rc::new(RefCell::new(Timers {
                dwell: None,
                grace: None,
                peek: None,
            })),
            open: Rc::new(RefCell::new(false)),
            pointer_in: Rc::new(RefCell::new(false)),
        };
        me.wire_hover();
        me
    }

    fn wire_hover(&self) {
        let motion = gtk4::EventControllerMotion::new();

        {
            let this = self.weakish();
            motion.connect_enter(move |_, _, _| {
                this.on_enter();
            });
        }
        {
            let this = self.weakish();
            motion.connect_leave(move |_| {
                this.on_leave();
            });
        }
        self.window.add_controller(motion);

        // La region de entrada se aplica cuando la superficie EXISTE y, sobre todo,
        // cuando ya tiene su tamaño: al mapear mide 1x1 y la franja caeria fuera.
        // `layout` salta en cada cambio de tamaño, asi que vale para las dos cosas.
        let this = self.weakish();
        self.window.connect_map(move |w| {
            this.apply_input_region(false);
            let Some(surface) = w.surface() else { return };
            let this = this.clone();
            surface.connect_layout(move |_, _, _| {
                if !this.is_open() {
                    this.apply_input_region(false);
                }
            });
        });
    }

    /// Un clon barato con lo que hace falta para los callbacks. Todo lo compartido
    /// ya es Rc o un objeto de GTK (que es contado por referencia de por si).
    fn weakish(&self) -> Handle {
        Handle {
            window: self.window.clone(),
            revealer: self.revealer.clone(),
            body: self.body.clone(),
            spacer: self.spacer.clone(),
            root: self.root.clone(),
            overlay: self.overlay.clone(),
            spec: self.spec.clone(),
            size: self.size.clone(),
            marca: self.marca.clone(),
            timers: self.timers.clone(),
            open: self.open.clone(),
            pointer_in: self.pointer_in.clone(),
        }
    }

    pub fn handle(&self) -> Handle {
        self.weakish()
    }

    /// Atajos a los mismos metodos del Handle, para quien tenga la superficie.
    pub fn resize(&self, width: i32, height: i32) {
        self.handle().resize(width, height);
    }
    pub fn fit_length(&self) {
        self.handle().fit_length();
    }
    pub fn fit_depth(&self) {
        self.handle().fit_depth();
    }

    /// Fija en que monitor vive la superficie. Hay que llamarlo ANTES de present():
    /// una vez mapeada, el compositor ya la ha colocado.
    ///
    /// Sin esto el compositor la pone en el monitor que tenga el foco al arrancar y
    /// ahi se queda, asi que al pasar a la otra pantalla el panel no esta. Por eso el
    /// escritorio crea un juego de superficies por monitor.
    pub fn set_monitor(&self, monitor: &gtk4::gdk::Monitor) {
        self.window.set_monitor(Some(monitor));
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Cierra la superficie y la quita de la pantalla. Lo usa el reconstruir al
    /// enchufar o desenchufar un monitor.
    pub fn destroy(&self) {
        self.window.destroy();
    }
}

/// Lo mismo que EdgeSurface pero clonable, para los callbacks y para quien quiera
/// abrir el panel desde fuera (las teclas de volumen, por ejemplo).
#[derive(Clone)]
pub struct Handle {
    window: gtk4::ApplicationWindow,
    revealer: gtk4::Revealer,
    body: gtk4::Box,
    spacer: gtk4::DrawingArea,
    root: gtk4::Overlay,
    overlay: gtk4::Overlay,
    spec: Spec,
    size: Rc<Cell<(i32, i32)>>,
    marca: Rc<Cell<Option<(f64, f64, f64)>>>,
    timers: Rc<RefCell<Timers>>,
    open: Rc<RefCell<bool>>,
    pointer_in: Rc<RefCell<bool>>,
}

impl Handle {

    /// Enciende o apaga la marca del canto. `None` la apaga.
    ///
    /// Repintar solo si cambia: esto lo llama un sondeo de 1 Hz y pedir un frame cada
    /// segundo para no cambiar nada seria despertar al compositor por gusto.
    pub fn set_mark(&self, color: Option<hfs_core::Rgb>) {
        let nuevo = color.map(|c| c.as_f64());
        if self.marca.get() == nuevo {
            return;
        }
        self.marca.set(nuevo);
        self.spacer.queue_draw();
    }
    /// Cambia el tamaño del CUERPO del panel; las muescas se suman aparte.
    ///
    /// Hace falta porque el contenido de varias superficies no es de tamaño fijo: el
    /// rail de espacios tiene tantos puntos como espacios haya, y una lista de avisos
    /// crece. Un panel de altura fija con dos puntos dentro se ve como un fallo.
    ///
    /// Hay que tocar las TRES piezas que reservan sitio. El rectangulo transparente
    /// es el que de verdad le dice su tamaño al compositor (ver mas abajo por que),
    /// asi que si solo se cambiase la ventana no cambiaria nada.
    pub fn resize(&self, width: i32, height: i32) {
        let n = self.spec.notch.ceil() as i32;
        let (a, b) = self.spec.notches();
        let extra = n * (a as i32 + b as i32);
        if self.spec.side.vertical_edge() {
            self.set_size(width, height + extra);
        } else {
            self.set_size(width + extra, height);
        }
    }

    /// Ajusta el LARGO de la superficie (lo que ocupa a lo largo de su canto) a lo
    /// que de verdad mide su contenido, y deja la profundidad como esta.
    ///
    /// Es lo que quiere el rail de espacios: tiene la anchura que tiene, y crece
    /// hacia arriba y hacia abajo segun cuantos espacios haya.
    ///
    /// Medir es mejor que calcular la altura a mano: el numero sale del arbol de
    /// widgets ya con su CSS, asi que cambiar un relleno en la hoja de estilos no
    /// deja el panel con el contenido cortado. `measure` ya incluye los margenes del
    /// cuerpo, que son justo el sitio de las muescas, asi que lo que devuelve es la
    /// superficie entera y no hay que sumarle nada.
    pub fn fit_length(&self) {
        let (sw, sh) = self.size.get();
        if self.spec.side.vertical_edge() {
            let (_, nat, _, _) = self.body.measure(gtk4::Orientation::Vertical, sw);
            self.set_size(sw, nat.max(1));
        } else {
            let (_, nat, _, _) = self.body.measure(gtk4::Orientation::Horizontal, sh);
            self.set_size(nat.max(1), sh);
        }
    }

    /// Ajusta la PROFUNDIDAD (lo que el panel entra en la pantalla) y deja el largo
    /// como esta.
    ///
    /// Es lo que quiere el reloj: el ancho lo manda el diseño, y lo que no se sabe de
    /// antemano es cuanto alto pide el calendario, que cambia con el mes.
    pub fn fit_depth(&self) {
        let (sw, sh) = self.size.get();
        if self.spec.side.vertical_edge() {
            let (_, nat, _, _) = self.body.measure(gtk4::Orientation::Horizontal, sh);
            self.set_size(nat.max(1), sh);
        } else {
            let (_, nat, _, _) = self.body.measure(gtk4::Orientation::Vertical, sw);
            self.set_size(sw, nat.max(1));
        }
    }

    /// Hay que tocar las TRES piezas que reservan sitio. El rectangulo transparente
    /// es el que de verdad le dice su tamaño al compositor (ver mas abajo por que),
    /// asi que cambiar solo la ventana no cambiaria nada.
    fn set_size(&self, sw: i32, sh: i32) {
        if self.size.get() == (sw, sh) {
            return;
        }
        self.size.set((sw, sh));
        self.spacer.set_size_request(sw, sh);
        self.overlay.set_size_request(sw, sh);
        self.root.set_size_request(sw, sh);
        self.window.set_default_size(sw, sh);
        // `layout` volvera a aplicar la region de entrada con la medida nueva, pero
        // solo si el panel esta cerrado; si esta abierto ya es la superficie entera.
    }

    fn apply_input_region(&self, full: bool) {
        let Some(surface) = self.window.surface() else {
            return;
        };
        let (sw, sh) = self.size.get();
        if full {
            let todo = cairo::RectangleInt::new(0, 0, sw, sh);
            surface.set_input_region(Some(&cairo::Region::create_rectangle(&todo)));
            return;
        }

        let t = self.spec.hover;
        // La franja del canto del que nace el panel.
        let propia = match self.spec.side {
            Side::Right => cairo::RectangleInt::new(sw - t, 0, t, sh),
            Side::Left => cairo::RectangleInt::new(0, 0, t, sh),
            Side::Top => cairo::RectangleInt::new(0, 0, sw, t),
            Side::Bottom => cairo::RectangleInt::new(0, sh - t, sw, t),
        };
        let region = cairo::Region::create_rectangle(&propia);

        // Y si el panel nace de una ESQUINA, tambien la del otro canto que la forma:
        // una esquina son dos cantos, y buscarla por uno solo obliga a recordar cual.
        // La region queda en L, con el vertice compartido.
        if !matches!(self.spec.place, Place::Center(_)) {
            let perpendicular = if self.spec.side.vertical_edge() {
                // Panel de canto vertical: los cantos perpendiculares son arriba y
                // abajo.
                match self.spec.place {
                    Place::Start(_) => cairo::RectangleInt::new(0, 0, sw, t),
                    _ => cairo::RectangleInt::new(0, sh - t, sw, t),
                }
            } else {
                match self.spec.place {
                    Place::Start(_) => cairo::RectangleInt::new(0, 0, t, sh),
                    _ => cairo::RectangleInt::new(sw - t, 0, t, sh),
                }
            };
            region.union_rectangle(&perpendicular).ok();
        }
        surface.set_input_region(Some(&region));
    }

    fn on_enter(&self) {
        *self.pointer_in.borrow_mut() = true;
        {
            let mut t = self.timers.borrow_mut();
            Timers::cancel(&mut t.grace);
            Timers::cancel(&mut t.peek);
        }
        // Ensanchar ANTES de esperar: si no, el puntero se sale de la franja al
        // moverse hacia dentro y el compositor cancela la entrada.
        self.apply_input_region(true);
        if *self.open.borrow() {
            return;
        }
        let this = self.clone();
        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(self.spec.dwell_ms as u64),
            move || {
                this.timers.borrow_mut().dwell = None;
                if *this.pointer_in.borrow() {
                    this.open_now();
                }
            },
        );
        let mut t = self.timers.borrow_mut();
        Timers::cancel(&mut t.dwell);
        t.dwell = Some(id);
    }

    fn on_leave(&self) {
        *self.pointer_in.borrow_mut() = false;
        {
            let mut t = self.timers.borrow_mut();
            Timers::cancel(&mut t.dwell);
        }
        if !*self.open.borrow() {
            self.apply_input_region(false);
            return;
        }
        let this = self.clone();
        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(self.spec.grace_ms as u64),
            move || {
                this.timers.borrow_mut().grace = None;
                if !*this.pointer_in.borrow() {
                    this.close_now();
                }
            },
        );
        let mut t = self.timers.borrow_mut();
        Timers::cancel(&mut t.grace);
        t.grace = Some(id);
    }

    pub fn open_now(&self) {
        *self.open.borrow_mut() = true;
        self.apply_input_region(true);
        self.revealer.set_reveal_child(true);
    }

    pub fn close_now(&self) {
        *self.open.borrow_mut() = false;
        self.revealer.set_reveal_child(false);
        self.apply_input_region(false);
    }

    /// Avisa cuando el panel se abre o se cierra. Lo usa el contenido para dejar de
    /// pedir datos mientras no se le ve: un panel cerrado no debe costar nada.
    pub fn connect_toggled<F: Fn(bool) + 'static>(&self, f: F) {
        self.revealer
            .connect_reveal_child_notify(move |r| f(r.reveals_child()));
    }

    pub fn is_open(&self) -> bool {
        *self.open.borrow()
    }

    /// Asomarse un momento y volver a esconderse. Es lo que hacen las teclas de
    /// volumen y brillo: asi el dock hace de OSD y no hay que escribir uno aparte.
    pub fn peek(&self, ms: u32) {
        self.open_now();
        let this = self.clone();
        let id = glib::timeout_add_local_once(
            std::time::Duration::from_millis(ms as u64),
            move || {
                this.timers.borrow_mut().peek = None;
                if !*this.pointer_in.borrow() {
                    this.close_now();
                }
            },
        );
        let mut t = self.timers.borrow_mut();
        Timers::cancel(&mut t.peek);
        Timers::cancel(&mut t.grace);
        t.peek = Some(id);
    }
}
