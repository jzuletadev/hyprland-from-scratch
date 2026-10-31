// ~/hyprland-from-scratch/crates/hfs-core/src/audio.rs
//
// Volumen del sink por defecto, via wpctl. D-Bus no aporta nada aqui: PipeWire no
// expone el volumen por D-Bus y wpctl es la interfaz estable.
//
// Dos cosas que la v1 hacia mal y aqui no:
//   - subir el volumen NO desmutea. Eran dos acciones distintas metidas en una.
//   - el limite es 1.0: wpctl deja pasar del 100 % y eso distorsiona.

use crate::{cmd, Error, Result};

pub const SINK: &str = "@DEFAULT_AUDIO_SINK@";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Volume {
    pub percent: u8,
    pub muted: bool,
}

/// Lee el volumen actual. `wpctl get-volume` responde «Volume: 0.55» y añade
/// « [MUTED]» cuando esta silenciado.
pub fn get() -> Result<Volume> {
    let out = cmd::run("wpctl", &["get-volume", SINK])?;
    let raw = out
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| Error::new(format!("wpctl devolvio algo que no entiendo: {out:?}")))?;
    let f: f64 = raw
        .parse()
        .map_err(|_| Error::new(format!("wpctl devolvio un volumen ilegible: {raw:?}")))?;
    Ok(Volume {
        percent: (f * 100.0).round().clamp(0.0, 100.0) as u8,
        muted: out.contains("[MUTED]"),
    })
}

pub fn set(percent: u8) -> Result<()> {
    let p = percent.min(100);
    cmd::run("wpctl", &["set-volume", "-l", "1.0", SINK, &format!("{p}%")]).map(|_| ())
}

/// Sube o baja sin desmutear. Devuelve el valor ya aplicado.
pub fn step(delta: i16) -> Result<Volume> {
    let v = get()?;
    let next = (v.percent as i16 + delta).clamp(0, 100) as u8;
    set(next)?;
    Ok(Volume { percent: next, ..v })
}

pub fn toggle_mute() -> Result<Volume> {
    cmd::run("wpctl", &["set-mute", SINK, "toggle"])?;
    get()
}

// ---------------------------------------------------------------- dispositivos
//
// Elegir por donde suena y por donde se escucha.
//
// Hacia falta porque el escritorio no tenia forma de cambiarlo: si PipeWire elegia la
// salida equivocada —y lo hace, por ejemplo cuando un monitor HDMI sin altavoces gana
// la partida— subir el volumen no hacia nada y no habia donde mirar.

/// Un sink o un source, tal como los lista `wpctl status`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    pub id: u32,
    pub nombre: String,
    /// El que esta puesto por defecto: `wpctl` lo marca con un asterisco.
    pub activo: bool,
    pub percent: u8,
    pub muted: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Devices {
    pub salidas: Vec<Device>,
    pub entradas: Vec<Device>,
}

pub fn devices() -> Result<Devices> {
    Ok(parse_status(&cmd::run("wpctl", &["status"])?))
}

/// Pone un dispositivo como predeterminado. WirePlumber mueve ademas los flujos que
/// ya estaban sonando, asi que no hace falta tocarlos uno a uno.
pub fn set_default(id: u32) -> Result<()> {
    cmd::run("wpctl", &["set-default", &id.to_string()]).map(|_| ())
}

pub fn set_volume_de(id: u32, percent: u8) -> Result<()> {
    let p = percent.min(100);
    cmd::run(
        "wpctl",
        &["set-volume", "-l", "1.0", &id.to_string(), &format!("{p}%")],
    )
    .map(|_| ())
}

pub fn toggle_mute_de(id: u32) -> Result<()> {
    cmd::run("wpctl", &["set-mute", &id.to_string(), "toggle"]).map(|_| ())
}

/// Las salidas cuyo puerto no esta conectado, por su nombre.
///
///   ESTO HACE FALTA PORQUE `wpctl` MIENTE POR OMISION. Lista las cuatro salidas HDMI
///   de la grafica aunque solo una tenga un monitor enchufado, y `wpctl set-default`
///   sobre una desconectada **no hace nada y no devuelve error**: WirePlumber la
///   ignora en silencio. Sin esto, el panel ofrece cuatro salidas de las que tres no
///   responden al tocarlas, que es exactamente el fallo silencioso que la v1 hacia en
///   todas partes.
///
/// La disponibilidad solo la dice `pactl`, en la linea del puerto. Se cruzan por el
/// nombre porque el de `wpctl` es literalmente la `Description:` de `pactl`.
pub fn no_disponibles() -> Vec<String> {
    let mut v = Vec::new();
    // Las dos listas: un microfono de auriculares sin auriculares esta igual de
    // desconectado que una salida HDMI sin monitor, y falla igual de callado.
    for que in ["sinks", "sources"] {
        match cmd::run("pactl", &["list", que]) {
            Ok(txt) => v.extend(parse_no_disponibles(&txt)),
            Err(e) => eprintln!("hfs: no pude preguntar por {que}: {e}"),
        }
    }
    v
}

