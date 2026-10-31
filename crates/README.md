# crates/ — el escritorio v2

El escritorio v2 es **un solo binario**, `hfs`, con subcomandos, y además **un solo
proceso**: `hfs shell` levanta todas las superficies a la vez. Cada superficie
(paneles de borde, rueda, controles) es un módulo suyo, no un programa aparte.

```
crates/
  hfs-core/   datos y estado. Sin GTK y SIN DEPENDENCIAS externas.
  hfs-ui/     la capa de dibujo: layer-shell, la forma del panel, el CSS.
  hfs/        el binario y sus subcomandos.
```

## Por qué un binario y no uno por superficie

Porque ya hacía falta de todas formas: en la v1, Waybar invoca
`screen-record-tui.py stop`, así que un TUI ya tenía que tener subcomandos. Con un
binario único eso deja de ser un caso especial. Además el núcleo se enlaza una vez
en lugar de once, y el lanzamiento queda uniforme: `$bin <lo que sea>`.

## Por qué un proceso y no uno por superficie

Cada proceso de GTK4 se trae su copia del toolkit, sus temas de iconos y su conexión
a Wayland. Ocho superficies serían ocho veces eso para enseñar un reloj y unos puntos.
Con un proceso, la paleta se lee una vez, el CSS se carga una vez y el socket de
eventos de Hyprland se escucha una vez para todas.

## Un juego de superficies por pantalla

Una superficie de layer-shell vive clavada en el monitor donde el compositor la pone
al mapearla, y ahí se queda. Con una sola copia, al pasarte a la otra pantalla el
panel sencillamente no está. Así que `shell.rs` recorre los monitores y crea un juego
entero en cada uno, con `set_monitor()` antes de `present()`, y los rehace cuando se
enchufa o se quita una pantalla.

Esto además resuelve dos cosas gratis: cada panel se dimensiona con la geometría de SU
monitor, y el rail de espacios enseña los espacios de SU monitor, que es como funciona
Hyprland de verdad. Los avisos (las teclas multimedia, `hfs all`) van solo a la
pantalla que tiene el foco: un aviso en la que no estás mirando es un aviso que no
existe.

## Lo que hay hoy

```bash
hfs shell                 # el escritorio: controles, espacios, reloj y rueda
hfs vol up|down|mute|N    # volumen (N = 0-100)
hfs bri up|down|N         # brillo  (N = 1-100)
hfs wheel                 # abre o cierra la rueda y el buscador
hfs all                   # enseña todas las superficies a la vez
hfs palette               # la paleta activa, con sus cuadritos de color
```

Siete superficies de borde más la rueda, cada una en su sitio:

| superficie | sitio | datos |
|---|---|---|
| **controles** | canto derecho, centrado | `wpctl`, `brightnessctl` |
| **espacios** | canto izquierdo, centrado | socket de eventos de Hyprland |
| **reloj** | esquina abajo-derecha | glib (hora local e idioma) |
| **cabecera** | arriba-centro | MPRIS, `/proc`, `/sys` |
| **energía** | esquina arriba-izquierda | PowerProfiles (D-Bus), `/sys` |
| **notificaciones** | esquina arriba-derecha | historial de dunst (D-Bus) |
| **portapapeles** | esquina abajo-izquierda | `wl-paste --watch` |
| **rueda** + buscador | centro, con SUPER | `.desktop` via GIO |
| └ **Wi-Fi** y **Bluetooth** | dentro de la rueda | NetworkManager, BlueZ |
| └ **captura** | dentro de la rueda, con SUPER+R | grim, slurp, satty, wf-recorder |
| └ **fondo** | dentro de la rueda, con SUPER+W | `~/Pictures/wallpapers` |
| └ **tema** | dentro de la rueda, con SUPER+A | `palettes/*.toml` |
| └ **atajos** | dentro de la rueda, con SUPER+/ | `hyprctl binds` |
| **bandeja** | abajo-centro | StatusNotifierItem (watcher + host) |
| **sistema** | ventana normal, centrada | `/sys`, `/proc`, `pci.ids` |

`hfs vol` y `hfs bri` **sin argumento** solo consultan: no sacan el panel. Con
argumento cambian el valor y avisan al escritorio, que asoma el dock un segundo y hace
de OSD. Por eso las teclas multimedia de `hyprland.conf` pasan por aquí.

