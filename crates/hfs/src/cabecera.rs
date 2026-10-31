// ~/hyprland-from-scratch/crates/hfs/src/cabecera.rs
//
// El panel de cabecera, arriba en el centro. Dos pestañas en una sola superficie:
//
//   Medios        lo que esta sonando, con su caratula y su transporte (MPRIS)
//   Rendimiento   CPU, memoria, disco, red y bateria
//
// Van juntas porque son las dos cosas que se consultan de reojo y no se tocan: si
// cada una tuviera su borde, serian dos zonas de raton gastadas en mirar.
//
// Esta pestaña es la que **jubila a btop**: el monitor de recursos deja de ser una
// ventana a pantalla completa que hay que abrir y cerrar, y pasa a ser un vistazo.
//
// Nada de esto se mide con el panel cerrado. El uso de CPU y la velocidad de red son
// derivadas —hacen falta dos muestras— asi que al abrir se toma la primera y el
// primer dibujo llega un segundo despues; es el precio de no gastar nada mientras no
// se mira.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::sys;
use hfs_core::{Palette, Rgb};
use hfs_ui::widgets::{self, Bar, Ring};
use hfs_ui::{dbus, EdgeSurface, Handle, Place, Side, Spec};

/// Un segundo: es el intervalo natural para una derivada de /proc y el que hace que
/// el porcentaje se lea sin bailar.
const LATIDO_MS: u64 = 1000;
/// Cuantas muestras guarda el grafico de red.
const PUNTOS: usize = 48;

const MPRIS_RUTA: &str = "/org/mpris/MediaPlayer2";
const MPRIS_PLAYER: &str = "org.mpris.MediaPlayer2.Player";
const MPRIS_RAIZ: &str = "org.mpris.MediaPlayer2";

pub struct Cabecera {
    surface: EdgeSurface,
    pub handle: Handle,
}

struct Medios {
    caratula: gtk::Image,
    fuente: gtk::Label,
    titulo: gtk::Label,
    artista: gtk::Label,
    posicion: gtk::Label,
    duracion: gtk::Label,
    barra: Bar,
    play: gtk::Image,
    vacio: gtk::Label,
    fila: gtk::Box,
}

struct Rendimiento {
    cpu: Ring,
    cpu_sub: gtk::Label,
    cpu_temp: gtk::Label,
    mem: Ring,
    mem_sub: gtk::Label,
    swap: gtk::Label,
    disco: Ring,
    disco_sub: gtk::Label,
    disco_dev: gtk::Label,
    red: gtk::DrawingArea,
    red_tasas: gtk::Label,
    red_nombre: gtk::Label,
    bat_estado: gtk::Label,
    bat_pct: gtk::Label,
    bat_detalle: gtk::Label,
    bat_barra: Bar,
    bat_salud: gtk::Label,
}

