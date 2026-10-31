// ~/hyprland-from-scratch/crates/hfs/src/captura.rs
//
// El panel de Captura, invocado desde su radio de la rueda. Jubila a
// `screen-record-tui.py`, y recoge tambien las cuatro capturas que hoy solo tienen
// tecla, porque el boceto las queria juntas: una captura y una grabacion son la misma
// intencion con distinta duracion.
//
//   VUELVE EL INDICADOR DE GRABACION. Vivia en Waybar y se fue con ella, asi que
//   entre el paso A del desmontaje y este panel no habia forma de saber que
//   `wf-recorder` seguia corriendo. Aqui el panel se sondea a 1 Hz **solo mientras
//   esta abierto** y ensena el tiempo transcurrido.
//
//   Y NO HAY MONITOR INVENTADO. La v1 preguntaba a `hyprctl` cual era el monitor con
//   foco y, si fallaba, devolvia «eDP-1» a ciegas: en esta maquina hay dos pantallas,
//   asi que grababa la equivocada sin decir nada.
//
// Todo lo que dispara va por `sh -c` y detras del cierre de la rueda: `slurp` se queda
// esperando a que arrastres el raton, y bloquear el bucle de GTK ahi congelaria el
// escritorio entero. Por eso tampoco se espera el resultado.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::{Duration, Instant};

use gtk::prelude::*;
use hfs_core::palette::Palette;
use hfs_ui::gtk::glib;
use hfs_ui::{gtk, widgets};

/// Donde caen los videos, igual que en la v1.
const DIR: &str = "Videos/recordings";

/// Cada cuanto se mira si `wf-recorder` sigue vivo, mientras el panel esta abierto.
const SONDEO: Duration = Duration::from_secs(1);

/// Las cuatro capturas, tal como las nombra el boceto. El segundo campo es el
/// argumento de `screenshot.sh`, que ya existe y funciona.
const CAPTURAS: &[(&str, &str, &str)] = &[
    ("Monitor completo", "screen", "camera-photo-symbolic"),
    ("Región", "region", "select-rectangular-symbolic"),
    ("Ventana activa", "window", "window-symbolic"),
    ("Región anotada", "annotate", "document-edit-symbolic"),
];

thread_local! {
    /// Quien sigue la pista a la grabacion. Es uno solo para todo el proceso: lo
    /// consultan este panel y la bandeja de cada pantalla, y comparten el PID que
    /// lleva dentro para no barrer /proc tres veces por segundo.
    static VIGIA: crate::proc::Vigia = const { crate::proc::Vigia::new("wf-recorder") };
}

/// Si `wf-recorder` esta grabando ahora mismo.
///
/// Dos cosas que no son obvias. Una: **un zombi conserva su nombre**, asi que con un
/// `pgrep -x` una grabacion ya parada seguia contando como viva y el panel se quedaba
/// diciendo «grabando» para siempre. Dos: esto se consulta **una vez por segundo
/// mientras el escritorio este vivo**, porque la marca del canto tiene que aparecer
/// sin que nadie roce nada, asi que no puede permitirse barrer /proc. Ver `proc.rs`.
pub fn grabando() -> bool {
    VIGIA.with(|v| v.vivo())
}

/// El monitor con foco, preguntado de verdad. `None` si Hyprland no contesta, y
/// entonces no se graba: es mejor no grabar que grabar la pantalla equivocada.
fn monitor_con_foco() -> Option<String> {
    let st = hfs_core::hyprland::State::query().ok()?;
    st.focused_monitor().map(|s| s.to_string())
}

fn ejecutar(cmd: String) {
    crate::proc::sh(&cmd);
}

pub fn parar() {
    // -INT y no -TERM: wf-recorder cierra el contenedor al recibir INT, y con TERM
    // deja el mp4 sin el indice al final, que es un fichero que no se puede buscar.
    ejecutar("pkill -INT -x wf-recorder".into());
}