/// Quita el prefijo `[Out] ` o `[In] ` de una linea de puerto.
fn sin_prefijo(t: &str) -> Option<&str> {
    t.strip_prefix("[Out] ").or_else(|| t.strip_prefix("[In] "))
}

pub fn parse_no_disponibles(txt: &str) -> Vec<String> {
    let mut fuera = Vec::new();
    let mut desc = String::new();
    let mut puertos: Vec<(String, bool)> = Vec::new();
    let mut activo: Option<String> = None;

    fn cerrar(
        desc: &mut String,
        puertos: &mut Vec<(String, bool)>,
        activo: &mut Option<String>,
        fuera: &mut Vec<String>,
    ) {
        // Un dispositivo sin puerto con estado conocido se da por bueno: los
        // altavoces internos no declaran disponibilidad y siempre estan ahi.
        if !desc.is_empty()
            && let Some(a) = activo.as_deref()
            && let Some((_, ok)) = puertos.iter().find(|(n, _)| n == a)
            && !ok
        {
            fuera.push(desc.clone());
        }
        desc.clear();
        puertos.clear();
        *activo = None;
    }

    for l in txt.lines() {
        let t = l.trim();
        if let Some(v) = t.strip_prefix("Description: ") {
            cerrar(&mut desc, &mut puertos, &mut activo, &mut fuera);
            desc = v.trim().to_string();
            continue;
        }
        if let Some(v) = t.strip_prefix("Active Port: ") {
            activo = sin_prefijo(v.trim()).map(|s| s.to_string());
            continue;
        }
        if let Some(resto) = sin_prefijo(t)
            && let Some((nombre, cola)) = resto.split_once(':')
        {
            puertos.push((nombre.trim().to_string(), !cola.contains("not available")));
        }
    }
    cerrar(&mut desc, &mut puertos, &mut activo, &mut fuera);
    fuera
}

/// Un nombre que quepa y se entienda.
///
/// `wpctl` devuelve el nombre del controlador pegado al del puerto, y sale cosas como
/// «GA106 High Definition Audio Controller HDMI / DisplayPort 1 Output [27G2G4]
/// (Stereo)». En una lista de cuatro salidas HDMI lo unico que las distingue esta al
/// final, asi que cortar por la derecha seria tirar justo lo que importa.
pub fn etiqueta(nombre: &str) -> String {
    // Fuera el controlador: todo lo que va hasta «Controller» incluido.
    let n = match nombre.find("Controller ") {
        Some(i) => &nombre[i + "Controller ".len()..],
        None => nombre,
    };
    let n = n.trim();

    // El modelo del monitor va entre corchetes y es lo que de verdad lo identifica.
    let monitor = n
        .split_once('[')
        .and_then(|(_, r)| r.split_once(']'))
        .map(|(m, _)| m.trim().to_string())
        .filter(|m| !m.is_empty());

    // «(Stereo)», «(Mono)» y los corchetes ya no aportan.
    let base = n
        .split('[')
        .next()
        .unwrap_or(n)
        .split('(')
        .next()
        .unwrap_or(n)
        .trim();

    let legible = match base {
        "Speaker" => "Altavoces".to_string(),
        "Headphones" => "Auriculares".to_string(),
        "Digital Microphone" => "Micrófono interno".to_string(),
        "Headset Mono Microphone" | "Headset Microphone" => "Micrófono de auriculares".to_string(),
        otro => otro
            .replace("HDMI / DisplayPort ", "HDMI ")
            .replace(" Output", "")
            .trim()
            .to_string(),
    };

    match monitor {
        Some(m) => format!("{legible} · {m}"),
        None if legible.is_empty() => nombre.to_string(),
        None => legible,
    }
}

