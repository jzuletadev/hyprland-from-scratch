// ~/hyprland-from-scratch/crates/hfs-core/src/lib.rs
//
// El nucleo del escritorio: estado, paleta y los backends de datos.
// No sabe nada de GTK ni de dibujar, para poder probarlo sin levantar una pantalla.

pub mod audio;
pub mod backlight;
pub mod binds;
pub mod clipboard;
pub mod cmd;
pub mod error;
pub mod fuzzy;
pub mod hyprland;
pub mod palette;
pub mod paths;
pub mod sys;

pub use error::{Error, Result};
pub use palette::{Mode, Palette, Rgb};
