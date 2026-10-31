// ~/hyprland-from-scratch/crates/hfs-core/src/binds.rs
//
// Los atajos de Hyprland, tal como los cuenta el propio compositor.
//
//   AQUI NO HAY LISTA A MANO. El TUI de la v1 ensenaba los atajos con descripcion y
//   le anadia una lista `EXTRA` escrita a mano para tapar el hueco, asi que la ayuda
//   y la configuracion se iban separando cada vez que alguien tocaba una sin la otra.
//   De los 68 binds de esta maquina **35 no tienen descripcion**, y en la v1 eran
//   invisibles salvo los que alguien se acordo de copiar a mano.
//
// Aqui salen los 68. Al que no tiene descripcion se le compone una con su dispatcher
// y su argumento, que es informacion de verdad y no un hueco, y se marca como
// derivada: asi la propia ayuda ensena cuales les falta un `bindd`.
//
// Se parsea el texto del socket y no `hyprctl binds -j` a proposito: este crate no
// tiene dependencias, y meter un parser de JSON por esto seria desproporcionado
// cuando el formato de texto es un registro por bloque y `clave: valor` por linea.

use crate::hyprland;
use crate::Result;

/// Los modificadores y el bit que ocupan en `modmask`.
///
/// El orden de esta tabla es el orden en que se pintan, y **no** es el de los bits:
/// es el que se usa al escribirlos, que es el mismo de `hyprland.conf` (`$mainMod
/// SHIFT, E`). Por bits saldria «SHIFT + SUPER + E», que nadie dice ni escribe.
const MODS: &[(u32, &str)] = &[
    (64, "SUPER"),
    (4, "CTRL"),
    (8, "ALT"),
    (1, "SHIFT"),
    (2, "CAPS"),
    (16, "MOD2"),
    (32, "MOD3"),
    (128, "MOD5"),
];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bind {
    /// `bindd`, `bindrd`, `bindel`... tal como lo escribe Hyprland.
    pub kind: String,
    pub modmask: u32,
    pub key: String,
    pub submap: String,
    /// Vacia cuando el bind no se declaro con `bindd`.
    pub description: String,
    pub dispatcher: String,
    pub arg: String,
}

impl Bind {
    /// «SUPER + SHIFT», o vacio si el atajo no lleva modificadores.
    pub fn mods(&self) -> String {
        let v: Vec<&str> = MODS
            .iter()
            .filter(|(bit, _)| self.modmask & bit != 0)
            .map(|(_, n)| *n)
            .collect();
        v.join(" + ")
    }

    /// El atajo entero, listo para pintar: «SUPER + SHIFT + S».
    pub fn combo(&self) -> String {
        let m = self.mods();
        if m.is_empty() {
            self.key.clone()
        } else if self.key.is_empty() {
            m
        } else {
            format!("{m} + {}", self.key)
        }
    }

    /// `true` cuando la descripcion la he compuesto yo porque el bind no la trae.
    pub fn derivada(&self) -> bool {
        self.description.trim().is_empty()
    }

    /// Que hace el atajo. La descripcion si la tiene; si no, lo que de verdad ejecuta.
    pub fn label(&self) -> String {
        if !self.derivada() {
            return self.description.clone();
        }
        let arg = self.arg.trim();
        if arg.is_empty() {
            return self.dispatcher.clone();
        }
        // Un `exec` describe mejor su orden que la palabra «exec».
        if self.dispatcher == "exec" {
            return arg.to_string();
        }
        format!("{} {arg}", self.dispatcher)
    }

    /// El texto sobre el que busca el panel: todo lo que el usuario podria escribir.
    pub fn buscable(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.combo(),
            self.description,
            self.dispatcher,
            self.arg,
            self.submap
        )
    }
}

