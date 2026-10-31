// ~/hyprland-from-scratch/crates/hfs-core/src/paths.rs
//
// Donde vive cada cosa. El repo esta clavado en ~/hyprland-from-scratch (es la regla
// del proyecto: install.sh se niega a correr en otro sitio), pero se puede mover con
// HFS_REPO para poder probar sin tocar el escritorio de verdad.

use std::path::PathBuf;

pub fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

pub fn repo() -> PathBuf {
    std::env::var_os("HFS_REPO")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("hyprland-from-scratch"))
}

pub fn palettes() -> PathBuf {
    repo().join("palettes")
}

/// Estado que sobrevive al reinicio: tema activo, modo claro/oscuro.
pub fn state() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(".local/state"))
        .join("hyprland-from-scratch")
}

/// Estado de esta sesion: pid de cada superficie viva.
pub fn runtime() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("hfs")
}

/// El fichero que escribe theme-manager.sh y que lee todo lo demas.
pub fn current_theme_file() -> PathBuf {
    home().join(".config/hypr/current_theme")
}
