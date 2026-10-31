// ~/hyprland-from-scratch/crates/hfs/src/red.rs
//
// Wi-Fi por D-Bus contra NetworkManager, no por `nmcli`.
//
// Aqui es donde mas se nota el cambio, y no es cosmetico:
//
//   EL SSID LLEGA COMO BYTES (`ay`), no como una linea de texto que hay que partir.
//   El TUI de la v1 hacia `line.split(":")` sobre la salida de nmcli, asi que
//   cualquier red con dos puntos en el nombre lo rompia. Aqui no hay nada que
//   partir.
//
//   LA SEGURIDAD SE DECODIFICA de `RsnFlags`/`WpaFlags` en vez de adivinarse, asi
//   que se distingue WPA3 de WPA2, y abierta de WEP.
//
//   LA BANDA sale de `Frequency`, que es un dato que nmcli no da de forma directa.
//
//   CONECTAR DEVUELVE UN ERROR DE VERDAD. La v1 lanzaba `nmcli connect` y hacia
//   `break` al instante: la ventana se congelaba y luego desaparecia, y una
//   contraseña mal puesta no decia nada. Aqui la llamada responde, y si falla el
//   mensaje de NetworkManager se ve en el panel.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::dbus;
use hfs_ui::widgets;

const NM: &str = "org.freedesktop.NetworkManager";
const NM_RUTA: &str = "/org/freedesktop/NetworkManager";
const NM_IFACE: &str = "org.freedesktop.NetworkManager";
const DEV: &str = "org.freedesktop.NetworkManager.Device";
const WIFI: &str = "org.freedesktop.NetworkManager.Device.Wireless";
const AP: &str = "org.freedesktop.NetworkManager.AccessPoint";
const SETTINGS_RUTA: &str = "/org/freedesktop/NetworkManager/Settings";
const SETTINGS: &str = "org.freedesktop.NetworkManager.Settings";

/// NM_DEVICE_TYPE_WIFI.
const TIPO_WIFI: u32 = 2;
/// NM_802_11_AP_FLAGS_PRIVACY.
const PRIVACY: u32 = 0x1;
/// NM_802_11_AP_SEC_KEY_MGMT_802_1X y _SAE.
const KEY_MGMT_8021X: u32 = 0x200;
const KEY_MGMT_SAE: u32 = 0x400;

#[derive(Clone)]
pub struct Punto {
    pub ssid: String,
    pub strength: u8,
    pub freq_mhz: u32,
    pub seguridad: Seguridad,
    pub ruta: String,
    pub activo: bool,
    /// La conexion guardada que le corresponde, si la hay.
    pub guardada: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Seguridad {
    Abierta,
    Wep,
    Wpa2,
    Wpa3,
    Empresa,
}

impl Seguridad {
    pub fn texto(self) -> &'static str {
        match self {
            Seguridad::Abierta => "abierta",
            Seguridad::Wep => "WEP",
            Seguridad::Wpa2 => "WPA2",
            Seguridad::Wpa3 => "WPA3",
            Seguridad::Empresa => "empresa",
        }
    }
    fn necesita_clave(self) -> bool {
        !matches!(self, Seguridad::Abierta)
    }
}

impl Punto {
    pub fn banda(&self) -> &'static str {
        match self.freq_mhz {
            0..=2999 => "2.4 GHz",
            3000..=5899 => "5 GHz",
            _ => "6 GHz",
        }
    }
    /// Un icono de tema segun la fuerza, para no dibujar barras a mano.
    fn icono(&self) -> &'static str {
        match self.strength {
            0..=25 => "network-wireless-signal-weak-symbolic",
            26..=50 => "network-wireless-signal-ok-symbolic",
            51..=75 => "network-wireless-signal-good-symbolic",
            _ => "network-wireless-signal-excellent-symbolic",
        }
    }
}

/// El estado del Wi-Fi: el dispositivo, lo que se ve y lo que esta guardado.
pub struct Wifi {
    nm: gio::DBusProxy,
    dev_ruta: String,
    dev: gio::DBusProxy,
    wifi: gio::DBusProxy,
    /// La ultima ruta de punto de acceso y su SSID. Leer el nombre cuesta crear un
    /// proxy, y la cabecera lo pide una vez por segundo: mientras no cambies de red,
    /// no hay que volver a preguntarlo.
    cache: RefCell<(String, Option<String>)>,
}

