// ~/hyprland-from-scratch/crates/hfs-ui/src/dbus.rs
//
// D-Bus por GIO, no por zbus.
//
// Es la decision que mantiene a `hfs-core` sin dependencias: GIO ya esta enlazado
// (viene con GTK), habla D-Bus entero, y sus proxys viven en el bucle de glib, que es
// justo donde tienen que estar para que una señal repinte un panel sin hilos ni
// canales. Meter zbus seria traer tokio o async-std para algo que ya tenemos.
//
// Un proxy ademas cachea las propiedades y avisa cuando cambian, asi que leer el
// perfil de energia o el estado del reproductor no cuesta ni una ida y vuelta.

use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;

/// Un proxy, o el error de D-Bus tal cual viene.
///
/// Que exista no garantiza que el servicio este: si nadie tiene el nombre,
/// `name_owner()` es None y las propiedades vienen vacias. Es a proposito, porque
/// hay servicios que van y vienen (un reproductor, por ejemplo).
pub fn proxy(
    bus: gio::BusType,
    servicio: &str,
    ruta: &str,
    iface: &str,
) -> Result<gio::DBusProxy, glib::Error> {
    gio::DBusProxy::for_bus_sync(
        bus,
        gio::DBusProxyFlags::DO_NOT_AUTO_START,
        None,
        servicio,
        ruta,
        iface,
        gio::Cancellable::NONE,
    )
}

pub fn session(servicio: &str, ruta: &str, iface: &str) -> Result<gio::DBusProxy, glib::Error> {
    proxy(gio::BusType::Session, servicio, ruta, iface)
}

pub fn system(servicio: &str, ruta: &str, iface: &str) -> Result<gio::DBusProxy, glib::Error> {
    proxy(gio::BusType::System, servicio, ruta, iface)
}

/// ¿Hay alguien al otro lado ahora mismo?
pub fn vivo(p: &gio::DBusProxy) -> bool {
    p.name_owner().is_some()
}

/// Una propiedad cacheada, ya convertida.
pub fn prop<T: glib::variant::FromVariant>(p: &gio::DBusProxy, nombre: &str) -> Option<T> {
    p.cached_property(nombre)?.get::<T>()
}

/// Una propiedad que es un diccionario `a{sv}` (los metadatos de MPRIS, por ejemplo).
///
/// Comprueba el tipo antes de envolverlo: `VariantDict::new` no devuelve un error si
/// le das otra cosa, aborta el proceso.
pub fn prop_dict(p: &gio::DBusProxy, nombre: &str) -> Option<glib::VariantDict> {
    let v = p.cached_property(nombre)?;
    let esperado = glib::VariantTy::new("a{sv}").ok()?;
    v.is_type(esperado).then(|| glib::VariantDict::new(Some(&v)))
}

/// Busca una clave en CUALQUIER diccionario de GVariant.
///
/// `glib::VariantDict` solo sabe de `a{sv}` y **aborta el proceso** si le das otra
/// cosa: no devuelve un error, revienta dentro de una llamada de C que no puede
/// desenrollar la pila. Y los dos servicios que mas se usan aqui devuelven
/// `a{sa{sv}}` —la configuracion de NetworkManager y el `GetManagedObjects` de
/// BlueZ—, asi que hay que recorrer el diccionario a mano.
///
/// Si el valor viene envuelto en un variante (que es lo que pasa en un `a{sv}`), lo
/// desenvuelve, para que quien llame no tenga que saber de que tipo era el dicho.
pub fn dict_get(d: &glib::Variant, clave: &str) -> Option<glib::Variant> {
    for i in 0..d.n_children() {
        let e = d.child_value(i);
        if e.n_children() == 2 && e.child_value(0).str() == Some(clave) {
            let v = e.child_value(1);
            // `as_variant()` no comprueba el tipo: si el valor no es un `v`, GLib
            // suelta un CRITICAL y devuelve nada. Hay que mirar antes.
            if v.type_() == glib::VariantTy::VARIANT {
                return v.as_variant().or(Some(v));
            }
            return Some(v);
        }
    }
    None
}

/// Construye un `a{sa{sv}}`, que es el tipo que pide NetworkManager para una
/// conexion. No se puede hacer con `VariantDict`: ese siempre produce `a{sv}`,
/// porque mete cada valor dentro de un variante.
pub fn dict_of_dicts(secciones: &[(&str, glib::Variant)]) -> glib::Variant {
    let entradas: Vec<glib::Variant> = secciones
        .iter()
        .map(|(k, v)| glib::Variant::from_dict_entry(&k.to_variant(), v))
        .collect();
    let tipo = glib::VariantTy::new("{sa{sv}}").expect("tipo fijo y valido");
    glib::Variant::array_from_iter_with_type(tipo, entradas)
}

/// Llama a un metodo y se queda con la respuesta. Sincrono a proposito: todo lo que
/// se llama asi aqui es una accion del usuario sobre un servicio local, que responde
/// en microsegundos; lo que no puede bloquear son los sondeos, y esos no pasan por
/// aqui.
pub fn call(
    p: &gio::DBusProxy,
    metodo: &str,
    args: Option<&glib::Variant>,
) -> Result<glib::Variant, glib::Error> {
    p.call_sync(
        metodo,
        args,
        gio::DBusCallFlags::NONE,
        2000,
        gio::Cancellable::NONE,
    )
}

/// Escribe una propiedad. D-Bus no deja hacerlo por el proxy directamente: hay que
/// pasar por `org.freedesktop.DBus.Properties.Set`, que es lo que hace esto.
pub fn set_prop(
    p: &gio::DBusProxy,
    iface: &str,
    nombre: &str,
    valor: &glib::Variant,
) -> Result<(), glib::Error> {
    let destino = p.name().map(|n| n.to_string()).unwrap_or_default();
    p.connection().call_sync(
        Some(&destino),
        &p.object_path(),
        "org.freedesktop.DBus.Properties",
        "Set",
        Some(&(iface, nombre, valor).to_variant()),
        None,
        gio::DBusCallFlags::NONE,
        2000,
        gio::Cancellable::NONE,
    )?;
    Ok(())
}