`hfs all` existe porque los bordes no llevan ninguna marca: es la única forma de ver
dónde está todo sin ir canto por canto. Va en `SUPER+SHIFT+A` (con SHIFT porque
`SUPER+A` lo tiene el selector de temas).

`hfs wheel` va en **SUPER a secas** y es lo que sustituyó a **Rofi**. Abre la rueda;
escribir cualquier letra entra en el buscador y borrar hasta vaciarlo vuelve a la
rueda. `Esc` retrocede un paso cada vez: del buscador a la rueda, y de la rueda al
escritorio. Los radios abren de momento la herramienta de la v1 que les toca, y cada
uno se cambiará por su propio panel cuando le llegue el turno.

## Compilar

```bash
cargo build --release                 # el binario sale en target/release/hfs
cargo test --workspace
cargo clippy --workspace --all-targets
cargo run -p hfs-ui --example shape -- /tmp/forma.png   # la forma del panel, a lo grande
cargo run -p hfs-core --example hypr                   # el estado de Hyprland, ya parseado
cargo run -p hfs-core --example hypr -- "workspace 3"  # y una orden, para probar dispatch
```

Hace falta `gtk4-layer-shell` instalado (está en `packages/base.txt`). `install.sh`
compila y deja el binario en `~/.local/bin/hfs`, que es la ruta absoluta a la que
apunta `$bin` en `hyprland.conf`: Hyprland arranca por uwsm y no hereda el `PATH` de
una shell interactiva, así que por nombre no se encuentra.

## Las tres piezas

### hfs-core — los datos

Sin dependencias externas, a propósito. El formato de paleta es nuestro y es plano,
así que un parser de cuarenta líneas sale más barato que `serde` + `toml` y no ata la
versión de nada. Lo mismo con los errores.

La regla que gobierna este crate: **nada se traga un fallo en silencio.** La v1
devolvía `50 %` cuando no podía leer el volumen, `balanced` cuando no podía leer el
perfil y `#cba6f7` cuando no podía leer el acento, y nadie se enteraba nunca. Aquí el
error llega entero hasta quien lo pueda enseñar.

Dos errores de la v1 que ya no se pueden repetir, porque están arreglados en el único
sitio donde se tocan:

- **Subir el volumen ya no desmutea.** Eran dos acciones distintas metidas en una.
- **El dispositivo de brillo se autodetecta.** Estaba clavado `amdgpu_bl1` en el TUI
  y otras tres veces en la config de Waybar.

### hfs-ui — que el panel nazca del borde

`shape.rs` dibuja el contorno de una pieza: pegado al canto, radio solo por dentro, y
las muescas cóncavas que funden el panel con el borde. Se dibuja **una sola
orientación** (la del canto derecho) y las otras tres salen de una transformación, así
que la geometría se cambia en un sitio. El ejemplo `shape` la saca a un PNG sobre
fondo claro, que es la única forma razonable de comprobarla: sobre un escritorio oscuro
las muescas no se ven en una captura.

Un panel de **esquina** nace de la esquina, no cerca de ella: se ancla a los dos
cantos a la vez y entonces uno de sus extremos ya no tiene pantalla al lado. Ese va a
escuadra y sin muesca. De ahí los dos booleanos de `path()`, que salen de `Place`:
`Center` lleva las dos muescas, `Start` y `End` solo una.

`edge.rs` monta la superficie. Dos decisiones son el escritorio entero:

- **zona exclusiva = 0.** El compositor no reserva sitio, así que nada de lo que hay
  debajo se encoge. Es la diferencia entre esto y una barra: Waybar reserva, esto cae
  encima.
- **región de entrada.** La superficie mide lo que mide el panel abierto, pero
  mientras está cerrada solo recibe ratón una franja de unos pocos píxeles pegada al
  canto. Por eso el escritorio no lleva ninguna marca y aun así se puede invocar.

Y el tamaño no es fijo: `fit_length()` mide el contenido y ajusta lo que el panel
ocupa a lo largo de su canto (el rail de espacios crece con los espacios que haya), y
`fit_depth()` ajusta lo que entra en la pantalla (el reloj, cuyo ancho lo manda el
diseño pero cuyo alto cambia con el mes). Medir es mejor que calcular a mano: el número
sale del árbol de widgets ya con su CSS, así que tocar un relleno en la hoja de estilos
no deja el contenido cortado.