impl Wifi {
    pub fn nueva() -> Option<Self> {
        let nm = dbus::system(NM, NM_RUTA, NM_IFACE).ok()?;
        if !dbus::vivo(&nm) {
            return None;
        }
        let rutas = rutas_de(&dbus::call(&nm, "GetDevices", None).ok()?)?;
        for r in rutas {
            let Ok(dev) = dbus::system(NM, &r, DEV) else {
                continue;
            };
            if dbus::prop::<u32>(&dev, "DeviceType") != Some(TIPO_WIFI) {
                continue;
            }
            let Ok(wifi) = dbus::system(NM, &r, WIFI) else {
                continue;
            };
            return Some(Self {
                nm,
                dev_ruta: r,
                dev,
                wifi,
                cache: RefCell::new((String::new(), None)),
            });
        }
        None
    }

    /// El SSID al que estamos conectados ahora mismo.
    pub fn ssid_activo(&self) -> Option<String> {
        let ap = dbus::prop::<String>(&self.wifi, "ActiveAccessPoint").unwrap_or_default();
        {
            let c = self.cache.borrow();
            if c.0 == ap {
                return c.1.clone();
            }
        }
        let ssid = if ap == "/" || ap.is_empty() {
            None
        } else {
            dbus::system(NM, &ap, AP).ok().and_then(|p| ssid_de(&p))
        };
        *self.cache.borrow_mut() = (ap, ssid.clone());
        ssid
    }

    /// Pide un barrido. No bloquea: NetworkManager contesta al momento y va
    /// publicando los puntos segun los encuentra.
    pub fn rescan(&self) {
        let vacio: std::collections::HashMap<String, glib::Variant> = Default::default();
        if let Err(e) = dbus::call(&self.wifi, "RequestScan", Some(&(vacio,).to_variant())) {
            // Pedir dos barridos seguidos da error y no es un problema que haya que
            // contarle a nadie: significa que ya esta barriendo.
            let msg = e.to_string();
            if !msg.contains("Scanning not allowed") {
                eprintln!("hfs: no pude pedir un barrido: {e}");
            }
        }
    }

    /// Las redes visibles, sin repetir SSID (se queda con la mas fuerte de cada una)
    /// y ordenadas por señal, con la activa siempre arriba.
    pub fn puntos(&self) -> Vec<Punto> {
        let guardadas = self.conexiones_guardadas();
        let activa = dbus::prop::<String>(&self.wifi, "ActiveAccessPoint").unwrap_or_default();

        let Some(rutas) = dbus::call(&self.wifi, "GetAllAccessPoints", None)
            .ok()
            .as_ref()
            .and_then(rutas_de)
        else {
            return Vec::new();
        };

        let mut v: Vec<Punto> = Vec::new();
        for r in rutas {
            let Ok(p) = dbus::system(NM, &r, AP) else {
                continue;
            };
            let Some(ssid) = ssid_de(&p) else { continue };
            if ssid.is_empty() {
                continue;
            }
            let flags = dbus::prop::<u32>(&p, "Flags").unwrap_or(0);
            let wpa = dbus::prop::<u32>(&p, "WpaFlags").unwrap_or(0);
            let rsn = dbus::prop::<u32>(&p, "RsnFlags").unwrap_or(0);
            let punto = Punto {
                guardada: guardadas.get(&ssid).cloned(),
                activo: r == activa,
                strength: dbus::prop::<u8>(&p, "Strength").unwrap_or(0),
                freq_mhz: dbus::prop::<u32>(&p, "Frequency").unwrap_or(0),
                seguridad: seguridad_de(flags, wpa, rsn),
                ruta: r,
                ssid,
            };
            // Un mismo SSID aparece una vez por banda y por punto de acceso; en una
            // lista para elegir eso es ruido, asi que se queda la mejor.
            match v.iter_mut().find(|x| x.ssid == punto.ssid) {
                Some(x) if punto.activo || punto.strength > x.strength => *x = punto,
                Some(_) => {}
                None => v.push(punto),
            }
        }
        v.sort_by(|a, b| {
            b.activo
                .cmp(&a.activo)
                .then(b.strength.cmp(&a.strength))
                .then(a.ssid.cmp(&b.ssid))
        });
        v
    }

