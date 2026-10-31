// ~/hyprland-from-scratch/crates/hfs/src/fondo.rs
//
// El panel de Fondo, invocado desde su radio de la rueda. Jubila a `wallpaper-tui.py`.
//
//   MINIATURAS DE VERDAD. El TUI de la v1 ensenaba una lista de nombres de fichero
//   dentro de una terminal; elegir fondo a ciegas es lo contrario de elegir fondo.
//   Aqui cada imagen se carga a su tamano real de miniatura con `GdkTexture`.
//
//   Y NO HAY «VISTA PREVIA» QUE MIENTA. En la v1 la tecla `p` se llamaba preview y
//   aplicaba el fondo de golpe, asi que no se podia mirar sin cambiar. Aqui no hay
//   ninguna tecla que prometa mirar: moverse por la rejilla no toca nada y solo
//   activar una miniatura la aplica.
//
// Aplicar es `wallpaper.sh set <fichero>`, que sigue siendo el que habla con awww,
// mantiene el symlink del estado y copia el fondo para SDDM.

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use hfs_core::palette::Palette;
use hfs_ui::{gtk, widgets};

/// Columnas de la rejilla, como en el boceto.
const COLUMNAS: i32 = 4;
/// Ancho de cada miniatura, en px. El alto sale de 16:10.
const MINI_W: i32 = 96;
const MINI_H: i32 = 60;

/// Las extensiones que awww sabe poner. Filtrar por extension y no por contenido es
/// suficiente aqui: un fichero que no sea imagen fallara al cargar la miniatura y se
/// queda fuera de la rejilla igualmente.
const EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp"];

fn directorio() -> PathBuf {
    hfs_core::paths::home().join("Pictures/wallpapers")
}

/// Los fondos disponibles, por nombre. Ordenados para que la rejilla no baile entre
/// aperturas: `read_dir` no promete ningun orden.
fn disponibles() -> Vec<PathBuf> {
    let Ok(entradas) = std::fs::read_dir(directorio()) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = entradas
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|e| e.to_str())
                    .map(|e| EXTS.contains(&e.to_ascii_lowercase().as_str()))
                    .unwrap_or(false)
        })
        .collect();
    v.sort();
    v
}

/// El fondo puesto ahora mismo, siguiendo el symlink que mantiene `wallpaper.sh`.
fn actual() -> Option<PathBuf> {
    std::fs::canonicalize(hfs_core::paths::state().join("wallpaper")).ok()
}

fn aplicar(fichero: &Path) {
    let script = hfs_core::paths::repo().join("scripts/wallpaper.sh");
    crate::proc::exec(
        &script.to_string_lossy(),
        ["set".as_ref(), fichero.as_os_str()],
    );
}

/// Una miniatura. Devuelve `None` si la imagen no se puede leer, que es lo que deja
/// fuera de la rejilla a un fichero con extension de imagen que no lo es.
///
/// La imagen se escala **al cargarla**, y eso es el punto, no un detalle. Un
/// `GtkPicture` pide como tamano natural el de su textura, asi que un fondo de
/// 1920x1080 reclama 1920x1080 y la rejilla se lo da: `set_size_request` no lo evita,
/// porque solo fija un minimo. Cargando ya escalado la textura nace de 96x60 y el
/// problema no existe. De paso, 24 fondos en memoria son 24 miniaturas y no 24
/// imagenes enteras, que serian ~190 MB.
fn miniatura(fichero: &Path) -> Option<gtk::Picture> {
    let pb = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(fichero, MINI_W, MINI_H, true).ok()?;
    let tex = gtk::gdk::Texture::for_pixbuf(&pb);
    let pic = gtk::Picture::for_paintable(&tex);
    pic.set_content_fit(gtk::ContentFit::Contain);
    pic.add_css_class("mini");
    Some(pic)
}


/// Mueve el cursor del teclado.
///
/// Va con una clase propia y no con el foco de GTK: en una superficie de layer-shell
/// `grab_focus()` no agarra —devuelve falso sin decir nada— asi que las flechas no
/// tenian de donde partir. Y aunque agarrara, el `outline` con el que GTK pinta el
/// foco es el mismo que marca el fondo que esta puesto, asi que se taparian.
fn marcar(botones: &[(PathBuf, gtk::Button)], n: usize) {
    for (i, (_, b)) in botones.iter().enumerate() {
        if i == n {
            b.add_css_class("cursor");
        } else {
            b.remove_css_class("cursor");
        }
    }
}

