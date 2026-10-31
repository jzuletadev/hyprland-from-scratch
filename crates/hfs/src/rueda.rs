// ~/hyprland-from-scratch/crates/hfs/src/rueda.rs
//
// La rueda del centro y el buscador de aplicaciones. Es lo que sustituye a Rofi.
//
// Dos cosas en una superficie, porque son el mismo gesto: tocas SUPER y o eliges un
// radio con el raton, o te pones a escribir. El centro de la rueda ya no repite la
// hora ni la bateria ni la red —cada una tiene su borde y duplicarlas solo mete
// ruido—: lo que queda es lo unico que un radio no puede dar, que es buscar.
//
// Escribir una letra cualquiera entra en el buscador, y borrar hasta vaciarlo vuelve
// a la rueda. No hay que apuntar a ningun sitio ni pulsar nada antes.
//
// Wi-Fi y Bluetooth ya abren su propio panel dentro de la rueda. Los otros tres
// radios siguen abriendo la herramienta de la v1 que les toca, y cada uno se
// cambiara por su panel cuando le llegue el turno en V2-PLAN.md.
// `Ajustes` no esta: es el unico de los seis del boceto que no tiene equivalente en
// la v1, y un radio que no hace nada es peor que un radio que falta.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gdk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::{fuzzy, Palette};
use hfs_ui::{CenterHandle, CenterSurface};

/// Cuantos caracteres del nombre de una aplicacion caben antes de recortarlo. Es el
/// tope que impide que un nombre largo ensanche la lista.
const NOMBRE_MAX: i32 = 28;
/// Alto del campo de busqueda, para poder centrarlo sin medirlo.
const ALTO_CAMPO: i32 = 38;
const LADO_MAX_ALTO_PCT: f64 = 0.62;
/// Proporciones del boceto, relativas al lado de la rueda.
const NODO_REL: f64 = 5.4 / 34.0;
/// El hub mide lo mismo que un radio. Era mas grande y se notaba: rompia el anillo
/// en vez de cerrarlo.
const HUB_REL: f64 = NODO_REL;
const RADIO_REL: f64 = 128.0 / 360.0;
const RADIO_DENTRO: f64 = 52.0 / 360.0;
const RADIO_FUERA: f64 = 94.0 / 360.0;
const RESULTADOS: usize = 6;

// ---------------------------------------------------------------- los radios

