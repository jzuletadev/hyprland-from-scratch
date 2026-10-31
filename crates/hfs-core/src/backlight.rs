// ~/hyprland-from-scratch/crates/hfs-core/src/backlight.rs
//
// Brillo de la pantalla. Se LEE de sysfs (es instantaneo y no lanza procesos) y se
// ESCRIBE con brightnessctl, que es quien tiene el permiso via udev.
//
// El dispositivo se autodetecta. En la v1 estaba clavado «amdgpu_bl1» en el TUI y
// otras tres veces en la config de Waybar, asi que el escritorio no arrancaba igual
// en otra maquina.

use crate::{cmd, Error, Result};
use std::fs;
use std::path::PathBuf;

const SYS: &str = "/sys/class/backlight";

/// Nunca bajamos a 0: eso apaga la pantalla y deja al usuario a ciegas sin forma
/// evidente de volver.
pub const MIN_PERCENT: u8 = 1;

#[derive(Debug, Clone)]
pub struct Backlight {
    pub device: String,
    pub max: u32,
}

impl Backlight {
    /// El primer dispositivo de retroiluminacion de pantalla. Se saltan los teclados
    /// (`*kbd*`), que son otra cosa y en esta maquina ni existen.
    pub fn detect() -> Result<Self> {
        let mut nombres: Vec<String> = fs::read_dir(SYS)
            .map_err(|e| Error::new(format!("no puedo leer {SYS}: {e}")))?
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| !n.contains("kbd"))
            .collect();
        nombres.sort();

        let device = nombres
            .into_iter()
            .next()
            .ok_or_else(|| Error::new(format!("no hay ningun backlight en {SYS}")))?;
        let max = read_u32(&PathBuf::from(SYS).join(&device).join("max_brightness"))?;
        if max == 0 {
            return Err(Error::new(format!("{device} dice que su brillo maximo es 0")));
        }
        Ok(Self { device, max })
    }

    pub fn raw(&self) -> Result<u32> {
        read_u32(&PathBuf::from(SYS).join(&self.device).join("brightness"))
    }

    pub fn percent(&self) -> Result<u8> {
        let raw = self.raw()?;
        Ok(((raw as f64 / self.max as f64) * 100.0).round().clamp(0.0, 100.0) as u8)
    }

    pub fn set(&self, percent: u8) -> Result<()> {
        let p = percent.clamp(MIN_PERCENT, 100);
        cmd::run(
            "brightnessctl",
            &["-q", "-d", &self.device, "set", &format!("{p}%")],
        )
        .map(|_| ())
    }

    /// Sube o baja. Devuelve el valor ya aplicado.
    pub fn step(&self, delta: i16) -> Result<u8> {
        let next = (self.percent()? as i16 + delta).clamp(MIN_PERCENT as i16, 100) as u8;
        self.set(next)?;
        Ok(next)
    }
}

fn read_u32(p: &PathBuf) -> Result<u32> {
    let s = fs::read_to_string(p).map_err(|e| Error::new(format!("no puedo leer {}: {e}", p.display())))?;
    s.trim()
        .parse()
        .map_err(|_| Error::new(format!("{} no contiene un numero: {:?}", p.display(), s.trim())))
}
