// ~/hyprland-from-scratch/crates/hfs-core/examples/hypr.rs
//
// Comprobacion a mano contra el Hyprland que esta corriendo:
//   cargo run -p hfs-core --example hypr                 enseña el estado
//   cargo run -p hfs-core --example hypr -- "workspace 3" manda una orden y lo enseña

fn main() -> hfs_core::Result<()> {
    if let Some(orden) = std::env::args().nth(1) {
        hfs_core::hyprland::dispatch(&orden)?;
        println!("ok: dispatch {orden}");
    }
    let s = hfs_core::hyprland::State::query()?;
    for m in &s.monitors {
        let act = s.active_on(&m.name);
        let ws: Vec<String> = s
            .on_monitor(&m.name)
            .iter()
            .map(|w| {
                format!(
                    "{}{}({})",
                    w.id,
                    if Some(w.id) == act { "*" } else { "" },
                    w.windows
                )
            })
            .collect();
        println!(
            "{:<10} {:<8} {}",
            m.name,
            if m.focused { "[foco]" } else { "" },
            ws.join(" ")
        );
    }
    Ok(())
}