`css.rs` genera la hoja de estilo desde la paleta. Ésta es la razón de elegir GTK y no
una biblioteca donde el estilo vive en el código: cambiar de tema es regenerar un texto
y volver a cargarlo, sin recompilar nada.

`center.rs` es la otra clase de superficie: ocupa el monitor entero y se abre en el
centro. No comparte casi nada con los paneles de borde, y las tres diferencias son el
motivo de que sea un fichero aparte:

- **Teclado.** Pide `KeyboardMode::Exclusive`. Los atajos de Hyprland siguen
  funcionando —el compositor los resuelve antes de reenviar la tecla, comprobado:
  tocar SUPER con la rueda abierta la cierra—, pero cualquier otra tecla es suya.
- **Se mapea y se desmapea.** Justo por lo anterior: dejarla mapeada y escondida le
  robaría el teclado a todo lo demás. Un panel de borde vive mapeado siempre y se
  esconde con la región de entrada; ésta se oculta de verdad, y así se ahorra además
  el truco del rectángulo transparente.
- **Zona exclusiva −1.** Los paneles de borde van a 0 para no reservar sitio. Aquí es
  −1, que es «no me recortes por lo que hayan reservado otros»: sin eso, la barra de
  la v1 le quita su franja al velo y la rueda sale centrada en lo que sobra, no en la
  pantalla.

`dbus.rs` habla D-Bus **por GIO, no por zbus**. Es la decisión que mantiene a
`hfs-core` sin dependencias: GIO ya está enlazado, habla D-Bus entero, y sus proxys
viven en el bucle de glib, que es justo donde tienen que estar para que una señal
repinte un panel sin hilos ni canales. Meter zbus sería traer tokio para algo que ya
tenemos.

`widgets.rs` es el kit: el anillo, la barra, la cabecera de panel, el botón con
icono y la baldosa. Existe por lo que pasó en la v1, donde el marco de ventana estaba
copiado byte a byte en 11 de 11 ficheros.

### hfs — las superficies

Un módulo por superficie: `ctrl.rs`, `espacios.rs`, `reloj.rs`, `cabecera.rs`,
`energia.rs`, `notificaciones.rs`, `portapapeles.rs` y `rueda.rs`, más `red.rs`,
`bluetooth.rs`, `captura.rs`, `fondo.rs`, `tema.rs` y `atajos.rs` para los paneles que
viven dentro de la rueda, `bandeja.rs` con `sni.rs` debajo (el protocolo), y `sistema.rs`, que es el único que **no** es una superficie
de layer-shell sino una ventana normal: la ficha de la máquina se consulta de un
vistazo y se cierra, así que se puede mover y no debe pedir el teclado. Su clase la fija
el `app_id` de la aplicación (`dev.hfs.shell`), porque GTK4 no deja fijarla por ventana;
la `windowrule` de `hyprland.conf` casa con eso. `shell.rs` las
monta por pantalla y reparte los avisos; `hypr.rs` es el puente entre el socket de
eventos de Hyprland y el bucle de glib.

La **rueda** es la única que se construye en diferido, la primera vez que se pide.
Construirla cuesta poco (~2 MB de widgets); lo caro es mapearla, que son ~125 MB de
buffers de una superficie a pantalla completa que no se devuelven al cerrarla. Eso se
paga igual, pero solo en la pantalla donde de verdad se abra, y de paso no hay una
copia de la lista de aplicaciones por monitor.

El **buscador** saca las aplicaciones de `gio::AppInfo::all()` y las lanza con
`AppInfo::launch()`, no parseando `.desktop` a mano: GIO ya hace los códigos de campo
(`%U`, `%f`), `Terminal=true`, la activación por D-Bus y el *startup notification*, y
ya es una dependencia. Un `AppInfoMonitor` hace que lo que instales aparezca sin
reiniciar nada. El orden lo pone `hfs-core::fuzzy`, que no depende de GTK y se prueba
sin pantalla.

**El evento solo dispara, no informa.** Al llegar una línea de `socket2` se vuelve a
pedir el estado entero por el otro socket. Mantener un estado incremental a base de
deltas («`openwindow` suma uno al contador de ese espacio») es donde viven los fallos,
y la consulta es una ida y vuelta por un socket local. Los eventos se agrupan 20 ms
antes de consultar, porque una sola acción del usuario suelta tres o cuatro líneas
seguidas.