/// Arranca la grabacion. `region` elige entre arrastrar un recuadro y la pantalla con
/// foco; `audio` anade el monitor del sink por omision.
///
/// Se monta como una orden de shell porque `slurp` se queda esperando a que arrastres
/// el raton: hacerlo desde Rust obligaria a un hilo y a comunicar el resultado de
/// vuelta al bucle de GTK, y aqui el shell ya sabe encadenar las dos cosas.
fn arrancar(region: bool, audio: bool) {
    let dir = hfs_core::paths::home().join(DIR);

    // La geometria primero, y si cancelas con Esc `slurp` no escribe nada: ese `exit 0`
    // es lo que distingue «he cambiado de idea» de un fallo.
    let (geo, fuente) = if region {
        ("G=\"$(slurp)\"; [ -n \"$G\" ] || exit 0; ".to_string(), "-g \"$G\"".to_string())
    } else {
        match monitor_con_foco() {
            Some(m) => (String::new(), format!("-o {m}")),
            None => {
                eprintln!("hfs: Hyprland no dice que monitor tiene el foco; no grabo");
                return;
            }
        }
    };

    let sonido = if audio {
        " --audio=\"$(pactl get-default-sink).monitor\""
    } else {
        ""
    };

    // El `exec` de dentro hace que este PID acabe siendo el de `wf-recorder`, asi
    // que apuntarlo evita el primer barrido y la marca del canto sale al momento.
    let pid = crate::proc::sh(&format!(
        "mkdir -p \"{dir}\"; {geo}exec wf-recorder \
         -f \"{dir}/$(date +%Y-%m-%d_%H-%M-%S).mp4\" \
         -p preset=veryfast -p crf=23 {fuente}{sonido}",
        dir = dir.display(),
    ));
    if let Some(pid) = pid {
        VIGIA.with(|v| v.cebar(pid));
    }
}

