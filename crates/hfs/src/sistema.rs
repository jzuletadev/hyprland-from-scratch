// ~/hyprland-from-scratch/crates/hfs/src/sistema.rs
//
// La ficha de la maquina. Jubila a `sysinfo.sh`, que era fastfetch dentro de una kitty
// con el logo de arch escalado por ImageMagick y cacheado a mano.
//
//   NO NACE DE UN CANTO, Y ESO ES DELIBERADO. Es lo unico del boceto que se abre en
//   medio de la pantalla: se consulta de vez en cuando y de un vistazo. Asi que no es
//   una superficie de layer-shell sino una **ventana normal**, con la `windowrule` de
//   float + size + center que ya existe para la clase `sysinfo`. Una ventana se puede
//   mover, se cierra con SUPER+Q como todo lo demas y no se come el teclado.
//
//   Y NO HAY SUBPROCESOS. Todo sale de /sys, /proc y /etc a traves de `hfs-core::sys`,
//   incluido el numero de paquetes (los directorios de la base de datos de pacman) y
//   el nombre de verdad de las dos graficas (resuelto contra `pci.ids` de hwdata).
//   La v1 forkeaba fastfetch, que forkea a su vez bastante mas.

use gtk::prelude::*;
use hfs_core::palette::Palette;
use hfs_core::sys;
use hfs_ui::{gtk, widgets};

/// Una fila de la ficha: etiqueta a la izquierda, valor a la derecha.
fn dato(etiqueta: &str, valor: &str) -> gtk::Box {
    let f = widgets::row();
    let e = gtk::Label::new(Some(etiqueta));
    e.add_css_class("k");
    e.set_xalign(0.0);
    e.set_size_request(86, -1);
    f.append(&e);
    let v = widgets::ellipsized(Some(valor), "nm");
    v.set_hexpand(true);
    v.set_xalign(0.0);
    f.append(&v);
    f
}

/// El espacio del sistema de ficheros de `/`. Va por GIO y no por `df` porque GIO ya
/// esta enlazado y responde sin forkear.
fn disco() -> Option<String> {
    let info = gtk::gio::File::for_path("/")
        .query_filesystem_info("filesystem::size,filesystem::used", gtk::gio::Cancellable::NONE)
        .ok()?;
    let total = info.attribute_uint64("filesystem::size") as f64;
    let usado = info.attribute_uint64("filesystem::used") as f64;
    if total <= 0.0 {
        return None;
    }
    Some(format!(
        "{} de {} ({:.0} %){}",
        sys::human_bytes(usado),
        sys::human_bytes(total),
        usado / total * 100.0,
        match sys::root_device() {
            Some(d) => format!(" · {d}"),
            None => String::new(),
        }
    ))
}

