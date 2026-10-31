// ~/hyprland-from-scratch/crates/hfs/src/main.rs
//
// Un solo binario con subcomandos, no once binarios.
//
//   hfs shell           el escritorio: todas las superficies, en un solo proceso
//   hfs vol up|down|mute|<0-100>
//   hfs bri up|down|<1-100>
//   hfs wheel           abre o cierra la rueda del centro y el buscador de aplicaciones
//   hfs all             enseña todas las superficies a la vez (los bordes no llevan marca)
//   hfs palette         la paleta activa, para comprobar que el tema se lee bien
//
// Los subcomandos de ajuste son los que se atan a las teclas multimedia: cambian el
// valor y le dan un toque al dock para que se asome. Asi el dock hace de OSD y no
// hay que escribir un OSD aparte.

/// Lo que devuelve un panel que quiere gobernar alguna tecla: `true` si la ha
/// consumido. Lo consulta el manejador de la rueda antes de hacer nada con ella.
pub type Teclas = std::rc::Rc<
    dyn Fn(hfs_ui::gtk::gdk::Key, hfs_ui::gtk::gdk::ModifierType) -> bool,
>;

mod atajos;
mod bandeja;
mod bluetooth;
mod cabecera;
mod captura;
mod ctrl;
mod energia;
mod espacios;
mod fondo;
mod hypr;
mod ipc;
mod notificaciones;
mod portapapeles;
mod proc;
mod sni;
mod red;
mod reloj;
mod rueda;
mod shell;
mod sonido;
mod sistema;
mod tema;

use hfs_core::{audio, backlight, palette, Palette};