/// Lo que hace un radio hoy. Al llegar su turno, cada uno pasa a abrir su panel.
struct Nodo {
    titulo: &'static str,
    icono: &'static str,
    /// Su panel propio, si ya existe. Si no, se cae a la herramienta de la v1.
    panel: Panel,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Panel {
    Wifi,
    Bluetooth,
    Captura,
    Fondo,
    Tema,
    Sonido,
    /// No tiene radio: se llega con SUPER+/. Es una referencia que se consulta, no un
    /// ajuste, asi que la rueda se queda con los radios del boceto.
    Atajos,
}


/// Si lo que tiene el foco dentro de la pila es algo donde se escribe. Decide si
/// Retroceso es «volver a la rueda» o «borrar una letra».
fn hay_foco_de_texto(pila: &gtk::Stack) -> bool {
    let Some(raiz) = pila.root() else { return false };
    let Ok(ventana) = raiz.downcast::<gtk::Window>() else {
        return false;
    };
    let Some(f) = gtk::prelude::GtkWindowExt::focus(&ventana) else {
        return false;
    };
    // `GtkEntry` delega el texto en un `GtkText` interno, asi que el foco real suele
    // estar en el hijo y preguntar solo por `Entry` no lo pilla.
    f.is::<gtk::Text>()
        || f.is::<gtk::Entry>()
        || f.ancestor(gtk::Entry::static_type()).is_some()
}

/// En el orden del boceto: desde arriba y en el sentido de las agujas del reloj.
const NODOS: &[Nodo] = &[
    Nodo {
        titulo: "Wi-Fi",
        icono: "network-wireless-signal-excellent-symbolic",
        panel: Panel::Wifi,
    },
    Nodo {
        titulo: "Bluetooth",
        icono: "bluetooth-symbolic",
        panel: Panel::Bluetooth,
    },
    Nodo {
        titulo: "Captura",
        icono: "camera-photo-symbolic",
        panel: Panel::Captura,
    },
    Nodo {
        titulo: "Fondo",
        icono: "preferences-desktop-wallpaper-symbolic",
        panel: Panel::Fondo,
    },
    Nodo {
        titulo: "Tema",
        icono: "preferences-color-symbolic",
        panel: Panel::Tema,
    },
    Nodo {
        titulo: "Sonido",
        icono: "audio-volume-high-symbolic",
        panel: Panel::Sonido,
    },
];


// ---------------------------------------------------------------- la superficie

pub struct Rueda {
    surface: CenterSurface,
    pub handle: CenterHandle,
    reset: Rc<dyn Fn()>,
    /// Saltarse la rueda y abrir un panel. Lo usa SUPER+/ para los Atajos, que no
    /// tienen radio porque son una referencia y no un ajuste.
    ir_a_panel: Rc<dyn Fn(Panel)>,
}

/// El estado vivo de la rueda. Va en un Rc porque lo tocan el teclado, el raton y el
/// propio buscador.
struct Estado {
    seleccion: Cell<Option<usize>>,
    /// Lo que se ve ahora mismo en el buscador, en orden de puntuacion.
    resultados: RefCell<Vec<gio::AppInfo>>,
    elegido: Cell<usize>,
}

pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gdk::Monitor) -> Rueda {
    let g = monitor.geometry();
    // La rueda mide lo mismo que un panel de esquina. No es un porcentaje propio:
    // sale de las mismas medidas que todo lo demas, que es lo que evita que vuelva a
    // crecer por su cuenta. Antes era el 30 % del ancho —576 px— contra los 408 de
    // un panel, y se veia.
    let m = hfs_ui::medidas::Medidas::para(&g);
    let lado = (m.esquina as f64).min(g.height() as f64 * LADO_MAX_ALTO_PCT).round();
    let nodo_px = (lado * NODO_REL).round() as i32;
    let hub_px = (lado * HUB_REL).round() as i32;

    let surface = CenterSurface::new(app, "hfs-rueda", palette);
    surface.set_monitor(monitor);
    let handle = surface.handle();
    surface.body.add_css_class("rueda-raiz");

    // Si el buscador esta a la vista. Antes se deducia del nombre de la pagina de la
    // pila; ahora el buscador ya no es una pagina, asi que se lleva aparte.
    let buscando_ya = Rc::new(Cell::new(false));

    let estado = Rc::new(Estado {
        seleccion: Cell::new(None),
        resultados: RefCell::new(Vec::new()),
        elegido: Cell::new(0),
    });

    // --- las dos caras: la rueda y el buscador ---
    let pila = gtk::Stack::new();
    pila.set_transition_type(gtk::StackTransitionType::Crossfade);
    pila.set_transition_duration(140);

    // ---- cara 1: la rueda ----
    let lienzo = gtk::Fixed::new();
    lienzo.set_size_request(lado as i32, lado as i32);
    // La pila mide lo que su hijo mas ancho, que es el buscador; sin esto la rueda
    // se queda pegada a la izquierda de esa anchura y no sale centrada en pantalla.
    lienzo.set_halign(gtk::Align::Center);
    lienzo.set_valign(gtk::Align::Center);

    let radios = gtk::DrawingArea::new();
    radios.set_size_request(lado as i32, lado as i32);
    radios.set_can_target(false);
    {
        let (r, gc, b) = palette.fg.as_f64();
        let n = NODOS.len();
        radios.set_draw_func(move |_, ctx, w, h| {
            dibujar_radios(ctx, w as f64, h as f64, n, (r, gc, b));
        });
    }
    lienzo.put(&radios, 0.0, 0.0);

    let centro = lado / 2.0;
    let radio = lado * RADIO_REL;
    let mut botones = Vec::new();
    for (i, nodo) in NODOS.iter().enumerate() {
        let ang: f64 = (-90.0 + i as f64 * (360.0 / NODOS.len() as f64)).to_radians();
        let (x, y) = (centro + radio * ang.cos(), centro + radio * ang.sin());

        let boton = gtk::Button::new();
        boton.add_css_class("nodo");
        boton.set_has_frame(false);
        boton.set_size_request(nodo_px, nodo_px);
        let img = gtk::Image::from_icon_name(nodo.icono);
        img.set_pixel_size((nodo_px as f64 * 0.38).round() as i32);
        boton.set_child(Some(&img));

        let etiqueta = gtk::Label::new(Some(nodo.titulo));
        etiqueta.add_css_class("nodo-tag");

        let caja = gtk::Box::new(gtk::Orientation::Vertical, 6);
        caja.set_halign(gtk::Align::Center);
        caja.append(&boton);
        caja.append(&etiqueta);

        // El ancho de la caja lo manda la etiqueta, que es mas larga que el circulo,
        // asi que se centra sobre el punto en vez de colocarla por su esquina.
        let (_, nat, _, _) = caja.measure(gtk::Orientation::Horizontal, -1);
        lienzo.put(&caja, x - nat as f64 / 2.0, y - nodo_px as f64 / 2.0);

        botones.push((boton, etiqueta));
    }

    // El hub: la entrada al buscador.
    let hub = gtk::Button::new();
    hub.add_css_class("hub");
    hub.set_has_frame(false);
    hub.set_size_request(hub_px, hub_px);
    {
        // Solo el icono. El texto «escribe para buscar» era una instruccion que hay
        // que leer una vez y luego estorba para siempre, y ademas obligaba al hub a
        // ser mas ancho de lo que la lupa necesita.
        let img = gtk::Image::from_icon_name("system-search-symbolic");
        img.set_pixel_size((hub_px as f64 * 0.38).round() as i32);
        hub.set_child(Some(&img));
    }
    lienzo.put(&hub, centro - hub_px as f64 / 2.0, centro - hub_px as f64 / 2.0);

    pila.add_named(&lienzo, Some("rueda"));

    // ---- cara 2: el buscador ----
    //
    // Es una cara aparte, y por tanto **la rueda no se ve mientras buscas**. Se
    // intento meterlo dentro del anillo y sale mal por una razon concreta:
    // `gtk::Fixed` mide a partir de la posicion de sus hijos, asi que la lista de
    // resultados hacia crecer el lienzo con cada letra y, al estar centrado, el campo
    // subia en pantalla segun escribias.
    //
    // Lo que si se arregla aqui es lo que hacia que molestara: el campo se ancla a una
    // posicion FIJA —la del centro de la pantalla, donde estaba el hub— en vez de
    // centrarse con su contenido. Asi los resultados crecen hacia abajo y el campo no
    // se mueve.
    // Igual de ancho que la rueda y que un panel de esquina: una sola medida para
    // todo lo que se ve en el centro.
    let campo_w = m.esquina;

    let entrada = gtk::Entry::new();
    entrada.add_css_class("sq");
    entrada.set_has_frame(false);
    entrada.set_placeholder_text(Some("buscar"));
    entrada.set_hexpand(true);

    let campo = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    campo.add_css_class("sfield");
    let lupa = gtk::Image::from_icon_name("system-search-symbolic");
    lupa.set_pixel_size(16);
    lupa.add_css_class("sfield-ic");
    campo.append(&lupa);
    campo.append(&entrada);

    let lista = gtk::Box::new(gtk::Orientation::Vertical, 2);
    lista.add_css_class("slist");

    let buscador = gtk::Box::new(gtk::Orientation::Vertical, 8);
    buscador.set_size_request(campo_w, -1);
    buscador.set_valign(gtk::Align::Start);
    buscador.append(&campo);
    buscador.append(&lista);
    pila.add_named(&buscador, Some("buscador"));

    // ---- cara 3: el panel que se invoca desde un radio ----
    let contenedor = gtk::Box::new(gtk::Orientation::Vertical, 0);
    contenedor.set_size_request(m.esquina, -1);
    contenedor.set_halign(gtk::Align::Center);
    contenedor.set_valign(gtk::Align::Center);
    pila.add_named(&contenedor, Some("panel"));

    pila.set_visible_child_name("rueda");
    surface.body.append(&pila);

    // --- las aplicaciones ---
    let apps: Rc<RefCell<Vec<gio::AppInfo>>> = Rc::new(RefCell::new(cargar_apps()));
    {
        // El dia que instalas algo, que aparezca sin reiniciar el escritorio.
        let apps = apps.clone();
        let monitor_apps = gio::AppInfoMonitor::get();
        monitor_apps.connect_changed(move |_| *apps.borrow_mut() = cargar_apps());
        std::mem::forget(monitor_apps);
    }

    let pintar = {
        let (lista, apps, estado, entrada) = (
            lista.clone(),
            apps.clone(),
            estado.clone(),
            entrada.clone(),
        );
        let handle = handle.clone();
        Rc::new(move || {
            let consulta = entrada.text().to_string();
            let mut puntuados: Vec<(i32, gio::AppInfo)> = apps
                .borrow()
                .iter()
                .filter_map(|a| {
                    let nombre = a.name().to_string();
                    let desc = a.description().map(|d| d.to_string()).unwrap_or_default();
                    fuzzy::score_fields(&consulta, &[&nombre, &desc]).map(|s| (s, a.clone()))
                })
                .collect();
            puntuados.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.name().cmp(&b.1.name())));
            let elegidos: Vec<gio::AppInfo> = puntuados
                .into_iter()
                .take(RESULTADOS)
                .map(|(_, a)| a)
                .collect();

            estado.elegido.set(0);
            *estado.resultados.borrow_mut() = elegidos.clone();
            rellenar(&lista, &elegidos, 0, &consulta, &handle);
        })
    };

    {
        let pintar = pintar.clone();
        entrada.connect_changed(move |_| pintar());
    }

    // --- la seleccion: un solo sitio para el raton y para el teclado ---
    let marcar: Rc<dyn Fn()> = {
        let (botones, estado) = (botones.clone(), estado.clone());
        Rc::new(move || {
            let sel = estado.seleccion.get();
            for (i, (boton, etiqueta)) in botones.iter().enumerate() {
                let on = sel == Some(i);
                for w in [boton.upcast_ref::<gtk::Widget>(), etiqueta.upcast_ref()] {
                    if on {
                        w.add_css_class("sel");
                    } else {
                        w.remove_css_class("sel");
                    }
                }
            }
        })
    };
    let seleccionar: Rc<dyn Fn(Option<usize>)> = {
        let (estado, marcar) = (estado.clone(), marcar.clone());
        Rc::new(move |s| {
            estado.seleccion.set(s);
            marcar();
        })
    };
    for (i, (boton, _)) in botones.iter().enumerate() {
        let motion = gtk::EventControllerMotion::new();
        let seleccionar = seleccionar.clone();
        motion.connect_enter(move |_, _, _| seleccionar(Some(i)));
        boton.add_controller(motion);
    }

    // --- cambiar de cara ---
    let a_buscador: Rc<dyn Fn()> = {
        let (pila, entrada, body) = (pila.clone(), entrada.clone(), surface.body.clone());
        let buscando_ya = buscando_ya.clone();
        // Donde cae el campo: su borde de arriba, para que el campo quede sobre el
        // centro de la pantalla, que es donde estaba el hub.
        let arriba = (g.height() / 2 - ALTO_CAMPO / 2).max(0);
        Rc::new(move || {
            if buscando_ya.get() {
                return;
            }
            buscando_ya.set(true);
            // Anclado arriba con un margen fijo, NO centrado: centrado, el campo se
            // movia medio renglon por cada resultado que entraba o salia.
            body.set_valign(gtk::Align::Start);
            body.set_margin_top(arriba);
            pila.set_visible_child_name("buscador");
            entrada.grab_focus();
        })
    };
    // Los paneles se construyen la primera vez que se abren y luego se reutilizan:
    // cada uno levanta sus proxys de D-Bus, y hacerlo en cada apertura seria pagar
    // una conexion nueva por mirar el Wi-Fi.
    // El tercer elemento es opcional: un panel que quiera gobernar sus propias
    // teclas (las flechas de la rejilla de fondos, por ejemplo) lo devuelve, y el
    // manejador de la rueda se lo pregunta antes de hacer nada.
    type Hecho = (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>);
    let hechos: Rc<RefCell<std::collections::HashMap<Panel, Hecho>>> =
        Rc::new(RefCell::new(std::collections::HashMap::new()));

    // Cerrar la rueda desde dentro de un panel. Lo necesita Captura: `slurp` y
    // `grim` no pueden trabajar con una superficie a pantalla completa por encima.
    let cerrar: Rc<dyn Fn()> = {
        let handle = handle.clone();
        Rc::new(move || handle.close())
    };

    // Las teclas del panel que se ve ahora mismo, si tiene.
    let teclas_vivas: Rc<RefCell<Option<crate::Teclas>>> = Rc::new(RefCell::new(None));

    let a_panel: Rc<dyn Fn(Panel)> = {
        let teclas_vivas = teclas_vivas.clone();
        let (pila, contenedor, hechos, body) = (
            pila.clone(),
            contenedor.clone(),
            hechos.clone(),
            surface.body.clone(),
        );
        let (palette, cerrar) = (palette.clone(), cerrar.clone());
        Rc::new(move |cual| {
            let mut mapa = hechos.borrow_mut();
            let (w, refrescar, teclas) = mapa.entry(cual).or_insert_with(|| match cual {
                Panel::Wifi => crate::red::panel(&palette),
                Panel::Bluetooth => crate::bluetooth::panel(&palette),
                Panel::Captura => crate::captura::panel(&palette, cerrar.clone()),
                Panel::Fondo => crate::fondo::panel(&palette, cerrar.clone()),
                Panel::Tema => crate::tema::panel(&palette),
                Panel::Atajos => crate::atajos::panel(&palette),
                Panel::Sonido => crate::sonido::panel(&palette),
            });
            *teclas_vivas.borrow_mut() = teclas.clone();
            if w.parent().is_none() {
                contenedor.append(w);
            }
            // Solo uno a la vez dentro del contenedor.
            let mut hijo = contenedor.first_child();
            while let Some(h) = hijo {
                hijo = h.next_sibling();
                h.set_visible(h == *w);
            }
            refrescar();
            body.set_valign(gtk::Align::Center);
            body.set_margin_top(0);
            pila.set_visible_child_name("panel");
        })
    };

    let a_rueda: Rc<dyn Fn()> = {
        let (pila, entrada, body) = (pila.clone(), entrada.clone(), surface.body.clone());
        let (teclas_vivas, buscando_ya) = (teclas_vivas.clone(), buscando_ya.clone());
        Rc::new(move || {
            *teclas_vivas.borrow_mut() = None;
            buscando_ya.set(false);
            entrada.set_text("");
            body.set_valign(gtk::Align::Center);
            body.set_margin_top(0);
            pila.set_visible_child_name("rueda");
        })
    };

    let activar: Rc<dyn Fn(usize)> = {
        let a_panel = a_panel.clone();
        Rc::new(move |i| a_panel(NODOS[i].panel))
    };
    for (i, (boton, _)) in botones.iter().enumerate() {
        let activar = activar.clone();
        boton.connect_clicked(move |_| activar(i));
    }
    {
        let a_buscador = a_buscador.clone();
        hub.connect_clicked(move |_| a_buscador());
    }

    // Mover la marca del resultado elegido sin rehacer la lista.
    let repintar_elegido: Rc<dyn Fn()> = {
        let (lista, estado) = (lista.clone(), estado.clone());
        Rc::new(move || {
            let n = estado.elegido.get();
            let mut i = 0;
            let mut hijo = lista.first_child();
            while let Some(w) = hijo {
                hijo = w.next_sibling();
                if w.has_css_class("sitem") {
                    if i == n {
                        w.add_css_class("on");
                    } else {
                        w.remove_css_class("on");
                    }
                    i += 1;
                }
            }
        })
    };

    let lanzar_elegido: Rc<dyn Fn()> = {
        let (estado, handle) = (estado.clone(), handle.clone());
        Rc::new(move || {
            let r = estado.resultados.borrow();
            if let Some(app) = r.get(estado.elegido.get()) {
                let app = app.clone();
                drop(r);
                handle.close();
                lanzar(&app);
            }
        })
    };

    // --- el teclado ---
    {
        let (estado, pila, entrada) = (estado.clone(), pila.clone(), entrada.clone());
        let (a_buscador, a_rueda) = (a_buscador.clone(), a_rueda.clone());
        let (seleccionar, repintar_elegido, lanzar_elegido) = (
            seleccionar.clone(),
            repintar_elegido.clone(),
            lanzar_elegido.clone(),
        );
        let (handle, activar) = (handle.clone(), activar.clone());
        let (teclas_vivas, buscando_ya) = (teclas_vivas.clone(), buscando_ya.clone());
        let n = NODOS.len();
        surface.connect_keys(move |key, mods| {
            let en_panel = pila.visible_child_name().as_deref() == Some("panel");
            let buscando = buscando_ya.get();

            // Primero el panel: si gobierna esa tecla, aqui no se hace nada mas.
            if en_panel
                && let Some(t) = teclas_vivas.borrow().clone()
                && t(key, mods)
            {
                return glib::Propagation::Stop;
            }

            match key {
                gdk::Key::Escape => {
                    // Escape CIERRA, desde donde sea.
                    //
                    // Antes retrocedia un paso —panel a rueda, rueda al escritorio— y
                    // eso obligaba a `SUPER → rueda → opcion → rueda → salir`: la
                    // rueda es un sitio de paso, no un destino, asi que quedarse en
                    // ella al terminar es un paso de mas en el camino mas comun.
                    // Para volver a la rueda sin salir esta Retroceso, y el buscador
                    // ya se vaciaba asi.
                    handle.close();
                    glib::Propagation::Stop
                }
                gdk::Key::Return | gdk::Key::KP_Enter => {
                    if en_panel {
                        // Dentro de un panel la tecla es suya (la contraseña).
                        return glib::Propagation::Proceed;
                    }
                    if buscando {
                        lanzar_elegido();
                    } else if let Some(i) = estado.seleccion.get() {
                        activar(i);
                    }
                    glib::Propagation::Stop
                }
                gdk::Key::Up | gdk::Key::Down | gdk::Key::Left | gdk::Key::Right
                | gdk::Key::Tab | gdk::Key::ISO_Left_Tab => {
                    if en_panel {
                        return glib::Propagation::Proceed;
                    }
                    let adelante = matches!(
                        key,
                        gdk::Key::Down | gdk::Key::Right | gdk::Key::Tab
                    ) && !mods.contains(gdk::ModifierType::SHIFT_MASK);
                    if buscando {
                        let total = estado.resultados.borrow().len();
                        if total > 0 {
                            let i = estado.elegido.get();
                            estado
                                .elegido
                                .set(if adelante { (i + 1) % total } else { (i + total - 1) % total });
                            repintar_elegido();
                        }
                    } else {
                        let i = estado.seleccion.get();
                        seleccionar(Some(match i {
                            None => {
                                if adelante {
                                    0
                                } else {
                                    n - 1
                                }
                            }
                            Some(i) if adelante => (i + 1) % n,
                            Some(i) => (i + n - 1) % n,
                        }));
                    }
                    glib::Propagation::Stop
                }
                // Retroceso vuelve a la rueda. En el buscador, solo cuando ya esta
                // vacio; en un panel, solo si no hay un campo de texto con el foco,
                // porque ahi la tecla es suya (la contraseña del Wi-Fi, el filtro de
                // los atajos).
                gdk::Key::BackSpace if buscando && entrada.text().is_empty() => {
                    a_rueda();
                    glib::Propagation::Stop
                }
                gdk::Key::BackSpace if en_panel && !hay_foco_de_texto(&pila) => {
                    a_rueda();
                    glib::Propagation::Stop
                }
                _ => {
                    if buscando || en_panel {
                        // Que la escriba el GtkEntry que toque.
                        return glib::Propagation::Proceed;
                    }
                    // Una letra cualquiera entra en el buscador. No hay que apuntar
                    // al centro ni pulsar nada antes: escribes y ya estas buscando.
                    match key.to_unicode() {
                        Some(c) if !c.is_control() && !mods.contains(gdk::ModifierType::CONTROL_MASK) => {
                            a_buscador();
                            entrada.set_text(&c.to_string());
                            entrada.set_position(-1);
                            glib::Propagation::Stop
                        }
                        _ => glib::Propagation::Proceed,
                    }
                }
            }
        });
    }

    // Abrir siempre deja la rueda como nueva: sin consulta y sin nada marcado.
    let reset: Rc<dyn Fn()> = {
        let (a_rueda, seleccionar) = (a_rueda.clone(), seleccionar.clone());
        Rc::new(move || {
            a_rueda();
            seleccionar(None);
        })
    };

    Rueda {
        surface,
        handle,
        reset,
        ir_a_panel: a_panel,
    }
}