**Lo que no se ve no cuesta nada.** Ni el dock ni el reloj tienen ningún temporizador
corriendo con el panel cerrado; se enganchan a `connect_toggled`. El rail no tiene
temporizador en absoluto: es reactivo de verdad.

## Trampas que ya han costado una tarde

- **Un panel plegado no pinta, y si no pinta no hay buffer.** Sin buffer el compositor
  se queda con el tamaño inicial de `GtkWindow` (200x200) aunque GTK ya diga otra cosa.
  Medido: GTK decía 71x371 y Hyprland seguía diciendo 200x200, así que la franja de
  ratón caía a 130 px del canto y el panel no se abría nunca. Por eso la raíz lleva un
  rectángulo transparente del tamaño exacto que se dibuja siempre.
- **La región de entrada hay que ponerla cuando la superficie ya tiene tamaño**, no al
  mapear: al mapear mide 1x1 y la franja cae fuera. Va enganchada a `layout`.
- **Al entrar en la franja hay que ensancharla ANTES de esperar.** Si no, mover el
  ratón hacia dentro saca el puntero de la franja, el compositor manda un `leave` y el
  panel no llega a abrirse nunca.
- **Un GtkStack mide lo que su hijo más ancho**, y los hijos más estrechos se alinean
  dentro de esa anchura, no en la pantalla. La rueda salía 58 px a la izquierda del
  centro, que es justo la mitad de lo que el buscador le saca de ancho. Se arregla
  con `set_halign(Center)` en el hijo, no tocando el contenedor.
- **`activewindow` de Hyprland salta a cada cambio de TÍTULO**, o sea varias veces
  por minuto mientras escribes en un editor. Tratarlo como un cambio de espacios
  costaba una consulta de estado y un repintado de todos los railes por cada pulsación
  en otra aplicación.
- **En un buscador, los campos son niveles y no pesos.** Con un castigo pequeño por
  buscar en la descripción, escribir «bra» ponía «Yazi File Manager» por delante de
  «Brave»: su descripción («**B**lazing fast… **R**ust, based on **a**sync I/O»)
  encaja como acrónimo y eso puntúa más que las tres letras seguidas del nombre. El
  castigo tiene que ser enorme: cualquier coincidencia en el nombre gana a cualquiera
  en la descripción.
- **`VariantDict` solo sabe de `a{sv}` y ABORTA el proceso con cualquier otra cosa.**
  No devuelve un error: revienta dentro de una llamada de C que no puede desenrollar
  la pila. Y los dos servicios que más se usan aquí devuelven `a{sa{sv}}` —la
  configuración de NetworkManager y el `GetManagedObjects` de BlueZ—, así que abrir
  el panel de Wi-Fi mataba el escritorio entero. Por eso existe `dbus::dict_get`, que
  recorre cualquier diccionario a mano. Y `as_variant()` tampoco comprueba el tipo:
  suelta un CRITICAL y devuelve nada, que es por lo que el historial de
  notificaciones salía vacío teniendo dunst entradas dentro.
- **Un GtkStack mide lo que su hijo más ancho**, y los hijos más estrechos se alinean
  dentro de esa anchura, no en la pantalla. La rueda salía 58 px a la izquierda del
  centro, que es justo la mitad de lo que el buscador le saca de ancho. Se arregla
  con `set_halign(Center)` en el hijo, no tocando el contenedor.
- **`connect_g_properties_changed` exige que el cierre sea `Send + Sync`**, y un
  cierre con `Rc` y widgets de GTK no lo es ni lo puede ser. La salida es
  `connect_local("g-properties-changed", ...)`, que entrega la señal en el hilo del
  bucle principal, que es donde hace falta.
- **El CSS de GTK va por especificidad, como el de la web.** Un reset escrito
  `.rail button { min-height: 0 }` (dos selectores) le gana a `.ws-dot { min-height:
  16px }` (uno solo), así que los puntos del rail salían de altura cero y el panel se
  medía igual con uno que con tres. Las reglas de un widget tienen que ir igual de
  cualificadas que el reset que las precede.
- **`gtk_widget_measure` incluye los márgenes del widget.** Como los márgenes del
  cuerpo son justo el sitio de las muescas, lo que devuelve ya es la superficie entera
  y no hay que sumarle nada. Sumárselo deja el panel con una muesca de aire.