    /// SSID -> ruta de la conexion guardada.
    fn conexiones_guardadas(&self) -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        let Ok(s) = dbus::system(NM, SETTINGS_RUTA, SETTINGS) else {
            return m;
        };
        let Some(rutas) = dbus::call(&s, "ListConnections", None)
            .ok()
            .as_ref()
            .and_then(rutas_de)
        else {
            return m;
        };
        for r in rutas {
            let Ok(c) = dbus::system(NM, &r, "org.freedesktop.NetworkManager.Settings.Connection")
            else {
                continue;
            };
            let Ok(resp) = dbus::call(&c, "GetSettings", None) else {
                continue;
            };
            if resp.n_children() == 0 {
                continue;
            }
            // `GetSettings` devuelve `a{sa{sv}}`: un diccionario de secciones. No
            // se puede meter en un VariantDict, que solo sabe de `a{sv}` y aborta.
            let Some(w) = dbus::dict_get(&resp.child_value(0), "802-11-wireless") else {
                continue;
            };
            if let Some(bytes) = dbus::dict_get(&w, "ssid").and_then(|v| v.get::<Vec<u8>>()) {
                m.insert(String::from_utf8_lossy(&bytes).to_string(), r);
            }
        }
        m
    }

    /// Conecta. Si la red ya estaba guardada se reactiva; si no, se crea la conexion
    /// con la clave que se acaba de escribir.
    pub fn conectar(&self, p: &Punto, clave: Option<&str>) -> Result<(), String> {
        // Los argumentos se montan con `tuple_from_iter` y NO con `.to_variant()`
        // sobre una tupla de Rust.
        //
        // Esa diferencia es la que rompia el Wi-Fi entero. `(a, b, c).to_variant()`
        // con `a`, `b` y `c` ya `Variant` **mete cada uno en una caja**, asi que lo
        // que sale es `(vvv)` y NetworkManager contesta «does not match expected type
        // "(ooo)"». Pasaba en los dos caminos, el de red guardada y el de red nueva,
        // asi que conectar no funcionaba nunca.
        let r = match &p.guardada {
            Some(c) => dbus::call(
                &self.nm,
                "ActivateConnection",
                Some(&glib::Variant::tuple_from_iter([
                    ruta(c),
                    ruta(&self.dev_ruta),
                    ruta(&p.ruta),
                ])),
            ),
            None => dbus::call(
                &self.nm,
                "AddAndActivateConnection",
                Some(&glib::Variant::tuple_from_iter([
                    ajustes_nuevos(&p.ssid, p.seguridad, clave),
                    ruta(&self.dev_ruta),
                    ruta(&p.ruta),
                ])),
            ),
        };
        r.map(|_| ()).map_err(|e| limpiar_error(&e.to_string()))
    }

    /// Avisa cada vez que el dispositivo cambia de estado, **venga de donde venga**:
    /// del panel, de `nmcli` o de que la red se caiga sola.
    ///
    /// Sin esto el panel solo se enteraba al abrirlo: conectabas, funcionaba, y la
    /// cabecera se quedaba diciendo «desconectado» hasta que lo cerrabas y lo volvias
    /// a abrir. Es reactivo y no un sondeo, que es la regla de la casa.
    pub fn al_cambiar_estado(&self, f: impl Fn() + 'static) {
        // `connect_local` y no el enganche tipado: ese exige `Send + Sync` y aqui
        // dentro hay `Rc` y widgets de GTK. Ver `notificaciones.rs`.
        self.dev.connect_local("g-signal", false, move |args| {
            // (proxy, remitente, nombre de la señal, argumentos)
            let nombre = args.get(2).and_then(|v| v.get::<String>().ok());
            if nombre.as_deref() == Some("StateChanged") {
                f();
            }
            None
        });
    }

    pub fn desconectar(&self) -> Result<(), String> {
        dbus::call(&self.dev, "Disconnect", None)
            .map(|_| ())
            .map_err(|e| limpiar_error(&e.to_string()))
    }
}