/// Construye la ventana. Se llama una vez por apertura: es una ficha que se mira y se
/// cierra, asi que no merece la pena mantenerla viva ni sondear nada.
pub fn abrir(app: &gtk::Application, palette: &Palette) {
    let win = gtk::ApplicationWindow::new(app);
    win.set_title(Some("Sistema"));
    // Es lo que hace que la `windowrule` la agarre y la deje flotante y centrada.
    gtk::prelude::GtkWindowExt::set_default_size(&win, 900, 300);

    let caja = gtk::Box::new(gtk::Orientation::Vertical, 10);
    caja.add_css_class("invocado");
    caja.add_css_class("ficha");
    caja.set_margin_top(18);
    caja.set_margin_bottom(18);
    caja.set_margin_start(20);
    caja.set_margin_end(20);

    let (cab, estado) = widgets::header("Sistema");
    estado.set_text(&sys::os_pretty().unwrap_or_else(|| "Linux".into()));
    caja.append(&cab);

    let cols = gtk::Box::new(gtk::Orientation::Horizontal, 22);
    let izq = gtk::Box::new(gtk::Orientation::Vertical, 3);
    let der = gtk::Box::new(gtk::Orientation::Vertical, 3);
    izq.set_hexpand(true);
    der.set_hexpand(true);

    let nada = || "—".to_string();

    izq.append(&dato("Máquina", &sys::host_model().unwrap_or_else(nada)));
    izq.append(&dato("CPU", &sys::cpu_model().unwrap_or_else(nada)));
    let gpus = sys::gpus();
    if gpus.is_empty() {
        izq.append(&dato("Gráfica", &nada()));
    } else {
        // Esta maquina es hibrida: dos tarjetas, y la ficha tiene que decirlo.
        for (i, g) in gpus.iter().enumerate() {
            izq.append(&dato(if i == 0 { "Gráfica" } else { "" }, g));
        }
    }
    izq.append(&dato(
        "Memoria",
        &match sys::memory() {
            Ok(m) => format!(
                "{} de {}",
                sys::human_bytes(m.used_kb as f64 * 1024.0),
                sys::human_bytes(m.total_kb as f64 * 1024.0)
            ),
            Err(_) => nada(),
        },
    ));

    der.append(&dato("Kernel", &sys::kernel().unwrap_or_else(nada)));
    der.append(&dato("Shell", &sys::shell().unwrap_or_else(nada)));
    der.append(&dato(
        "Paquetes",
        &match sys::package_count() {
            Some(n) => format!("{n}"),
            None => nada(),
        },
    ));
    der.append(&dato("Disco", &disco().unwrap_or_else(nada)));
    der.append(&dato(
        "Encendido",
        &match sys::uptime() {
            Ok(d) => sys::human_duration(d),
            Err(_) => nada(),
        },
    ));

    cols.append(&izq);
    cols.append(&der);
    caja.append(&cols);
    caja.append(&widgets::sep());

    // La paleta activa, con sus colores. Es la linea que el boceto pedia y la que
    // explica de un vistazo de donde salen los colores de todo lo demas.
    let fila_pal = widgets::row();
    let et = gtk::Label::new(Some("Paleta"));
    et.add_css_class("k");
    et.set_xalign(0.0);
    et.set_size_request(86, -1);
    fila_pal.append(&et);
    let nombre = widgets::ellipsized(
        Some(&format!("{} · {}", palette.name, palette.mode.as_str())),
        "nm",
    );
    nombre.set_xalign(0.0);
    fila_pal.append(&nombre);
    let tiras = gtk::Box::new(gtk::Orientation::Horizontal, 3);
    tiras.set_hexpand(true);
    tiras.set_halign(gtk::Align::End);
    for c in [
        palette.accent,
        palette.ok,
        palette.warn,
        palette.bad,
        palette.info,
        palette.fg,
        palette.surface,
        palette.bg,
    ] {
        let a = gtk::DrawingArea::new();
        a.set_content_width(12);
        a.set_content_height(12);
        a.set_valign(gtk::Align::Center);
        a.set_draw_func(move |_, ctx, w, h| {
            let (r, g, b) = c.as_f64();
            ctx.set_source_rgb(r, g, b);
            widgets::redondeado(ctx, 0.0, 0.0, w as f64, h as f64, 2.5);
            let _ = ctx.fill();
        });
        tiras.append(&a);
    }
    fila_pal.append(&tiras);
    caja.append(&fila_pal);

    let pie = gtk::Label::new(Some("esc o SUPER+Q para cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);
    caja.append(&pie);

    win.set_child(Some(&caja));

    // Esc cierra, que es lo que uno intenta en una ficha que se consulta y se tira.
    let teclas = gtk::EventControllerKey::new();
    {
        let win = win.clone();
        teclas.connect_key_pressed(move |_, key, _, _| {
            if key == gtk::gdk::Key::Escape {
                win.close();
                return glib_true();
            }
            gtk::glib::Propagation::Proceed
        });
    }
    win.add_controller(teclas);

    win.present();
}

fn glib_true() -> gtk::glib::Propagation {
    gtk::glib::Propagation::Stop
}
