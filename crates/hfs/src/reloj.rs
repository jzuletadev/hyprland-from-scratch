// ~/hyprland-from-scratch/crates/hfs/src/reloj.rs
//
// El reloj de la esquina de abajo a la derecha: hora grande, la fecha al lado y el
// mes entero con el dia de hoy marcado.
//
// Es la primera superficie de ESQUINA: se ancla a los dos cantos a la vez y por eso
// solo lleva una muesca, la del lado que da a la pantalla. En la otra no hay pantalla
// de la que nacer.
//
// La fecha sale de glib, no de un crate de fechas. glib ya trae la zona horaria y el
// idioma del sistema, asi que `%a` y `%b` salen en el idioma que tenga puesto la
// sesion sin añadir ni una dependencia. La semana empieza en lunes, como el
// calendario de la v1.
//
// Mientras el panel esta cerrado no corre ningun temporizador. Cuesta poco, pero la
// regla del escritorio es que lo que no se ve no cuesta nada, y un reloj que solo da
// horas y minutos no tiene que despertarse cada segundo.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk4 as gtk;
use gtk4::glib;
use gtk4::prelude::*;

use hfs_core::Palette;
use hfs_ui::{EdgeSurface, Handle, Place, Side, Spec};

/// Cada cuanto se mira la hora con el panel abierto. Enseña horas y minutos, asi que
/// medio minuto basta para que nunca se vea un minuto de menos.
const LATIDO_S: u64 = 30;

pub struct Reloj {
    surface: EdgeSurface,
    pub handle: Handle,
}

impl Reloj {
    pub fn new(app: &gtk::Application, palette: &Palette, monitor: &gtk::gdk::Monitor) -> Self {
        let g = monitor.geometry();
        let m = hfs_ui::medidas::Medidas::para(&g);

        let mut spec = Spec::new("hfs-reloj", Side::Bottom, m.esquina, m.esquina_alto);
        // Abajo a la derecha, separado del vertice como todas las esquinas.
        spec.place = Place::End(0);
        spec.radius = m.radio;
        spec.notch = m.muesca;

        let surface = EdgeSurface::new(app, spec, palette);
        surface.set_monitor(monitor);
        let handle = surface.handle();
        surface.body.add_css_class("reloj");
        surface.body.set_orientation(gtk::Orientation::Vertical);

        // --- cabecera: hora grande y fecha al lado ---
        let hora = gtk::Label::new(None);
        hora.add_css_class("clock");
        hora.set_xalign(0.0);
        let fecha = gtk::Label::new(None);
        fecha.add_css_class("fecha");
        fecha.set_xalign(0.0);

        let cabecera = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        cabecera.set_valign(gtk::Align::Baseline);
        cabecera.append(&hora);
        cabecera.append(&fecha);

        let cal = gtk::Grid::new();
        cal.add_css_class("cal");
        cal.set_hexpand(true);
        cal.set_row_homogeneous(true);
        // Reparte las filas por el alto que sobre. El panel tiene alto fijo, asi que
        // sin esto el calendario se apelotona arriba y deja un hueco debajo.
        cal.set_vexpand(true);
        cal.set_column_homogeneous(true);

        surface.body.append(&cabecera);
        surface.body.append(&cal);

        // El alto se vuelve a medir despues de pintar: un mes de seis semanas ocupa
        // una fila mas que uno de cinco, y el panel tiene que seguirle.
        let pintar = {
            let (hora, fecha, cal) = (hora.clone(), fecha.clone(), cal.clone());
            Rc::new(move || {
                pintar(&hora, &fecha, &cal);
            })
        };
        pintar();

        // El latido solo corre mientras se ve.
        {
            let pintar = pintar.clone();
            let timer: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
            handle.connect_toggled(move |abierto| {
                let mut t = timer.borrow_mut();
                if let Some(id) = t.take() {
                    id.remove();
                }
                if abierto {
                    pintar();
                    let pintar = pintar.clone();
                    *t = Some(glib::timeout_add_local(
                        Duration::from_secs(LATIDO_S),
                        move || {
                            pintar();
                            glib::ControlFlow::Continue
                        },
                    ));
                }
            });
        }

        Self { surface, handle }
    }

    pub fn destroy(&self) {
        self.surface.destroy();
    }

    pub fn present(&self) {
        self.surface.present();
    }
}

fn fmt(d: &glib::DateTime, patron: &str) -> String {
    d.format(patron)
        .map(|s| s.to_string())
        .unwrap_or_else(|_| "--".into())
}

fn pintar(hora: &gtk::Label, fecha: &gtk::Label, cal: &gtk::Grid) {
    let Ok(ahora) = glib::DateTime::now_local() else {
        hora.set_text("--:--");
        return;
    };
    hora.set_text(&fmt(&ahora, "%H:%M"));
    fecha.set_text(fmt(&ahora, "%a %e %b").trim());

    while let Some(hijo) = cal.first_child() {
        cal.remove(&hijo);
    }

    // Las iniciales de los dias salen del idioma de la sesion: se formatea una semana
    // de verdad empezando por su lunes, en vez de escribirlas a mano.
    let lunes = ahora.add_days(1 - ahora.day_of_week());
    for i in 0..7 {
        let txt = match &lunes {
            Ok(l) => l
                .add_days(i)
                .map(|d| fmt(&d, "%a"))
                .unwrap_or_else(|_| String::new()),
            Err(_) => String::new(),
        };
        let l = gtk::Label::new(Some(txt.chars().take(2).collect::<String>().as_str()));
        l.add_css_class("cal-head");
        cal.attach(&l, i, 0, 1, 1);
    }

    let (y, m, hoy) = (ahora.year(), ahora.month(), ahora.day_of_month());
    let Ok(primero) = glib::DateTime::from_local(y, m, 1, 0, 0, 0.0) else {
        return;
    };
    // `day_of_week` da 1 el lunes y 7 el domingo, que es justo la columna menos uno.
    let hueco = primero.day_of_week() - 1;
    let dias = primero
        .add_months(1)
        .and_then(|d| d.add_days(-1))
        .map(|d| d.day_of_month())
        .unwrap_or(30);

    for dia in 1..=dias {
        let celda = hueco + dia - 1;
        let l = gtk::Label::new(Some(&dia.to_string()));
        l.add_css_class("cal-day");
        if dia == hoy {
            l.add_css_class("today");
        }
        cal.attach(&l, celda % 7, 1 + celda / 7, 1, 1);
    }
}
