// ~/hyprland-from-scratch/crates/hfs-ui/src/shape.rs
//
// La forma del panel, de una sola pieza.
//
// Un panel no flota cerca del borde: nace de el. Eso son tres cosas y las tres estan
// en este fichero:
//   1. pegado al canto, sin hueco
//   2. radio solo por dentro; el lado que toca la pantalla va recto
//   3. muescas concavas en las esquinas del canto, que funden el panel con el borde
//
// La muesca es un cuadrado de lado N pegado al canto, al que se le quita un cuarto de
// disco centrado en la esquina EXTERIOR. Asi la curva sale del canto y entra en el
// costado del panel.
//
// Un panel de ESQUINA nace de la esquina, no cerca de ella: se pega a los dos cantos
// a la vez, y entonces uno de sus dos extremos ya no tiene pantalla al lado. Ese
// extremo va a escuadra y sin muesca; el otro lleva muesca y radio como siempre. De
// ahi los dos booleanos.
//
// Se dibuja una sola orientacion (la del canto derecho) y las otras tres salen de una
// transformacion. Si cambias la geometria, cambiala una vez aqui.

use gtk4::cairo::Context;
use std::f64::consts::{FRAC_PI_2, PI};

use crate::edge::Side;

/// Que extremos del panel llevan muesca, en el orden de la pantalla: el primero es el
/// de arriba en un panel vertical y el de la izquierda en uno horizontal.
///
/// `(true, true)` es un panel de canto; una esquina pone a `false` el extremo que se
/// pega a la esquina de la pantalla.
pub type Notches = (bool, bool);

pub const BOTH: Notches = (true, true);

/// Contorno del panel dentro de una caja de `w` x `h`, pegado a `side`.
/// `r` es el radio interior y `n` el lado de la muesca.
///
/// La caja incluye las muescas: en un panel vertical con las dos, el cuerpo va de
/// `y = n` a `y = h - n`.
pub fn path(ctx: &Context, side: Side, w: f64, h: f64, r: f64, n: f64, notches: Notches) {
    let (a, b) = notches;
    ctx.save().ok();
    match side {
        Side::Right => canonical(ctx, w, h, r, n, a, b),
        Side::Left => {
            // Espejo horizontal: la vertical no cambia, asi que los extremos tampoco.
            ctx.translate(w, 0.0);
            ctx.scale(-1.0, 1.0);
            canonical(ctx, w, h, r, n, a, b);
        }
        Side::Top => {
            // El canto derecho pasa a ser el de arriba. Con este giro el extremo
            // «inicio» del canonico cae a la izquierda de la pantalla.
            ctx.translate(0.0, h);
            ctx.rotate(-FRAC_PI_2);
            canonical(ctx, h, w, r, n, a, b);
        }
        Side::Bottom => {
            // Aqui el giro es al otro lado, asi que el «inicio» del canonico cae a la
            // DERECHA de la pantalla y los dos extremos van cambiados.
            ctx.translate(w, 0.0);
            ctx.rotate(FRAC_PI_2);
            canonical(ctx, h, w, r, n, b, a);
        }
    }
    ctx.restore().ok();
}

/// El panel pegado al canto DERECHO. Todo lo demas es esto girado.
///
/// `ns` / `ne` son los extremos de arriba y de abajo: con muesca y radio interior, o
/// a escuadra porque ahi se acaba la pantalla.
fn canonical(ctx: &Context, w: f64, h: f64, r: f64, n: f64, ns: bool, ne: bool) {
    let top = if ns { n } else { 0.0 };
    let bot = if ne { h - n } else { h };
    let r = r.min((bot - top) / 2.0).min(w);

    ctx.new_path();

    if ns {
        // Muesca de arriba: del canto hacia dentro, por el arco concavo, y la esquina
        // interior redondeada.
        ctx.move_to(w, top - n);
        ctx.arc(w - n, top - n, n, 0.0, FRAC_PI_2);
        ctx.line_to(r, top);
        ctx.arc_negative(r, top + r, r, -FRAC_PI_2, PI);
    } else {
        // Esquina de la pantalla: recto y a escuadra.
        ctx.move_to(w, top);
        ctx.line_to(0.0, top);
    }

    if ne {
        ctx.line_to(0.0, bot - r);
        ctx.arc_negative(r, bot - r, r, PI, FRAC_PI_2);
        ctx.line_to(w - n, bot);
        ctx.arc(w - n, bot + n, n, -FRAC_PI_2, 0.0);
    } else {
        ctx.line_to(0.0, bot);
        ctx.line_to(w, bot);
    }

    // De vuelta por el canto.
    ctx.close_path();
}
