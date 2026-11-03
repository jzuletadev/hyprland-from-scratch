// ~/hyprland-from-scratch/crates/hfs-core/src/teclado.rs
//
// El estado del teclado segun Hyprland: bloqueos y distribucion.
//
// Caps Lock y Num Lock no tienen indicador en este portatil —ni luz ni nada— asi que
// la unica forma de saber si estan puestos era equivocarse al escribir. Hyprland los
// publica por su socket junto con la distribucion activa, que es ademas lo que hace
// falta para poder cambiarla.
//
// Se parsea el texto del socket y no `hyprctl devices -j` por lo mismo que los atajos:
// este crate no tiene dependencias y meter un parser de JSON por esto seria
// desproporcionado cuando el formato es `clave: valor` con sangria.

use crate::hyprland;
use crate::Result;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Teclado {
    pub nombre: String,
    /// Como lo llama Hyprland: «English (US)», «Spanish».
    pub keymap: String,
    pub caps: bool,
    pub num: bool,
    /// El teclado principal. Es el unico que importa: los demas son botones de
    /// encendido, teclas de volumen y cosas asi, y todos dicen lo mismo.
    pub main: bool,
}

impl Teclado {
    /// Un nombre corto para una pastilla. «English (US)» -> «US».
    pub fn distribucion(&self) -> String {
        if let Some((_, dentro)) = self.keymap.split_once('(')
            && let Some((d, _)) = dentro.split_once(')')
        {
            return d.trim().to_string();
        }
        // Sin parentesis, las tres primeras letras dicen bastante: «Spanish» -> «Spa».
        self.keymap.chars().take(3).collect::<String>().to_uppercase()
    }
}

/// Parsea la seccion `Keyboards:` de `hyprctl devices`.
pub fn parse(txt: &str) -> Vec<Teclado> {
    let mut v = Vec::new();
    let mut en_teclados = false;
    let mut actual: Option<Teclado> = None;
    // El nombre del dispositivo es la primera linea del bloque y NO lleva `clave:`,
    // asi que hace falta saber que toca leerlo.
    let mut esperando_nombre = false;

    for linea in txt.lines() {
        let t = linea.trim();
        if t == "Keyboards:" {
            en_teclados = true;
            continue;
        }
        // Cualquier otra seccion de primer nivel cierra la de teclados.
        if t.ends_with(':') && !linea.starts_with('\t') && t != "Keyboards:" {
            if let Some(k) = actual.take() {
                v.push(k);
            }
            en_teclados = false;
            continue;
        }
        if !en_teclados {
            continue;
        }
        if t.starts_with("Keyboard at") {
            if let Some(k) = actual.take() {
                v.push(k);
            }
            actual = Some(Teclado::default());
            esperando_nombre = true;
            continue;
        }
        let Some(k) = actual.as_mut() else { continue };
        if esperando_nombre && !t.is_empty() {
            k.nombre = t.to_string();
            esperando_nombre = false;
            continue;
        }
        let Some((clave, valor)) = t.split_once(':') else {
            continue;
        };
        let valor = valor.trim();
        match clave.trim() {
            "active keymap" => k.keymap = valor.to_string(),
            "capsLock" => k.caps = valor == "yes",
            "numLock" => k.num = valor == "yes",
            "main" => k.main = valor == "yes",
            _ => {}
        }
    }
    if let Some(k) = actual {
        v.push(k);
    }
    v
}

/// El estado del teclado, juntando todos los dispositivos.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Estado {
    pub caps: bool,
    pub num: bool,
    /// La distribucion activa, abreviada.
    pub distribucion: String,
}

/// Lo que hay que enseñar, mirando TODOS los teclados y no solo el principal.
///
///   EL «PRINCIPAL» DE HYPRLAND NO ES ESTABLE. Medido: entre dos consultas seguidas
///   paso de `sonix-usb-device` a `dell-wireless-hotkeys`, que es un receptor de
///   teclas de funcion. Colgar el indicador de ese campo lo haria parpadear solo.
///
/// Un bloqueo esta puesto si **algun** teclado lo dice, que ademas es lo correcto con
/// un teclado externo enchufado: el bloqueo es del usuario, no del dispositivo.
pub fn estado() -> Result<Estado> {
    Ok(resumir(&parse(&hyprland::request("devices")?)))
}

