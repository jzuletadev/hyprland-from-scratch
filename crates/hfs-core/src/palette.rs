// ~/hyprland-from-scratch/crates/hfs-core/src/palette.rs
//
// La paleta activa, en color real de 24 bits.
//
// En la v1 el tema se sacaba raspando flavor.toml y tmtheme.xml de Yazi con regex
// dentro de un heredoc de Python cuyo stdout se hacia `eval` como shell. Como Yazi
// sale del sistema, las paletas pasan a ser ficheros nuestros en palettes/*.toml:
// planos, legibles y sin nada que adivinar.
//
// El formato es a proposito trivial —  clave = "#rrggbb" por linea, comentarios con
// # al principio de la linea  — para no necesitar un parser de TOML entero.

use crate::{paths, Error, Result};
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn parse(s: &str) -> Result<Self> {
        let h = s.trim().trim_start_matches('#');
        let v = match h.len() {
            6 => u32::from_str_radix(h, 16).ok(),
            // #rgb abreviado
            3 => {
                let mut full = String::with_capacity(6);
                for c in h.chars() {
                    full.push(c);
                    full.push(c);
                }
                u32::from_str_radix(&full, 16).ok()
            }
            _ => None,
        }
        .ok_or_else(|| Error::new(format!("«{s}» no es un color hexadecimal")))?;
        Ok(Self::new((v >> 16) as u8, (v >> 8) as u8, v as u8))
    }

    /// Para CSS de GTK, que no admite `#rrggbbaa` en todas las versiones.
    pub fn rgba(&self, a: f64) -> String {
        format!("rgba({},{},{},{:.3})", self.r, self.g, self.b, a)
    }

    pub fn as_f64(&self) -> (f64, f64, f64) {
        (
            self.r as f64 / 255.0,
            self.g as f64 / 255.0,
            self.b as f64 / 255.0,
        )
    }

    /// Luminancia relativa, para decidir si un fondo es claro u oscuro y si encima
    /// de el va texto claro o texto oscuro.
    pub fn luminance(&self) -> f64 {
        let f = |c: u8| {
            let c = c as f64 / 255.0;
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * f(self.r) + 0.7152 * f(self.g) + 0.0722 * f(self.b)
    }

    /// Mezcla hacia otro color. `t = 0` es este, `t = 1` es el otro.
    pub fn mix(&self, other: Rgb, t: f64) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * t).round() as u8;
        Rgb::new(m(self.r, other.r), m(self.g, other.g), m(self.b, other.b))
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Dark => "dark",
            Mode::Light => "light",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub name: String,
    pub mode: Mode,
    /// El fondo del escritorio: lo mas oscuro (o lo mas claro) de la paleta.
    pub bg: Rgb,
    /// El fondo de un panel.
    pub surface: Rgb,
    /// Un escalon por encima de surface: pistas, pastillas, separadores rellenos.
    pub overlay: Rgb,
    pub fg: Rgb,
    pub muted: Rgb,
    pub dim: Rgb,
    pub accent: Rgb,
    /// Lo que va escrito ENCIMA del acento.
    pub on_accent: Rgb,
    pub ok: Rgb,
    pub warn: Rgb,
    pub bad: Rgb,
    pub info: Rgb,
}

impl Palette {
    /// La paleta del tema activo. Si algo falla devuelve la de reserva en vez de
    /// reventar: quedarse sin escritorio por un fichero mal escrito seria peor.
    /// El motivo del fallo viaja en el `Err` para quien quiera enseñarlo.
    pub fn load() -> (Self, Option<Error>) {
        match Self::try_load() {
            Ok(p) => (p, None),
            Err(e) => (Self::fallback(), Some(e)),
        }
    }

    pub fn try_load() -> Result<Self> {
        Self::by_name(&current_theme_name())
    }

    pub fn by_name(name: &str) -> Result<Self> {
        Self::from_file(&paths::palettes().join(format!("{name}.toml")))
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let txt = fs::read_to_string(path)
            .map_err(|e| Error::new(format!("no puedo leer la paleta {}: {e}", path.display())))?;
        let mut p = Self::parse(&txt)?;
        if p.name.is_empty() {
            p.name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
        }
        Ok(p)
    }

    pub fn parse(txt: &str) -> Result<Self> {
        let kv = parse_kv(txt);
        let color = |k: &str| -> Result<Rgb> {
            let v = kv
                .get(k)
                .ok_or_else(|| Error::new(format!("a la paleta le falta «{k}»")))?;
            Rgb::parse(v).map_err(|e| Error::new(format!("en «{k}»: {e}")))
        };

        let bg = color("bg")?;
        let accent = color("accent")?;
        let fg = color("fg")?;

        let mode = match kv.get("mode").map(|s| s.as_str()) {
            Some("light") => Mode::Light,
            Some("dark") => Mode::Dark,
            // Si no lo dice, lo decide el fondo.
            _ => {
                if bg.luminance() > 0.5 {
                    Mode::Light
                } else {
                    Mode::Dark
                }
            }
        };

        // Los opcionales se derivan en vez de obligar a escribirlos.
        let surface = kv
            .get("surface")
            .map(|v| Rgb::parse(v))
            .transpose()?
            .unwrap_or_else(|| bg.mix(fg, 0.06));
        let overlay = kv
            .get("overlay")
            .map(|v| Rgb::parse(v))
            .transpose()?
            .unwrap_or_else(|| bg.mix(fg, 0.14));
        let muted = kv
            .get("muted")
            .map(|v| Rgb::parse(v))
            .transpose()?
            .unwrap_or_else(|| fg.mix(bg, 0.40));
        let dim = kv
            .get("dim")
            .map(|v| Rgb::parse(v))
            .transpose()?
            .unwrap_or_else(|| fg.mix(bg, 0.62));
        // El texto sobre el acento se elige por contraste, no a mano: asi una paleta
        // con acento claro no acaba con letras blancas encima.
        let on_accent = kv
            .get("on_accent")
            .map(|v| Rgb::parse(v))
            .transpose()?
            .unwrap_or(if accent.luminance() > 0.4 { bg } else { fg });

        Ok(Self {
            name: kv.get("name").cloned().unwrap_or_default(),
            mode,
            bg,
            surface,
            overlay,
            fg,
            muted,
            dim,
            accent,
            on_accent,
            ok: kv.get("ok").map(|v| Rgb::parse(v)).transpose()?.unwrap_or(accent),
            warn: kv.get("warn").map(|v| Rgb::parse(v)).transpose()?.unwrap_or(accent),
            bad: kv.get("bad").map(|v| Rgb::parse(v)).transpose()?.unwrap_or(accent),
            info: kv.get("info").map(|v| Rgb::parse(v)).transpose()?.unwrap_or(accent),
        })
    }

