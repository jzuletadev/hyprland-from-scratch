// ~/hyprland-from-scratch/crates/hfs-ui/src/lib.rs
//
// La capa de dibujo. Aqui vive todo lo que hace que un panel «nazca del borde»:
// la superficie de layer-shell, la forma con las muescas concavas y el CSS que sale
// de la paleta.
//
// Que esto sea una biblioteca y no parte del binario es el punto: la segunda
// superficie (espacios, reloj, notificaciones...) no vuelve a escribir nada de esto.

pub mod center;
pub mod css;
pub mod dbus;
pub mod edge;
pub mod medidas;
pub mod shape;
pub mod widgets;

pub use center::{CenterHandle, CenterSurface};
pub use edge::{EdgeSurface, Handle, Place, Side, Spec};

pub use gtk4 as gtk;
pub use gtk4_layer_shell as layer_shell;