fn cargar_apps() -> Vec<gio::AppInfo> {
    gio::AppInfo::all()
        .into_iter()
        .filter(|a| a.should_show())
        .collect()
}

fn rellenar(
    lista: &gtk::Box,
    apps: &[gio::AppInfo],
    elegido: usize,
    consulta: &str,
    handle: &CenterHandle,
) {
    while let Some(hijo) = lista.first_child() {
        lista.remove(&hijo);
    }
    if apps.is_empty() {
        let l = gtk::Label::new(Some(&format!("sin resultados para «{consulta}»")));
        l.add_css_class("sempty");
        l.set_xalign(0.0);
        lista.append(&l);
        return;
    }
    for (i, app) in apps.iter().enumerate() {
        let fila = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        fila.add_css_class("sitem");
        if i == elegido {
            fila.add_css_class("on");
        }
        let img = gtk::Image::new();
        img.set_pixel_size(22);
        if let Some(icono) = app.icon() {
            img.set_from_gicon(&icono);
        } else {
            img.set_icon_name(Some("application-x-executable-symbolic"));
        }
        // `max_width_chars` NO sobra teniendo `ellipsize`: sin el, una etiqueta
        // elidida sigue PIDIENDO su ancho natural, asi que la lista se ensanchaba
        // hasta donde llegara la descripcion mas larga y cambiaba de tamano con cada
        // letra que escribias.
        //
        // El reparto no es simetrico a proposito. El NOMBRE es lo que se lee, asi que
        // pide lo suyo hasta un tope; la DESCRIPCION es de relleno, asi que pide lo
        // minimo y se queda con el hueco que sobre. Ponerle el tope de 1 a los dos
        // dejaba los nombres en «...».
        let nombre = gtk::Label::new(Some(&app.name()));
        nombre.set_xalign(0.0);
        nombre.set_ellipsize(gtk::pango::EllipsizeMode::End);
        nombre.set_max_width_chars(NOMBRE_MAX);
        let desc = gtk::Label::new(app.description().as_deref());
        desc.add_css_class("k");
        desc.set_xalign(1.0);
        desc.set_hexpand(true);
        desc.set_ellipsize(gtk::pango::EllipsizeMode::End);
        desc.set_max_width_chars(1);
        fila.append(&img);
        fila.append(&nombre);
        fila.append(&desc);

        let click = gtk::GestureClick::new();
        {
            let (app, handle) = (app.clone(), handle.clone());
            click.connect_released(move |_, _, _, _| {
                handle.close();
                lanzar(&app);
            });
        }
        fila.add_controller(click);
        lista.append(&fila);
    }
}