impl Cabecera {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);
        let mut spec = Spec::new(
            "hfs-cabecera",
            Side::Top,
            m.centro,
            m.alto,
        );
        spec.place = Place::Center(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("cabecera");
        surface.body.set_orientation(gtk::Orientation::Vertical);

        // --- las pestañas ---
        let pila = gtk::Stack::new();
        pila.set_transition_type(gtk::StackTransitionType::Crossfade);
        pila.set_transition_duration(120);

        let (medios, caja_medios) = construir_medios(palette);
        let (perf, caja_perf) = construir_rendimiento(palette);
        pila.add_named(&caja_medios, Some("medios"));
        pila.add_named(&caja_perf, Some("perf"));

        let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        tabs.add_css_class("tabs");
        let t_medios = tab("audio-x-generic-symbolic", "Medios");
        let t_perf = tab("utilities-system-monitor-symbolic", "Rendimiento");
        t_medios.add_css_class("on");
        let encendido = gtk::Label::new(None);
        encendido.add_css_class("sp");
        encendido.set_hexpand(true);
        encendido.set_xalign(1.0);
        tabs.append(&t_medios);
        tabs.append(&t_perf);
        tabs.append(&encendido);

        {
            let (pila, a, b) = (pila.clone(), t_medios.clone(), t_perf.clone());
            t_medios.connect_clicked(move |_| {
                pila.set_visible_child_name("medios");
                a.add_css_class("on");
                b.remove_css_class("on");
            });
        }
        {
            let (pila, a, b) = (pila.clone(), t_medios.clone(), t_perf.clone());
            t_perf.connect_clicked(move |_| {
                pila.set_visible_child_name("perf");
                b.add_css_class("on");
                a.remove_css_class("on");
            });
        }

        surface.body.append(&tabs);
        surface.body.append(&pila);

        // --- los datos ---
        let repro = Reproductor::new();
        cablear_transporte(&medios, repro.clone(), &t_medios);

        let historia: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(Vec::new()));
        {
            let (historia, acento) = (historia.clone(), palette.accent);
            perf.red.set_draw_func(move |_, ctx, w, h| {
                dibujar_grafico(ctx, w as f64, h as f64, &historia.borrow(), acento);
            });
        }

        let cpu_ant: Rc<Cell<sys::CpuSample>> = Rc::new(Cell::new(sys::CpuSample::default()));
        let net_ant: Rc<Cell<sys::NetSample>> = Rc::new(Cell::new(sys::NetSample::default()));
        let modelo = sys::cpu_model().unwrap_or_else(|| "procesador".into());

        // Un solo objeto de Wi-Fi para toda la vida del panel: crearlo en cada
        // latido serian cuatro proxys de D-Bus por segundo para leer un nombre.
        let wifi = Rc::new(crate::red::Wifi::nueva());

        let refrescar: Rc<dyn Fn()> = {
            let (medios, perf) = (Rc::new(medios), Rc::new(perf));
            let wifi = wifi.clone();
            let repro = repro.clone();
            let (cpu_ant, net_ant, historia) = (cpu_ant.clone(), net_ant.clone(), historia.clone());
            let (encendido, palette) = (encendido.clone(), palette.clone());
            Rc::new(move || {
                pintar_medios(&medios, &repro, &palette);
                pintar_rendimiento(
                    &perf,
                    &palette,
                    &modelo,
                    &cpu_ant,
                    &net_ant,
                    &historia,
                    wifi.as_ref().as_ref(),
                );
                if let Ok(u) = sys::uptime() {
                    encendido.set_text(&format!("arriba {}", sys::human_duration(u)));
                }
            })
        };

        {
            let refrescar = refrescar.clone();
            let (cpu_ant, net_ant, historia) = (cpu_ant.clone(), net_ant.clone(), historia.clone());
            let timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
            handle.connect_toggled(move |abierto| {
                let mut t = timer.borrow_mut();
                if let Some(id) = t.take() {
                    id.remove();
                }
                if !abierto {
                    return;
                }
                // Al abrir se tira la historia y se toma la muestra cero: los
                // contadores llevan parados desde la ultima vez y la diferencia con
                // lo de hace media hora no significaria nada.
                historia.borrow_mut().clear();
                if let Ok(s) = sys::cpu_sample() {
                    cpu_ant.set(s);
                }
                if let Ok(s) = sys::net_sample() {
                    net_ant.set(s);
                }
                refrescar();
                let refrescar = refrescar.clone();
                *t = Some(glib::timeout_add_local(
                    Duration::from_millis(LATIDO_MS),
                    move || {
                        refrescar();
                        glib::ControlFlow::Continue
                    },
                ));
            });
        }

        surface.fit_depth();
        Self { surface, handle }
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }
    pub fn present(&self) {
        self.surface.present();
    }
}

fn tab(icono: &str, texto: &str) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("tab");
    b.set_has_frame(false);
    let caja = gtk::Box::new(gtk::Orientation::Horizontal, 7);
    let img = gtk::Image::from_icon_name(icono);
    img.set_pixel_size(14);
    caja.append(&img);
    caja.append(&gtk::Label::new(Some(texto)));
    b.set_child(Some(&caja));
    b
}

// ---------------------------------------------------------------- medios

