// ~/hyprland-from-scratch/crates/hfs/src/tema.rs
//
// El panel de Tema, invocado desde su radio de la rueda. Jubila a `theme-tui.py`.
//
//   LAS PALETAS SE LEEN DEL DISCO. El TUI de la v1 llevaba los seis nombres escritos
//   a mano en una lista `PALETTES`, asi que anadir una paleta era editar Python y
//   acordarse de dos sitios. Aqui salen de `palettes/*.toml`, que es de donde el
//   escritorio ya saca sus colores.
//
//   Y SE VEN. Cada fila ensena el acento, el fondo y la superficie de su propia
//   paleta, con los colores de verdad: elegir tema deja de ser elegir un nombre.
//
// Aplicar sigue siendo `theme-manager.sh`, que es quien escribe los ficheros de kitty,
// dunst y los bordes de Hyprland. El escritorio no se entera por aqui: lo ve el
// vigilante de `~/.config/hypr` de `shell.rs` y se rehace solo.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use hfs_core::palette::{self, Palette};
use hfs_ui::{gtk, widgets};

/// Lado de cada muestra de color, en px.
const MUESTRA: i32 = 11;

/// Un cuadradito con un color. Va con Cairo y no con CSS porque por CSS haria falta
/// un proveedor nuevo por cada color de cada paleta.
fn muestra(c: hfs_core::Rgb) -> gtk::DrawingArea {
    let a = gtk::DrawingArea::new();
    a.set_content_width(MUESTRA);
    a.set_content_height(MUESTRA);
    a.set_valign(gtk::Align::Center);
    a.set_draw_func(move |_, ctx, w, h| {
        let (r, g, b) = c.as_f64();
        ctx.set_source_rgb(r, g, b);
        widgets::redondeado(ctx, 0.0, 0.0, w as f64, h as f64, 2.5);
        let _ = ctx.fill();
    });
    a
}

/// Lanza el motor de temas. No espero el resultado: tarda ~300 ms en recargar kitty y
/// dunst, y bloquear el bucle de GTK ahi se veria como un tiron.
fn aplicar(nombre: &str) {
    let script = hfs_core::paths::repo().join("scripts/theme-manager.sh");
    crate::proc::exec("bash", [script.as_os_str(), nombre.as_ref()]);
}

pub fn panel(_palette: &Palette) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Tema");
    let lista = gtk::Box::new(gtk::Orientation::Vertical, 3);

    let pie = gtk::Label::new(Some("⏎ aplicar a todo el sistema · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&lista);
    caja.append(&pie);

    // Lo que hay pintado ahora mismo, en el orden en que se ve: lo necesita el teclado
    // para saber a quien le toca.
    let filas: Rc<RefCell<Vec<(String, gtk::Button)>>> = Rc::new(RefCell::new(Vec::new()));

    let refrescar: Rc<dyn Fn()> = {
        let (lista, estado, filas) = (lista.clone(), estado.clone(), filas.clone());
        Rc::new(move || {
            widgets::clear(&lista);
            filas.borrow_mut().clear();

            let activa = palette::current_theme_name();
            let nombres = palette::available();
            estado.set_text(&match nombres.len() {
                1 => "1 paleta".to_string(),
                n => format!("{n} paletas"),
            });

            if nombres.is_empty() {
                let v = gtk::Label::new(Some("no hay ninguna paleta en palettes/"));
                v.add_css_class("aviso");
                v.set_xalign(0.0);
                lista.append(&v);
                return;
            }

            for nombre in nombres {
                let fila = gtk::Button::new();
                fila.add_css_class("row");
                fila.set_has_frame(false);
                if nombre == activa {
                    fila.add_css_class("on");
                }

                let dentro = gtk::Box::new(gtk::Orientation::Horizontal, 8);

                // Las muestras solo se pueden pintar si la paleta carga. Si su fichero
                // esta roto lo digo en la fila, en vez de dejarla sin colores y que
                // parezca un fallo de dibujo.
                match Palette::by_name(&nombre) {
                    Ok(p) => {
                        let tiras = gtk::Box::new(gtk::Orientation::Horizontal, 3);
                        for c in [p.accent, p.bg, p.surface] {
                            tiras.append(&muestra(c));
                        }
                        dentro.append(&tiras);
                        dentro.append(&widgets::ellipsized(Some(&nombre), "nm"));
                        let modo = gtk::Label::new(Some(p.mode.as_str()));
                        modo.add_css_class("k");
                        modo.set_hexpand(true);
                        modo.set_xalign(1.0);
                        dentro.append(&modo);
                    }
                    Err(e) => {
                        eprintln!("hfs: la paleta «{nombre}» no carga: {e}");
                        dentro.append(&widgets::ellipsized(Some(&nombre), "nm"));
                        let mal = gtk::Label::new(Some("ilegible"));
                        mal.add_css_class("k");
                        mal.set_hexpand(true);
                        mal.set_xalign(1.0);
                        dentro.append(&mal);
                    }
                }

                if nombre == activa {
                    let tag = gtk::Label::new(Some("activo"));
                    tag.add_css_class("k");
                    tag.add_css_class("acc");
                    dentro.append(&tag);
                }

                fila.set_child(Some(&dentro));
                {
                    let nombre = nombre.clone();
                    fila.connect_clicked(move |_| aplicar(&nombre));
                }
                lista.append(&fila);
                filas.borrow_mut().push((nombre, fila));
            }
        })
    };

    (caja.upcast(), refrescar, None)
}