pub fn panel(_palette: &Palette, cerrar: Rc<dyn Fn()>) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Captura");

    let lista = gtk::Box::new(gtk::Orientation::Vertical, 3);
    for (titulo, arg, icono) in CAPTURAS {
        let fila = gtk::Button::new();
        fila.add_css_class("row");
        fila.set_has_frame(false);
        let dentro = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let img = gtk::Image::from_icon_name(icono);
        img.set_pixel_size(14);
        dentro.append(&img);
        dentro.append(&widgets::ellipsized(Some(titulo), "nm"));
        fila.set_child(Some(&dentro));
        {
            let (cerrar, arg) = (cerrar.clone(), arg.to_string());
            fila.connect_clicked(move |_| {
                // Primero fuera la rueda: una seleccion de region no se puede hacer
                // con una superficie a pantalla completa por encima.
                cerrar();
                let script = hfs_core::paths::repo().join("scripts/screenshot.sh");
                let cmd = format!("sleep 0.25; exec {} {arg}", script.display());
                ejecutar(cmd);
            });
        }
        lista.append(&fila);
    }

    caja.append(&cab);
    caja.append(&lista);
    caja.append(&widgets::sep());

    // --- la grabacion ---
    //
    // El modo es un par excluyente y no una casilla suelta: «region» a secas no dice
    // contra que, y una casilla sin estado visible no es un control, es un adorno.
    // Ese fue el fallo de la primera version: los dos conmutadores funcionaban, pero
    // `.key` solo tenia `:hover`, asi que pulsarlos no cambiaba nada a la vista y
    // parecia que solo se podia grabar la pantalla entera.
    let region = Rc::new(Cell::new(false));
    let audio = Rc::new(Cell::new(false));

    let opciones = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let modo = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    modo.add_css_class("seg");
    let b_pantalla = gtk::ToggleButton::with_label("pantalla");
    let b_region = gtk::ToggleButton::with_label("región");
    for b in [&b_pantalla, &b_region] {
        b.add_css_class("key");
        b.set_has_frame(false);
    }
    b_pantalla.set_active(true);
    // `group` los hace excluyentes y garantiza que siempre hay uno activo.
    b_region.set_group(Some(&b_pantalla));
    modo.append(&b_pantalla);
    modo.append(&b_region);

    let b_audio = gtk::ToggleButton::with_label("audio");
    b_audio.add_css_class("key");
    b_audio.set_has_frame(false);

    opciones.append(&modo);
    opciones.append(&widgets::sep());
    opciones.append(&b_audio);

    let grabar = gtk::Button::new();
    grabar.add_css_class("row");
    grabar.set_has_frame(false);
    let g_dentro = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let g_punto = gtk::Image::from_icon_name("media-record-symbolic");
    g_punto.set_pixel_size(14);
    let g_texto = widgets::ellipsized(Some("Grabar la pantalla"), "nm");
    let g_tiempo = gtk::Label::new(None);
    g_tiempo.add_css_class("k");
    g_tiempo.set_hexpand(true);
    g_tiempo.set_xalign(1.0);
    g_dentro.append(&g_punto);
    g_dentro.append(&g_texto);
    g_dentro.append(&g_tiempo);
    grabar.set_child(Some(&g_dentro));

    {
        // El texto del boton dice que va a grabar, para que el modo se lea tambien
        // ahi y no solo en el conmutador.
        let (region, g_texto) = (region.clone(), g_texto.clone());
        b_region.connect_toggled(move |b| {
            region.set(b.is_active());
            if !grabando() {
                g_texto.set_text(if b.is_active() {
                    "Grabar una región"
                } else {
                    "Grabar la pantalla"
                });
            }
        });
    }
    {
        let audio = audio.clone();
        b_audio.connect_toggled(move |b| audio.set(b.is_active()));
    }

    caja.append(&opciones);
    caja.append(&grabar);

    let pie = gtk::Label::new(Some("esc cerrar · retroceso vuelve a la rueda"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);
    caja.append(&pie);

    // Desde cuando se graba. Es una estimacion: si la grabacion ya estaba en marcha
    // antes de abrir el panel no hay forma de saber cuando empezo sin leer el mtime
    // del fichero, asi que en ese caso no se ensena tiempo.
    let desde: Rc<RefCell<Option<Instant>>> = Rc::new(RefCell::new(None));

    {
        let (region, audio, cerrar, desde) =
            (region.clone(), audio.clone(), cerrar.clone(), desde.clone());
        grabar.connect_clicked(move |_| {
            if grabando() {
                parar();
                *desde.borrow_mut() = None;
            } else {
                let r = region.get();
                cerrar();
                *desde.borrow_mut() = Some(Instant::now());
                arrancar(r, audio.get());
            }
        });
    }

    // Pintar el estado. Se llama al abrir y una vez por segundo mientras este abierto.
    let pintar: Rc<dyn Fn()> = {
        let (estado, g_texto, g_tiempo, grabar, opciones, desde, region) = (
            estado.clone(),
            g_texto.clone(),
            g_tiempo.clone(),
            grabar.clone(),
            opciones.clone(),
            desde.clone(),
            region.clone(),
        );
        Rc::new(move || {
            if grabando() {
                estado.set_text("grabando");
                g_texto.set_text("Parar la grabación");
                grabar.add_css_class("on");
                opciones.set_sensitive(false);
                let t = desde.borrow().map(|i| i.elapsed().as_secs());
                g_tiempo.set_text(&match t {
                    Some(s) => format!("{}:{:02}", s / 60, s % 60),
                    None => "en marcha".into(),
                });
            } else {
                estado.set_text("");
                g_texto.set_text(if region.get() {
                    "Grabar una región"
                } else {
                    "Grabar la pantalla"
                });
                grabar.remove_css_class("on");
                opciones.set_sensitive(true);
                g_tiempo.set_text("");
            }
        })
    };

    // El sondeo se ata al panel y no al escritorio: `SourceId` muere cuando el widget
    // se va, y asi nada mira a /proc mientras la rueda esta cerrada.
    let tic: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    {
        let (tic, pintar) = (tic.clone(), pintar.clone());
        caja.connect_unmap(move |_| {
            if let Some(id) = tic.borrow_mut().take() {
                id.remove();
            }
            let _ = &pintar;
        });
    }

    let refrescar: Rc<dyn Fn()> = {
        let (tic, pintar) = (tic.clone(), pintar.clone());
        Rc::new(move || {
            pintar();
            if tic.borrow().is_none() {
                let pintar = pintar.clone();
                let id = glib::timeout_add_local(SONDEO, move || {
                    pintar();
                    glib::ControlFlow::Continue
                });
                *tic.borrow_mut() = Some(id);
            }
        })
    };

    (caja.upcast(), refrescar, None)
}