const PASO: i16 = 5;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (c.as_str(), r),
        None => ("ayuda", &[][..]),
    };

    let r = match cmd {
        // `dock` era el nombre de cuando solo existia el dock de controles.
        "shell" | "dock" => return shell::run(),
        "vol" => volumen(rest),
        "bri" => brillo(rest),
        "all" | "todo" => {
            ipc::nudge(ipc::Aviso::Todo);
            Ok(())
        }
        "wheel" | "rueda" => {
            ipc::nudge(ipc::Aviso::Rueda);
            Ok(())
        }
        // Las teclas que la v1 tenia para sus TUIs, ahora apuntando al panel que
        // los sustituye.
        "atajos" | "binds" => {
            ipc::nudge(ipc::Aviso::Panel("atajos"));
            Ok(())
        }
        "tema" | "theme" => {
            ipc::nudge(ipc::Aviso::Panel("tema"));
            Ok(())
        }
        "fondo" | "wallpaper" => {
            ipc::nudge(ipc::Aviso::Panel("fondo"));
            Ok(())
        }
        "captura" | "shot" => {
            ipc::nudge(ipc::Aviso::Panel("captura"));
            Ok(())
        }
        "sonido" | "audio" => {
            ipc::nudge(ipc::Aviso::Panel("sonido"));
            Ok(())
        }
        "sistema" | "sysinfo" => {
            ipc::nudge(ipc::Aviso::Sistema);
            Ok(())
        }
        // Lo llama `wl-paste --watch`, una vez por cada copia.
        "clip-add" => clip_add(),
        "palette" => paleta(),
        "-h" | "--help" | "ayuda" | "help" => {
            ayuda();
            Ok(())
        }
        otro => Err(hfs_core::Error::new(format!(
            "no conozco «{otro}». Prueba con: hfs --help"
        ))),
    };

    match r {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("hfs: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn ayuda() {
    println!(
        "hfs {} — el escritorio

  hfs shell                 levanta el escritorio entero (un proceso, 8 superficies
                            por pantalla). Es el `exec-once` de hyprland.conf.

  hfs vol up|down|mute|N    volumen; N es 0-100
  hfs bri up|down|N         brillo; N es 1-100

  hfs wheel                 abre o cierra la rueda y el buscador        SUPER
  hfs all                   enseña todas las superficies a la vez       SUPER+SHIFT+A

  hfs tema                  abre la rueda en el panel de temas          SUPER+A
  hfs fondo                 ... en el de fondos                         SUPER+W
  hfs captura               ... en el de capturas y grabacion           SUPER+R
  hfs sonido                ... en el de entradas y salidas de audio    SUPER+V
  hfs atajos                ... en la ayuda de atajos                   SUPER+/
  hfs sistema               la ficha de la maquina (tambien el icono de arch)

  hfs clip-add              guarda lo que venga por la entrada en el historial
  hfs palette               enseña la paleta activa

Las teclas multimedia van a `hfs vol` y `hfs bri`: ademas de cambiar el valor,
hacen que el dock se asome un segundo, asi que no hace falta un OSD aparte.

Dentro de la rueda: ESC cierra desde donde sea, RETROCESO vuelve a la rueda,
y escribir cualquier letra entra en el buscador de aplicaciones.",
        env!("CARGO_PKG_VERSION")
    );
}

fn volumen(args: &[String]) -> hfs_core::Result<()> {
    let accion = args.first().map(|s| s.as_str()).unwrap_or("");
    let v = match accion {
        "up" | "+" => nudged(audio::step(PASO)?),
        "down" | "-" => nudged(audio::step(-PASO)?),
        "mute" | "toggle" => nudged(audio::toggle_mute()?),
        // Sin argumento es una consulta: no debe sacar el OSD.
        "" => audio::get()?,
        n => {
            let p: u8 = n
                .parse()
                .map_err(|_| hfs_core::Error::new(format!("«{n}» no es ni up, ni down, ni mute, ni un numero")))?;
            audio::set(p)?;
            nudged(audio::get()?)
        }
    };
    println!("{}{}", v.percent, if v.muted { " (mudo)" } else { "" });
    Ok(())
}

fn brillo(args: &[String]) -> hfs_core::Result<()> {
    let bl = backlight::Backlight::detect()?;
    let accion = args.first().map(|s| s.as_str()).unwrap_or("");
    let p = match accion {
        "up" | "+" => nudged(bl.step(PASO)?),
        "down" | "-" => nudged(bl.step(-PASO)?),
        "" => bl.percent()?,
        n => {
            let p: u8 = n
                .parse()
                .map_err(|_| hfs_core::Error::new(format!("«{n}» no es ni up, ni down, ni un numero")))?;
            bl.set(p)?;
            nudged(bl.percent()?)
        }
    };
    println!("{p}");
    Ok(())
}

/// Avisa al dock de que algo ha cambiado y devuelve el valor tal cual, para poder
/// encadenarlo donde se calcula.
fn nudged<T>(v: T) -> T {
    ipc::nudge(ipc::Aviso::Ctrl);
    v
}

/// Lee la seleccion de la entrada estandar y la guarda. Es lo que `wl-paste --watch`
/// invoca en cada copia, asi que tiene que ser barato y callado.
fn clip_add() -> hfs_core::Result<()> {
    use std::io::Read;
    let mut buf = Vec::new();
    std::io::stdin()
        .read_to_end(&mut buf)
        .map_err(|e| hfs_core::Error::new(format!("no pude leer la seleccion: {e}")))?;
    hfs_core::clipboard::add(&buf)
}

fn paleta() -> hfs_core::Result<()> {
    let nombre = palette::current_theme_name();
    let (p, err) = Palette::load();
    if let Some(e) = err {
        eprintln!("hfs: uso la paleta de reserva porque {e}");
    }
    println!("tema activo : {nombre}");
    println!("paleta      : {} ({})", p.name, p.mode.as_str());
    println!("disponibles : {}", palette::available().join(", "));
    println!();
    for (k, c) in [
        ("bg", p.bg),
        ("surface", p.surface),
        ("overlay", p.overlay),
        ("fg", p.fg),
        ("muted", p.muted),
        ("dim", p.dim),
        ("accent", p.accent),
        ("on_accent", p.on_accent),
        ("ok", p.ok),
        ("warn", p.warn),
        ("bad", p.bad),
        ("info", p.info),
    ] {
        // El cuadrito de color usa el fondo del terminal en 24 bits.
        println!("\x1b[48;2;{};{};{}m    \x1b[0m  {:<10} {}", c.r, c.g, c.b, k, c);
    }
    Ok(())
}
