// ~/hyprland-from-scratch/crates/hfs-core/src/hyprland.rs
//
// Hablar con Hyprland por su socket, sin forkear `hyprctl`.
//
// Son dos sockets en $XDG_RUNTIME_DIR/hypr/$HYPRLAND_INSTANCE_SIGNATURE/:
//
//   .socket.sock    ordenes y consultas. Conectar, escribir, leer hasta EOF.
//   .socket2.sock   un flujo de eventos, una linea por evento: `nombre>>datos`.
//                   Se queda abierto para siempre y no hay que preguntar nada.
//
// El reparto es a proposito: el evento solo dice QUE ha cambiado, y entonces se
// vuelve a pedir el estado entero. Mantener un estado incremental a base de deltas
// es donde viven los fallos (un `movewindow` que no se ve, un contador que se queda
// desfasado), y la consulta cuesta una ida y vuelta por un socket local.
//
// Se parsea la salida de texto, no el JSON: este crate no tiene dependencias, y un
// parser de JSON a mano para ahorrarse un `split_whitespace` no se paga.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use crate::{Error, Result};

fn socket_dir() -> Result<PathBuf> {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE").map_err(|_| {
        Error::new("HYPRLAND_INSTANCE_SIGNATURE no esta puesta: ¿esto corre dentro de Hyprland?")
    })?;
    let rt = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    Ok(rt.join("hypr").join(sig))
}

/// El socket de ordenes y consultas.
pub fn command_socket() -> Result<PathBuf> {
    Ok(socket_dir()?.join(".socket.sock"))
}

/// El socket de eventos. Quien lo use se conecta y lee lineas.
pub fn event_socket() -> Result<PathBuf> {
    Ok(socket_dir()?.join(".socket2.sock"))
}

/// Una consulta u orden. Varias separadas por `;;` detras de `[[BATCH]]` viajan en
/// una sola ida y vuelta.
pub fn request(cmd: &str) -> Result<String> {
    let path = command_socket()?;
    let mut s = UnixStream::connect(&path).map_err(|e| {
        Error::new(format!(
            "no pude conectar con Hyprland en {}: {e}",
            path.display()
        ))
    })?;
    s.write_all(cmd.as_bytes())
        .map_err(|e| Error::new(format!("no pude mandarle «{cmd}» a Hyprland: {e}")))?;
    let mut out = String::new();
    s.read_to_string(&mut out)
        .map_err(|e| Error::new(format!("Hyprland no contesto a «{cmd}»: {e}")))?;
    Ok(out)
}

/// Un `dispatch`. Hyprland contesta «ok» cuando lo acepta; cualquier otra cosa es un
/// fallo y hay que verlo.
pub fn dispatch(args: &str) -> Result<()> {
    let r = request(&format!("dispatch {args}"))?;
    if r.trim() == "ok" {
        Ok(())
    } else {
        Err(Error::new(format!(
            "Hyprland rechazo «dispatch {args}»: {}",
            r.trim()
        )))
    }
}