/// Los ajustes minimos de una red nueva. NetworkManager rellena el resto.
fn ajustes_nuevos(ssid: &str, seg: Seguridad, clave: Option<&str>) -> glib::Variant {
    let conn = dict(&[
        ("id", ssid.to_variant()),
        ("type", "802-11-wireless".to_variant()),
        ("autoconnect", true.to_variant()),
    ]);
    let wireless = dict(&[
        ("ssid", ssid.as_bytes().to_vec().to_variant()),
        ("mode", "infrastructure".to_variant()),
    ]);
    let auto = dict(&[("method", "auto".to_variant())]);

    let mut secciones: Vec<(&str, glib::Variant)> = vec![
        ("connection", conn),
        ("802-11-wireless", wireless),
        ("ipv4", auto.clone()),
        ("ipv6", auto),
    ];
    if seg.necesita_clave() {
        let km = if seg == Seguridad::Wpa3 { "sae" } else { "wpa-psk" };
        secciones.push((
            "802-11-wireless-security",
            dict(&[
                ("key-mgmt", km.to_variant()),
                ("psk", clave.unwrap_or_default().to_variant()),
            ]),
        ));
    }
    dbus::dict_of_dicts(&secciones)
}

fn dict(pares: &[(&str, glib::Variant)]) -> glib::Variant {
    let d = glib::VariantDict::new(None);
    for (k, v) in pares {
        d.insert_value(k, v);
    }
    d.end()
}

/// Una ruta de objeto de D-Bus. Tiene su propio tipo en GVariant: pasar una cadena
/// donde se espera una `o` no cuela, porque GVariant va tipado de verdad.
fn ruta(p: &str) -> glib::Variant {
    glib::variant::ObjectPath::try_from(p.to_string())
        .map(glib::Variant::from)
        // Si NetworkManager nos ha dado algo que no es una ruta, el error util es el
        // que devuelva la llamada, no un panico aqui.
        .unwrap_or_else(|_| glib::Variant::from("/"))
}

/// D-Bus pone un prefijo larguisimo delante del mensaje util; aqui solo estorba.
fn limpiar_error(e: &str) -> String {
    e.rsplit(": ").next().unwrap_or(e).trim().to_string()
}

fn rutas_de(v: &glib::Variant) -> Option<Vec<String>> {
    if v.n_children() == 0 {
        return None;
    }
    let arr = v.child_value(0);
    Some(
        (0..arr.n_children())
            .filter_map(|i| arr.child_value(i).str().map(|s| s.to_string()))
            .collect(),
    )
}

/// El SSID viene como array de bytes. Esto es lo que mata el fallo de los `:`.
fn ssid_de(p: &gio::DBusProxy) -> Option<String> {
    let bytes = p.cached_property("Ssid")?.get::<Vec<u8>>()?;
    Some(String::from_utf8_lossy(&bytes).to_string())
}

fn seguridad_de(flags: u32, wpa: u32, rsn: u32) -> Seguridad {
    if rsn & KEY_MGMT_SAE != 0 {
        Seguridad::Wpa3
    } else if (rsn | wpa) & KEY_MGMT_8021X != 0 {
        Seguridad::Empresa
    } else if rsn != 0 || wpa != 0 {
        Seguridad::Wpa2
    } else if flags & PRIVACY != 0 {
        Seguridad::Wep
    } else {
        Seguridad::Abierta
    }
}

// ---------------------------------------------------------------- el panel

