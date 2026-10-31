// ~/hyprland-from-scratch/crates/hfs-core/src/sys.rs
//
// CPU, memoria, red, batería y tiempo encendido, leídos de /proc y /sys.
//
// Todo son lecturas de ficheros: ni un subproceso y ni una dependencia. Es lo que
// permite que esto viva en el nucleo y se pruebe contra texto de ejemplo sin tener
// una maquina delante.
//
// El uso de CPU y la velocidad de red no son valores, son DERIVADAS: hay que tomar
// dos muestras y restarlas. Por eso la API devuelve muestras y no porcentajes; quien
// la use guarda la anterior. La v1 llamaba a `top` para esto, que cuesta un proceso y
// 1,5 s de espera.

use std::fs;
use std::path::Path;
use std::time::Duration;

use crate::{Error, Result};

fn leer(ruta: &str) -> Result<String> {
    std::fs::read_to_string(ruta).map_err(|e| Error::new(format!("no pude leer {ruta}: {e}")))
}

fn leer_num(ruta: &str) -> Option<u64> {
    std::fs::read_to_string(ruta).ok()?.trim().parse().ok()
}

// ---------------------------------------------------------------- CPU

/// Una muestra del contador acumulado de /proc/stat. No significa nada por si sola.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuSample {
    pub total: u64,
    pub idle: u64,
}

pub fn cpu_sample() -> Result<CpuSample> {
    parse_cpu_sample(&leer("/proc/stat")?)
}

fn parse_cpu_sample(texto: &str) -> Result<CpuSample> {
    let linea = texto
        .lines()
        .find(|l| l.starts_with("cpu "))
        .ok_or_else(|| Error::new("/proc/stat no trae la linea «cpu»"))?;
    let campos: Vec<u64> = linea
        .split_whitespace()
        .skip(1)
        .filter_map(|c| c.parse().ok())
        .collect();
    if campos.len() < 5 {
        return Err(Error::new("/proc/stat trae menos campos de los esperados"));
    }
    // idle + iowait: los dos cuentan como «sin hacer nada».
    Ok(CpuSample {
        total: campos.iter().sum(),
        idle: campos[3] + campos[4],
    })
}

/// El uso entre dos muestras. Si no ha pasado nada entre ellas, devuelve 0.
pub fn cpu_percent(antes: CpuSample, ahora: CpuSample) -> u8 {
    let dt = ahora.total.saturating_sub(antes.total);
    if dt == 0 {
        return 0;
    }
    let di = ahora.idle.saturating_sub(antes.idle);
    let uso = 100.0 * (1.0 - di as f64 / dt as f64);
    uso.clamp(0.0, 100.0).round() as u8
}