/// Parsea la salida de `hyprctl binds`. Un bloque por atajo, separado por una linea
/// en blanco; la primera linea del bloque es el tipo y el resto son `clave: valor`
/// con sangria.
pub fn parse(txt: &str) -> Vec<Bind> {
    let mut v = Vec::new();
    let mut actual: Option<Bind> = None;

    for linea in txt.lines() {
        // Una linea sin sangria y no vacia abre un bloque nuevo.
        if !linea.starts_with([' ', '\t']) {
            if let Some(b) = actual.take() {
                v.push(b);
            }
            let t = linea.trim();
            if !t.is_empty() {
                actual = Some(Bind {
                    kind: t.to_string(),
                    ..Default::default()
                });
            }
            continue;
        }
        let Some(b) = actual.as_mut() else { continue };
        let Some((clave, valor)) = linea.split_once(':') else {
            continue;
        };
        // El valor puede llevar `:` dentro (una orden de `exec`, por ejemplo), asi que
        // solo se parte en el primero.
        let valor = valor.trim();
        match clave.trim() {
            "modmask" => b.modmask = valor.parse().unwrap_or(0),
            "submap" => b.submap = valor.to_string(),
            "key" => b.key = valor.to_string(),
            "description" => b.description = valor.to_string(),
            "dispatcher" => b.dispatcher = valor.to_string(),
            "arg" => b.arg = valor.to_string(),
            _ => {}
        }
    }
    if let Some(b) = actual {
        v.push(b);
    }
    v
}

/// Los atajos vivos, preguntados al compositor.
pub fn load() -> Result<Vec<Bind>> {
    Ok(parse(&hyprland::request("binds")?))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MUESTRA: &str = "\
bindrd
\tmodmask: 64
\tsubmap: 
\tkey: Super_L
\tkeycode: 0
\tcatchall: false
\tdescription: App search
\tdispatcher: exec
\targ: ~/.local/bin/hfs wheel

bind
\tmodmask: 65
\tsubmap: 
\tkey: S
\tkeycode: 0
\tcatchall: false
\tdescription: 
\tdispatcher: movetoworkspace
\targ: 3
";

    #[test]
    fn parsea_dos_bloques() {
        let v = parse(MUESTRA);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].kind, "bindrd");
        assert_eq!(v[0].key, "Super_L");
        assert_eq!(v[1].dispatcher, "movetoworkspace");
    }

    #[test]
    fn decodifica_los_modificadores() {
        let v = parse(MUESTRA);
        assert_eq!(v[0].mods(), "SUPER");
        // 65 = SHIFT (1) + SUPER (64). Salen como se escriben en hyprland.conf, no
        // en el orden de los bits: «SUPER + SHIFT», no «SHIFT + SUPER».
        assert_eq!(v[1].mods(), "SUPER + SHIFT");
        assert_eq!(v[1].combo(), "SUPER + SHIFT + S");
    }

    #[test]
    fn un_exec_se_describe_con_su_orden_y_no_con_la_palabra_exec() {
        // El caso real: `hyprctl binds` llama «exec» a 20 atajos distintos, asi que
        // usar el dispatcher como etiqueta los dejaria indistinguibles.
        let b = Bind {
            dispatcher: "exec".into(),
            arg: "hyprlock".into(),
            ..Default::default()
        };
        assert!(b.derivada());
        assert_eq!(b.label(), "hyprlock");
    }

    #[test]
    fn los_35_sin_descripcion_dejan_de_ser_invisibles() {
        let v = parse(MUESTRA);
        assert!(!v[0].derivada());
        assert_eq!(v[0].label(), "App search");
        // Este es de los que la v1 no ensenaba: sin descripcion, pero con algo que
        // decir.
        assert!(v[1].derivada());
        assert_eq!(v[1].label(), "movetoworkspace 3");
    }

    #[test]
    fn un_valor_con_dos_puntos_dentro_no_se_parte() {
        let txt = "bind\n\tdispatcher: exec\n\targ: sh -c 'echo a:b'\n";
        assert_eq!(parse(txt)[0].arg, "sh -c 'echo a:b'");
    }

    #[test]
    fn texto_vacio_no_da_atajos() {
        assert!(parse("").is_empty());
        assert!(parse("\n\n").is_empty());
    }
}