// ---------------------------------------------------------------- el estado

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    pub windows: u16,
    pub monitor: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Monitor {
    pub name: String,
    pub active_ws: i32,
    pub focused: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct State {
    pub workspaces: Vec<Workspace>,
    pub monitors: Vec<Monitor>,
}

impl State {
    /// Los dos listados en una sola ida y vuelta.
    pub fn query() -> Result<Self> {
        Ok(Self::parse(&request("[[BATCH]]workspaces;;monitors")?))
    }

    /// Los espacios de un monitor, ordenados por id.
    pub fn on_monitor(&self, monitor: &str) -> Vec<&Workspace> {
        let mut v: Vec<&Workspace> = self
            .workspaces
            .iter()
            .filter(|w| w.monitor == monitor)
            // Los espacios especiales (scratchpad) tienen id negativo y no van en el rail.
            .filter(|w| w.id > 0)
            .collect();
        v.sort_by_key(|w| w.id);
        v
    }

    pub fn active_on(&self, monitor: &str) -> Option<i32> {
        self.monitors
            .iter()
            .find(|m| m.name == monitor)
            .map(|m| m.active_ws)
    }

    pub fn focused_monitor(&self) -> Option<&str> {
        self.monitors
            .iter()
            .find(|m| m.focused)
            .map(|m| m.name.as_str())
    }

    /// Un solo recorrido sobre la salida de los dos listados concatenados: una linea
    /// sin sangrar abre un registro y las sangradas son sus campos.
    fn parse(text: &str) -> Self {
        let mut me = Self::default();
        enum Cur {
            None,
            Ws,
            Mon,
        }
        let mut cur = Cur::None;

        for line in text.lines() {
            let campo = line.starts_with('\t') || line.starts_with(' ');
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if !campo {
                if let Some(ws) = parse_ws_header(line) {
                    me.workspaces.push(ws);
                    cur = Cur::Ws;
                } else if let Some(mon) = parse_mon_header(line) {
                    me.monitors.push(mon);
                    cur = Cur::Mon;
                } else {
                    cur = Cur::None;
                }
                continue;
            }

            let Some((clave, valor)) = line.split_once(':') else {
                continue;
            };
            let valor = valor.trim();
            match cur {
                Cur::Ws if clave == "windows" => {
                    if let (Some(w), Ok(n)) = (me.workspaces.last_mut(), valor.parse()) {
                        w.windows = n;
                    }
                }
                Cur::Mon if clave == "active workspace" => {
                    // «1 (nombre)»: el id es lo primero.
                    if let (Some(m), Some(Ok(n))) = (
                        me.monitors.last_mut(),
                        valor.split_whitespace().next().map(str::parse),
                    ) {
                        m.active_ws = n;
                    }
                }
                Cur::Mon if clave == "focused" => {
                    if let Some(m) = me.monitors.last_mut() {
                        m.focused = valor == "yes";
                    }
                }
                _ => {}
            }
        }
        me
    }
}

/// `workspace ID 2 (nombre) on monitor HDMI-A-1:`
fn parse_ws_header(line: &str) -> Option<Workspace> {
    let resto = line.strip_prefix("workspace ID ")?;
    let (id, resto) = resto.split_once(' ')?;
    let id: i32 = id.parse().ok()?;
    // El nombre puede llevar espacios, y el monitor va al final: se corta por detras.
    let (nombre, monitor) = resto.trim_end_matches(':').rsplit_once(" on monitor ")?;
    Some(Workspace {
        id,
        name: nombre
            .trim()
            .trim_start_matches('(')
            .trim_end_matches(')')
            .to_string(),
        windows: 0,
        monitor: monitor.trim().to_string(),
    })
}

/// `Monitor eDP-1 (ID 0):`
fn parse_mon_header(line: &str) -> Option<Monitor> {
    let resto = line.strip_prefix("Monitor ")?;
    let (nombre, _) = resto.split_once(" (ID ")?;
    Some(Monitor {
        name: nombre.trim().to_string(),
        active_ws: 0,
        focused: false,
    })
}

// ---------------------------------------------------------------- los eventos

/// Para que sirve un evento de socket2. No interesa el detalle: interesa si hay que
/// volver a pedir el estado y si hay que repintar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Algo que cambia los espacios o el foco: hay que releer el estado.
    Workspaces,
    /// Cambio de monitores: hay que reconstruir las superficies.
    Monitors,
    /// Cualquier otra cosa.
    Other,
}

