// ~/hyprland-from-scratch/crates/hfs/src/atajos.rs
//
// El panel de Atajos. Jubila a `keybinds-tui.py`, y es el hueco que el boceto no vio:
// no tiene ficha en `design/js/data.js`, asi que ninguna superficie lo reemplazaba.
//
//   SE PUEDE ESCRIBIR EN EL FILTRO. El TUI de la v1 evaluaba las teclas antes que el
//   texto, asi que `i`, `k`, `q` y `Q` nunca llegaban al buscador: buscar «quit»,
//   «kill» o «window» era literalmente imposible. Aqui el filtro es un `GtkEntry` y
//   las teclas de navegacion son las que no son texto.
//
//   Y SE VEN LOS 68, no 33. Los que no tienen `bindd` salen con lo que de verdad
//   ejecutan y marcados como derivados, asi que la ayuda ensena cuales le falta
//   describir en vez de esconderlos.
//
// Vive dentro de la rueda, no como radio: es una referencia que se consulta con
// SUPER+/, no un ajuste, y la rueda se queda con los radios del boceto.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use hfs_core::binds::{self, Bind};
use hfs_core::fuzzy;
use hfs_core::palette::Palette;
use hfs_ui::{gtk, widgets};

/// Mueve el cursor y arrastra el scroll con el.
///
/// Va con una clase propia y no con el foco de GTK por lo mismo que la rejilla de
/// fondos: `grab_focus()` no agarra en una superficie de layer-shell. Ver
/// `crates/README.md`.
fn marcar(lista: &gtk::Box, rollo: &gtk::ScrolledWindow, n: usize) {
    let mut i = 0;
    let mut hijo = lista.first_child();
    while let Some(w) = hijo {
        hijo = w.next_sibling();
        if i == n {
            w.add_css_class("on");
            // Que la fila elegida entre en la parte visible. Sin esto, bajar con las
            // flechas mueve la marca pero la lista no sigue, y el cursor se pierde
            // por debajo del borde.
            // `compute_bounds` contra la lista da la posicion de la fila dentro de
            // ella, que es contra lo que mide el ajuste del scroll.
            let (y, alto_fila) = match w.compute_bounds(lista) {
                Some(b) => (b.y() as f64, (b.height() as f64).max(1.0)),
                None => continue,
            };
            let v = rollo.vadjustment();
            if y < v.value() {
                v.set_value(y);
            } else if y + alto_fila > v.value() + v.page_size() {
                v.set_value(y + alto_fila - v.page_size());
            }
        } else {
            w.remove_css_class("on");
        }
        i += 1;
    }
}

/// Hasta donde crece la lista antes de hacer scroll, en px. No es un numero de filas:
/// antes se cortaba en 11 y los demas no habia forma de verlos.
const MAX_ALTO: i32 = 420;

/// Una fila: el atajo a la izquierda y lo que hace a la derecha.
fn fila(b: &Bind) -> gtk::Box {
    let f = widgets::row();
    let combo = gtk::Label::new(Some(&b.combo()));
    combo.add_css_class("key-tag");
    combo.set_xalign(0.0);
    f.append(&combo);

    let que = widgets::ellipsized(Some(&b.label()), "nm");
    que.set_hexpand(true);
    que.set_xalign(0.0);
    if b.derivada() {
        // Sin `bindd`: se ensena, pero mas apagado, porque lo que se lee es la orden
        // y no una descripcion escrita para leerse.
        que.add_css_class("k");
    }
    f.append(&que);

    if !b.submap.trim().is_empty() {
        let s = gtk::Label::new(Some(&b.submap));
        s.add_css_class("k");
        f.append(&s);
    }
    f
}

