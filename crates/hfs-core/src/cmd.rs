// ~/hyprland-from-scratch/crates/hfs-core/src/cmd.rs
//
// Ejecutar un programa y quedarse con su salida. Si falla, el error que sale de aqui
// lleva el stderr real del programa, no un "algo ha pasado".

use crate::{Error, Result};
use std::process::Command;

pub fn run(bin: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| Error::new(format!("no se pudo ejecutar «{bin}»: {e}")))?;

    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let detalle = if err.is_empty() {
            match out.status.code() {
                Some(c) => format!("salio con codigo {c}"),
                None => "lo mato una señal".into(),
            }
        } else {
            err
        };
        return Err(Error::new(format!(
            "«{bin} {}» {detalle}",
            args.join(" ")
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