/// El panel que se abre desde el radio de Wi-Fi de la rueda.
pub fn panel(palette: &Palette) -> (gtk::Widget, Rc<dyn Fn()>, Option<crate::Teclas>) {
    let _ = palette;
    let caja = gtk::Box::new(gtk::Orientation::Vertical, 8);
    caja.add_css_class("invocado");

    let (cab, estado) = widgets::header("Wi-Fi");
    let buscar = gtk::Button::new();
    buscar.add_css_class("bic");
    buscar.set_has_frame(false);
    let img = gtk::Image::from_icon_name("view-refresh-symbolic");
    img.set_pixel_size(14);
    buscar.set_child(Some(&img));
    cab.append(&buscar);

    let lista = gtk::Box::new(gtk::Orientation::Vertical, 3);

    let clave = gtk::Entry::new();
    clave.set_visibility(false);
    clave.set_placeholder_text(Some("contraseña"));
    clave.add_css_class("clave");
    let clave_fila = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    clave_fila.append(&clave);
    clave_fila.set_visible(false);

    let aviso = gtk::Label::new(None);
    aviso.add_css_class("aviso");
    aviso.set_xalign(0.0);
    aviso.set_wrap(true);
    aviso.set_visible(false);

    let pie = gtk::Label::new(Some("⏎ conectar · esc cerrar"));
    pie.add_css_class("foot");
    pie.set_xalign(0.0);

    caja.append(&cab);
    caja.append(&lista);
    caja.append(&clave_fila);
    caja.append(&aviso);
    caja.append(&pie);

    let wifi = Rc::new(Wifi::nueva());
    let pendiente: Rc<RefCell<Option<Punto>>> = Rc::new(RefCell::new(None));

    let refrescar: Rc<dyn Fn()> = {
        let (lista, estado, aviso, clave_fila, clave) = (
            lista.clone(),
            estado.clone(),
            aviso.clone(),
            clave_fila.clone(),
            clave.clone(),
        );
        let (wifi, pendiente) = (wifi.clone(), pendiente.clone());
        Rc::new(move || {
            let Some(w) = wifi.as_ref() else {
                estado.set_text("sin NetworkManager");
                return;
            };
            estado.set_text(&match w.ssid_activo() {
                Some(s) => format!("conectado a {s}"),
                None => "desconectado".into(),
            });

            widgets::clear(&lista);
            let puntos = w.puntos();
            if puntos.is_empty() {
                let l = gtk::Label::new(Some("no se ve ninguna red"));
                l.add_css_class("sempty");
                l.set_xalign(0.0);
                lista.append(&l);
                return;
            }
            for p in puntos.into_iter().take(7) {
                let fila = widgets::row();
                if p.activo {
                    fila.add_css_class("on");
                }
                let img = gtk::Image::from_icon_name(p.icono());
                img.set_pixel_size(15);
                let nombre = widgets::ellipsized(Some(&p.ssid), "");
                nombre.set_hexpand(true);
                let detalle = gtk::Label::new(Some(&format!(
                    "{} · {}{}",
                    p.banda(),
                    p.seguridad.texto(),
                    if p.guardada.is_some() { " · guardada" } else { "" }
                )));
                detalle.add_css_class("k");
                fila.append(&img);
                fila.append(&nombre);
                fila.append(&detalle);

                let click = gtk::GestureClick::new();
                {
                    let (wifi, pendiente, aviso, clave_fila, clave) = (
                        wifi.clone(),
                        pendiente.clone(),
                        aviso.clone(),
                        clave_fila.clone(),
                        clave.clone(),
                    );
                    let p = p.clone();
                    click.connect_released(move |_, _, _, _| {
                        let Some(w) = wifi.as_ref() else { return };
                        aviso.set_visible(false);
                        if p.activo {
                            if let Err(e) = w.desconectar() {
                                aviso.set_text(&e);
                                aviso.set_visible(true);
                            }
                            return;
                        }
                        // Una red nueva y con clave es lo unico que no se puede
                        // resolver con un solo clic.
                        if p.guardada.is_none() && p.seguridad.necesita_clave() {
                            *pendiente.borrow_mut() = Some(p.clone());
                            clave.set_text("");
                            clave_fila.set_visible(true);
                            clave.grab_focus();
                            return;
                        }
                        if let Err(e) = w.conectar(&p, None) {
                            aviso.set_text(&e);
                            aviso.set_visible(true);
                        }
                    });
                }
                fila.add_controller(click);
                lista.append(&fila);
            }
            let _ = &pendiente;
        })
    };

    // Seguir al dispositivo: conectar desde el panel, desde nmcli o perder la red se
    // ven los tres igual.
    if let Some(w) = wifi.as_ref() {
        let refrescar = refrescar.clone();
        w.al_cambiar_estado(move || refrescar());
    }

    {
        let (wifi, refrescar) = (wifi.clone(), refrescar.clone());
        buscar.connect_clicked(move |_| {
            if let Some(w) = wifi.as_ref() {
                w.rescan();
            }
            refrescar();
        });
    }
    {
        let (wifi, pendiente, aviso, clave_fila, refrescar) = (
            wifi.clone(),
            pendiente.clone(),
            aviso.clone(),
            clave_fila.clone(),
            refrescar.clone(),
        );
        clave.connect_activate(move |e| {
            let Some(w) = wifi.as_ref() else { return };
            let p = pendiente.borrow_mut().take();
            clave_fila.set_visible(false);
            if let Some(p) = p
                && let Err(err) = w.conectar(&p, Some(&e.text()))
            {
                aviso.set_text(&err);
                aviso.set_visible(true);
            }
            refrescar();
        });
    }

    (caja.upcast(), refrescar, None)
}


