// ~/hyprland-from-scratch/crates/hfs/src/sni.rs
//
// StatusNotifierItem: el protocolo de la bandeja, por los dos lados.
//
//   HAY QUE SER TAMBIEN EL «WATCHER», NO SOLO EL QUE PINTA. El protocolo tiene tres
//   papeles: los programas publican un *item*, un *host* los pinta, y un *watcher*
//   hace de registro entre ambos. Casi todos los escritorios dan por hecho que el
//   watcher ya existe porque lo trae el panel del sistema. Aqui lo traia Waybar, y al
//   quitarla **no quedo ninguno**: comprobado, nadie posee
//   `org.kde.StatusNotifierWatcher` en este bus. Asi que la bandeja tiene que
//   registrarse ella.
//
//   UN PROGRAMA QUE ARRANCO SIN WATCHER NO SE ENTERA SOLO. Los items preguntan al
//   arrancar; si no habia nadie, muchos se rinden y no vuelven a intentarlo. Por eso
//   `reenganchar()` existe: cuando aparece el watcher, a los nuestros se les da un
//   empujon.
//
// Todo va por GIO, como el resto del escritorio: `register_object` exporta la
// interfaz y `bus_own_name` toma el nombre. Ni zbus ni un hilo aparte.

use std::cell::RefCell;
use std::rc::Rc;

/// A quien avisar cuando la lista de iconos cambia: una bandeja por pantalla.
type Suscriptores = Rc<RefCell<Vec<Rc<dyn Fn()>>>>;

use hfs_ui::gtk::{gio, glib};
use hfs_ui::gtk::prelude::*;

const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const ITEM_IFACE: &str = "org.kde.StatusNotifierItem";

/// La interfaz que exportamos. Es la de KDE, que es la que usan de verdad los
/// programas, no la de freedesktop que nunca se estandarizo.
const XML: &str = r#"
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg type="s" direction="in" name="service"/>
    </method>
    <method name="RegisterStatusNotifierHost">
      <arg type="s" direction="in" name="service"/>
    </method>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
    <property name="ProtocolVersion" type="i" access="read"/>
    <signal name="StatusNotifierItemRegistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierItemUnregistered">
      <arg type="s" name="service"/>
    </signal>
    <signal name="StatusNotifierHostRegistered"/>
    <signal name="StatusNotifierHostUnregistered"/>
  </interface>
</node>
"#;

/// Un icono de bandeja, ya resuelto a lo que hace falta para pintarlo.
#[derive(Clone, Debug)]
pub struct Item {
    pub bus: String,
    pub ruta: String,
    pub id: String,
    pub titulo: String,
    /// `Active`, `Passive` o `NeedsAttention`.
    pub estado: String,
    pub icono: String,
    /// Carpeta propia de iconos, si el programa trae los suyos.
    pub tema_propio: Option<String>,
    /// Mapa de bits en ARGB32, cuando no hay nombre de icono que valga.
    pub pixmap: Option<(i32, i32, Vec<u8>)>,
}

impl Item {
    /// Lo que se enseña debajo del icono o en el tooltip.
    pub fn etiqueta(&self) -> &str {
        if !self.titulo.trim().is_empty() {
            &self.titulo
        } else {
            &self.id
        }
    }
}

/// Parte un `service` del protocolo en (bus, ruta).
///
/// Viene en dos formas: un nombre de bus suelto, o una ruta de objeto, y entonces el
/// bus es quien llamo. Hay programas que mandan `bus/ruta` de una pieza.
fn partir(service: &str, quien_llama: Option<&str>) -> (String, String) {
    // Este orden importa: una ruta suelta («/StatusNotifierItem») tambien casa con
    // `split_once('/')`, y da un bus vacio. Mirarlo al reves dejaba fuera de la
    // bandeja a todo lo basado en libappindicator, que es lo que manda solo la ruta.
    if service.starts_with('/') {
        return (
            quien_llama.unwrap_or_default().to_string(),
            service.to_string(),
        );
    }
    if let Some((bus, ruta)) = service.split_once('/') {
        return (bus.to_string(), format!("/{ruta}"));
    }
    (service.to_string(), "/StatusNotifierItem".to_string())
}

