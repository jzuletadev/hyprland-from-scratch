// ~/hyprland-from-scratch/crates/hfs/src/sonido.rs
//
// Elegir por donde suena y por donde se escucha. Panel de la rueda, SUPER+V.
//
//   NO EXISTIA, Y SE NOTO. Ni la v1 ni la v2 tenian forma de cambiar de dispositivo:
//   el dock de Controles sube y baja el volumen del que haya puesto, y punto. Cuando
//   PipeWire elige mal —y lo hace: tras un arranque con la tarjeta interna sin
//   configurar, el predeterminado acabo siendo un monitor HDMI sin altavoces— subir
//   el volumen no hacia nada y no habia donde mirar.
//
//   VA EN LA RUEDA Y NO EN CONTROLES. El dock del canto derecho mide un 3,7 % de la
//   pantalla a proposito, para que el pulgar suba y baje por el mismo canto. Una
//   lista de siete dispositivos ahi dentro seria romperlo. Esto es un ajuste
//   ocasional, que es justo lo que la rueda recoge.
//
// Tocar una fila la pone de predeterminada; WirePlumber mueve ademas lo que ya estaba
// sonando, asi que no hay que ir flujo por flujo.

use std::rc::Rc;

use gtk::prelude::*;
use hfs_core::audio::{self, Device};
use hfs_core::palette::Palette;
use hfs_ui::{gtk, widgets};

/// El propio cierre de repintado, guardado para poder llamarse desde dentro.
type Repintar = Rc<std::cell::RefCell<Option<Rc<dyn Fn()>>>>;

/// Una fila de dispositivo. `conectado` en falso la deja a la vista pero sin poder
/// elegirse: ofrecer algo que al tocarlo no hace nada es peor que no ofrecerlo.
fn fila(d: &Device, icono: &str, conectado: bool, refrescar: &Rc<dyn Fn()>) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("row");
    b.set_has_frame(false);
    if d.activo {
        b.add_css_class("on");
    }
    b.set_sensitive(conectado);

    let caja = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let img = gtk::Image::from_icon_name(icono);
    img.set_pixel_size(14);
    caja.append(&img);

    let nombre = widgets::ellipsized(Some(&audio::etiqueta(&d.nombre)), "nm");
    nombre.set_hexpand(true);
    nombre.set_xalign(0.0);
    caja.append(&nombre);

    // El volumen de cada uno, no solo el del activo: es lo que explica un dispositivo
    // que parece bien elegido y no suena porque esta a cero o silenciado.
    let vol = gtk::Label::new(Some(&if !conectado {
        "sin conectar".to_string()
    } else if d.muted {
        "silenciado".to_string()
    } else {
        format!("{} %", d.percent)
    }));
    vol.add_css_class("k");
    if d.muted && conectado {
        vol.add_css_class("mal");
    }
    caja.append(&vol);

    if d.activo {
        let tag = gtk::Label::new(Some("en uso"));
        tag.add_css_class("k");
        tag.add_css_class("acc");
        caja.append(&tag);
    }

    b.set_child(Some(&caja));
    {
        let (id, activo, refrescar) = (d.id, d.activo, refrescar.clone());
        b.connect_clicked(move |_| {
            // Tocar el que ya esta puesto silencia o devuelve el sonido: es la accion
            // util que queda cuando no hay nada que cambiar.
            let r = if activo {
                audio::toggle_mute_de(id)
            } else {
                audio::set_default(id)
            };
            if let Err(e) = r {
                eprintln!("hfs: no pude cambiar el dispositivo: {e}");
            }
            refrescar();
        });
    }
    b
}

fn seccion(titulo: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(titulo));
    l.add_css_class("sec");
    l.set_xalign(0.0);
    l
}

pub fn panel(_palette: &Palette) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Sonido");
    let cuerpo = gtk::Box::new(gtk::Orientation::Vertical, 4);

    let pie = gtk::Label::new(Some("tocar elige · tocar el activo silencia · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&cuerpo);
    caja.append(&pie);

    // El cierre se necesita a si mismo para repintar tras cada cambio, asi que se
    // construye en dos tiempos.
    let hueco: Repintar = Rc::new(std::cell::RefCell::new(None));

    let refrescar: Rc<dyn Fn()> = {
        let (cuerpo, estado, hueco) = (cuerpo.clone(), estado.clone(), hueco.clone());
        Rc::new(move || {
            widgets::clear(&cuerpo);
            let yo = hueco.borrow().clone();
            let Some(yo) = yo else { return };

            // Lo que no tiene nada enchufado —una salida HDMI sin monitor, un
            // microfono de auriculares sin auriculares—. `wpctl` lo lista igual y
            // `set-default` sobre ello no hace nada ni falla.
            let fuera = audio::no_disponibles();

            let d = match audio::devices() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("hfs: no pude listar los dispositivos: {e}");
                    estado.set_text("sin PipeWire");
                    let v = gtk::Label::new(Some("wpctl no contesta"));
                    v.add_css_class("aviso");
                    v.set_xalign(0.0);
                    cuerpo.append(&v);
                    return;
                }
            };

            let utiles = |v: &[hfs_core::audio::Device]| {
                v.iter().filter(|x| !fuera.contains(&x.nombre)).count()
            };
            let plural = |n: usize, uno: &str, varias: &str| {
                if n == 1 {
                    format!("1 {uno}")
                } else {
                    format!("{n} {varias}")
                }
            };
            estado.set_text(&format!(
                "{} · {}",
                plural(utiles(&d.salidas), "salida", "salidas"),
                plural(utiles(&d.entradas), "entrada", "entradas")
            ));

            cuerpo.append(&seccion("Salida"));
            if d.salidas.is_empty() {
                cuerpo.append(&seccion("  ninguna"));
            }
            for s in &d.salidas {
                let ok = !fuera.contains(&s.nombre);
                cuerpo.append(&fila(s, "audio-speakers-symbolic", ok, &yo));
            }

            cuerpo.append(&widgets::sep());
            cuerpo.append(&seccion("Entrada"));
            if d.entradas.is_empty() {
                cuerpo.append(&seccion("  ninguna"));
            }
            for e in &d.entradas {
                let ok = !fuera.contains(&e.nombre);
                cuerpo.append(&fila(e, "audio-input-microphone-symbolic", ok, &yo));
            }
        })
    };
    *hueco.borrow_mut() = Some(refrescar.clone());

    (caja.upcast(), refrescar, None)
}