pub fn panel(_palette: &Palette) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Atajos");

    let filtro = gtk::Entry::new();
    filtro.set_placeholder_text(Some("filtrar…"));
    filtro.add_css_class("clave");
    filtro.set_visibility(true);

    let lista = gtk::Box::new(gtk::Orientation::Vertical, 2);

    // Con scroll salen los 68, no once. Y el scroll hace falta de verdad: sin filtro
    // la lista no cabe en ninguna pantalla.
    let rollo = gtk::ScrolledWindow::new();
    rollo.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    rollo.set_propagate_natural_height(true);
    rollo.set_max_content_height(MAX_ALTO);
    rollo.set_child(Some(&lista));

    let pie = gtk::Label::new(Some("↑↓ mover · escribe para filtrar · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&filtro);
    caja.append(&rollo);
    caja.append(&pie);

    let todos: Rc<RefCell<Vec<Bind>>> = Rc::new(RefCell::new(Vec::new()));
    let elegido = Rc::new(std::cell::Cell::new(0usize));

    let pintar: Rc<dyn Fn()> = {
        let (lista, estado, filtro, todos) =
            (lista.clone(), estado.clone(), filtro.clone(), todos.clone());
        let (rollo, elegido) = (rollo.clone(), elegido.clone());
        Rc::new(move || {
            widgets::clear(&lista);
            let aguja = filtro.text().to_string();
            let v = todos.borrow();

            // Sin filtro, el orden es el del compositor, que es el de hyprland.conf:
            // lo mas parecido a «como estan agrupados en el fichero».
            let elegidos: Vec<&Bind> = if aguja.trim().is_empty() {
                v.iter().collect()
            } else {
                let mut con_nota: Vec<(i32, &Bind)> = v
                    .iter()
                    .filter_map(|b| {
                        let campos = [b.combo(), b.label(), b.buscable()];
                        let refs: Vec<&str> = campos.iter().map(|s| s.as_str()).collect();
                        fuzzy::score_fields(&aguja, &refs).map(|n| (n, b))
                    })
                    .collect();
                con_nota.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
                con_nota.into_iter().map(|(_, b)| b).collect()
            };

            let total = elegidos.len();
            for b in &elegidos {
                lista.append(&fila(b));
            }
            // Filtrar devuelve el cursor arriba: la fila que estaba marcada ya no es
            // la misma despues de cambiar el filtro.
            elegido.set(0);
            marcar(&lista, &rollo, 0);
            rollo.vadjustment().set_value(0.0);

            if total == 0 {
                let nada = gtk::Label::new(Some("ningún atajo coincide"));
                nada.add_css_class("aviso");
                nada.set_xalign(0.0);
                lista.append(&nada);
            }

            let sin = v.iter().filter(|b| b.derivada()).count();
            estado.set_text(&if aguja.trim().is_empty() {
                format!("{} · {sin} sin describir", v.len())
            } else {
                format!("{total} de {}", v.len())
            });
        })
    };

    {
        let pintar = pintar.clone();
        filtro.connect_changed(move |_| pintar());
    }

    let refrescar: Rc<dyn Fn()> = {
        let (todos, filtro, pintar, estado) =
            (todos.clone(), filtro.clone(), pintar.clone(), estado.clone());
        Rc::new(move || {
            // Se vuelven a pedir en cada apertura: un `hyprctl reload` cambia la lista
            // y la ayuda tiene que contar la de ahora, no la de cuando se abrio.
            match binds::load() {
                Ok(v) => *todos.borrow_mut() = v,
                Err(e) => {
                    eprintln!("hfs: no pude leer los atajos: {e}");
                    estado.set_text("Hyprland no contesta");
                    todos.borrow_mut().clear();
                }
            }
            filtro.set_text("");
            pintar();
            filtro.grab_focus();
        })
    };

    // --- el teclado ---
    let teclas: crate::Teclas = {
        let (lista, rollo, elegido) = (lista.clone(), rollo.clone(), elegido.clone());
        Rc::new(move |key, _mods| {
            let total = {
                let mut n = 0;
                let mut h = lista.first_child();
                while let Some(w) = h {
                    h = w.next_sibling();
                    n += 1;
                }
                n
            };
            if total == 0 {
                return false;
            }
            let i = elegido.get().min(total - 1);
            let nuevo = match key {
                gtk::gdk::Key::Down => (i + 1) % total,
                gtk::gdk::Key::Up => (i + total - 1) % total,
                gtk::gdk::Key::Page_Down => (i + 8).min(total - 1),
                gtk::gdk::Key::Page_Up => i.saturating_sub(8),
                gtk::gdk::Key::Home => 0,
                gtk::gdk::Key::End => total - 1,
                _ => return false,
            };
            elegido.set(nuevo);
            marcar(&lista, &rollo, nuevo);
            true
        })
    };

    (caja.upcast(), refrescar, Some(teclas))
}