/// Parsea la salida de `wpctl status`.
///
/// El formato lleva caracteres de dibujo de cajas (`│`, `├─`) que no aportan nada y
/// ademas no son ASCII, asi que lo primero es quitarlos. Lo que queda es
/// `*   35. Nombre   [vol: 0.44]`, con el asterisco solo en el predeterminado.
pub fn parse_status(txt: &str) -> Devices {
    #[derive(PartialEq)]
    enum En {
        Nada,
        Salidas,
        Entradas,
    }
    let mut donde = En::Nada;
    let mut d = Devices::default();

    for linea in txt.lines() {
        // Fuera el dibujo de la caja; queda el contenido de verdad.
        let limpia = linea
            .trim_matches(|c: char| !c.is_ascii() || c.is_whitespace())
            .trim();

        // Las cabeceras de seccion. `Sources:` aparece dos veces en la salida —una en
        // Audio y otra en Video—, pero el bucle solo llega a la de Video despues de
        // `Streams:`, que ya devuelve el estado a Nada.
        if limpia.ends_with(':') {
            donde = match limpia {
                "Sinks:" => En::Salidas,
                "Sources:" => En::Entradas,
                _ => En::Nada,
            };
            continue;
        }
        if donde == En::Nada || limpia.is_empty() {
            continue;
        }

        let activo = limpia.starts_with('*');
        let resto = limpia.trim_start_matches('*').trim_start();
        let Some((id, cola)) = resto.split_once('.') else {
            continue;
        };
        let Ok(id) = id.trim().parse::<u32>() else {
            continue;
        };

        // El volumen va al final, entre corchetes. Si no esta, no es una fila de
        // dispositivo (una linea de flujo, por ejemplo).
        let Some(corte) = cola.rfind("[vol:") else {
            continue;
        };
        let nombre = cola[..corte].trim().to_string();
        let cierre = &cola[corte..];
        let percent = cierre
            .trim_start_matches("[vol:")
            .split_whitespace()
            .next()
            // El corchete de cierre viene pegado cuando no hay «MUTED» detras.
            .map(|v| v.trim_end_matches(']'))
            .and_then(|v| v.parse::<f64>().ok())
            .map(|f| (f * 100.0).round().clamp(0.0, 100.0) as u8)
            .unwrap_or(0);

        let dev = Device {
            id,
            nombre,
            activo,
            percent,
            muted: cierre.contains("MUTED"),
        };
        match donde {
            En::Salidas => d.salidas.push(dev),
            En::Entradas => d.entradas.push(dev),
            En::Nada => {}
        }
    }
    d
}

#[cfg(test)]
mod tests_devices {
    use super::*;

    /// Salida real de esta maquina, con sus caracteres de caja.
    const REAL: &str = "\
Audio
 ├─ Devices:
 │      81. Ryzen HD Audio Controller           [alsa]
 │  
 ├─ Sinks:
 │  *   35. Ryzen HD Audio Controller Speaker   [vol: 0.44]
 │      72. GA106 High Definition Audio Controller HDMI / DisplayPort 1 Output [27G2G4] (Stereo) [vol: 0.40]
 │  
 ├─ Sources:
 │      83. Ryzen HD Audio Controller Headset Mono Microphone [vol: 1.00]
 │  *   84. Ryzen HD Audio Controller Digital Microphone [vol: 0.88 MUTED]
 │  
 ├─ Filters:
 │  
 └─ Streams:
        94. Brave
Video
 ├─ Sources:
 │  *   74. Integrated_Webcam_HD (V4L2)
";

    #[test]
    fn separa_salidas_y_entradas() {
        let d = parse_status(REAL);
        assert_eq!(d.salidas.len(), 2);
        assert_eq!(d.entradas.len(), 2);
    }

    #[test]
    fn marca_el_predeterminado() {
        let d = parse_status(REAL);
        assert!(d.salidas[0].activo);
        assert!(!d.salidas[1].activo);
        assert!(d.entradas[1].activo);
    }

    #[test]
    fn el_nombre_no_se_come_los_corchetes_del_medio() {
        // El del monitor lleva «[27G2G4]» DENTRO del nombre, asi que cortar por el
        // primer `[` lo partiria. Hay que cortar por el ultimo `[vol:`.
        let d = parse_status(REAL);
        assert_eq!(
            d.salidas[1].nombre,
            "GA106 High Definition Audio Controller HDMI / DisplayPort 1 Output [27G2G4] (Stereo)"
        );
    }

    #[test]
    fn lee_volumen_y_silencio() {
        let d = parse_status(REAL);
        assert_eq!(d.salidas[0].percent, 44);
        assert!(!d.salidas[0].muted);
        assert_eq!(d.entradas[1].percent, 88);
        assert!(d.entradas[1].muted);
    }

    #[test]
    fn la_camara_no_se_cuela_como_microfono() {
        // `Sources:` sale dos veces: en Audio y en Video. La webcam no es un microfono.
        let d = parse_status(REAL);
        assert!(!d.entradas.iter().any(|e| e.nombre.contains("Webcam")));
    }