- **Las `layerrule` de Hyprland se aplican al mapear la capa**, no al recargar la
  configuración: después de un `hyprctl reload` hay que reiniciar el proceso para ver
  el efecto. Y ojo con la sintaxis: la forma plana con prefijo `match:` se parsea sin
  quejarse pero puede no coincidir nunca; la de bloque (`layerrule { name = x;
  match:namespace = ...; blur = true }`) sí avisa cuando te equivocas. Aun con la
  sintaxis buena no se ha conseguido que el desenfoque llegue a estas superficies; está
  apuntado en `hyprland.conf` y en el plan.
- **`glib` 0.22 ya no trae el enganche de señales Unix** al bucle principal, así que
  avisar al dock desde la CLI va por un socket de dominio Unix. Sale mejor igualmente:
  no hay pid que se quede viejo y llega sin retardo (un `GFileMonitor` habría metido
  hasta 800 ms, que para un OSD se nota).

## Dos trampas más, de los últimos paneles

**`GtkPicture` pide como tamaño natural el de su textura.** Una miniatura de un fondo
de 1920x1080 reclama 1920x1080 y la rejilla se lo da: el panel de Fondo salía con una
sola imagen ocupándolo entero. `set_size_request` no lo arregla, porque sólo fija un
mínimo. La salida es cargar ya escalado — `Pixbuf::from_file_at_scale` — para que la
textura nazca pequeña; de paso, una galería de 24 fondos son 24 miniaturas y no 24
imágenes enteras.

**El orden de los modificadores de `hyprctl binds` no es el de los bits.** `modmask`
pone SHIFT en el bit 1 y SUPER en el 64, así que recorrer la máscara en orden da
«SHIFT + SUPER + E», que no es como se escribe en `hyprland.conf` ni como se dice. La
tabla de `hfs-core/src/binds.rs` está en orden de escritura a propósito.


## La bandeja, y por qué hay que ser también el *watcher*

El protocolo StatusNotifierItem tiene **tres** papeles: los programas publican un
*item*, un *host* los pinta, y un *watcher* hace de registro entre ambos. Casi todo el
mundo da por hecho que el watcher ya existe porque lo trae el panel del sistema. Aquí
lo traía Waybar, y al quitarla no quedó ninguno — comprobado: nadie poseía
`org.kde.StatusNotifierWatcher` en el bus. Así que `sni.rs` exporta la interfaz del
watcher con `gio::DBusConnection::register_object` y toma el nombre con
`bus_own_name`, además de ser el host.

Tres trampas que costaron:

**Un `service` que empieza por `/` también casa con `split_once('/')`.** Los programas
pueden registrarse con un nombre de bus, con una ruta de objeto (y entonces el bus es
quien llamó) o con `bus/ruta` de una pieza. Comprobar el `split_once` antes que el
`starts_with('/')` daba bus vacío para la forma de ruta suelta, que es justo la que
mandan todos los basados en libappindicator — Steam incluido. Lo cazó una prueba
unitaria antes de que llegara a correr.

**Nadie manda un «me voy».** Un programa que se cierra simplemente desaparece del bus,
así que hay que suscribirse a `NameOwnerChanged` y quitar sus items a mano; si no, la
bandeja acumula iconos muertos. Y `subscribe_to_signal` devuelve una `SignalSubscription`
que **cancela la suscripción al soltarla**: hay que guardarla.

**Un programa que arrancó sin watcher no se entera solo.** Los items preguntan al
arrancar; si no había nadie, muchos se rinden. Por eso al tomar el nombre se le da un
empujón a `udiskie`, que es nuestro. Comprobar si la lista está vacía no vale: basta
con que otro programa sí se haya registrado para que parezca que todo está bien, así
que hay que preguntar por él en concreto.

## El único sondeo permanente, y lo que costó

La marca de grabación del canto tiene que aparecer sin que nadie roce nada, y
`wf-recorder` no avisa por D-Bus ni por fichero. Es el único sondeo permanente del
escritorio, y la primera versión costaba **1,57 % de CPU en reposo** — más que el
`waybar-autohide.sh` de la v1 (1,3 %) que esta v2 presume de haber jubilado. Dos
motivos: barría `/proc` entero (~800 ficheros) cada segundo, y había **un temporizador
por pantalla** haciendo el mismo trabajo.

