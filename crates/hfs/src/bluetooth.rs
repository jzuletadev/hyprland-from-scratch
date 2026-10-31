// ~/hyprland-from-scratch/crates/hfs/src/bluetooth.rs
//
// Bluetooth por D-Bus contra BlueZ.
//
// La diferencia de fondo con la v1 es una sola llamada: `GetManagedObjects` devuelve
// el adaptador y TODOS los dispositivos con TODAS sus propiedades de golpe. El TUI de
// la v1 hacia un `bluetoothctl info` por dispositivo, asi que con ocho emparejados
// eran nueve procesos en fila y hasta ocho segundos de ventana congelada en cada
// refresco. Aqui es una ida y vuelta.
//
// Ademas se ve lo que antes se cambiaba a ciegas: si el adaptador esta encendido, si
// esta buscando, y la bateria del dispositivo cuando la publica.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::dbus;
use hfs_ui::widgets;

const BLUEZ: &str = "org.bluez";
const OM: &str = "org.freedesktop.DBus.ObjectManager";
const ADAPTER: &str = "org.bluez.Adapter1";
const DEVICE: &str = "org.bluez.Device1";
const BATTERY: &str = "org.bluez.Battery1";

#[derive(Clone)]
pub struct Dispositivo {
    pub ruta: String,
    pub nombre: String,
    pub conectado: bool,
    pub emparejado: bool,
    /// El tipo real que publica BlueZ: input-mouse, audio-headset, phone...
    pub icono: String,
    pub bateria: Option<u8>,
}

pub struct Adaptador {
    pub ruta: String,
    pub encendido: bool,
    pub buscando: bool,
}

pub struct Bluetooth {
    raiz: gio::DBusProxy,
}

impl Bluetooth {
    pub fn nuevo() -> Option<Self> {
        let raiz = dbus::system(BLUEZ, "/", OM).ok()?;
        dbus::vivo(&raiz).then_some(Self { raiz })
    }

    /// El adaptador y los dispositivos, en UNA llamada.
    pub fn estado(&self) -> (Option<Adaptador>, Vec<Dispositivo>) {
        let Ok(r) = dbus::call(&self.raiz, "GetManagedObjects", None) else {
            return (None, Vec::new());
        };
        if r.n_children() == 0 {
            return (None, Vec::new());
        }
        let objetos = r.child_value(0);

        let mut adaptador = None;
        let mut dispositivos = Vec::new();

        for i in 0..objetos.n_children() {
            let par = objetos.child_value(i);
            let Some(ruta) = par.child_value(0).str().map(|s| s.to_string()) else {
                continue;
            };
            // `a{sa{sv}}`: un diccionario de interfaces. VariantDict no vale aqui.
            let ifaces = par.child_value(1);

            if let Some(a) = dbus::dict_get(&ifaces, ADAPTER) {
                let d = glib::VariantDict::new(Some(&a));
                adaptador = Some(Adaptador {
                    ruta: ruta.clone(),
                    encendido: b(&d, "Powered"),
                    buscando: b(&d, "Discovering"),
                });
            }
            if let Some(dev) = dbus::dict_get(&ifaces, DEVICE) {
                let d = glib::VariantDict::new(Some(&dev));
                let nombre = s(&d, "Alias");
                if nombre.is_empty() {
                    continue;
                }
                let bateria = dbus::dict_get(&ifaces, BATTERY)
                    .and_then(|bt| dbus::dict_get(&bt, "Percentage"))
                    .and_then(|v| v.get::<u8>());
                dispositivos.push(Dispositivo {
                    ruta,
                    nombre,
                    conectado: b(&d, "Connected"),
                    emparejado: b(&d, "Paired"),
                    icono: s(&d, "Icon"),
                    bateria,
                });
            }
        }

        // Conectados primero, luego emparejados, y lo demas por nombre.
        dispositivos.sort_by(|a, c| {
            c.conectado
                .cmp(&a.conectado)
                .then(c.emparejado.cmp(&a.emparejado))
                .then(a.nombre.cmp(&c.nombre))
        });
        (adaptador, dispositivos)
    }