pub struct Bandeja {
    conn: gio::DBusConnection,
    /// Los servicios registrados, en orden de llegada.
    servicios: Rc<RefCell<Vec<String>>>,
    al_cambiar: Suscriptores,
    somos_el_watcher: Rc<std::cell::Cell<bool>>,
    /// Hay que guardarla: al soltarla se cancela la suscripcion y dejariamos de
    /// enterarnos de que un programa se ha ido.
    _caidas: RefCell<Option<gio::SignalSubscription>>,
}

impl Bandeja {
    /// Arranca el watcher y el host. Devuelve `None` si no hay bus de sesion, que no
    /// deberia pasar nunca pero tampoco debe tumbar el escritorio.
    pub fn new() -> Option<Rc<Self>> {
        let conn = gio::bus_get_sync(gio::BusType::Session, gio::Cancellable::NONE).ok()?;
        let yo = Rc::new(Bandeja {
            conn,
            servicios: Rc::new(RefCell::new(Vec::new())),
            al_cambiar: Rc::new(RefCell::new(Vec::new())),
            somos_el_watcher: Rc::new(std::cell::Cell::new(false)),
            _caidas: RefCell::new(None),
        });
        yo.exportar();
        yo.tomar_nombres();
        yo.vigilar_caidas();
        Some(yo)
    }

    pub fn cuando_cambie(&self, f: Rc<dyn Fn()>) {
        self.al_cambiar.borrow_mut().push(f);
    }

    /// Se olvida de los suscriptores anteriores. Lo llama `shell.rs` antes de rehacer
    /// las superficies: si no, cada cambio de tema dejaria otra tanda de cierres
    /// apuntando a paneles ya destruidos.
    pub fn olvidar_suscriptores(&self) {
        self.al_cambiar.borrow_mut().clear();
    }

    fn aviso(&self) {
        // Se clona antes de llamar: un suscriptor podria tocar la lista.
        let v = self.al_cambiar.borrow().clone();
        for f in v {
            f();
        }
    }