pub fn panel(
    _palette: &Palette,
    cerrar: Rc<dyn Fn()>,
) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Fondo");
    let rejilla = gtk::Grid::new();
    rejilla.set_row_spacing(6);
    rejilla.set_column_spacing(6);

    let aviso = gtk::Label::new(None);
    aviso.add_css_class("aviso");
    aviso.set_xalign(0.0);
    aviso.set_wrap(true);
    aviso.set_visible(false);

    let pie = gtk::Label::new(Some("flechas mueven · ⏎ aplicar · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&rejilla);
    caja.append(&aviso);
    caja.append(&pie);

    let botones: Rc<RefCell<Vec<(PathBuf, gtk::Button)>>> = Rc::new(RefCell::new(Vec::new()));
    let elegido = Rc::new(Cell::new(0usize));

    let refrescar: Rc<dyn Fn()> = {
        let (rejilla, estado, aviso, botones, elegido) = (
            rejilla.clone(),
            estado.clone(),
            aviso.clone(),
            botones.clone(),
            elegido.clone(),
        );
        let cerrar = cerrar.clone();
        Rc::new(move || {
            widgets::clear(&rejilla);
            botones.borrow_mut().clear();
            elegido.set(0);

            let fondos = disponibles();
            let puesto = actual();

            estado.set_text(&match fondos.len() {
                1 => "1 imagen".to_string(),
                n => format!("{n} imágenes"),
            });

            if fondos.is_empty() {
                aviso.set_text(&format!(
                    "no hay imágenes en {}",
                    directorio().display()
                ));
                aviso.set_visible(true);
                rejilla.set_visible(false);
                return;
            }
            aviso.set_visible(false);
            rejilla.set_visible(true);

            let mut i = 0i32;
            let mut inicial = 0usize;
            for f in fondos {
                let Some(pic) = miniatura(&f) else {
                    eprintln!("hfs: «{}» no se puede leer como imagen", f.display());
                    continue;
                };
                let b = gtk::Button::new();
                b.add_css_class("minib");
                b.set_has_frame(false);
                b.set_child(Some(&pic));
                b.set_tooltip_text(f.file_name().and_then(|n| n.to_str()));
                if puesto.as_deref() == Some(f.as_path()) {
                    b.add_css_class("on");
                }
                {
                    let (f, cerrar) = (f.clone(), cerrar.clone());
                    b.connect_clicked(move |_| {
                        aplicar(&f);
                        // La accion termino: quedarse mirando la rejilla del fondo
                        // que acabas de poner no sirve para nada, y encima la tapa.
                        cerrar();
                    });
                }
                rejilla.attach(&b, i % COLUMNAS, i / COLUMNAS, 1, 1);
                // El que esta puesto arranca con el foco, y si no hay, el primero.
                // Sin esto las flechas no mueven nada: GTK las reparte desde el
                // widget enfocado, y al abrir el panel no lo estaba ninguno.
                if puesto.as_deref() == Some(f.as_path()) {
                    inicial = i as usize;
                }
                botones.borrow_mut().push((f, b));
                i += 1;
            }
            // El cursor arranca sobre el fondo que esta puesto.
            elegido.set(inicial);
            marcar(&botones.borrow(), inicial);
        })
    };

    // --- el teclado ---
    let teclas: crate::Teclas = {
        let (botones, elegido, cerrar) = (botones.clone(), elegido.clone(), cerrar.clone());
        Rc::new(move |key, _mods| {
            let b = botones.borrow();
            let total = b.len();
            if total == 0 {
                return false;
            }
            let i = elegido.get().min(total - 1);
            let cols = COLUMNAS as usize;
            let nuevo = match key {
                gtk::gdk::Key::Right => (i + 1) % total,
                gtk::gdk::Key::Left => (i + total - 1) % total,
                // Arriba y abajo saltan una fila entera, y se quedan en el sitio si
                // no hay fila a la que ir: dar la vuelta aqui desorienta.
                gtk::gdk::Key::Down => (i + cols).min(total - 1),
                gtk::gdk::Key::Up => i.saturating_sub(cols),
                gtk::gdk::Key::Return | gtk::gdk::Key::KP_Enter => {
                    aplicar(&b[i].0);
                    cerrar();
                    return true;
                }
                _ => return false,
            };
            elegido.set(nuevo);
            marcar(&b, nuevo);
            true
        })
    };

    (caja.upcast(), refrescar, Some(teclas))
}
