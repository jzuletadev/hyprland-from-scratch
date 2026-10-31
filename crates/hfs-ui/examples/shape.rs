// ~/hyprland-from-scratch/crates/hfs-ui/examples/shape.rs
//
// Saca la forma del panel a un PNG, a lo grande y sobre un fondo claro.
//
// Existe porque las muescas concavas son casi invisibles sobre un escritorio oscuro:
// mirar una captura no sirve para saber si la geometria esta bien. Aqui se ve.
//
//   cargo run -p hfs-ui --example shape -- /tmp/forma.png
//
// Dibuja los cuatro cantos y las cuatro esquinas de una vez, asi que tambien vale de
// comprobacion cuando se añada una superficie nueva en un sitio que no se habia usado.

use hfs_ui::{shape, Side};

const R: f64 = 18.0;
const N: f64 = 16.0;

fn main() {
    let salida = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "forma.png".to_string());

    // Cada panel ocupa su cuarto del lienzo, pegado a su canto.
    let (lw, lh) = (900, 620);
    let surf = gtk4::cairo::ImageSurface::create(gtk4::cairo::Format::ARgb32, lw, lh).unwrap();
    let ctx = gtk4::cairo::Context::new(&surf).unwrap();

    // Fondo a cuadros: con un color plano no se distingue «transparente» de «relleno».
    for y in 0..(lh / 20) {
        for x in 0..(lw / 20) {
            let c = if (x + y) % 2 == 0 { 0.85 } else { 0.72 };
            ctx.set_source_rgb(c, c * 0.95, c * 0.86);
            ctx.rectangle((x * 20) as f64, (y * 20) as f64, 20.0, 20.0);
            ctx.fill().ok();
        }
    }

    let (w, h) = (lw as f64, lh as f64);
    // Los cuatro paneles de canto, centrados, con sus dos muescas.
    let cantos = [
        (Side::Right, w - 120.0, 220.0, 120.0, 190.0),
        (Side::Left, 0.0, 220.0, 120.0, 190.0),
        (Side::Top, 340.0, 0.0, 220.0, 90.0),
        (Side::Bottom, 340.0, h - 90.0, 220.0, 90.0),
    ];
    // Las cuatro esquinas: una sola muesca, la del lado que da a la pantalla.
    // `Start` se pega al extremo de arriba o de la izquierda; `End`, al otro.
    let esquinas = [
        (Side::Top, 0.0, 0.0, 240.0, 90.0, (false, true)),
        (Side::Top, w - 240.0, 0.0, 240.0, 90.0, (true, false)),
        (Side::Bottom, 0.0, h - 90.0, 240.0, 90.0, (false, true)),
        (Side::Bottom, w - 240.0, h - 90.0, 240.0, 90.0, (true, false)),
    ];

    let paneles = cantos
        .iter()
        .map(|&(s, x, y, w, h)| (s, x, y, w, h, shape::BOTH))
        .chain(esquinas.iter().copied());

    for (side, x, y, w, h, notches) in paneles {
        ctx.save().unwrap();
        ctx.translate(x, y);
        shape::path(&ctx, side, w, h, R, N, notches);
        ctx.set_source_rgba(0.08, 0.09, 0.11, 0.94);
        ctx.fill_preserve().unwrap();
        ctx.set_source_rgba(1.0, 0.85, 0.4, 0.9);
        ctx.set_line_width(1.5);
        ctx.stroke().unwrap();
        ctx.restore().unwrap();
    }

    drop(ctx);
    let mut f = std::fs::File::create(&salida).unwrap();
    surf.write_to_png(&mut f).unwrap();
    println!("escrito {salida}");
}