    /// Publica la interfaz del watcher en nuestra conexion.
    fn exportar(self: &Rc<Self>) {
        let info = match gio::DBusNodeInfo::for_xml(XML) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("hfs: el XML del watcher no vale: {e}");
                return;
            }
        };
        let Some(iface) = info.lookup_interface(WATCHER) else {
            eprintln!("hfs: el XML del watcher no trae su interfaz");
            return;
        };

        let yo = self.clone();
        let yo_prop = self.clone();
        let r = self
            .conn
            .register_object(WATCHER_PATH, &iface)
            .method_call(move |_, quien, _, _, metodo, args, inv| {
                match metodo {
                    "RegisterStatusNotifierItem" => {
                        let service: String = args.child_value(0).get().unwrap_or_default();
                        yo.registrar(&service, quien);
                        inv.return_value(None);
                    }
                    // Decimos que si: el host somos nosotros y ya estamos aqui.
                    "RegisterStatusNotifierHost" => inv.return_value(None),
                    otro => inv.return_error(
                        gio::DBusError::UnknownMethod,
                        &format!("no conozco {otro}"),
                    ),
                }
            })
            .property(move |_, _, _, _, nombre| match nombre {
                "RegisteredStatusNotifierItems" => {
                    let v = yo_prop.servicios.borrow();
                    let refs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
                    refs.to_variant()
                }
                // Si esto fuera `false`, los programas educados ni publicarian icono.
                "IsStatusNotifierHostRegistered" => true.to_variant(),
                _ => 0i32.to_variant(),
            })
            .build();
        if let Err(e) = r {
            eprintln!("hfs: no pude publicar el watcher: {e}");
        }
    }

    /// Toma `org.kde.StatusNotifierWatcher` y el nombre de host.
    fn tomar_nombres(self: &Rc<Self>) {
        let yo = self.clone();
        gio::bus_own_name(
            gio::BusType::Session,
            WATCHER,
            // Sin `REPLACE`: si ya hay un watcher de verdad, el suyo manda y nosotros
            // nos quedamos de host a secas. Pelearse por el nombre deja a los
            // programas sin saber a quien hablarle.
            gio::BusNameOwnerFlags::NONE,
            |_, _| {},
            move |_, _| {
                yo.somos_el_watcher.set(true);
                yo.anunciar_host();
                yo.reenganchar();
            },
            |_, _| {
                eprintln!("hfs: otro programa ya es el watcher de la bandeja; cedo");
            },
        );

        // El host tambien tiene nombre propio, y hay programas que lo comprueban.
        let host = format!("org.kde.StatusNotifierHost-{}", std::process::id());
        gio::bus_own_name(
            gio::BusType::Session,
            &host,
            gio::BusNameOwnerFlags::NONE,
            |_, _| {},
            |_, _| {},
            |_, _| {},
        );
    }

    fn anunciar_host(&self) {
        let _ = self.conn.emit_signal(
            None,
            WATCHER_PATH,
            WATCHER,
            "StatusNotifierHostRegistered",
            None,
        );
    }

    fn registrar(&self, service: &str, quien: Option<&str>) {
        let (bus, ruta) = partir(service, quien);
        if bus.is_empty() {
            return;
        }
        let clave = format!("{bus}{ruta}");
        {
            let mut v = self.servicios.borrow_mut();
            if v.contains(&clave) {
                return;
            }
            v.push(clave.clone());
        }
        let _ = self.conn.emit_signal(
            None,
            WATCHER_PATH,
            WATCHER,
            "StatusNotifierItemRegistered",
            Some(&(clave.as_str(),).to_variant()),
        );
        self.aviso();
    }

    /// Quita los items cuyo programa se ha ido. Sin esto la bandeja acumula iconos
    /// muertos: nadie manda un «me voy», simplemente desaparece del bus.
    fn vigilar_caidas(self: &Rc<Self>) {
        let yo = self.clone();
        let sub = self.conn.subscribe_to_signal(
            Some("org.freedesktop.DBus"),
            Some("org.freedesktop.DBus"),
            Some("NameOwnerChanged"),
            Some("/org/freedesktop/DBus"),
            None,
            gio::DBusSignalFlags::NONE,
            move |sig| {
                let args = sig.parameters;
                let nombre: String = args.child_value(0).get().unwrap_or_default();
                let nuevo: String = args.child_value(2).get().unwrap_or_default();
                if !nuevo.is_empty() {
                    return;
                }
                let fuera: Vec<String> = yo
                    .servicios
                    .borrow()
                    .iter()
                    .filter(|s| s.starts_with(&nombre))
                    .cloned()
                    .collect();
                if fuera.is_empty() {
                    return;
                }
                yo.servicios.borrow_mut().retain(|s| !fuera.contains(s));
                for s in fuera {
                    let _ = yo.conn.emit_signal(
                        None,
                        WATCHER_PATH,
                        WATCHER,
                        "StatusNotifierItemUnregistered",
                        Some(&(s.as_str(),).to_variant()),
                    );
                }
                yo.aviso();
            },
        );
        *self._caidas.borrow_mut() = Some(sub);
    }

    /// Lee ahora mismo el estado de cada item registrado.
    pub fn items(&self) -> Vec<Item> {
        self.servicios
            .borrow()
            .iter()
            .filter_map(|s| self.leer(s))
            .collect()
    }

    fn leer(&self, servicio: &str) -> Option<Item> {
        let corte = servicio.find('/')?;
        let (bus, ruta) = servicio.split_at(corte);
        let p = hfs_ui::dbus::session(bus, ruta, ITEM_IFACE).ok()?;

        let txt = |n: &str| hfs_ui::dbus::prop::<String>(&p, n).unwrap_or_default();
        let estado = {
            let e = txt("Status");
            if e.is_empty() { "Active".to_string() } else { e }
        };

        Some(Item {
            bus: bus.to_string(),
            ruta: ruta.to_string(),
            id: txt("Id"),
            titulo: txt("Title"),
            estado,
            icono: txt("IconName"),
            tema_propio: Some(txt("IconThemePath")).filter(|s| !s.is_empty()),
            pixmap: pixmap(&p),
        })
    }

    /// Un clic. `Activate` es lo que hace casi todo: enseñar u ocultar la ventana.
    pub fn activar(&self, it: &Item) {
        self.llamar(it, "Activate");
    }

    /// El clic secundario. Lo correcto seria sacar el menu de DBusMenu; `ContextMenu`
    /// le pide al propio programa que lo enseñe, que es lo que funciona sin
    /// implementar un segundo protocolo entero.
    pub fn menu(&self, it: &Item) {
        self.llamar(it, "ContextMenu");
    }

    fn llamar(&self, it: &Item, metodo: &str) {
        let Ok(p) = hfs_ui::dbus::session(&it.bus, &it.ruta, ITEM_IFACE) else {
            return;
        };
        // Las coordenadas son para que el programa coloque su menu; en Wayland no las
        // puede usar para nada, asi que van a cero y punto.
        p.call(
            metodo,
            Some(&(0i32, 0i32).to_variant()),
            gio::DBusCallFlags::NONE,
            2000,
            gio::Cancellable::NONE,
            move |r| {
                if let Err(e) = r {
                    eprintln!("hfs: la bandeja no pudo con la acción: {e}");
                }
            },
        );
    }

    /// A los programas que arrancaron antes que el watcher se les da un empujon.
    ///
    /// Solo a los nuestros: `udiskie` lo lanza nuestro `exec-once`, asi que
    /// reiniciarlo es reponer algo que nosotros pusimos. Lo demas se avisa y ya.
    fn reenganchar(self: &Rc<Self>) {
        let yo = self.clone();
        // Un respiro: los programas que esperan al watcher suelen reintentar solos en
        // cuanto ven el nombre, y reiniciar a uno que iba a volver seria peor.
        glib::timeout_add_seconds_local_once(3, move || {
            if !crate::proc::vivo("udiskie") {
                return;
            }
            // Mirar si la lista esta vacia no vale: basta con que otro programa
            // (Steam, sin ir mas lejos) si se haya registrado para que parezca que
            // todo esta bien. Hay que preguntar por udiskie en concreto.
            let registrado = yo.items().iter().any(|i| {
                i.id.eq_ignore_ascii_case("udiskie") || i.titulo.to_lowercase().contains("udiskie")
            });
            if registrado {
                return;
            }
            eprintln!("hfs: udiskie arrancó sin bandeja; lo repongo");
            crate::proc::sh("pkill -x udiskie; sleep 0.5; exec udiskie -s");
        });
    }
}

