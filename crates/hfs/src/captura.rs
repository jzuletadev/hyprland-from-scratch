// ~/hyprland-from-scratch/crates/hfs/src/captura.rs
//
// El MOTOR de las capturas y la grabacion. La interfaz vive en `accesos.rs`, en la
// esquina de abajo a la derecha; aqui solo queda lo que arranca y para.
//
//   NO HAY MONITOR INVENTADO. La v1 preguntaba a `hyprctl` cual era el monitor con
//   foco y, si fallaba, devolvia «eDP-1» a ciegas: en esta maquina hay dos pantallas,
//   asi que grababa la equivocada sin decir nada.
//
// Todo lo que dispara va por `sh -c` y detras de cerrar el panel: `slurp` se queda
// esperando a que arrastres el raton, y bloquear el bucle de GTK ahi congelaria el
// escritorio entero. Por eso tampoco se espera el resultado.

/// Donde caen los videos, igual que en la v1.
const DIR: &str = "Videos/recordings";



thread_local! {
    /// Quien sigue la pista a la grabacion. Es uno solo para todo el proceso: lo
    /// consultan este panel y la bandeja de cada pantalla, y comparten el PID que
    /// lleva dentro para no barrer /proc tres veces por segundo.
    static VIGIA: crate::proc::Vigia = const { crate::proc::Vigia::new("wf-recorder") };
}

/// Si `wf-recorder` esta grabando ahora mismo.
///
/// Dos cosas que no son obvias. Una: **un zombi conserva su nombre**, asi que con un
/// `pgrep -x` una grabacion ya parada seguia contando como viva y el panel se quedaba
/// diciendo «grabando» para siempre. Dos: esto se consulta **una vez por segundo
/// mientras el escritorio este vivo**, porque la marca del canto tiene que aparecer
/// sin que nadie roce nada, asi que no puede permitirse barrer /proc. Ver `proc.rs`.
pub fn grabando() -> bool {
    VIGIA.with(|v| v.vivo())
}

/// El monitor con foco, preguntado de verdad. `None` si Hyprland no contesta, y
/// entonces no se graba: es mejor no grabar que grabar la pantalla equivocada.
fn monitor_con_foco() -> Option<String> {
    let st = hfs_core::hyprland::State::query().ok()?;
    st.focused_monitor().map(|s| s.to_string())
}

fn ejecutar(cmd: String) {
    crate::proc::sh(&cmd);
}

pub fn parar() {
    // -INT y no -TERM: wf-recorder cierra el contenedor al recibir INT, y con TERM
    // deja el mp4 sin el indice al final, que es un fichero que no se puede buscar.
    ejecutar("pkill -INT -x wf-recorder".into());
}

/// Arranca la grabacion. `region` elige entre arrastrar un recuadro y la pantalla con
/// foco; `audio` anade el monitor del sink por omision.
///
/// Se monta como una orden de shell porque `slurp` se queda esperando a que arrastres
/// el raton: hacerlo desde Rust obligaria a un hilo y a comunicar el resultado de
/// vuelta al bucle de GTK, y aqui el shell ya sabe encadenar las dos cosas.
pub fn arrancar(region: bool, audio: bool) {
    let dir = hfs_core::paths::home().join(DIR);

    // La geometria primero, y si cancelas con Esc `slurp` no escribe nada: ese `exit 0`
    // es lo que distingue «he cambiado de idea» de un fallo.
    let (geo, fuente) = if region {
        ("G=\"$(slurp)\"; [ -n \"$G\" ] || exit 0; ".to_string(), "-g \"$G\"".to_string())
    } else {
        match monitor_con_foco() {
            Some(m) => (String::new(), format!("-o {m}")),
            None => {
                eprintln!("hfs: Hyprland no dice que monitor tiene el foco; no grabo");
                return;
            }
        }
    };

    let sonido = if audio {
        " --audio=\"$(pactl get-default-sink).monitor\""
    } else {
        ""
    };

    // El `exec` de dentro hace que este PID acabe siendo el de `wf-recorder`, asi
    // que apuntarlo evita el primer barrido y la marca del canto sale al momento.
    let pid = crate::proc::sh(&format!(
        "mkdir -p \"{dir}\"; {geo}exec wf-recorder \
         -f \"{dir}/$(date +%Y-%m-%d_%H-%M-%S).mp4\" \
         -p preset=veryfast -p crf=23 {fuente}{sonido}",
        dir = dir.display(),
    ));
    if let Some(pid) = pid {
        VIGIA.with(|v| v.cebar(pid));
    }
}