/// El nombre comercial del procesador, para no enseñar solo un porcentaje.
pub fn cpu_model() -> Option<String> {
    let texto = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    let bruto = texto
        .lines()
        .find(|l| l.starts_with("model name"))?
        .split_once(':')?
        .1
        .trim()
        .to_string();
    // «AMD Ryzen 5 7520U with Radeon Graphics» -> «Ryzen 5 7520U»
    let limpio = bruto
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("CPU", "");
    let limpio = limpio
        .split(" with ")
        .next()
        .unwrap_or(&limpio)
        .split(" @ ")
        .next()
        .unwrap_or(&limpio)
        .trim()
        .trim_start_matches("AMD ")
        .trim_start_matches("Intel ")
        .trim();
    Some(limpio.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// La temperatura del paquete, en grados. Busca el hwmon del procesador por nombre:
/// el numero de hwmon cambia de arranque a arranque, asi que clavarlo no vale.
pub fn cpu_temp() -> Option<f64> {
    const CHIPS: [&str; 3] = ["k10temp", "coretemp", "zenpower"];
    let dir = std::fs::read_dir("/sys/class/hwmon").ok()?;
    for e in dir.flatten() {
        let nombre = std::fs::read_to_string(e.path().join("name")).ok()?;
        if !CHIPS.contains(&nombre.trim()) {
            continue;
        }
        let ruta = e.path().join("temp1_input");
        if let Some(m) = leer_num(ruta.to_str()?) {
            return Some(m as f64 / 1000.0);
        }
    }
    None
}

// ---------------------------------------------------------------- memoria

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Memory {
    pub total_kb: u64,
    pub used_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

impl Memory {
    pub fn percent(&self) -> u8 {
        if self.total_kb == 0 {
            return 0;
        }
        ((self.used_kb as f64 / self.total_kb as f64) * 100.0).round() as u8
    }
}

pub fn memory() -> Result<Memory> {
    parse_memory(&leer("/proc/meminfo")?)
}

fn parse_memory(texto: &str) -> Result<Memory> {
    let campo = |clave: &str| -> u64 {
        texto
            .lines()
            .find(|l| l.starts_with(clave))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    let total = campo("MemTotal:");
    if total == 0 {
        return Err(Error::new("/proc/meminfo no trae MemTotal"));
    }
    // «Usada» es total menos disponible, no total menos libre: lo que esta en cache
    // se puede recuperar y contarlo como usado es lo que hace que todos los monitores
    // enseñen la memoria al 90 %.
    let disponible = campo("MemAvailable:");
    let swap_total = campo("SwapTotal:");
    Ok(Memory {
        total_kb: total,
        used_kb: total.saturating_sub(disponible),
        swap_total_kb: swap_total,
        swap_used_kb: swap_total.saturating_sub(campo("SwapFree:")),
    })
}

// ---------------------------------------------------------------- red

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct NetSample {
    pub rx: u64,
    pub tx: u64,
}

pub fn net_sample() -> Result<NetSample> {
    parse_net(&leer("/proc/net/dev")?)
}

fn parse_net(texto: &str) -> Result<NetSample> {
    let mut s = NetSample::default();
    for linea in texto.lines().skip(2) {
        let Some((iface, resto)) = linea.split_once(':') else {
            continue;
        };
        let iface = iface.trim();
        // La interfaz de loopback no es trafico: contarla dobla todo lo local.
        if iface == "lo" || iface.starts_with("docker") || iface.starts_with("veth") {
            continue;
        }
        let campos: Vec<u64> = resto
            .split_whitespace()
            .filter_map(|c| c.parse().ok())
            .collect();
        if campos.len() >= 9 {
            s.rx += campos[0];
            s.tx += campos[8];
        }
    }
    Ok(s)
}

/// Bytes por segundo entre dos muestras.
pub fn net_rate(antes: NetSample, ahora: NetSample, segundos: f64) -> (f64, f64) {
    if segundos <= 0.0 {
        return (0.0, 0.0);
    }
    (
        ahora.rx.saturating_sub(antes.rx) as f64 / segundos,
        ahora.tx.saturating_sub(antes.tx) as f64 / segundos,
    )
}

// ---------------------------------------------------------------- tiempo encendido

pub fn uptime() -> Result<Duration> {
    let texto = leer("/proc/uptime")?;
    let s: f64 = texto
        .split_whitespace()
        .next()
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| Error::new("/proc/uptime ilegible"))?;
    Ok(Duration::from_secs_f64(s))
}

// ---------------------------------------------------------------- disco

/// El nombre del dispositivo montado en `/`, para que el panel de disco diga de que
/// disco habla. Solo el nombre: «/dev/nvme0n1p2» -> «nvme0n1p2».
pub fn root_device() -> Option<String> {
    let texto = std::fs::read_to_string("/proc/mounts").ok()?;
    let linea = texto
        .lines()
        .find(|l| l.split_whitespace().nth(1) == Some("/"))?;
    let dev = linea.split_whitespace().next()?;
    Some(dev.rsplit('/').next().unwrap_or(dev).to_string())
}

// ---------------------------------------------------------------- bateria

#[derive(Debug, Clone, PartialEq)]
pub struct Battery {
    pub percent: u8,
    /// Tal cual lo dice el kernel: Charging, Discharging, Full, Not charging.
    pub status: String,
    /// Consumo o carga, en vatios.
    pub watts: Option<f64>,
    /// Cuanta capacidad le queda frente a la que tenia de fabrica.
    pub health: Option<u8>,
    pub time_left: Option<Duration>,
    pub cycles: Option<u32>,
    pub model: Option<String>,
}

impl Battery {
    pub fn charging(&self) -> bool {
        self.status == "Charging"
    }
    pub fn full(&self) -> bool {
        self.status == "Full"
    }
}

/// La primera bateria que encuentre. Devuelve None en un sobremesa, que no es un
/// fallo: es que no hay.
pub fn battery() -> Option<Battery> {
    let dir = std::fs::read_dir("/sys/class/power_supply").ok()?;
    for e in dir.flatten() {
        let p = e.path();
        let tipo = std::fs::read_to_string(p.join("type")).unwrap_or_default();
        if tipo.trim() != "Battery" {
            continue;
        }
        let f = |n: &str| p.join(n).to_str().and_then(leer_num);
        let txt = |n: &str| {
            std::fs::read_to_string(p.join(n))
                .ok()
                .map(|s| s.trim().to_string())
        };

        // Unos kernels dan carga (uAh) y otros energia (uWh). Las dos sirven igual
        // mientras se comparen entre si; el vatiaje necesita ademas el voltaje.
        let (ahora, lleno, diseño, ritmo) = match (f("charge_now"), f("energy_now")) {
            (Some(c), _) => (
                Some(c),
                f("charge_full"),
                f("charge_full_design"),
                f("current_now"),
            ),
            (None, Some(e)) => (
                Some(e),
                f("energy_full"),
                f("energy_full_design"),
                f("power_now"),
            ),
            _ => (None, None, None, None),
        };
        let voltios = f("voltage_now").map(|v| v as f64 / 1e6);
        let usa_carga = f("charge_now").is_some();

        let watts = match (ritmo, voltios, usa_carga) {
            // current_now esta en uA: vatios = amperios x voltios.
            (Some(r), Some(v), true) if r > 0 => Some(r as f64 / 1e6 * v),
            // power_now ya viene en uW.
            (Some(r), _, false) if r > 0 => Some(r as f64 / 1e6),
            _ => None,
        };
        let time_left = match (ahora, ritmo) {
            (Some(a), Some(r)) if r > 0 => {
                Some(Duration::from_secs_f64(a as f64 / r as f64 * 3600.0))
            }
            _ => None,
        };

        return Some(Battery {
            percent: f("capacity").unwrap_or(0).min(100) as u8,
            status: txt("status").unwrap_or_else(|| "Unknown".into()),
            watts,
            health: match (lleno, diseño) {
                (Some(l), Some(d)) if d > 0 => {
                    Some(((l as f64 / d as f64) * 100.0).round().min(100.0) as u8)
                }
                _ => None,
            },
            time_left,
            // Muchos portatiles lo publican a cero, que no es un dato: es «no lo se».
            cycles: f("cycle_count").filter(|c| *c > 0).map(|c| c as u32),
            // Hay portatiles que publican aqui bytes en crudo en vez del modelo
            // («0x46,0x39,...» en este). Si no parece texto, no es un dato.
            model: txt("model_name").filter(|m| {
                !m.is_empty()
                    && m.len() <= 32
                    && !m.contains("0x")
                    && m.chars().any(|c| c.is_ascii_alphabetic())
            }),
        });
    }
    None
}

// ---------------------------------------------------------------- formato

/// Bytes en unidades que se leen de un vistazo.
pub fn human_bytes(b: f64) -> String {
    const U: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut v = b.abs();
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{v:.0} {}", U[i])
    } else if v < 10.0 {
        format!("{v:.1} {}", U[i])
    } else {
        format!("{v:.0} {}", U[i])
    }
}

/// Una duracion en «3 h 40 min» / «12 min».
pub fn human_duration(d: Duration) -> String {
    let m = d.as_secs() / 60;
    let (h, m) = (m / 60, m % 60);
    if h > 0 {
        format!("{h} h {m} min")
    } else {
        format!("{m} min")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_uso_de_cpu_sale_de_la_diferencia() {
        let a = parse_cpu_sample("cpu  100 0 100 800 0 0 0 0 0 0\ncpu0 1 2 3\n").unwrap();
        let b = parse_cpu_sample("cpu  200 0 200 1400 0 0 0 0 0 0\ncpu0 1 2 3\n").unwrap();
        // 200 ticks de trabajo y 600 de reposo sobre 800 nuevos -> 25 %.
        assert_eq!(cpu_percent(a, b), 25);
    }

    #[test]
    fn dos_muestras_iguales_son_cero_y_no_una_division_por_cero() {
        let a = parse_cpu_sample("cpu  1 1 1 1 1\n").unwrap();
        assert_eq!(cpu_percent(a, a), 0);
    }

    #[test]
    fn la_memoria_usada_descuenta_la_cache() {
        let m = parse_memory(
            "MemTotal:       16000 kB\nMemFree:          500 kB\nMemAvailable:    9000 kB\nSwapTotal:       2000 kB\nSwapFree:        1500 kB\n",
        )
        .unwrap();
        assert_eq!(m.used_kb, 7000, "total menos disponible, no menos libre");
        assert_eq!(m.swap_used_kb, 500);
        assert_eq!(m.percent(), 44);
    }

    #[test]
    fn la_red_no_cuenta_el_loopback() {
        let texto = "Inter-|   Receive\n face |bytes packets errs drop fifo frame compressed multicast|bytes packets errs drop fifo colls carrier compressed
    lo: 9999 1 0 0 0 0 0 0 9999 1 0 0 0 0 0 0
 wlan0: 1000 1 0 0 0 0 0 0 2000 1 0 0 0 0 0 0
";
        let s = parse_net(texto).unwrap();
        assert_eq!(s.rx, 1000);
        assert_eq!(s.tx, 2000);
    }

    #[test]
    fn formato_de_bytes() {
        assert_eq!(human_bytes(512.0), "512 B");
        assert_eq!(human_bytes(1536.0), "1.5 kB");
        assert_eq!(human_bytes(15.0 * 1024.0 * 1024.0), "15 MB");
    }

    #[test]
    fn formato_de_duracion() {
        assert_eq!(human_duration(Duration::from_secs(3600 * 3 + 60 * 40)), "3 h 40 min");
        assert_eq!(human_duration(Duration::from_secs(60 * 12)), "12 min");
    }
}

// ---------------------------------------------------------------- la ficha de la maquina
//
// Lo que pide el panel de Sistema. Todo sale de /sys, /proc y /etc: ni un subproceso,
// que es lo que separa esto del `sysinfo.sh` de la v1 (fastfetch dentro de una kitty).

/// Modelo de la maquina, segun la DMI de la placa. «Dell Inc. G15 5515».
pub fn host_model() -> Option<String> {
    let dmi = Path::new("/sys/devices/virtual/dmi/id");
    let leer = |f: &str| {
        fs::read_to_string(dmi.join(f))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };
    let producto = leer("product_name")?;
    // El fabricante suele venir ya dentro del producto, y comparar las cadenas
    // enteras no lo pilla: esta maquina dice sys_vendor «Dell Inc.» y product_name
    // «Dell G15 5515», que daria «Dell Inc. Dell G15 5515». Comparo la primera
    // palabra, que es la marca.
    let vendor = leer("sys_vendor")?;
    let marca = vendor.split_whitespace().next().unwrap_or(&vendor);
    if producto.to_lowercase().starts_with(&marca.to_lowercase()) {
        Some(producto)
    } else {
        Some(format!("{vendor} {producto}"))
    }
}

pub fn kernel() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// `PRETTY_NAME` de /etc/os-release.
pub fn os_pretty() -> Option<String> {
    let txt = fs::read_to_string("/etc/os-release").ok()?;
    for l in txt.lines() {
        if let Some(v) = l.strip_prefix("PRETTY_NAME=") {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// El shell de la sesion, por su nombre. `$SHELL` es lo unico que lo dice sin mirar
/// al padre del proceso, que aqui seria Hyprland y no diria nada util.
pub fn shell() -> Option<String> {
    std::env::var("SHELL")
        .ok()
        .and_then(|s| s.rsplit('/').next().map(|n| n.to_string()))
        .filter(|s| !s.is_empty())
}

/// Cuantos paquetes hay instalados. Se cuentan los directorios de la base de datos
/// local de pacman, que es exactamente lo que cuenta `pacman -Q` pero sin forkear.
pub fn package_count() -> Option<usize> {
    let d = fs::read_dir("/var/lib/pacman/local").ok()?;
    Some(d.flatten().filter(|e| e.path().is_dir()).count())
}

/// Busca «vendor:device» en un fichero con el formato de `pci.ids`: la linea del
/// fabricante empieza en la columna 0 y las de sus dispositivos llevan una tabulacion.
pub fn lookup_pci(ids: &str, vendor: &str, device: &str) -> Option<String> {
    let (vendor, device) = (vendor.to_ascii_lowercase(), device.to_ascii_lowercase());
    let mut dentro = false;
    for l in ids.lines() {
        if l.starts_with('#') || l.trim().is_empty() {
            continue;
        }
        if !l.starts_with('\t') {
            // Una linea de fabricante cierra el bloque anterior.
            dentro = l
                .split_once("  ")
                .map(|(id, _)| id.eq_ignore_ascii_case(&vendor))
                .unwrap_or(false);
            continue;
        }
        if !dentro {
            continue;
        }
        // Dos tabulaciones son un subsistema, no un dispositivo.
        if l.starts_with("\t\t") {
            continue;
        }
        let l = l.trim_start_matches('\t');
        if let Some((id, nombre)) = l.split_once("  ")
            && id.eq_ignore_ascii_case(&device)
        {
            return Some(nombre.trim().to_string());
        }
    }
    None
}

/// Las tarjetas graficas: su nombre de verdad si se puede resolver, y si no su driver.
///
/// Para la NVIDIA el propio driver publica el modelo en /proc, que es mas exacto que
/// la tabla de PCI. Para el resto se resuelve `PCI_ID` contra `pci.ids` de hwdata, que
/// esta instalado porque lo trae `pciutils`. Si falta, queda el driver, que es poco
/// pero es verdad.
pub fn gpus() -> Vec<String> {
    let ids = fs::read_to_string("/usr/share/hwdata/pci.ids").ok();
    let nvidia = nvidia_model();

    let Ok(cards) = fs::read_dir("/sys/class/drm") else {
        return Vec::new();
    };
    let mut nombres: Vec<String> = Vec::new();
    let mut rutas: Vec<_> = cards
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| {
                    n.starts_with("card") && n["card".len()..].chars().all(|c| c.is_ascii_digit())
                })
                .unwrap_or(false)
        })
        .collect();
    rutas.sort();

    for card in rutas {
        let Ok(uevent) = fs::read_to_string(card.join("device/uevent")) else {
            continue;
        };
        let campo = |k: &str| {
            uevent
                .lines()
                .find_map(|l| l.strip_prefix(k))
                .map(|v| v.trim().to_string())
        };
        let driver = campo("DRIVER=").unwrap_or_default();

        let nombre = if driver == "nvidia" {
            nvidia.clone()
        } else {
            campo("PCI_ID=").and_then(|pci| {
                let (v, d) = pci.split_once(':')?;
                lookup_pci(ids.as_deref()?, v, d)
            })
        };

        let nombre = match nombre {
            Some(n) => n,
            None if !driver.is_empty() => driver,
            None => continue,
        };
        if !nombres.contains(&nombre) {
            nombres.push(nombre);
        }
    }
    nombres
}

/// El modelo que publica el driver propietario de NVIDIA, si esta cargado.
fn nvidia_model() -> Option<String> {
    let d = fs::read_dir("/proc/driver/nvidia/gpus").ok()?;
    for e in d.flatten() {
        if let Ok(txt) = fs::read_to_string(e.path().join("information")) {
            for l in txt.lines() {
                if let Some(v) = l.strip_prefix("Model:") {
                    let v = v.trim();
                    if !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests_ficha {
    use super::*;

    // Un trozo de pci.ids de verdad, con las dos tarjetas de esta maquina y un
    // subsistema por medio para comprobar que no se cuela como dispositivo.
    const IDS: &str = "\
# comentario
1002  Advanced Micro Devices, Inc. [AMD/ATI]
\t1636  Renoir
\t1638  Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]
\t\t1043 1d12  Subsistema que no es un dispositivo
10de  NVIDIA Corporation
\t2560  GA106M [GeForce RTX 3060 Mobile]
";

    #[test]
    fn resuelve_las_dos_tarjetas() {
        assert_eq!(
            lookup_pci(IDS, "1002", "1638").as_deref(),
            Some("Cezanne [Radeon Vega Series / Radeon Vega Mobile Series]")
        );
        assert_eq!(
            lookup_pci(IDS, "10de", "2560").as_deref(),
            Some("GA106M [GeForce RTX 3060 Mobile]")
        );
    }

    #[test]
    fn el_id_no_distingue_mayusculas() {
        // El uevent del kernel los escribe en mayusculas y pci.ids en minusculas.
        assert_eq!(
            lookup_pci(IDS, "10DE", "2560").as_deref(),
            Some("GA106M [GeForce RTX 3060 Mobile]")
        );
    }

    #[test]
    fn un_subsistema_no_pasa_por_dispositivo() {
        // "1043 1d12" lleva dos tabulaciones: si se tomara por dispositivo, buscar
        // "1043" devolveria basura.
        assert_eq!(lookup_pci(IDS, "1002", "1043"), None);
    }

    #[test]
    fn un_dispositivo_de_otro_fabricante_no_se_cruza() {
        // 1636 es de AMD, asi que preguntando por NVIDIA no debe aparecer.
        assert_eq!(lookup_pci(IDS, "10de", "1636"), None);
    }

    #[test]
    fn id_desconocido_da_nada() {
        assert_eq!(lookup_pci(IDS, "beef", "cafe"), None);
        assert_eq!(lookup_pci("", "1002", "1638"), None);
    }
}