La salida está en `proc.rs::Vigia`: cachea el PID, así que un tick normal son dos
lecturas de `/proc`, y sólo barre entero cada 8 s para enterarse de algo lanzado por
fuera. Lo que lanza el propio panel ceba el PID —el `exec` del `sh -c` hace que ese PID
acabe siendo el de `wf-recorder`— y se ve al instante. Más un solo temporizador en
`shell.rs` para todas las pantallas. Medido: **0,43 %**, por debajo del 0,48 % de antes
de que la bandeja existiera.

## Recoger a los hijos

`hfs shell` no sale nunca, así que un hijo lanzado y olvidado se queda de **zombi**:
ya no corre, pero su entrada en la tabla de procesos sigue ahí con su `/proc/<pid>/comm`
intacto. El panel de Captura se quedó diciendo «grabando» con la grabación parada por
esto, y `pgrep` también daba el zombi por bueno (lo lista como `<defunct>`), o sea que
la v1 tenía el mismo fallo latente y se salvaba sólo porque sus TUIs eran procesos
cortos. Todo lo que lanza el escritorio pasa por `proc.rs`, que recoge con
`glib::child_watch_add`, y toda comprobación de «¿está vivo?» salta los zombis mirando
el estado en `/proc/<pid>/stat`.

## Un conmutador tiene que verse encendido

Los conmutadores de región y audio de Captura funcionaban desde el primer día, pero
`.key` sólo tenía `:hover`: pulsarlos no cambiaba nada a la vista. El usuario reportó
que «ya no se puede grabar por región». Un control invisible es, desde donde está el
usuario, una función que no existe. Ahora hay `.key:checked` y el modo es un par
excluyente (`set_group`) en vez de una casilla suelta, porque «región» a secas no dice
contra qué.

## Dos trampas de D-Bus que rompieron cosas ya «hechas»

**`(a, b, c).to_variant()` sobre una tupla de Rust mete cada elemento en una caja.**
Si `a`, `b` y `c` ya son `glib::Variant`, lo que sale es `(vvv)` y no `(ooo)`.
NetworkManager lo rechaza con *«Type of message, "(vvv)", does not match expected type
"(ooo)"»*, que es exactamente lo que se veía en el panel de Wi-Fi. Pasaba en los **dos**
caminos —`ActivateConnection` para una red guardada y `AddAndActivateConnection` para
una nueva—, así que conectar no funcionó nunca, y el panel se dio por bueno porque
nadie llegó a pulsar una red de verdad. Los argumentos se montan con
`glib::Variant::tuple_from_iter`, que no envuelve nada.

**Un servicio «activable» no se activa solo por crear un proxy.** La pestaña de Medios
hablaba con `playerctld`, que sigue al último reproductor que usaste. Lo levantaba el
módulo `mpris` de **Waybar**, y al quitarla dejó de levantarlo nadie: D-Bus sólo activa
un servicio cuando alguien le **llama**, y aquí no se llegaba a llamar porque
`dbus::vivo()` veía el nombre sin dueño y se rendía antes. Resultado: «no hay nada
sonando» con YouTube sonando delante. Ahora `Reproductor` enumera los nombres
`org.mpris.MediaPlayer2.*` del bus y elige —playerctld si está, si no el que esté
sonando—, así que `playerctl` pasa de requisito a mejora.

Las dos tienen la misma moraleja: **una superficie que se construye sin error no está
probada**. Las dos partes rotas se habían dado por hechas y las encontró el usuario
usándolas.

## `wpctl` ofrece cosas que no se pueden elegir

`wpctl status` lista las cuatro salidas HDMI de la gráfica aunque solo una tenga un
monitor enchufado, y **`wpctl set-default` sobre una desconectada no hace nada y no
devuelve error**: WirePlumber la ignora en silencio. El panel de Sonido las ofrecía las
cuatro y tres no respondían al tocarlas.

La disponibilidad solo la dice `pactl`, en la línea del puerto (`not available`), así
que `audio::no_disponibles()` cruza las dos herramientas: se emparejan por el nombre,
porque el de `wpctl` es literalmente la `Description:` de `pactl`. Y hay que mirar las
**dos** listas: un micrófono de auriculares sin auriculares falla igual de callado, y
sus puertos se declaran con `[In]` en vez de `[Out]`, así que arreglarlo solo para las
salidas dejaba la mitad del fallo en pie.


