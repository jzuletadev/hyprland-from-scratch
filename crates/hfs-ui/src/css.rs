// ~/hyprland-from-scratch/crates/hfs-ui/src/css.rs
//
// El CSS sale de la paleta, no al reves.
//
// Esta es la razon de elegir GTK y no una biblioteca donde el estilo vive en el
// codigo: cambiar de tema es regenerar este texto y volver a cargarlo. No hay que
// recompilar nada, y el dia que el motor de temas sea nuestro, esto no cambia.

use hfs_core::Palette;

pub fn stylesheet(p: &Palette) -> String {
    format!(
        r#"
/* generado por hfs-ui desde la paleta «{nombre}» ({modo}) */

window.hfs {{
  background-color: transparent;
}}
window.hfs .root {{
  background-color: transparent;
}}

.panel {{
  padding: 10px 7px;
  color: {fg};
  font-family: "IBM Plex Mono", "JetBrainsMono Nerd Font", monospace;
}}

/* ---- dock de controles ---- */
.meter-value {{
  font-size: 10px;
  color: {muted};
  margin-bottom: 5px;
}}
.meter-icon {{
  margin-bottom: 7px;
  color: {on_accent};
}}
/* Cuando el relleno no llega al icono, el icono queda sobre la pista: hay que
   devolverle el color del texto normal o desaparece. */
.meter-icon.on-track {{
  color: {muted};
}}
.meter-icon.muted {{
  color: {bad};
}}

.dock-sep {{
  background-color: {sep};
  min-height: 1px;
  margin: 9px 4px;
}}

/* ---- rail de espacios ---- */
.panel.rail {{
  padding: 12px 0;
}}
/* GTK le pone a todo boton un tamaño minimo y un fondo; aqui no hace falta ninguno
   de los dos, el punto ES el boton.
   OJO: las reglas de abajo van cualificadas con `.rail button` a proposito. Este
   reset tiene dos selectores y `.ws-dot` uno solo, asi que por especificidad el
   `min-height: 0` de aqui le ganaba y los puntos salian de altura cero. */
.rail button {{
  min-width: 0;
  min-height: 0;
  padding: 0;
  margin: 0;
  border: none;
  background: none;
  box-shadow: none;
}}
.rail button.arch {{
  color: {accent};
  opacity: 0.85;
  margin-bottom: 2px;
}}
.rail button.arch:hover {{
  opacity: 1;
}}
.rail-sep {{
  background-color: {sep};
  min-height: 1px;
  margin: 7px 10px 9px 10px;
}}
.rail button.ws-dot {{
  min-width: 9px;
  min-height: 16px;
  margin: 3px 0;
  border-radius: 99px;
  background-color: {punto};
  transition: min-height 160ms ease, background-color 160ms ease;
}}
.rail button.ws-dot.has {{
  background-color: {punto_has};
}}
.rail button.ws-dot.on {{
  min-height: 36px;
  background-color: {accent};
}}
.rail button.ws-dot:hover {{
  background-color: {punto_hover};
}}

/* ---- reloj ---- */
.panel.reloj {{
  padding: 12px 16px 14px 16px;
}}
.clock {{
  font-size: 34px;
  font-weight: 500;
  color: {fg};
}}
.fecha {{
  font-size: 11px;
  color: {muted};
}}
.cal {{
  margin-top: 10px;
}}
.cal-head {{
  font-size: 9px;
  color: {dim};
  padding-bottom: 3px;
}}
.cal-day {{
  font-size: 10px;
  color: {muted};
  padding: 2px 0;
  border-radius: 4px;
}}
.cal-day.today {{
  background-color: {accent};
  color: {on_accent};
  font-weight: 600;
}}

/* ---- rueda y buscador ---- */
.rueda-raiz {{
  font-family: "IBM Plex Mono", "JetBrainsMono Nerd Font", monospace;
  color: {fg};
}}
/* El circulo: un radio enorme sobre un widget cuadrado. GTK4 no tiene `transform`,
   asi que el nodo marcado no crece como en el boceto; se distingue por el color. */
.nodo, .hub {{
  border-radius: 9999px;
  background-color: {nodo_bg};
  border: 1px solid {nodo_borde};
  color: {muted};
  padding: 0;
  transition: background-color 150ms ease, border-color 150ms ease, color 150ms ease;
}}
.nodo:hover, .nodo.sel, .hub:hover {{
  border-color: {accent};
  background-color: {acento_suave};
  color: {accent};
}}
.nodo-tag {{
  font-size: 12px;
  color: {muted};
  opacity: 0;
  transition: opacity 150ms ease;
}}
.nodo-tag.sel {{
  opacity: 1;
  color: {accent};
}}

.sfield {{
  background-color: {surface};
  border: 1px solid {accent};
  border-radius: 9999px;
  padding: 7px 15px;
}}
.sfield-ic {{
  color: {accent};
}}
/* El GtkEntry trae su propio marco, su fondo y su relleno; aqui el marco es la
   pastilla de fuera. El texto vive en un nodo `text` aparte, asi que el color y el
   tamaño hay que decirselos a los dos. */
entry.sq, entry.sq > text {{
  background: none;
  background-image: none;
  border: none;
  box-shadow: none;
  outline: none;
  padding: 0;
  min-height: 0;
  color: {fg};
  font-size: 15px;
  caret-color: {accent};
}}
entry.sq > text > placeholder {{
  color: {dim};
}}

.slist {{
  background-color: {surface};
  border: 1px solid {borde};
  border-radius: 10px;
  padding: 6px;
}}
.sitem {{
  padding: 8px 12px;
  border-radius: 7px;
  color: {fg};
}}
.sitem:hover {{
  background-color: {acento_suave};
}}
.sitem.on {{
  background-color: {accent};
  color: {on_accent};
}}
.sitem .k {{
  color: {dim};
  font-size: 11px;
}}
.sitem.on .k {{
  color: {on_accent};
}}
.sempty {{
  padding: 10px 12px;
  color: {dim};
}}

/* ---- piezas compartidas ---- */
.ph {{
  font-size: 12px;
  color: {fg};
  margin-bottom: 2px;
}}
.ph .sp {{
  font-size: 11px;
  color: {dim};
}}
.th {{
  font-size: 11px;
  color: {fg};
}}
.sub {{
  font-size: 10px;
  color: {dim};
}}
.big {{
  font-size: 22px;
  color: {fg};
}}
.ring-val {{
  font-size: 11px;
  color: {fg};
}}
.tile {{
  background-color: {tile};
  border-radius: 8px;
  padding: 8px 10px;
}}
.row {{
  padding: 5px 6px;
  border-radius: 6px;
  font-size: 11px;
  color: {fg};
}}
.row:hover {{
  background-color: {acento_suave};
}}
.row .k {{
  color: {dim};
  font-size: 10px;
}}
.bic {{
  min-width: 0;
  min-height: 0;
  padding: 5px;
  border: none;
  background: none;
  box-shadow: none;
  color: {muted};
  border-radius: 8px;
}}
.bic:hover {{
  color: {accent};
  background-color: {acento_suave};
}}
.bic.main {{
  color: {fg};
}}

/* ---- cabecera ---- */
.panel.cabecera {{
  padding: 12px 14px 14px 14px;
}}
.tabs {{
  margin-bottom: 10px;
}}
.tab {{
  min-width: 0;
  min-height: 0;
  padding: 5px 10px;
  border: none;
  background: none;
  box-shadow: none;
  border-radius: 7px;
  font-size: 11px;
  color: {muted};
}}
.tab:hover {{
  color: {fg};
  background-color: {tile};
}}
.tab.on {{
  color: {accent};
  background-color: {acento_suave};
}}
.tabs .sp {{
  font-size: 10px;
  color: {dim};
}}
.art {{
  background-color: {tile};
  border-radius: 8px;
  color: {dim};
  min-width: 64px;
  min-height: 64px;
}}
.media-src {{
  font-size: 10px;
  color: {accent};
}}
.media-title {{
  font-size: 17px;
  color: {fg};
}}
.media-sub {{
  font-size: 11px;
  color: {muted};
}}
.media-time {{
  font-size: 10px;
  color: {dim};
}}

/* ---- energia ---- */
.panel.energia {{
  padding: 12px 14px 14px 14px;
}}
.key {{
  min-width: 0;
  min-height: 0;
  padding: 9px 4px;
  border: none;
  box-shadow: none;
  background-color: {tile};
  border-radius: 8px;
  color: {muted};
}}
.key:hover {{
  color: {accent};
  background-color: {acento_suave};
}}
.key.danger:hover {{
  color: {bad};
  background-color: {rojo_suave};
}}
/* Un conmutador **tiene que verse encendido**. Sin esto, `.key` solo cambiaba al
   pasar el raton por encima, asi que pulsar «region» no se notaba y parecia que la
   opcion no existia. */
.key:checked {{
  background-color: {accent};
  color: {on_accent};
}}
.key:checked:hover {{
  background-color: {accent};
  color: {on_accent};
}}
.key:disabled {{
  color: {dim};
}}
/* Par excluyente: se leen como una sola pieza partida en dos. */
.seg button.key {{
  border-radius: 0;
}}
.seg button.key:first-child {{
  border-top-left-radius: 8px;
  border-bottom-left-radius: 8px;
}}
.seg button.key:last-child {{
  border-top-right-radius: 8px;
  border-bottom-right-radius: 8px;
}}
.key-tag {{
  font-size: 10px;
}}
.confirm {{
  background-color: {rojo_suave};
  border-radius: 8px;
  padding: 8px 10px;
  font-size: 11px;
  color: {fg};
}}
.confirm button {{
  min-height: 0;
  padding: 4px 12px;
  border: none;
  box-shadow: none;
  background-color: {tile};
  border-radius: 6px;
  font-size: 11px;
  color: {muted};
}}
.confirm button.confirm-si {{
  background-color: {bad};
  color: {on_bad};
}}
.seg {{
  background-color: {tile};
  border-radius: 8px;
  padding: 3px;
}}
.seg-item {{
  min-width: 0;
  min-height: 0;
  padding: 5px 8px;
  border: none;
  background: none;
  box-shadow: none;
  border-radius: 6px;
  font-size: 10px;
  color: {muted};
}}
.seg-item:hover {{
  color: {fg};
}}
.seg-item.on {{
  background-color: {accent};
  color: {on_accent};
}}

/* ---- notificaciones ---- */
.panel.notifs {{
  padding: 12px 14px 14px 14px;
}}
.note {{
  padding: 7px 8px;
  border-radius: 8px;
  background-color: {tile};
}}
.note:hover {{
  background-color: {acento_suave};
}}
.note.urgente {{
  border-left: 2px solid {bad};
}}
.note .thumb {{
  color: {muted};
}}
.note .src {{
  font-size: 10px;
  color: {accent};
}}
.note .k {{
  font-size: 10px;
  color: {dim};
}}
.note .ti {{
  font-size: 12px;
  color: {fg};
}}
.note .dt {{
  font-size: 10px;
  color: {muted};
}}

/* ---- panel invocado desde un radio de la rueda ---- */
.invocado {{
  background-color: {surface};
  border: 1px solid {borde};
  border-radius: 12px;
  padding: 14px 16px;
  color: {fg};
}}
.invocado .row.on {{
  background-color: {acento_suave};
  color: {accent};
}}
.foot {{
  font-size: 10px;
  color: {dim};
  margin-top: 4px;
}}
.aviso {{
  font-size: 11px;
  color: {bad};
}}
entry.clave, entry.clave > text {{
  background-color: {tile};
  border: 1px solid {accent};
  border-radius: 8px;
  padding: 6px 10px;
  color: {fg};
  font-size: 12px;
  caret-color: {accent};
}}
entry.clave {{
  min-width: 200px;
}}

/* ---- Tema: la fila de una paleta ---- */
/* `.row` ya trae el hueco y el hover; aqui solo el nombre y la marca de activa. */
.invocado .row .nm {{
  font-size: 12px;
  color: {fg};
}}
.invocado .row .k.acc {{
  color: {accent};
}}

/* ---- Fondo: la rejilla de miniaturas ---- */
/* El boton no pinta nada y deja ver la imagen; el borde de la elegida va fuera del
   contorno para que no recorte la miniatura. */
/* Dos señales distintas que pueden coincidir en la misma miniatura:
   `outline` = el fondo que esta puesto ahora.
   `box-shadow` = donde esta el cursor del teclado.
   Iban las dos por `outline` y se pisaban: como GTK dibuja el foco con `outline`,
   ponerlo transparente aqui hacia que moverse con las flechas no se viera. */
.minib {{
  padding: 0;
  margin: 0;
  min-width: 0;
  min-height: 0;
  background: none;
  border: none;
  border-radius: 6px;
  outline: 2px solid transparent;
  outline-offset: 2px;
}}
.minib:hover {{
  outline-color: {borde};
}}
.minib.on {{
  outline-color: {accent};
}}
.minib.cursor {{
  box-shadow: 0 0 0 2px {fg};
}}
.mini {{
  border-radius: 6px;
}}

/* ---- Atajos: el combo de teclas ---- */
.invocado .key-tag {{
  min-width: 140px;
}}

/* ---- Sistema: la ficha del centro de la pantalla ---- */
/* Es una ventana normal, no una superficie, asi que tiene que pintarse su propio
   fondo: no hay forma de layer-shell debajo que lo haga por ella. */
window.background {{
  background-color: transparent;
}}
.ficha {{
  border-radius: 14px;
}}
.ficha .row {{
  padding: 1px 0;
}}

/* ---- Sonido: las dos secciones ---- */
.invocado .sec {{
  font-size: 10px;
  color: {muted};
  margin-top: 2px;
}}
.invocado .row .k.mal {{
  color: {bad};
}}

/* ---- bandeja ---- */
.panel.bandeja {{
  padding: 12px 16px 14px 16px;
}}
.tray {{
  padding: 6px;
  min-width: 0;
  min-height: 0;
  border: none;
  box-shadow: none;
  background: none;
  border-radius: 9px;
  color: {fg};
}}
.tray:hover {{
  background-color: {acento_suave};
  color: {accent};
}}
/* `NeedsAttention` del protocolo: el programa pide que lo mires. */
.tray.atencion {{
  background-color: {rojo_suave};
  color: {bad};
}}
.tray.rec:hover {{
  background-color: {rojo_suave};
}}

/* ---- portapapeles ---- */
.panel.clip {{
  padding: 12px 14px 14px 14px;
}}
"#,
        nombre = p.name,
        modo = p.mode.as_str(),
        fg = p.fg,
        muted = p.muted,
        on_accent = p.on_accent,
        bad = p.bad,
        accent = p.accent,
        dim = p.dim,
        sep = p.fg.mix(p.surface, 0.80),
        punto = p.fg.rgba(0.16),
        punto_has = p.fg.rgba(0.42),
        punto_hover = p.fg.rgba(0.62),
        surface = p.surface,
        borde = p.fg.mix(p.surface, 0.80),
        nodo_bg = p.bg.mix(p.fg, 0.10),
        nodo_borde = p.fg.rgba(0.16),
        acento_suave = p.accent.rgba(0.16),
        rojo_suave = p.bad.rgba(0.16),
        on_bad = if p.bad.luminance() > 0.45 { p.bg } else { p.fg },
        tile = p.fg.rgba(0.07),
    )
}

/// Carga la hoja en la pantalla por defecto. Llamar otra vez con otra paleta
/// reemplaza la anterior.
pub fn apply(p: &Palette) -> gtk4::CssProvider {
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&stylesheet(p));
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    provider
}