fn lanzar(app: &gio::AppInfo) {
    let ctx = gdk::Display::default().map(|d| d.app_launch_context());
    let r = match &ctx {
        Some(c) => app.launch(&[], Some(c.upcast_ref::<gio::AppLaunchContext>())),
        None => app.launch(&[], None::<&gio::AppLaunchContext>),
    };
    if let Err(e) = r {
        eprintln!("hfs: no pude abrir «{}»: {e}", app.name());
    }
}

/// Los radios: una linea fina desde el hub hacia cada nodo, con su punta de flecha.
fn dibujar_radios(ctx: &gtk::cairo::Context, w: f64, h: f64, n: usize, color: (f64, f64, f64)) {
    let (cx, cy) = (w / 2.0, h / 2.0);
    let lado = w.min(h);
    let (dentro, fuera) = (lado * RADIO_DENTRO, lado * RADIO_FUERA);
    let (r, g, b) = color;

    for i in 0..n {
        let a = (-90.0 + i as f64 * (360.0 / n as f64)).to_radians();
        let (dx, dy) = (a.cos(), a.sin());
        ctx.set_source_rgba(r, g, b, 0.17);
        ctx.set_line_width(1.0);
        ctx.move_to(cx + dentro * dx, cy + dentro * dy);
        ctx.line_to(cx + fuera * dx, cy + fuera * dy);
        ctx.stroke().ok();

        // La punta, para que el radio se lea como «va hacia alli» y no como un palo.
        let (px, py) = (cx + fuera * dx, cy + fuera * dy);
        let (ah, aw) = (lado * 0.019, lado * 0.012);
        ctx.set_source_rgba(r, g, b, 0.22);
        ctx.move_to(px, py);
        ctx.line_to(px - ah * dx - aw * dy, py - ah * dy + aw * dx);
        ctx.line_to(px - ah * dx + aw * dy, py - ah * dy - aw * dx);
        ctx.close_path();
        ctx.fill().ok();
    }
}