/// `IconPixmap` es `a(iiay)`: ancho, alto y los pixeles en ARGB32 big-endian. Se coge
/// el mas grande, que es el que mejor se ve al escalarlo hacia abajo.
fn pixmap(p: &gio::DBusProxy) -> Option<(i32, i32, Vec<u8>)> {
    let v = p.cached_property("IconPixmap")?;
    let mut mejor: Option<(i32, i32, Vec<u8>)> = None;
    for i in 0..v.n_children() {
        let e = v.child_value(i);
        if e.n_children() != 3 {
            continue;
        }
        let w: i32 = e.child_value(0).get()?;
        let h: i32 = e.child_value(1).get()?;
        let datos: Vec<u8> = e.child_value(2).get()?;
        if w <= 0 || h <= 0 || datos.len() < (w * h * 4) as usize {
            continue;
        }
        if mejor.as_ref().is_none_or(|(mw, _, _)| w > *mw) {
            mejor = Some((w, h, datos));
        }
    }
    mejor
}

#[cfg(test)]
mod tests {
    use super::partir;

    #[test]
    fn un_nombre_de_bus_suelto_usa_la_ruta_de_siempre() {
        let (b, r) = partir("org.kde.StatusNotifierItem-1234-1", Some(":1.9"));
        assert_eq!(b, "org.kde.StatusNotifierItem-1234-1");
        assert_eq!(r, "/StatusNotifierItem");
    }

    #[test]
    fn una_ruta_suelta_usa_a_quien_llamo() {
        // Es lo que mandan los programas basados en libappindicator.
        let (b, r) = partir("/StatusNotifierItem", Some(":1.42"));
        assert_eq!(b, ":1.42");
        assert_eq!(r, "/StatusNotifierItem");
    }

    #[test]
    fn la_forma_de_una_pieza_se_parte_bien() {
        let (b, r) = partir(":1.42/org/ayatana/NotificationItem/x", Some(":1.42"));
        assert_eq!(b, ":1.42");
        assert_eq!(r, "/org/ayatana/NotificationItem/x");
    }
}
