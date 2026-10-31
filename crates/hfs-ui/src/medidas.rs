// ~/hyprland-from-scratch/crates/hfs-ui/src/medidas.rs
//
// Las medidas del escritorio, en un solo sitio.
//
//   ESTO EXISTE PORQUE NO LAS HABIA. Cada superficie traia su propio porcentaje, y
//   acabaron siendo siete distintos (0,028 · 0,037 · 0,19 · 0,23 · 0,25 · 0,25 ·
//   0,44) con dos radios de esquina diferentes. Separados se veian razonables; juntos
//   en pantalla no parecian del mismo escritorio.
//
// Todo sale de UNA unidad, `x`, y del lado corto del monitor. Cambiar `x` mueve el
// escritorio entero a la vez, que es justo lo que no se podia hacer antes.

use crate::gtk::gdk;

/// Un panel de esquina, en tanto por uno del ancho. Los cuatro miden igual.
const ESQUINA_REL: f64 = 0.195;
/// El panel de arriba al centro, que lleva dos pestañas.
const CENTRO_REL: f64 = 0.38;
/// El de abajo al centro: la bandeja crece con los iconos, esto es su minimo.
const BANDEJA_REL: f64 = 0.11;
/// Lo que un rail entra en la pantalla. Va contra el ANCHO y no contra el lado
/// corto: es una medida horizontal, y sacarla del alto dejaba el dock de controles
/// en 32 px, demasiado estrecho para arrastrar una barra de volumen.
const RAIL_REL: f64 = 0.030;

/// Alto inicial de un panel que luego se mide solo (`fit_depth`). No es el final:
/// solo evita que la superficie nazca con el 200x200 de GtkWindow.
const ALTO_REL: f64 = 0.185;

/// Alto de un panel de ESQUINA. Es fijo, no sale del contenido.
///
/// Dejarlo al contenido se veia mal de dos maneras a la vez: Notificaciones con siete
/// avisos crecia hasta tapar el dock de controles del canto derecho, y el calendario
/// se quedaba en 189 px, enano al lado de los demas.
///
/// El numero sale de Energia, que es el panel cuyo contenido no se puede recortar:
/// seis acciones, la bateria y los perfiles. Lo que pide de forma natural es la medida
/// buena, asi que ese panel no necesita tocarse. El calendario reparte el alto que le
/// sobra y los que llevan lista se desplazan con scroll.
/// 0,215 de 1080 son 232 px, que es justo lo que `fit_depth` le daba a Energia
/// cuando cada panel se medía solo. Con 0,185 —el alto de PARTIDA— se quedaba corto
/// y le recortaba la fila de perfiles.
const ESQUINA_ALTO_REL: f64 = 0.215;

#[derive(Debug, Clone, Copy)]
pub struct Medidas {
    /// Ancho de un rail (lo que entra en la pantalla).
    pub rail: i32,
    /// Ancho de un panel de esquina. Los cuatro iguales.
    pub esquina: i32,
    /// Ancho del panel de arriba al centro.
    pub centro: i32,
    /// Ancho minimo de la bandeja.
    pub bandeja: i32,
    /// Alto de partida de un panel que se mide solo.
    pub alto: i32,
    /// Alto de un panel de esquina. Los cuatro iguales, y fijo: lo que sobre se
    /// desplaza con scroll y lo que falte queda en blanco.
    pub esquina_alto: i32,
    pub radio: f64,
    pub muesca: f64,
}

impl Medidas {
    pub fn para(g: &gdk::Rectangle) -> Self {
        let (w, h) = (g.width() as f64, g.height() as f64);
        Self {
            rail: (w * RAIL_REL).round() as i32,
            esquina: (w * ESQUINA_REL).round() as i32,
            centro: (w * CENTRO_REL).round() as i32,
            bandeja: (w * BANDEJA_REL).round() as i32,
            alto: (h * ALTO_REL).round() as i32,
            esquina_alto: (h * ESQUINA_ALTO_REL).round() as i32,
            // El radio y la muesca no escalan con la pantalla: son el grosor del
            // trazo del escritorio, y a 4K se verian igual de gruesos que a 1080p
            // porque el ojo los mide contra el texto, no contra la pantalla.
            radio: 15.0,
            muesca: 17.0,
        }
    }
}