pub fn resumir(v: &[Teclado]) -> Estado {
    Estado {
        caps: v.iter().any(|k| k.caps),
        num: v.iter().any(|k| k.num),
        // Para el nombre vale cualquiera con keymap: todos comparten la configuracion.
        distribucion: v
            .iter()
            .find(|k| !k.keymap.is_empty())
            .map(|k| k.distribucion())
            .unwrap_or_default(),
    }
}

/// Pasa a la siguiente distribucion configurada en `kb_layout`.
///
/// `all` y no un dispositivo concreto: cambiarsela a uno solo deja el teclado del
/// portatil y el externo en distribuciones distintas, que no es lo que nadie quiere.
///
/// Y va por `request` y NO por `dispatch`: `switchxkblayout` es un comando de hyprctl
/// por derecho propio, no un dispatcher, asi que `dispatch switchxkblayout …` lo
/// rechaza. Es facil equivocarse porque desde la terminal los dos se escriben igual.
pub fn siguiente_distribucion() -> Result<()> {
    let r = hyprland::request("switchxkblayout all next")?;
    if r.trim() == "ok" {
        Ok(())
    } else {
        Err(crate::Error::new(format!(
            "Hyprland rechazo el cambio de distribucion: {}",
            r.trim()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REAL: &str = "\
mice:
\tMouse at 55c55df00660:
\t\tlogitech-usb-optical-mouse
\t\t\tdefault speed: 0.00000

Keyboards:
\tKeyboard at 55c55e31c280:
\t\tvideo-bus
\t\t\trules: r \"\", m \"\", l \"us\", v \"\", o \"\"
\t\t\tactive layout index: 0
\t\t\tactive keymap: English (US)
\t\t\tcapsLock: no
\t\t\tnumLock: no
\t\t\tmain: no
\tKeyboard at 55c55df00ac0:
\t\tsonix-usb-device
\t\t\trules: r \"\", m \"\", l \"us,es\", v \"\", o \"\"
\t\t\tactive layout index: 1
\t\t\tactive keymap: Spanish
\t\t\tcapsLock: yes
\t\t\tnumLock: yes
\t\t\tmain: yes

Tablets:
";

    #[test]
    fn solo_hay_un_principal_y_es_el_que_importa() {
        let v = parse(REAL);
        assert_eq!(v.len(), 2);
        let p: Vec<&Teclado> = v.iter().filter(|k| k.main).collect();
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].nombre, "sonix-usb-device");
    }

    #[test]
    fn lee_los_bloqueos() {
        let v = parse(REAL);
        let p = v.iter().find(|k| k.main).unwrap();
        assert!(p.caps);
        assert!(p.num);
        // El otro teclado dice que no, y no debe contaminar al principal.
        assert!(!v[0].caps);
    }

    #[test]
    fn la_distribucion_se_abrevia() {
        assert_eq!(
            Teclado {
                keymap: "English (US)".into(),
                ..Default::default()
            }
            .distribucion(),
            "US"
        );
        // Sin parentesis no hay nada entre que cortar, asi que se abrevia a mano.
        assert_eq!(
            Teclado {
                keymap: "Spanish".into(),
                ..Default::default()
            }
            .distribucion(),
            "SPA"
        );
    }

    #[test]
    fn el_raton_no_se_cuela() {
        // La seccion de teclados empieza despues de la de ratones; sin cerrar la
        // anterior, el raton entraria como teclado sin nombre.
        let v = parse(REAL);
        assert!(!v.iter().any(|k| k.nombre.contains("mouse")));
    }

    #[test]
    fn el_bloqueo_lo_pone_cualquier_teclado() {
        // El «main» de Hyprland cambia entre consultas, asi que no se puede colgar el
        // indicador de el: aqui el principal dice que si y el otro que no.
        let e = resumir(&parse(REAL));
        assert!(e.caps);
        assert!(e.num);
        assert_eq!(e.distribucion, "US"); // la del primero con keymap
    }

    #[test]
    fn sin_teclados_el_estado_es_todo_apagado() {
        assert_eq!(resumir(&[]), Estado::default());
    }

    #[test]
    fn un_texto_sin_teclados_no_revienta() {
        assert!(parse("").is_empty());
        assert!(parse("mice:\n\tMouse at 1:\n\t\tx\n").is_empty());
    }
}
