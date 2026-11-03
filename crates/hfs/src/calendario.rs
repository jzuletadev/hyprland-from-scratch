// ~/hyprland-from-scratch/crates/hfs/src/calendario.rs
//
// El calendario, como pieza suelta y no como parte de una superficie.
//
// Deja de vivir en la esquina de abajo a la derecha y pasa al panel de cabecera, al
// lado del reloj y de lo que suena. Sacarlo a su propio modulo es lo que permite el
// cambio sin copiar el dibujo del mes a otro sitio.
//
//   SE PUEDE CAMBIAR DE MES. Antes solo pintaba el actual, que sirve para mirar que
//   dia es y para nada mas. Mirar si el 14 del mes que viene cae en sabado obligaba a
//   abrir otra cosa.
//
// La fecha sale de glib y no de un crate de fechas: ya trae la zona horaria y el
// idioma de la sesion, asi que `%a` y `%b` salen traducidos sin anadir dependencias.
// La semana empieza en lunes.

use std::cell::Cell;
use std::rc::Rc;

use gtk4 as gtk;
use gtk4::glib;
use gtk4::prelude::*;

pub struct Calendario {
    pub root: gtk::Box,
    /// Año y mes que se estan enseñando. No tiene por que ser el de hoy.
    vista: Rc<Cell<(i32, i32)>>,
    grid: gtk::Grid,
    titulo: gtk::Label,
}

impl Calendario {
    pub fn new() -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 4);
        root.add_css_class("cal-caja");

        let cab = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let atras = flecha("pan-start-symbolic");
        let adelante = flecha("pan-end-symbolic");
        let titulo = gtk::Label::new(None);
        titulo.add_css_class("cal-titulo");
        titulo.set_hexpand(true);
        cab.append(&atras);
        cab.append(&titulo);
        cab.append(&adelante);

        let grid = gtk::Grid::new();
        grid.add_css_class("cal");
        grid.set_hexpand(true);
        // SIN `vexpand`. Lo llevaba de cuando vivia en un panel de esquina de alto
        // fijo y tenia que llenarlo; en el dashboard estirar sus filas es lo que
        // hacia el panel mas alto de lo necesario.
        grid.set_row_homogeneous(true);
        grid.set_column_homogeneous(true);

        root.append(&cab);
        root.append(&grid);

        let hoy = hoy_ym();
        let yo = Rc::new(Self {
            root,
            vista: Rc::new(Cell::new(hoy)),
            grid,
            titulo,
        });

        for (b, paso) in [(&atras, -1), (&adelante, 1)] {
            let yo = yo.clone();
            b.connect_clicked(move |_| yo.mover(paso));
        }
        // Tocar el titulo vuelve a hoy. Es la salida de haberse ido seis meses
        // adelante mirando una fecha: sin ella habria que contar hacia atras.
        {
            let clic = gtk::GestureClick::new();
            let suyo = yo.clone();
            clic.connect_released(move |_, _, _, _| suyo.hoy());
            yo.titulo.add_controller(clic);
        }

        yo.pintar();
        yo
    }

    /// Vuelve al mes de hoy. Lo llama tambien quien abre el panel: volver a abrirlo
    /// y seguir en marzo seria desconcertante.
    pub fn hoy(&self) {
        self.vista.set(hoy_ym());
        self.pintar();
    }

    fn mover(&self, meses: i32) {
        let (y, m) = self.vista.get();
        // La resta en base 0 es lo que hace que diciembre + 1 sea enero del año que
        // viene sin casos especiales.
        let total = y * 12 + (m - 1) + meses;
        self.vista.set((total.div_euclid(12), total.rem_euclid(12) + 1));
        self.pintar();
    }

    fn pintar(&self) {
        let (y, m) = self.vista.get();
        while let Some(h) = self.grid.first_child() {
            self.grid.remove(&h);
        }
        let Ok(primero) = glib::DateTime::from_local(y, m, 1, 0, 0, 0.0) else {
            self.titulo.set_text("--");
            return;
        };
        self.titulo.set_text(&fmt(&primero, "%B %Y"));

        // Las iniciales salen del idioma de la sesion: se formatea una semana de
        // verdad empezando por su lunes, en vez de escribirlas a mano.
        if let Ok(lunes) = primero.add_days(1 - primero.day_of_week()) {
            for i in 0..7 {
                let txt = lunes
                    .add_days(i)
                    .map(|d| fmt(&d, "%a"))
                    .unwrap_or_default();
                let l = gtk::Label::new(Some(txt.chars().take(2).collect::<String>().as_str()));
                l.add_css_class("cal-head");
                self.grid.attach(&l, i, 0, 1, 1);
            }
        }

        // `day_of_week` da 1 el lunes y 7 el domingo: la columna, menos uno.
        let hueco = primero.day_of_week() - 1;
        let dias = primero
            .add_months(1)
            .and_then(|d| d.add_days(-1))
            .map(|d| d.day_of_month())
            .unwrap_or(30);

        let (hy, hm) = hoy_ym();
        let hoy_dia = glib::DateTime::now_local().map(|d| d.day_of_month()).unwrap_or(0);

        for dia in 1..=dias {
            let celda = hueco + dia - 1;
            let l = gtk::Label::new(Some(&dia.to_string()));
            l.add_css_class("cal-day");
            // Hoy solo se marca en SU mes: en marzo no hay un «hoy».
            if (y, m) == (hy, hm) && dia == hoy_dia {
                l.add_css_class("today");
            }
            self.grid.attach(&l, celda % 7, 1 + celda / 7, 1, 1);
        }
    }
}

fn flecha(icono: &str) -> gtk::Button {
    let b = gtk::Button::new();
    b.add_css_class("cal-nav");
    b.set_has_frame(false);
    let img = gtk::Image::from_icon_name(icono);
    img.set_pixel_size(12);
    b.set_child(Some(&img));
    b
}

fn hoy_ym() -> (i32, i32) {
    glib::DateTime::now_local()
        .map(|d| (d.year(), d.month()))
        .unwrap_or((2026, 1))
}

fn fmt(d: &glib::DateTime, patron: &str) -> String {
    d.format(patron)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "--".into())
}
