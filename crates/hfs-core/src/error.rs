// ~/hyprland-from-scratch/crates/hfs-core/src/error.rs
//
// Un error con mensaje, y nada mas. La regla de la v2 es que NADA se traga un fallo
// en silencio: la v1 devolvia 50 % cuando no podia leer el volumen y nadie se enteraba.
// Aqui el fallo llega entero hasta quien lo pueda enseñar.

use std::fmt;

#[derive(Debug, Clone)]
pub struct Error(String);

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
