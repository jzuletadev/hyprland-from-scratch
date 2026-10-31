// ~/hyprland-from-scratch/crates/hfs-core/src/clipboard.rs
//
// El historial del portapapeles.
//
// No hay ninguno en la v1: `wl-copy` guarda lo ultimo y ya esta. Aqui lo alimenta
// `wl-paste --watch`, que llama a `hfs clip-add` cada vez que cambia la seleccion.
//
// El almacen es un directorio con un fichero por entrada, nombrado con el instante en
// milisegundos. Sin indice, sin base de datos y sin formato: el nombre ordena, el
// contenido es el contenido, y borrar es borrar un fichero. Para un historial de
// doscientas entradas de texto, cualquier otra cosa es maquinaria de mas.
//
// Solo texto a proposito. `wl-paste` se invoca con `--type text`, asi que copiar una
// imagen no deja un fichero binario de diez megas en el historial.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::{paths, Error, Result};

/// Cuantas entradas se guardan. Pasado eso, se borran las mas viejas.
const MAXIMO: usize = 200;
/// Lo que se guarda como mucho de una entrada. Pegar un fichero entero en el
/// portapapeles no deberia llenar el disco.
const LIMITE_BYTES: usize = 256 * 1024;

pub fn dir() -> PathBuf {
    paths::state().join("clipboard")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entrada {
    /// El nombre del fichero, que es tambien su instante en milisegundos.
    pub id: String,
    /// La primera linea, recortada, para enseñarla en la lista.
    pub vista: String,
    pub bytes: usize,
    pub edad: Duration,
    /// Si el texto parece una ruta, una URL o codigo, para elegir el icono.
    pub clase: Clase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clase {
    Texto,
    Url,
    Ruta,
    Codigo,
}

fn ahora_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Guarda una entrada nueva. Si es igual a la ultima, no hace nada: al pegar desde
/// el propio panel se vuelve a copiar lo mismo y `wl-paste --watch` lo anuncia otra
/// vez, asi que sin esto el historial se llenaria de duplicados.
pub fn add(contenido: &[u8]) -> Result<()> {
    let texto = String::from_utf8_lossy(contenido);
    let texto = texto.trim_end_matches(['\n', '\r']);
    if texto.trim().is_empty() {
        return Ok(());
    }
    let recorte = &texto.as_bytes()[..texto.len().min(LIMITE_BYTES)];

    let d = dir();
    std::fs::create_dir_all(&d)
        .map_err(|e| Error::new(format!("no pude crear {}: {e}", d.display())))?;

    if let Some(ultima) = ficheros(&d).last()
        && std::fs::read(ultima).ok().as_deref() == Some(recorte)
    {
        return Ok(());
    }

    let destino = d.join(ahora_ms().to_string());
    std::fs::write(&destino, recorte)
        .map_err(|e| Error::new(format!("no pude escribir {}: {e}", destino.display())))?;
    podar(&d);
    Ok(())
}

/// Las entradas mas recientes primero.
pub fn list(limite: usize) -> Vec<Entrada> {
    let d = dir();
    let ahora = ahora_ms();
    ficheros(&d)
        .into_iter()
        .rev()
        .take(limite)
        .filter_map(|f| {
            let bytes = std::fs::read(&f).ok()?;
            let id = f.file_name()?.to_str()?.to_string();
            let ms: u128 = id.parse().unwrap_or(ahora);
            let texto = String::from_utf8_lossy(&bytes);
            Some(Entrada {
                clase: clasificar(&texto),
                vista: vista_de(&texto),
                bytes: bytes.len(),
                edad: Duration::from_millis(ahora.saturating_sub(ms) as u64),
                id,
            })
        })
        .collect()
}

pub fn leer(id: &str) -> Result<Vec<u8>> {
    // Nada de rutas: el id es un nombre de fichero y solo eso.
    if id.is_empty() || id.contains('/') || id.contains("..") {
        return Err(Error::new(format!("«{id}» no es una entrada")));
    }
    let f = dir().join(id);
    std::fs::read(&f).map_err(|e| Error::new(format!("no pude leer {}: {e}", f.display())))
}

pub fn vaciar() -> Result<()> {
    let d = dir();
    for f in ficheros(&d) {
        let _ = std::fs::remove_file(f);
    }
    Ok(())
}

/// Los ficheros del almacen, de mas viejo a mas nuevo.
fn ficheros(d: &Path) -> Vec<PathBuf> {
    let Ok(dir) = std::fs::read_dir(d) else {
        return Vec::new();
    };
    let mut v: Vec<PathBuf> = dir
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.chars().all(|c| c.is_ascii_digit()))
        })
        .collect();
    // Por nombre no vale: «9» iria despues de «10». Por numero si.
    v.sort_by_key(|p| {
        p.file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.parse::<u128>().ok())
            .unwrap_or(0)
    });
    v
}

fn podar(d: &Path) {
    let v = ficheros(d);
    if v.len() <= MAXIMO {
        return;
    }
    for f in &v[..v.len() - MAXIMO] {
        let _ = std::fs::remove_file(f);
    }
}

/// La primera linea con algo, sin espacios de sobra y sin pasarse de larga.
fn vista_de(texto: &str) -> String {
    let linea = texto
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let limpio: String = linea.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpio.chars().count() > 90 {
        let corto: String = limpio.chars().take(89).collect();
        format!("{corto}…")
    } else {
        limpio
    }
}

fn clasificar(texto: &str) -> Clase {
    let t = texto.trim();
    let primera = t.lines().next().unwrap_or("").trim();
    if primera.starts_with("http://") || primera.starts_with("https://") {
        Clase::Url
    } else if (primera.starts_with('/') || primera.starts_with("~/")) && !primera.contains(' ') {
        Clase::Ruta
    } else if t.lines().count() > 1
        || primera.contains("fn ")
        || primera.contains("def ")
        || primera.contains(" => ")
        || primera.contains("();")
    {
        Clase::Codigo
    } else {
        Clase::Texto
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_vista_es_la_primera_linea_util() {
        assert_eq!(vista_de("\n\n  hola   mundo \n otra "), "hola mundo");
    }

    #[test]
    fn la_vista_no_crece_sin_limite() {
        let largo = "a".repeat(300);
        let v = vista_de(&largo);
        assert!(v.chars().count() <= 90, "{}", v.chars().count());
        assert!(v.ends_with('…'));
    }

    #[test]
    fn clasifica_lo_que_se_copia_de_verdad() {
        assert_eq!(clasificar("https://wayland.app/protocols"), Clase::Url);
        assert_eq!(clasificar("/home/why/.config/hypr"), Clase::Ruta);
        assert_eq!(clasificar("fn main() {"), Clase::Codigo);
        assert_eq!(clasificar("una nota cualquiera"), Clase::Texto);
        // Una ruta con espacios no es una ruta para esto: casi siempre es una frase.
        assert_eq!(clasificar("/ algo con espacios"), Clase::Texto);
    }

    #[test]
    fn un_id_con_ruta_no_se_lee() {
        assert!(leer("../../etc/passwd").is_err());
        assert!(leer("").is_err());
    }
}