impl Rueda {
    pub fn destroy(&self) {
        self.surface.destroy();
    }

    /// Abre o cierra. Al abrir, la deja como nueva.
    pub fn toggle(&self) {
        if self.handle.is_open() {
            self.handle.close();
        } else {
            (self.reset)();
            self.handle.open();
        }
    }

    pub fn close(&self) {
        self.handle.close();
    }

    /// Abre la rueda con uno de sus paneles ya puesto. Si ya estaba abierta en otro
    /// sitio, cambia a el: tocar la tecla dos veces no deberia cerrar nada.
    ///
    /// El nombre viene del IPC, asi que un nombre que no existe no es un fallo que
    /// deba tirar nada: se dice y se abre la rueda normal.
    pub fn abrir_panel(&self, nombre: &str) {
        let cual = match nombre {
            "atajos" => Panel::Atajos,
            "sonido" => Panel::Sonido,
            "tema" => Panel::Tema,
            "fondo" => Panel::Fondo,
            "captura" => Panel::Captura,
            otro => {
                eprintln!("hfs: no conozco el panel «{otro}»; abro la rueda");
                if !self.handle.is_open() {
                    (self.reset)();
                    self.handle.open();
                }
                return;
            }
        };
        if !self.handle.is_open() {
            (self.reset)();
            self.handle.open();
        }
        (self.ir_a_panel)(cual);
    }
}