fn construir_medios(palette: &Palette) -> (Medios, gtk::Box) {
    let caratula = gtk::Image::from_icon_name("audio-x-generic-symbolic");
    caratula.add_css_class("art");
    caratula.set_pixel_size(64);

    let fuente = widgets::ellipsized(None, "media-src");
    let titulo = widgets::ellipsized(None, "media-title");
    let artista = widgets::ellipsized(None, "media-sub");

    let posicion = gtk::Label::new(Some("0:00"));
    posicion.add_css_class("media-time");
    let duracion = gtk::Label::new(Some("0:00"));
    duracion.add_css_class("media-time");
    let barra = Bar::new(palette, 5);

    let tiempos = gtk::Box::new(gtk::Orientation::Horizontal, 9);
    tiempos.append(&posicion);
    tiempos.append(&barra.root);
    tiempos.append(&duracion);

    let texto = gtk::Box::new(gtk::Orientation::Vertical, 3);
    texto.set_hexpand(true);
    texto.set_valign(gtk::Align::Center);
    texto.append(&fuente);
    texto.append(&titulo);
    texto.append(&artista);
    texto.append(&tiempos);

    let play = gtk::Image::from_icon_name("media-playback-start-symbolic");
    play.set_pixel_size(22);

    let fila = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    fila.append(&caratula);
    fila.append(&texto);

    let vacio = gtk::Label::new(Some("no hay nada sonando"));
    vacio.add_css_class("sempty");

    let caja = gtk::Box::new(gtk::Orientation::Vertical, 0);
    caja.append(&fila);
    caja.append(&vacio);

    (
        Medios {
            caratula,
            fuente,
            titulo,
            artista,
            posicion,
            duracion,
            barra,
            play,
            vacio,
            fila,
        },
        caja,
    )
}

/// Con quien hablamos de lo que suena.
///
///   NO SE PUEDE DEPENDER DE `playerctld`. Sigue al ultimo reproductor que usaste, asi
///   que cuando esta levantado es la mejor respuesta. Pero quien lo levantaba era el
///   modulo `mpris` de **Waybar**, y al quitarla dejo de levantarlo nadie: la pestaña
///   de Medios se quedo diciendo «no hay nada sonando» con YouTube sonando delante.
///   El servicio esta declarado como activable, pero D-Bus solo activa al **llamar**,
///   y aqui no se llegaba a llamar nunca porque `vivo()` veia el nombre sin dueño y se
///   rendia antes.
///
/// Asi que si no esta, se buscan los reproductores a mano por el bus. playerctld pasa
/// de requisito a mejora: si alguien lo levanta, se usa.
struct Reproductor {
    /// (nombre de bus, proxy del Player, proxy de la raiz).
    actual: RefCell<Option<(String, gio::DBusProxy, gio::DBusProxy)>>,
}

impl Reproductor {
    fn new() -> Rc<Self> {
        Rc::new(Self {
            actual: RefCell::new(None),
        })
    }

    /// Los nombres `org.mpris.MediaPlayer2.*` que hay ahora mismo en el bus.
    fn nombres() -> Vec<String> {
        let Ok(bus) = dbus::session(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
        ) else {
            return Vec::new();
        };
        let Ok(r) = dbus::call(&bus, "ListNames", None) else {
            return Vec::new();
        };
        if r.n_children() == 0 {
            return Vec::new();
        }
        let arr = r.child_value(0);
        (0..arr.n_children())
            .filter_map(|i| arr.child_value(i).str().map(|s| s.to_string()))
            .filter(|n| n.starts_with("org.mpris.MediaPlayer2."))
            .collect()
    }

    fn estado_de(nombre: &str) -> String {
        dbus::session(nombre, MPRIS_RUTA, MPRIS_PLAYER)
            .ok()
            .and_then(|p| dbus::prop::<String>(&p, "PlaybackStatus"))
            .unwrap_or_default()
    }

    /// Elige a quien escuchar: playerctld si esta, y si no el que este sonando.
    fn elegir() -> Option<String> {
        let v = Self::nombres();
        if let Some(p) = v.iter().find(|n| n.ends_with(".playerctld")) {
            return Some(p.clone());
        }
        v.iter()
            .find(|n| Self::estado_de(n) == "Playing")
            .or_else(|| v.iter().find(|n| Self::estado_de(n) == "Paused"))
            .or_else(|| v.first())
            .cloned()
    }