    fn dev(&self, ruta: &str) -> Option<gio::DBusProxy> {
        dbus::system(BLUEZ, ruta, DEVICE).ok()
    }

    pub fn alternar(&self, d: &Dispositivo) -> Result<(), String> {
        let Some(p) = self.dev(&d.ruta) else {
            return Err("no encuentro el dispositivo".into());
        };
        // Un dispositivo sin emparejar hay que emparejarlo primero; BlueZ lo rechaza
        // si no, y el TUI de la v1 no distinguia los dos casos.
        let metodo = match (d.conectado, d.emparejado) {
            (true, _) => "Disconnect",
            (false, true) => "Connect",
            (false, false) => "Pair",
        };
        dbus::call(&p, metodo, None)
            .map(|_| ())
            .map_err(|e| limpiar(&e.to_string()))
    }

    pub fn encender(&self, a: &Adaptador, on: bool) -> Result<(), String> {
        let Ok(p) = dbus::system(BLUEZ, &a.ruta, ADAPTER) else {
            return Err("no encuentro el adaptador".into());
        };
        dbus::set_prop(&p, ADAPTER, "Powered", &on.to_variant())
            .map_err(|e| limpiar(&e.to_string()))
    }

    pub fn buscar(&self, a: &Adaptador, on: bool) -> Result<(), String> {
        let Ok(p) = dbus::system(BLUEZ, &a.ruta, ADAPTER) else {
            return Err("no encuentro el adaptador".into());
        };
        dbus::call(&p, if on { "StartDiscovery" } else { "StopDiscovery" }, None)
            .map(|_| ())
            .map_err(|e| limpiar(&e.to_string()))
    }
}

fn b(d: &glib::VariantDict, k: &str) -> bool {
    d.lookup_value(k, None)
        .and_then(|v| v.get::<bool>())
        .unwrap_or(false)
}

fn s(d: &glib::VariantDict, k: &str) -> String {
    d.lookup_value(k, None)
        .and_then(|v| v.get::<String>())
        .unwrap_or_default()
}

fn limpiar(e: &str) -> String {
    e.rsplit(": ").next().unwrap_or(e).trim().to_string()
}

/// El icono del tema que le toca a cada tipo que publica BlueZ.
fn icono_de(tipo: &str) -> &'static str {
    match tipo {
        "input-mouse" => "input-mouse-symbolic",
        "input-keyboard" => "input-keyboard-symbolic",
        "input-gaming" => "input-gaming-symbolic",
        "audio-headset" | "audio-headphones" => "audio-headphones-symbolic",
        "audio-card" | "audio-speakers" => "audio-speakers-symbolic",
        "phone" => "phone-symbolic",
        "computer" => "computer-symbolic",
        _ => "bluetooth-symbolic",
    }
}

// ---------------------------------------------------------------- el panel

