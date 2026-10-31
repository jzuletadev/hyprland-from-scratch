// ~/hyprland-from-scratch/crates/hfs-core/src/fuzzy.rs
//
// El emparejador del buscador: «vsc» tiene que encontrar «Visual Studio Code» y
// «term» tiene que poner «Terminal» por encima de «Gestor de terminales».
//
// No es un Levenshtein: para listas de doscientas aplicaciones y consultas de cuatro
// letras lo que importa no es la distancia de edicion, sino DONDE cae cada letra.
//
// Son dos pasadas y se queda con la mejor:
//
//   1. INICIALES. Si TODAS las letras de la consulta caen en principio de palabra,
//      es un acronimo y gana de calle. Es lo que hace que «vsc» encuentre «Visual
//      Studio Code».
//   2. CODICIOSA, de izquierda a derecha, con estas reglas por orden de peso:
//      principio de nombre > principio de palabra > letra seguida > letra suelta,
//      restando por cada hueco y por nombre largo.
//
// La primera pasada existe porque la segunda sola se equivoca justo en el caso que
// mas se usa: buscando «vsc», la codiciosa coge la «s» de «Visual» (la primera que
// encuentra) en vez de la de «Studio», y entonces «Vivaldi Scorer» adelanta a
// «Visual Studio Code».
//
// El hueco solo resta cuando la letra NO cae en principio de palabra: un acronimo se
// salta medio nombre por definicion y no se le puede cobrar por ello.
//
// Sin dependencias: este crate no tiene ninguna y un emparejador de cien lineas no
// justifica la primera.

/// Separadores tras los que empieza una «palabra».
fn es_frontera(c: char) -> bool {
    c.is_whitespace() || matches!(c, '-' | '_' | '.' | '/' | ':' | '(' | '[')
}

const INICIO: i32 = 18;
const PALABRA: i32 = 12;
const SEGUIDA: i32 = 8;
const SUELTA: i32 = 2;
const HUECO: i32 = -1;
/// Lo que vale que la consulta entera sean iniciales.
const ACRONIMO: i32 = 25;

/// Donde empieza cada palabra del texto.
fn fronteras(texto: &[char]) -> Vec<usize> {
    (0..texto.len())
        .filter(|&i| i == 0 || es_frontera(texto[i - 1]))
        .collect()
}

/// Cuanto encaja `aguja` en `pajar`. `None` si no encaja; a mas alto, mejor.
/// Una aguja vacia encaja con todo y puntua cero.
pub fn score(aguja: &str, pajar: &str) -> Option<i32> {
    if aguja.is_empty() {
        return Some(0);
    }
    let texto: Vec<char> = pajar.chars().collect();
    let bajo: Vec<char> = pajar.to_lowercase().chars().collect();
    // Si la minuscula cambia el numero de caracteres (alguna ligadura rara), se
    // compara sobre el original: vale mas acertar poco que indexar mal.
    let bajo = if bajo.len() == texto.len() { bajo } else { texto.clone() };
    let aguja_bajo = aguja.to_lowercase();

    let por_iniciales = acronimo(&aguja_bajo, &texto, &bajo);
    let codiciosa = codiciosa(&aguja_bajo, &texto, &bajo);
    match (por_iniciales, codiciosa) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (a, b) => a.or(b),
    }
}

/// La consulta entera en principios de palabra.
fn acronimo(aguja: &str, texto: &[char], bajo: &[char]) -> Option<i32> {
    let fronteras = fronteras(texto);
    let mut desde = 0;
    let mut total = ACRONIMO;
    for a in aguja.chars() {
        if a.is_whitespace() {
            continue;
        }
        let p = fronteras[desde..].iter().position(|&f| bajo[f] == a)?;
        let f = fronteras[desde + p];
        total += if f == 0 { INICIO } else { PALABRA };
        desde += p + 1;
    }
    Some(total - (texto.len() as i32) / 8)
}

fn codiciosa(aguja: &str, texto: &[char], bajo: &[char]) -> Option<i32> {
    let mut total = 0;
    let mut i = 0;
    let mut anterior: Option<usize> = None;

    for a in aguja.chars() {
        if a.is_whitespace() {
            continue;
        }
        let encontrado = (i..bajo.len()).find(|&j| bajo[j] == a)?;
        let en_palabra = encontrado > 0 && es_frontera(texto[encontrado - 1]);

        total += if encontrado == 0 {
            INICIO
        } else if en_palabra {
            PALABRA
        } else if anterior == Some(encontrado - 1) {
            SEGUIDA
        } else {
            SUELTA
        };
        // El hueco solo se cobra si la letra cae en medio de una palabra.
        if let (Some(p), false) = (anterior, en_palabra) {
            total += HUECO * (encontrado - p - 1).min(20) as i32;
        }
        anterior = Some(encontrado);
        i = encontrado + 1;
    }

    // Entre dos que encajan igual de bien, gana el nombre mas corto: si escribes
    // «term» quieres «Terminal», no «Gestor de terminales de red».
    Some(total - (texto.len() as i32) / 8)
}