/// Clasifica una linea `nombre>>datos` de socket2.
///
/// `activewindow` NO esta en la lista a proposito, aunque lo parezca: salta cada vez
/// que cambia el TITULO de la ventana enfocada, que en un editor es varias veces por
/// minuto, y no cambia ni los espacios ni cuantas ventanas tiene cada uno. Incluirlo
/// costaba una consulta de estado y un repintado de todos los railes por cada
/// pulsacion de tecla en otra aplicacion.
pub fn classify(line: &str) -> Event {
    let nombre = line.split_once(">>").map(|(n, _)| n).unwrap_or(line);
    match nombre {
        "workspace"
        | "workspacev2"
        | "focusedmon"
        | "focusedmonv2"
        | "createworkspace"
        | "createworkspacev2"
        | "destroyworkspace"
        | "destroyworkspacev2"
        | "moveworkspace"
        | "moveworkspacev2"
        | "renameworkspace"
        | "openwindow"
        | "closewindow"
        | "movewindow"
        | "movewindowv2" => Event::Workspaces,
        "monitoradded" | "monitoraddedv2" | "monitorremoved" | "monitorremovedv2" => {
            Event::Monitors
        }
        _ => Event::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SALIDA: &str = "\
workspace ID 1 (1) on monitor eDP-1:
\tmonitorID: 0
\twindows: 0
\thasfullscreen: 0
\tlastwindowtitle: 

workspace ID 2 (2) on monitor HDMI-A-1:
\tmonitorID: 1
\twindows: 3
\thasfullscreen: 0

workspace ID -99 (special:magic) on monitor eDP-1:
\tmonitorID: 0
\twindows: 1

Monitor eDP-1 (ID 0):
\t1920x1080@120.01500 at 0x0
\tactive workspace: 1 (1)
\tspecial workspace: 0 ()
\tfocused: no
Monitor HDMI-A-1 (ID 1):
\t1920x1080@60.00000 at 1920x0
\tactive workspace: 2 (2)
\tfocused: yes
";

    #[test]
    fn lee_los_dos_listados_de_una_pasada() {
        let s = State::parse(SALIDA);
        assert_eq!(s.workspaces.len(), 3);
        assert_eq!(s.monitors.len(), 2);
        assert_eq!(s.workspaces[1].windows, 3);
        assert_eq!(s.workspaces[1].monitor, "HDMI-A-1");
    }

    #[test]
    fn el_rail_ignora_los_especiales_y_el_otro_monitor() {
        let s = State::parse(SALIDA);
        let edp = s.on_monitor("eDP-1");
        assert_eq!(edp.len(), 1, "el -99 especial no va en el rail");
        assert_eq!(edp[0].id, 1);
    }

    #[test]
    fn el_activo_es_por_monitor() {
        let s = State::parse(SALIDA);
        assert_eq!(s.active_on("eDP-1"), Some(1));
        assert_eq!(s.active_on("HDMI-A-1"), Some(2));
        assert_eq!(s.active_on("DP-9"), None);
        assert_eq!(s.focused_monitor(), Some("HDMI-A-1"));
    }

    #[test]
    fn el_nombre_con_espacios_no_rompe_el_monitor() {
        let s = State::parse("workspace ID 7 (mi espacio) on monitor DP-1:\n\twindows: 2\n");
        assert_eq!(s.workspaces[0].name, "mi espacio");
        assert_eq!(s.workspaces[0].monitor, "DP-1");
        assert_eq!(s.workspaces[0].windows, 2);
    }

    #[test]
    fn clasifica_los_eventos() {
        assert_eq!(classify("workspace>>3"), Event::Workspaces);
        assert_eq!(classify("openwindow>>a,b,c,d"), Event::Workspaces);
        assert_eq!(classify("monitorremoved>>HDMI-A-1"), Event::Monitors);
        assert_eq!(classify("submap>>resize"), Event::Other);
    }

    #[test]
    fn cambiar_de_titulo_no_dispara_nada() {
        // Salta a cada tecla que escribes en un editor y no cambia el rail.
        assert_eq!(classify("activewindow>>kitty,nvim"), Event::Other);
        assert_eq!(classify("activewindowv2>>5587999aef80"), Event::Other);
    }
}