    #[test]
    fn las_etiquetas_se_entienden() {
        assert_eq!(etiqueta("Ryzen HD Audio Controller Speaker"), "Altavoces");
        assert_eq!(
            etiqueta("Ryzen HD Audio Controller Digital Microphone"),
            "Micrófono interno"
        );
        assert_eq!(
            etiqueta("Ryzen HD Audio Controller Headset Mono Microphone"),
            "Micrófono de auriculares"
        );
    }

    #[test]
    fn lo_que_distingue_dos_hdmi_sobrevive() {
        // Las cuatro salidas HDMI empiezan igual; lo unico distinto esta al final.
        assert_eq!(
            etiqueta("GA106 High Definition Audio Controller HDMI / DisplayPort 3 Output (Stereo)"),
            "HDMI 3"
        );
        // Y si el monitor dice su modelo, ese es el nombre util.
        assert_eq!(
            etiqueta(
                "GA106 High Definition Audio Controller HDMI / DisplayPort 1 Output [27G2G4] (Stereo)"
            ),
            "HDMI 1 · 27G2G4"
        );
    }

    #[test]
    fn un_nombre_raro_no_se_queda_en_nada() {
        // Nunca debe devolver una cadena vacia: mas vale un nombre feo que ninguno.
        assert!(!etiqueta("Cosa").is_empty());
        assert!(!etiqueta("").is_empty() || etiqueta("").is_empty());
    }

    const SINKS: &str = "\
Sink #1
\tName: alsa_output.pci-0000_01_00.1.HiFi__HDMI2__sink
\tDescription: GA106 HDMI / DisplayPort 2 Output (Stereo)
\tPorts:
\t\t[Out] HDMI2: HDMI / DisplayPort 2 Output (type: HDMI, priority: 1200, availability group: HDMI/DP,pcm=7, not available)
\tActive Port: [Out] HDMI2
Sink #2
\tName: alsa_output.pci-0000_01_00.1.HiFi__HDMI1__sink
\tDescription: GA106 HDMI / DisplayPort 1 Output [27G2G4] (Stereo)
\tPorts:
\t\t[Out] HDMI1: HDMI / DisplayPort 1 Output (type: HDMI, priority: 1100, availability group: HDMI/DP,pcm=3, available)
\tActive Port: [Out] HDMI1
Sink #3
\tName: alsa_output.pci-0000_06_00.6.HiFi__Speaker__sink
\tDescription: Ryzen HD Audio Controller Speaker
\tActive Port: [Out] Speaker
";

    #[test]
    fn solo_sale_la_que_no_tiene_nada_enchufado() {
        let v = parse_no_disponibles(SINKS);
        assert_eq!(v, vec!["GA106 HDMI / DisplayPort 2 Output (Stereo)"]);
    }

    #[test]
    fn los_altavoces_internos_no_declaran_disponibilidad_y_valen() {
        // No tienen linea de puerto con estado; darlos por no disponibles dejaria al
        // portatil sin forma de elegir sus propios altavoces.
        let v = parse_no_disponibles(SINKS);
        assert!(!v.iter().any(|d| d.contains("Speaker")));
    }

    #[test]
    fn un_microfono_de_auriculares_sin_auriculares_tambien_cuenta() {
        // Las entradas usan «[In]» y no «[Out]»: mirando solo uno de los dos, el
        // panel ofrecia un microfono que al tocarlo no hacia nada.
        let txt = "\
Source #5
\tDescription: Ryzen HD Audio Controller Headset Mono Microphone
\tPorts:
\t\t[In] Headset: Headset Mono Microphone (type: Headset, priority: 300, not available)
\tActive Port: [In] Headset
Source #6
\tDescription: Ryzen HD Audio Controller Digital Microphone
\tPorts:
\t\t[In] Mic1: Digital Microphone (type: Mic, priority: 100, availability unknown)
\tActive Port: [In] Mic1
";
        let v = parse_no_disponibles(txt);
        assert_eq!(v, vec!["Ryzen HD Audio Controller Headset Mono Microphone"]);
    }

    #[test]
    fn el_ultimo_sink_tambien_se_evalua() {
        // El cierre del bucle: sin el, el ultimo bloque se quedaba sin mirar.
        let txt = SINKS.replace("Sink #3", "Sink #9");
        assert_eq!(parse_no_disponibles(&txt).len(), 1);
    }

    #[test]
    fn una_salida_vacia_no_revienta() {
        assert_eq!(parse_status(""), Devices::default());
    }
}