    /// El reproductor de ahora. Se queda con el que tiene mientras siga sonando; en
    /// cuanto para o desaparece, vuelve a mirar. Asi cambiar de Brave a Spotify se
    /// nota sin preguntarle al bus una vez por segundo para nada.
    fn actual(&self) -> Option<(gio::DBusProxy, gio::DBusProxy)> {
        if let Some((nombre, p, r)) = self.actual.borrow().as_ref()
            && dbus::vivo(p)
            && Self::estado_de(nombre) == "Playing"
        {
            return Some((p.clone(), r.clone()));
        }
        let nombre = Self::elegir()?;
        let p = dbus::session(&nombre, MPRIS_RUTA, MPRIS_PLAYER).ok()?;
        let r = dbus::session(&nombre, MPRIS_RUTA, MPRIS_RAIZ).ok()?;
        *self.actual.borrow_mut() = Some((nombre, p.clone(), r.clone()));
        Some((p, r))
    }
}

fn cablear_transporte(m: &Medios, player: Rc<Reproductor>, _t: &gtk::Button) {
    let botones = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    botones.set_valign(gtk::Align::Center);
    for (icono, metodo, principal) in [
        ("media-skip-backward-symbolic", "Previous", false),
        ("", "PlayPause", true),
        ("media-skip-forward-symbolic", "Next", false),
    ] {
        let b = gtk::Button::new();
        b.add_css_class("bic");
        b.set_has_frame(false);
        if principal {
            b.add_css_class("main");
            b.set_child(Some(&m.play));
        } else {
            let img = gtk::Image::from_icon_name(icono);
            img.set_pixel_size(16);
            b.set_child(Some(&img));
        }
        let player = player.clone();
        b.connect_clicked(move |_| {
            // Se resuelve al pulsar y no al construir: el reproductor de entonces
            // podria haberse cerrado, o ser otro.
            if let Some((p, _)) = player.actual()
                && let Err(e) = dbus::call(&p, metodo, None)
            {
                eprintln!("hfs: el reproductor no acepto «{metodo}»: {e}");
            }
        });
        botones.append(&b);
    }
    m.fila.append(&botones);
}