    /// Lo que se usa cuando no hay paleta que valga. Es la del boceto.
    pub fn fallback() -> Self {
        Self {
            name: "reserva".into(),
            mode: Mode::Dark,
            bg: Rgb::new(0x0b, 0x0d, 0x10),
            surface: Rgb::new(0x13, 0x16, 0x1b),
            overlay: Rgb::new(0x24, 0x29, 0x32),
            fg: Rgb::new(0xe7, 0xe9, 0xee),
            muted: Rgb::new(0x8a, 0x91, 0x9d),
            dim: Rgb::new(0x5d, 0x64, 0x6f),
            accent: Rgb::new(0xf2, 0xcd, 0x63),
            on_accent: Rgb::new(0x19, 0x14, 0x06),
            ok: Rgb::new(0x8f, 0xd4, 0x7a),
            warn: Rgb::new(0xe8, 0xa0, 0x5c),
            bad: Rgb::new(0xe2, 0x75, 0x6b),
            info: Rgb::new(0x7f, 0xbb, 0xb3),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.mode == Mode::Dark
    }
}

/// El tema activo, tal como lo deja theme-manager.sh.
pub fn current_theme_name() -> String {
    if let Some(v) = std::env::var_os("HFS_THEME") {
        return v.to_string_lossy().into_owned();
    }
    fs::read_to_string(paths::current_theme_file())
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "monokai".into())
}

/// Las paletas disponibles, por nombre, leidas del disco.
pub fn available() -> Vec<String> {
    let mut v = Vec::new();
    if let Ok(dir) = fs::read_dir(paths::palettes()) {
        for e in dir.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("toml") {
                continue;
            }
            if let Some(stem) = p.file_stem().and_then(|x| x.to_str()) {
                v.push(stem.to_string());
            }
        }
    }
    v.sort();
    v
}

/// `clave = valor`, una por linea. Comentario = linea que empieza por #.
/// El valor puede ir entre comillas o no.
fn parse_kv(txt: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for linea in txt.lines() {
        let l = linea.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }
        let Some((k, v)) = l.split_once('=') else {
            continue;
        };
        let v = v.trim();
        let v = v
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(v);
        out.insert(k.trim().to_string(), v.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        assert_eq!(Rgb::parse("#ffd866").unwrap(), Rgb::new(0xff, 0xd8, 0x66));
        assert_eq!(Rgb::parse("FFD866").unwrap(), Rgb::new(0xff, 0xd8, 0x66));
        assert_eq!(Rgb::parse("#fff").unwrap(), Rgb::new(255, 255, 255));
        assert!(Rgb::parse("#zz").is_err());
    }

    #[test]
    fn el_modo_sale_del_fondo_si_no_se_dice() {
        let oscuro = Palette::parse("bg=\"#111111\"\nfg=\"#eeeeee\"\naccent=\"#ffd866\"").unwrap();
        assert_eq!(oscuro.mode, Mode::Dark);
        let claro = Palette::parse("bg=\"#fafafa\"\nfg=\"#111111\"\naccent=\"#3344ff\"").unwrap();
        assert_eq!(claro.mode, Mode::Light);
    }

    #[test]
    fn el_texto_sobre_el_acento_se_elige_por_contraste() {
        let p = Palette::parse("bg=\"#111111\"\nfg=\"#eeeeee\"\naccent=\"#ffd866\"").unwrap();
        assert_eq!(p.on_accent, p.bg, "acento claro -> texto oscuro");
        let p = Palette::parse("bg=\"#111111\"\nfg=\"#eeeeee\"\naccent=\"#3b4bd8\"").unwrap();
        assert_eq!(p.on_accent, p.fg, "acento oscuro -> texto claro");
    }

    #[test]
    fn falta_una_clave_obligatoria() {
        let e = Palette::parse("bg=\"#111111\"").unwrap_err();
        assert!(e.message().contains("accent"), "{}", e);
    }

    #[test]
    fn los_comentarios_no_se_comen_los_colores() {
        let p = Palette::parse("# un comentario\nbg = \"#221F22\"  \nfg=\"#fcfcfa\"\naccent=\"#ffd866\"")
            .unwrap();
        assert_eq!(p.bg, Rgb::new(0x22, 0x1f, 0x22));
    }
}