pub fn panel(palette: &Palette) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let _ = palette;
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Bluetooth");
    let encender = gtk::Button::new();
    encender.add_css_class("bic");
    encender.set_has_frame(false);
    let img_power = gtk::Image::from_icon_name("system-shutdown-symbolic");
    img_power.set_pixel_size(14);
    encender.set_child(Some(&img_power));
    let buscar = gtk::Button::new();
    buscar.add_css_class("bic");
    buscar.set_has_frame(false);
    let img_buscar = gtk::Image::from_icon_name("view-refresh-symbolic");
    img_buscar.set_pixel_size(14);
    buscar.set_child(Some(&img_buscar));
    cab.append(&encender);
    cab.append(&buscar);

    let lista = gtk::Box::new(gtk::Orientation::Vertical, 3);
    let aviso = gtk::Label::new(None);
    aviso.add_css_class("aviso");
    aviso.set_xalign(0.0);
    aviso.set_wrap(true);
    aviso.set_visible(false);
    let pie = gtk::Label::new(Some("tocar conecta o desconecta · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&lista);
    caja.append(&aviso);
    caja.append(&pie);

    let bt = Rc::new(Bluetooth::nuevo());
    let adaptador: Rc<RefCell<Option<Adaptador>>> = Rc::new(RefCell::new(None));

    let refrescar: Rc<dyn Fn()> = {
        let (lista, estado, aviso, bt, adaptador) = (
            lista.clone(),
            estado.clone(),
            aviso.clone(),
            bt.clone(),
            adaptador.clone(),
        );
        Rc::new(move || {
            let Some(b) = bt.as_ref() else {
                estado.set_text("sin BlueZ");
                return;
            };
            let (ad, devs) = b.estado();
            estado.set_text(&match &ad {
                Some(a) if a.buscando => "buscando…".to_string(),
                Some(a) if a.encendido => format!("{} dispositivos", devs.len()),
                Some(_) => "apagado".to_string(),
                None => "sin adaptador".to_string(),
            });
            *adaptador.borrow_mut() = ad;

            widgets::clear(&lista);
            if devs.is_empty() {
                let l = gtk::Label::new(Some("ningún dispositivo"));
                l.add_css_class("sempty");
                l.set_xalign(0.0);
                lista.append(&l);
                return;
            }
            for d in devs.into_iter().take(7) {
                let fila = widgets::row();
                if d.conectado {
                    fila.add_css_class("on");
                }
                let img = gtk::Image::from_icon_name(icono_de(&d.icono));
                img.set_pixel_size(15);
                let nombre = widgets::ellipsized(Some(&d.nombre), "");
                nombre.set_hexpand(true);
                let mut detalle = Vec::new();
                if d.conectado {
                    detalle.push("conectado".to_string());
                } else if d.emparejado {
                    detalle.push("emparejado".to_string());
                }
                if let Some(p) = d.bateria {
                    detalle.push(format!("{p} %"));
                }
                let k = gtk::Label::new(Some(&detalle.join(" · ")));
                k.add_css_class("k");
                fila.append(&img);
                fila.append(&nombre);
                fila.append(&k);

                let click = gtk::GestureClick::new();
                {
                    let (bt, aviso) = (bt.clone(), aviso.clone());
                    click.connect_released(move |_, _, _, _| {
                        let Some(b) = bt.as_ref() else { return };
                        aviso.set_visible(false);
                        if let Err(e) = b.alternar(&d) {
                            aviso.set_text(&e);
                            aviso.set_visible(true);
                        }
                    });
                }
                fila.add_controller(click);
                lista.append(&fila);
            }
        })
    };

    {
        let (bt, adaptador, aviso, refrescar) =
            (bt.clone(), adaptador.clone(), aviso.clone(), refrescar.clone());
        encender.connect_clicked(move |_| {
            let (Some(b), Some(a)) = (bt.as_ref(), adaptador.borrow().as_ref().map(|a| (a.ruta.clone(), a.encendido)))
            else {
                return;
            };
            let a = Adaptador {
                ruta: a.0,
                encendido: a.1,
                buscando: false,
            };
            if let Err(e) = b.encender(&a, !a.encendido) {
                aviso.set_text(&e);
                aviso.set_visible(true);
            }
            refrescar();
        });
    }
    {
        let (bt, adaptador, aviso, refrescar) =
            (bt.clone(), adaptador.clone(), aviso.clone(), refrescar.clone());
        buscar.connect_clicked(move |_| {
            let (Some(b), Some(a)) = (
                bt.as_ref(),
                adaptador
                    .borrow()
                    .as_ref()
                    .map(|a| (a.ruta.clone(), a.buscando)),
            ) else {
                return;
            };
            let a = Adaptador {
                ruta: a.0,
                encendido: true,
                buscando: a.1,
            };
            if let Err(e) = b.buscar(&a, !a.buscando) {
                aviso.set_text(&e);
                aviso.set_visible(true);
            }
            refrescar();
        });
    }

    (caja.upcast(), refrescar, None)
}
