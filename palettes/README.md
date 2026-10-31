# palettes/

Las paletas del escritorio, en ficheros nuestros.

Hasta ahora el tema se sacaba **raspando** `flavor.toml` y `tmtheme.xml` de Yazi con
expresiones regulares, dentro de un heredoc de Python cuyo `stdout` se ejecutaba con
`eval` como shell. Como Yazi sale del sistema (Thunar ocupa su sitio), las paletas
pasan a vivir aquí: planas, legibles y sin nada que adivinar.

El nombre del fichero es el nombre del tema. `hfs` lee el tema activo de
`~/.config/hypr/current_theme` y busca `<ese nombre>.toml` en esta carpeta.

## Formato

Una clave por línea, `clave = "#rrggbb"`. Comentarios con `#` al principio de la línea.
No es TOML de verdad: es un subconjunto plano, a propósito, para no necesitar un parser
entero por doce colores.

### Obligatorias

| clave | qué es |
|---|---|
| `bg` | el fondo del escritorio, lo más oscuro (o lo más claro) de la paleta |
| `fg` | el texto normal |
| `accent` | el color del tema |

### Opcionales

Si no están, se derivan de las tres anteriores, así que una paleta mínima son tres
líneas. Vale la pena escribirlas cuando la paleta original ya trae un color mejor que
el que saldría de mezclar.

| clave | si falta |
|---|---|
| `mode` | `dark` o `light`; si falta, lo decide la luminancia de `bg` |
| `surface` | `bg` mezclado un 6 % hacia `fg` |
| `overlay` | `bg` mezclado un 14 % hacia `fg` |
| `muted` | `fg` mezclado un 40 % hacia `bg` |
| `dim` | `fg` mezclado un 62 % hacia `bg` |
| `on_accent` | el texto que va **encima** del acento; se elige por contraste |
| `ok` `warn` `bad` `info` | el acento |

## Comprobarla

```bash
hfs palette          # enseña la paleta activa con sus cuadritos de color
```