/// Lo mismo, pero probando contra varios campos y quedandose con el mejor.
///
/// Los campos son NIVELES, no un promedio: cualquier coincidencia en el primero (el
/// nombre) gana a cualquier coincidencia en el segundo (la descripcion), por buena
/// que sea. Por eso el castigo es enorme y no un ajuste fino.
///
/// Sin esto, escribir «bra» pone «Yazi File Manager» por delante de «Brave», porque
/// su descripcion («**B**lazing fast... **R**ust, based on **a**sync I/O») encaja
/// como acronimo y eso puntua mas que las tres letras seguidas de «Brave». Es un
/// lanzador: lo que se escribe es un nombre.
const CAMPO: i32 = 1000;

pub fn score_fields(aguja: &str, campos: &[&str]) -> Option<i32> {
    let mut mejor: Option<i32> = None;
    for (n, campo) in campos.iter().enumerate() {
        if let Some(s) = score(aguja, campo) {
            let s = s - CAMPO * n as i32;
            mejor = Some(mejor.map_or(s, |m: i32| m.max(s)));
        }
    }
    mejor
}

#[cfg(test)]
mod tests {
    use super::*;

    fn orden(aguja: &str, nombres: &[&str]) -> Vec<String> {
        let mut v: Vec<(i32, &str)> = nombres
            .iter()
            .filter_map(|n| score(aguja, n).map(|s| (s, *n)))
            .collect();
        v.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.len().cmp(&b.1.len())));
        v.into_iter().map(|(_, n)| n.to_string()).collect()
    }

    #[test]
    fn la_aguja_vacia_encaja_con_todo() {
        assert_eq!(score("", "Lo que sea"), Some(0));
    }

    #[test]
    fn lo_que_no_esta_no_encaja() {
        assert!(score("zzz", "Brave").is_none());
        // El orden importa: las letras tienen que salir en secuencia.
        assert!(score("evarb", "Brave").is_none());
    }

    #[test]
    fn no_distingue_mayusculas() {
        assert!(score("BRAVE", "brave").is_some());
        assert!(score("brave", "BRAVE").is_some());
    }

    #[test]
    fn las_iniciales_encuentran_el_nombre_largo() {
        assert!(score("vsc", "Visual Studio Code").is_some());
        assert!(score("vsc", "Visual Studio Code").unwrap() > score("vsc", "Vivaldi Scorer").unwrap_or(i32::MIN));
    }

    #[test]
    fn el_prefijo_gana_a_la_coincidencia_de_en_medio() {
        assert_eq!(orden("term", &["Gestor de terminales", "Terminal"])[0], "Terminal");
    }

    #[test]
    fn el_principio_de_palabra_gana_al_medio() {
        assert_eq!(orden("code", &["Decode Helper", "Visual Studio Code"])[0], "Visual Studio Code");
    }

    #[test]
    fn entre_iguales_gana_el_nombre_mas_corto() {
        assert!(score("fire", "Firefox").unwrap() > score("fire", "Firefox Developer Edition").unwrap());
    }

    #[test]
    fn el_nombre_pesa_mas_que_la_descripcion() {
        let por_nombre = score_fields("brave", &["Brave", "navegador"]).unwrap();
        let por_descripcion = score_fields("brave", &["Otro", "brave browser"]).unwrap();
        assert!(por_nombre > por_descripcion);
    }

    /// El caso real que lo destapo: «bra» tiene que dar Brave, no Yazi.
    #[test]
    fn una_coincidencia_floja_en_el_nombre_gana_a_una_buena_en_la_descripcion() {
        let brave = score_fields("bra", &["Brave", "Access the Internet"]).unwrap();
        let yazi = score_fields(
            "bra",
            &[
                "Yazi File Manager",
                "Blazing fast terminal file manager written in Rust, based on async I/O",
            ],
        )
        .unwrap();
        assert!(
            brave > yazi,
            "Brave {brave} deberia ir por delante de Yazi {yazi}"
        );
    }

    #[test]
    fn dentro_de_la_descripcion_sigue_habiendo_orden() {
        let a = score_fields("rust", &["Uno", "Rust toolchain"]).unwrap();
        let b = score_fields("rust", &["Dos", "escrito en rust, mas o menos"]).unwrap();
        assert!(a > b);
    }
}