## Escape cierra, Retroceso retrocede

Escape retrocedía un paso cada vez: del panel a la rueda, y de la rueda al escritorio.
Eso obliga a `SUPER → rueda → opción → rueda → salir`, y la rueda es un **sitio de
paso**, no un destino: quedarse en ella al terminar una acción es un paso de más en el
camino más común. Ahora Escape cierra desde donde sea, y **Retroceso** es el que vuelve
a la rueda, lo que el buscador ya hacía al vaciarse.

Retroceso no se roba cuando hay dónde escribir: `hay_foco_de_texto()` mira el foco real
de la ventana, y pregunta por `GtkText` además de por `GtkEntry` porque un `Entry`
delega el texto en un hijo interno y el foco acaba ahí. Sin eso, borrar una letra de la
contraseña del Wi-Fi o del filtro de atajos te sacaría del panel.

## El foco de GTK no sirve en una superficie de layer-shell

La rejilla de fondos tenía que moverse con las flechas. El camino obvio —hacer los
botones enfocables y dejar que GTK navegue— **no funciona aquí**: `grab_focus()`
devuelve falso sin decir nada, así que las flechas no tienen de dónde partir. Y aunque
agarrara, GTK pinta el foco con `outline`, que es justo la propiedad con la que la
rejilla marca el fondo que está puesto: una señal taparía a la otra.

Así que el cursor del teclado es una clase CSS propia (`.minib.cursor`, con
`box-shadow`) y las flechas las gobierna el panel. Para eso los paneles pueden devolver
un manejador de teclas opcional —el tercer elemento de `Hecho`— que `rueda.rs` consulta
**antes** de hacer nada con la tecla. Devolver `true` la consume. Los demás paneles
devuelven `None` y se comportan como siempre.

## Una acción que termina, cierra

Aplicar un fondo dejaba la rejilla abierta encima del fondo que acabas de poner, que es
lo único que no te deja verlo. Fondo y Captura reciben un `cerrar` y lo llaman cuando
la acción se ha completado. Tema no lo necesita: cambiar de paleta rehace todas las
superficies, y la rueda se va con ellas.


## Una sola unidad para todo

Cada superficie traía su porcentaje. Acabaron siendo siete distintos —0,028 · 0,037 ·
0,19 · 0,23 · 0,25 · 0,25 · 0,44— con dos radios de esquina diferentes. Por separado
cada uno parecía razonable; juntos en pantalla no parecían del mismo escritorio.

`hfs-ui/src/medidas.rs` las tiene todas y las saca de una unidad `x` y del monitor.
Cambiar `x` mueve el escritorio entero a la vez, que es justo lo que antes no se podía
hacer. Las cuatro esquinas miden lo mismo; el alto sigue saliendo del contenido
(`fit_depth`), porque forzarlo igual sería recortar el portapapeles o dejar medio panel
vacío en las notificaciones.

El radio y la muesca **no** escalan con la pantalla: son el grosor del trazo del
escritorio y el ojo los mide contra el texto, no contra el monitor.

**Los paneles de esquina nacen DEL vértice**, no cerca de él: `Place::Start(0)` /
`End(0)`. El parámetro existe por si alguna vez hace falta separarlos, y entonces
dejarían de tocar la esquina y llevarían las dos muescas en vez de una.

Un detalle medido: sacar el ancho de los raíles del lado corto del monitor los dejaba
en 32 px, demasiado estrecho para arrastrar una barra de volumen. Es una medida
horizontal y va contra el ancho.

## El buscador: una cara aparte, pero anclada

Dos intentos y los dos fallaron por el mismo sitio, que merece quedar escrito.

**Primero** era una pagina aparte anclada arriba de la pantalla. La rueda desaparecia
—eso esta bien— pero el campo salia en un sitio raro y demasiado grande.

**Despues** se metio dentro del `gtk::Fixed` de la rueda, con la lista debajo del
anillo. Peor: `GtkFixed` **mide a partir de la posicion de sus hijos**, asi que la
lista hacia crecer el lienzo con cada resultado y, al estar centrado, el campo subia
en pantalla segun escribias. Ademas dejaba el anillo detras y los resultados a una
distancia absurda, porque el lienzo es bastante mas grande que el anillo que dibuja.