fn pintar_medios(m: &Medios, repro: &Reproductor, palette: &Palette) {
    let Some((p, raiz)) = repro.actual() else {
        m.fila.set_visible(false);
        m.vacio.set_visible(true);
        return;
    };
    let (p, raiz) = (&p, Some(&raiz));
    let meta = dbus::prop_dict(p, "Metadata");
    let titulo = meta
        .as_ref()
        .and_then(|d| d.lookup_value("xesam:title", None))
        .and_then(|v| v.get::<String>())
        .unwrap_or_default();
    if titulo.is_empty() {
        m.fila.set_visible(false);
        m.vacio.set_visible(true);
        return;
    }
    m.fila.set_visible(true);
    m.vacio.set_visible(false);

    let estado = dbus::prop::<String>(p, "PlaybackStatus").unwrap_or_default();
    let sonando = estado == "Playing";
    m.play.set_icon_name(Some(if sonando {
        "media-playback-pause-symbolic"
    } else {
        "media-playback-start-symbolic"
    }));

    let nombre = raiz
        .and_then(|r| dbus::prop::<String>(r, "Identity"))
        .unwrap_or_else(|| "reproductor".into());
    m.fuente.set_text(&format!(
        "{nombre} · {}",
        match estado.as_str() {
            "Playing" => "reproduciendo",
            "Paused" => "en pausa",
            _ => "parado",
        }
    ));
    m.titulo.set_text(&titulo);

    let artistas = meta
        .as_ref()
        .and_then(|d| d.lookup_value("xesam:artist", None))
        .and_then(|v| v.get::<Vec<String>>())
        .unwrap_or_default()
        .join(", ");
    let album = meta
        .as_ref()
        .and_then(|d| d.lookup_value("xesam:album", None))
        .and_then(|v| v.get::<String>())
        .unwrap_or_default();
    m.artista.set_text(&match (artistas.is_empty(), album.is_empty()) {
        (false, false) => format!("{artistas} — {album}"),
        (false, true) => artistas,
        (true, false) => album,
        _ => String::new(),
    });

    // La caratula solo si es un fichero local. Las de los navegadores son URLs
    // remotas y no se va a abrir una conexion de red para dibujar un cuadradito.
    let art = meta
        .as_ref()
        .and_then(|d| d.lookup_value("mpris:artUrl", None))
        .and_then(|v| v.get::<String>())
        .unwrap_or_default();
    if let Some(ruta) = art.strip_prefix("file://") {
        m.caratula.set_from_file(Some(ruta));
    } else {
        m.caratula
            .set_icon_name(Some("audio-x-generic-symbolic"));
    }

    let largo_us = meta
        .as_ref()
        .and_then(|d| d.lookup_value("mpris:length", None))
        .and_then(|v| v.get::<i64>())
        .unwrap_or(0)
        .max(0) as f64;
    // La posicion no se cachea: hay que pedirla cada vez.
    let pos_us = p
        .connection()
        .call_sync(
            p.name().as_deref(),
            &p.object_path(),
            "org.freedesktop.DBus.Properties",
            "Get",
            Some(&(MPRIS_PLAYER, "Position").to_variant()),
            None,
            gio::DBusCallFlags::NONE,
            800,
            gio::Cancellable::NONE,
        )
        .ok()
        // `Properties.Get` devuelve `(v)`: hay que sacar el hijo 0 y ademas
        // desenvolver el variante que lleva dentro.
        .filter(|r| r.n_children() > 0)
        .and_then(|r| r.child_value(0).as_variant())
        .and_then(|v| v.get::<i64>())
        .unwrap_or(0)
        .max(0) as f64;

    m.posicion.set_text(&reloj(pos_us));
    m.duracion.set_text(&reloj(largo_us));
    m.barra.set(
        if largo_us > 0.0 { pos_us / largo_us } else { 0.0 },
        palette.accent,
    );
}

/// Microsegundos a «4:18».
fn reloj(us: f64) -> String {
    let s = (us / 1e6).max(0.0) as u64;
    format!("{}:{:02}", s / 60, s % 60)
}

// ---------------------------------------------------------------- rendimiento

fn construir_rendimiento(palette: &Palette) -> (Rendimiento, gtk::Box) {
    let anillo = |t: &str, icono: &str| {
        let r = Ring::new(palette, 44);
        let th = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        th.add_css_class("th");
        let img = gtk::Image::from_icon_name(icono);
        img.set_pixel_size(13);
        th.append(&img);
        th.append(&gtk::Label::new(Some(t)));
        let a = widgets::ellipsized(None, "sub");
        let b = widgets::ellipsized(None, "sub");
        let texto = gtk::Box::new(gtk::Orientation::Vertical, 1);
        texto.set_hexpand(true);
        texto.set_valign(gtk::Align::Center);
        texto.append(&th);
        texto.append(&a);
        texto.append(&b);
        let caja = widgets::tile();
        caja.set_orientation(gtk::Orientation::Horizontal);
        caja.set_spacing(10);
        caja.set_hexpand(true);
        caja.append(&r.root);
        caja.append(&texto);
        (r, a, b, caja)
    };

    let (cpu, cpu_sub, cpu_temp, caja_cpu) = anillo("CPU", "computer-symbolic");
    let (mem, mem_sub, swap, caja_mem) = anillo("Memoria", "drive-harddisk-solidstate-symbolic");
    let (disco, disco_sub, disco_dev, caja_disco) = anillo("Disco", "drive-harddisk-symbolic");

    let arriba = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    arriba.set_homogeneous(true);
    arriba.append(&caja_cpu);
    arriba.append(&caja_mem);
    arriba.append(&caja_disco);

    // --- red ---
    let red = gtk::DrawingArea::new();
    red.set_content_height(34);
    red.set_hexpand(true);
    let red_tasas = gtk::Label::new(None);
    red_tasas.add_css_class("sub");
    red_tasas.set_xalign(0.0);
    let th_red = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    th_red.add_css_class("th");
    let img = gtk::Image::from_icon_name("network-wireless-symbolic");
    img.set_pixel_size(13);
    let red_nombre = gtk::Label::new(None);
    red_nombre.add_css_class("sub");
    red_nombre.set_hexpand(true);
    red_nombre.set_xalign(1.0);
    red_nombre.set_ellipsize(gtk::pango::EllipsizeMode::End);
    th_red.append(&img);
    th_red.append(&gtk::Label::new(Some("Red")));
    th_red.append(&red_nombre);
    let caja_red = widgets::tile();
    caja_red.set_hexpand(true);
    caja_red.append(&th_red);
    caja_red.append(&red);
    caja_red.append(&red_tasas);

    // --- bateria ---
    let bat_estado = gtk::Label::new(None);
    bat_estado.add_css_class("sub");
    bat_estado.set_hexpand(true);
    bat_estado.set_xalign(1.0);
    let th_bat = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    th_bat.add_css_class("th");
    let img = gtk::Image::from_icon_name("battery-symbolic");
    img.set_pixel_size(13);
    th_bat.append(&img);
    th_bat.append(&gtk::Label::new(Some("Batería")));
    th_bat.append(&bat_estado);

    let bat_pct = gtk::Label::new(None);
    bat_pct.add_css_class("big");
    let bat_detalle = gtk::Label::new(None);
    bat_detalle.add_css_class("sub");
    let linea = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    linea.append(&bat_pct);
    linea.append(&bat_detalle);

    let bat_barra = Bar::new(palette, 5);
    let bat_salud = gtk::Label::new(None);
    bat_salud.add_css_class("sub");
    bat_salud.set_xalign(0.0);

    let caja_bat = widgets::tile();
    caja_bat.set_hexpand(true);
    caja_bat.append(&th_bat);
    caja_bat.append(&linea);
    caja_bat.append(&bat_barra.root);
    caja_bat.append(&bat_salud);

    let abajo = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    abajo.set_homogeneous(true);
    abajo.append(&caja_red);
    abajo.append(&caja_bat);

    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.append(&arriba);
    caja.append(&abajo);

    (
        Rendimiento {
            cpu,
            cpu_sub,
            cpu_temp,
            mem,
            mem_sub,
            swap,
            disco,
            disco_sub,
            disco_dev,
            red,
            red_tasas,
            red_nombre,
            bat_estado,
            bat_pct,
            bat_detalle,
            bat_barra,
            bat_salud,
        },
        caja,
    )
}