**Lo que funciona** es la cara aparte otra vez, pero anclada donde toca: `valign:
Start` y un margen superior fijo que deja el campo sobre el centro de la pantalla,
justo donde estaba el hub. Los resultados crecen hacia abajo y el campo no se mueve
nunca. Centrarlo seria volver al baile: el contenido cambia de alto con cada letra.

La leccion es que **«no se mueve» y «esta centrado» son incompatibles** cuando el
contenido cambia de tamano. Hay que elegir, y para escribir gana no moverse.

El hub perdio su texto y mide **lo mismo que un radio**. Era mas grande y se notaba:
rompia el anillo en vez de cerrarlo. «Escribe para buscar» era ademas una instruccion
que se lee una vez y estorba para siempre.

**Y la rueda mide lo mismo que un panel de esquina**, porque sale de `Medidas` como
todo lo demas en vez de tener su propio porcentaje. Tenia el 30 % del ancho —576 px
contra los 408 de un panel— y se veia a simple vista. El campo de busqueda, la lista
de resultados y los paneles que se invocan desde un radio usan esa misma anchura: en
el centro de la pantalla hay **una sola medida**.

## `ellipsize` no basta: una etiqueta elidida sigue pidiendo su ancho

La lista de resultados cambiaba de ancho con cada letra. Las etiquetas ya tenian
`set_ellipsize(End)`, asi que parecia que no podian estirar nada — pero elidir es lo
que se hace al PINTAR; al MEDIR, la etiqueta sigue pidiendo el ancho natural de su
texto. Como las descripciones de las aplicaciones son largas y distintas, cada
busqueda daba una anchura distinta.

Lo que lo ata es `set_max_width_chars`. Y el reparto no es simetrico: el **nombre**
es lo que se lee, asi que pide lo suyo hasta un tope (`NOMBRE_MAX`); la
**descripcion** es relleno, asi que pide 1 y se queda con el hueco que sobre. Ponerles
el tope de 1 a las dos dejaba todos los nombres en «...».


## Una esquina son dos cantos

Un panel de esquina se invocaba sólo por el canto del que nace: Energía por el de
arriba, Portapapeles por el de abajo. Eso obliga a recordar cuál de los dos es, que es
justo lo que un escritorio sin marcas no puede pedir.

`apply_input_region` monta ahora una **L** para los paneles de esquina: la franja de su
propio canto más la del canto perpendicular que forma el vértice, unidas con
`cairo::Region::union_rectangle`. El panel ya está anclado a los dos cantos, así que su
superficie cubre el vértice y las dos franjas caben dentro sin tocar la geometría.

## Los paneles de esquina tienen alto FIJO, y las listas se desplazan dentro

Dejar el alto al contenido (`fit_depth`) parecía lo razonable y se veía mal de dos
maneras a la vez: Notificaciones con siete avisos crecía hasta **tapar el dock de
controles** del canto derecho, y el calendario se quedaba en 189 px, enano al lado de
los demás.

Los cuatro miden ahora lo mismo. El número no es inventado: es lo que `fit_depth` le
daba a **Energía**, que es el panel cuyo contenido no se puede recortar —seis acciones,
la batería y los perfiles—. Lo que pide de forma natural es la medida buena, y así ese
panel no necesita tocarse. Ojo con confundirlo con `alto`, que es el de PARTIDA (0,185)
y se queda corto: con él, Energía perdía la fila de perfiles.

El calendario reparte lo que le sobra (`vexpand` sobre el `Grid`, que ya era
`row_homogeneous`) y los que llevan lista la meten en un `ScrolledWindow` con
`vexpand`: el tope es el panel, así que el scroll no necesita el suyo. Los topes de
cantidad suben a lo que de verdad hay (40 notificaciones, 60 entradas, los 68 atajos);
los de antes —6, 8 y 11— existían sólo para que el panel no se estirara, y lo que
sobraba no se podía ver de ninguna manera.

En Atajos las flechas mueven un cursor **y arrastran el scroll con él**. La posición de
la fila se saca con `compute_bounds` contra la lista, que es contra lo que mide el
ajuste del scroll; sin eso, bajar mueve la marca pero la lista no sigue y el cursor se
pierde por debajo del borde.