#[allow(clippy::too_many_arguments)]
fn pintar_rendimiento(
    p: &Rendimiento,
    palette: &Palette,
    modelo: &str,
    cpu_ant: &Rc<Cell<sys::CpuSample>>,
    net_ant: &Rc<Cell<sys::NetSample>>,
    historia: &Rc<RefCell<Vec<f64>>>,
    wifi: Option<&crate::red::Wifi>,
) {
    // --- CPU ---
    if let Ok(s) = sys::cpu_sample() {
        let pct = sys::cpu_percent(cpu_ant.get(), s);
        cpu_ant.set(s);
        p.cpu
            .set(pct, &format!("{pct}"), widgets::umbral(palette, pct));
        p.cpu_sub.set_text(modelo);
        p.cpu_temp.set_text(
            &sys::cpu_temp()
                .map(|t| format!("{t:.0} °C"))
                .unwrap_or_default(),
        );
    }

    // --- memoria ---
    if let Ok(m) = sys::memory() {
        let pct = m.percent();
        p.mem
            .set(pct, &format!("{pct}"), widgets::umbral(palette, pct));
        p.mem_sub.set_text(&format!(
            "{} / {}",
            sys::human_bytes(m.used_kb as f64 * 1024.0),
            sys::human_bytes(m.total_kb as f64 * 1024.0)
        ));
        p.swap.set_text(&if m.swap_total_kb == 0 {
            "sin swap".to_string()
        } else {
            format!(
                "swap {}",
                sys::human_bytes(m.swap_used_kb as f64 * 1024.0)
            )
        });
    }

    // --- disco ---
    // Por GIO y no por `df`: es una llamada al sistema en vez de un proceso, y este
    // panel la hace cada segundo mientras esta abierto.
    if let Ok(info) = gio::File::for_path("/").query_filesystem_info("filesystem::*", gio::Cancellable::NONE) {
        let total = info.attribute_uint64("filesystem::size") as f64;
        let libre = info.attribute_uint64("filesystem::free") as f64;
        if total > 0.0 {
            let pct = (((total - libre) / total) * 100.0).round() as u8;
            p.disco
                .set(pct, &format!("{pct}"), widgets::umbral(palette, pct));
            p.disco_sub.set_text(&format!(
                "{} / {}",
                sys::human_bytes(total - libre),
                sys::human_bytes(total)
            ));
            p.disco_dev
                .set_text(&sys::root_device().unwrap_or_default());
        }
    }

    // --- red ---
    if let Ok(s) = sys::net_sample() {
        let (rx, tx) = sys::net_rate(net_ant.get(), s, LATIDO_MS as f64 / 1000.0);
        net_ant.set(s);
        let mut h = historia.borrow_mut();
        h.push(rx + tx);
        if h.len() > PUNTOS {
            h.remove(0);
        }
        drop(h);
        p.red.queue_draw();
        p.red_tasas.set_text(&format!(
            "↓ {}/s    ↑ {}/s",
            sys::human_bytes(rx),
            sys::human_bytes(tx)
        ));
        p.red_nombre
            .set_text(&wifi.and_then(|w| w.ssid_activo()).unwrap_or_default());
    }

    // --- bateria ---
    match sys::battery() {
        Some(b) => {
            let color = if b.charging() || b.full() {
                palette.ok
            } else {
                widgets::umbral(palette, 100 - b.percent)
            };
            p.bat_pct.set_text(&format!("{} %", b.percent));
            p.bat_estado.set_text(match b.status.as_str() {
                "Charging" => "cargando",
                "Discharging" => "descargando",
                "Full" => "llena",
                otro => otro,
            });
            let mut detalle = Vec::new();
            if let Some(t) = b.time_left {
                detalle.push(sys::human_duration(t));
            }
            if let Some(w) = b.watts {
                detalle.push(format!("{w:.1} W"));
            }
            p.bat_detalle.set_text(&detalle.join(" · "));
            p.bat_barra.set(b.percent as f64 / 100.0, color);
            let mut salud = Vec::new();
            if let Some(h) = b.health {
                salud.push(format!("salud {h} %"));
            }
            if let Some(c) = b.cycles {
                salud.push(format!("{c} ciclos"));
            }
            if let Some(m) = &b.model {
                salud.push(m.clone());
            }
            p.bat_salud.set_text(&salud.join(" · "));
        }
        None => {
            p.bat_pct.set_text("—");
            p.bat_estado.set_text("sin batería");
            p.bat_detalle.set_text("");
            p.bat_salud.set_text("");
            p.bat_barra.set(0.0, palette.muted);
        }
    }
}

/// El grafico de red: un area bajo la curva, escalada al maximo de la ventana.
/// La escala es relativa a proposito: lo que interesa es la forma, no el valor
/// absoluto, que ya esta escrito debajo en texto.
fn dibujar_grafico(ctx: &gtk::cairo::Context, w: f64, h: f64, pts: &[f64], color: Rgb) {
    if pts.len() < 2 {
        return;
    }
    let max = pts.iter().cloned().fold(1.0f64, f64::max);
    let paso = w / (PUNTOS - 1) as f64;
    let x0 = w - paso * (pts.len() - 1) as f64;

    ctx.new_path();
    ctx.move_to(x0, h);
    for (i, v) in pts.iter().enumerate() {
        ctx.line_to(x0 + paso * i as f64, h - (v / max) * (h - 2.0));
    }
    ctx.line_to(x0 + paso * (pts.len() - 1) as f64, h);
    ctx.close_path();
    let (r, g, b) = color.as_f64();
    ctx.set_source_rgba(r, g, b, 0.22);
    ctx.fill_preserve().ok();
    ctx.set_source_rgba(r, g, b, 0.85);
    ctx.set_line_width(1.2);
    ctx.stroke().ok();
}
