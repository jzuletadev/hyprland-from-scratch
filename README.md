Edit

# Hyprland From Scratch

Guía paso a paso para construir un escritorio basado en Arch Linux y Hyprland desde cero, con control total sobre los componentes instalados y sus configuraciones.

---

# Fase 0 - Instalación de Arch Linux

## Objetivo

Instalar una base mínima de Arch Linux sobre la cual construiremos todo el escritorio.

Al finalizar esta fase el equipo deberá iniciar únicamente en una consola TTY, sin ningún entorno gráfico instalado.

---

# Paso 1 - Iniciar desde la USB de Arch

Crear una memoria USB con la imagen oficial de Arch Linux y arrancar desde ella.

Cuando aparezca el prompt del live ISO verificar que existe conexión a Internet.

Para conexiones cableadas normalmente ya estará disponible.

Comprobar:

```bash
ping archlinux.org
```

Deberías obtener respuesta.

---

# Paso 2 - Iniciar el instalador

Ejecutar:

```bash
archinstall
```

---

# Paso 3 - Configurar archinstall

En el menú configurar las opciones en el siguiente orden.

## Mirrors

Seleccionar el país más cercano o dejar la selección automática.

---

## Locales

Configurar:

- Keyboard layout: `latam` (o `us`, según preferencia)
- Locale: `en_US.UTF-8`

---

## Disk Configuration

Seleccionar el disco donde se instalará Arch.

Elegir:

```
Use entire disk
```

Particionado automático.

No utilizar particionado manual para esta guía.

---

## Bootloader

Dejar la opción por defecto.

Actualmente suele ser:

```
systemd-boot
```

---

## Swap

Dejar la opción automática.

---

## Hostname

Elegir el nombre que tendrá la máquina.

Ejemplo:

```
arch-hypr
```

---

## Root Password

Configurar una contraseña para root.

---

## User Account

Crear el usuario principal.

Activar:

```
✓ Add user to wheel group
```

Esto permitirá utilizar `sudo`.

---

## Profile

Seleccionar:

```
Minimal
```

o

```
None
```

**No instalar ningún entorno gráfico.**

No seleccionar:

- GNOME
- KDE Plasma
- XFCE
- Cinnamon
- LXQt
- Hyprland

---

## Audio

Seleccionar:

```
PipeWire
```

---

## Network

Seleccionar:

```
NetworkManager
```

---

## Additional Packages

Agregar únicamente:

```text
git
curl
base-devel
```

No instalar nada más en esta etapa.

---

## Timezone

Seleccionar la zona horaria correspondiente.

Ejemplo:

```
America/Guatemala
```

---

## Confirmar instalación

Revisar el resumen.

Si todo está correcto seleccionar:

```
Install
```

Esperar a que finalice la instalación.

---

# Paso 4 - Reiniciar

Cuando finalice:

```bash
reboot
```

Retirar la memoria USB cuando el equipo lo solicite.

---

# Resultado esperado

Al arrancar nuevamente el equipo deberías ver únicamente una consola similar a esta:

```text
Arch Linux

arch-hypr login:
```

Todavía no existe:

- Entorno gráfico
- Hyprland
- Waybar
- Display Manager
- Login gráfico

Este será nuestro punto de partida para construir todo el escritorio.

---

# Fase 0.5 — Preparación del entorno de trabajo

## Objetivo

Poder administrar el equipo completamente por SSH y dejar preparado el repositorio donde se versionarán los dotfiles y la guía.

---

## 0.5.1 Configurar OpenSSH

Instalar (si no se instaló antes):

```
sudo pacman -S openssh
```

Habilitar el servicio:

```
sudo systemctl enable --now sshd
```

Verificar:

```
systemctl status sshd
```

Debe aparecer:

```
Active: active (running)
```

---

## 0.5.2 Obtener la IP del equipo

```
hostname -I
```

Ejemplo:

```
192.168.1.120
```

---

## 0.5.3 Conectarse desde la computadora principal

Desde la máquina principal:

```
ssh TU_USUARIO@192.168.1.120
```

Aceptar la huella digital la primera vez.

---

## 0.5.4 Generar llave SSH para GitHub

En la máquina de pruebas:

```
ssh-keygen -t ed25519 -C "tu_correo@example.com"
```

Aceptar la ruta por defecto.

Mostrar la clave pública:

```
cat ~/.ssh/id_ed25519.pub
```

Copiarla y agregarla en GitHub:

**Settings → SSH and GPG keys → New SSH key**

---

## 0.5.5 Probar conexión con GitHub

```
ssh -T git@github.com
```

La primera vez responder `yes`.

Debe aparecer un mensaje similar a:

```
Hi usuario! You've successfully authenticated...
```

---

## 0.5.6 Configurar Git

```
git config --global user.name "Tu Nombre"
git config --global user.email "tu_correo@example.com"
git config --global init.defaultBranch main
```

Verificar:

```
git config --list
```

---

## 0.5.7 Crear estructura del proyecto

```
mkdir -p ~/Projects/hyprland-from-scratch/{guide,dotfiles,scripts,assets}
cd ~/Projects/hyprland-from-scratch
```

Inicializar Git:

```
git init
```

---

## 0.5.8 Crear README inicial

```
cat > README.md <<'EOF'
# Hyprland From Scratch

Guía paso a paso para construir un escritorio basado en Arch Linux y Hyprland desde cero.
EOF
```

---

## 0.5.9 Primer commit

```
git add .
git commit -m "Initial project structure"
```

---

## Estado esperado

* Acceso SSH funcionando

* Llave SSH generada

* Conexión con GitHub funcionando

* Git configurado

* Repositorio inicial creado

* Primer commit realizado

---

# Fase 1 — Hyprland mínimo

## Objetivo

Instalar únicamente lo necesario para iniciar una sesión Wayland con Hyprland y abrir una terminal.

No se instalarán barras, launchers, wallpapers ni temas.

---

## 1.1 Instalar Hyprland y componentes mínimos

```
sudo pacman -S \
  hyprland \
  xdg-desktop-portal-hyprland \
  xdg-desktop-portal \
  kitty
```

Qué instala cada paquete:

* `hyprland`: gestor de ventanas Wayland.

* `xdg-desktop-portal`: interfaz estándar para aplicaciones Wayland.

* `xdg-desktop-portal-hyprland`: implementación específica para Hyprland.

* `kitty`: terminal gráfica para la sesión.

---

## 1.2 Crear directorio de configuración

```
mkdir -p ~/.config/hypr
```

---

## 1.3 Crear configuración mínima

```
cat > ~/.config/hypr/hyprland.conf <<'EOF'
monitor=,preferred,auto,1

$terminal = kitty
$mainMod = SUPER

bind = $mainMod, RETURN, exec, $terminal
bind = $mainMod SHIFT, Q, killactive
bind = $mainMod, M, exit
EOF
```

Esta configuración permite:

* `Super + Enter` → abrir terminal

* `Super + Shift + Q` → cerrar ventana activa

* `Super + M` → salir de Hyprland

---

## 1.4 Iniciar Hyprland manualmente

Desde la TTY:

```
Hyprland
```

Si todo funciona aparecerá una pantalla vacía.

Presionar `Super + Enter` para abrir Kitty.

Salir con `Super + M`.

---

## 1.5 Verificaciones

Dentro de la terminal en Hyprland:

```
echo $XDG_SESSION_TYPE
```

Debe devolver:

```
wayland
```

Comprobar que el portal está instalado:

```
pacman -Q xdg-desktop-portal-hyprland
```

---

## Estado esperado

* Hyprland inicia desde TTY

* Se puede abrir Kitty con `Super + Enter`

* Se puede cerrar ventanas

* Se puede salir de la sesión

* La sesión reporta `wayland`

---

# Fase 2 — Configuración base de Hyprland

## Objetivo

Tener una configuración mínima pero cómoda para uso diario, manteniendo el sistema lo más limpio posible.

---

## 2.1 Crear copia de seguridad del archivo actual

```
cp ~/.config/hypr/hyprland.conf ~/.config/hypr/hyprland.conf.bak
```

---

## 2.2 Reemplazar por configuración base

```
cat > ~/.config/hypr/hyprland.conf <<'EOF'
################
### MONITOR ###
################

monitor=,preferred,auto,1

###############
### INPUT ###
###############

input {
    kb_layout = latam
    follow_mouse = 1
    touchpad {
        natural_scroll = false
    }
    sensitivity = 0
}

####################
### VARIABLES ###
####################

$terminal = kitty
$mainMod = SUPER

#################
### GENERAL ###
#################

general {
    gaps_in = 5
    gaps_out = 10
    border_size = 2
}

####################
### DECORATION ###
####################

decoration {
    rounding = 8
}

##################
### ANIMATIONS ###
##################

animations {
    enabled = true
}

################
### KEYBINDS ###
################

bind = $mainMod, RETURN, exec, $terminal
bind = $mainMod SHIFT, Q, killactive
bind = $mainMod, M, exit

# Workspaces
bind = $mainMod, 1, workspace, 1
bind = $mainMod, 2, workspace, 2
bind = $mainMod, 3, workspace, 3
bind = $mainMod, 4, workspace, 4
bind = $mainMod, 5, workspace, 5

# Mover ventanas a workspaces
bind = $mainMod SHIFT, 1, movetoworkspace, 1
bind = $mainMod SHIFT, 2, movetoworkspace, 2
bind = $mainMod SHIFT, 3, movetoworkspace, 3
bind = $mainMod SHIFT, 4, movetoworkspace, 4
bind = $mainMod SHIFT, 5, movetoworkspace, 5

# Ventana flotante
bind = $mainMod, F, togglefloating

# Recargar configuración
bind = $mainMod SHIFT, R, exec, hyprctl reload
EOF
```

---

## 2.3 Recargar configuración

Dentro de Hyprland:

```
hyprctl reload
```

O usar el atajo:

`Super + Shift + R`

---

## 2.4 Probar workspaces

* `Super + 1` → workspace 1

* `Super + 2` → workspace 2

* `Super + Shift + 2` → mover ventana al workspace 2

---

## 2.5 Probar ventana flotante

Abrir una terminal y presionar:

`Super + F`

La ventana debe alternar entre modo tiling y flotante.

---

## 2.6 Guardar configuración en el repositorio

Copiar el archivo al repositorio:

```
mkdir -p ~/Projects/hyprland-from-scratch/dotfiles/hypr
cp ~/.config/hypr/hyprland.conf ~/Projects/hyprland-from-scratch/dotfiles/hypr/
```

Realizar commit:

```
cd ~/Projects/hyprland-from-scratch
git add dotfiles/hypr/hyprland.conf
git commit -m "Add minimal Hyprland configuration"
```

---

## Estado esperado

* Hyprland inicia correctamente

* Teclado configurado en español latinoamericano

* Workspaces funcionando

* Movimiento de ventanas entre workspaces funcionando

* Ventanas flotantes funcionando

* Recarga de configuración funcionando

* Configuración guardada en el repositorio

---

# Resultado al finalizar la Fase 2

En este punto se dispone de:

* Arch Linux mínimo y actualizado.

* Acceso remoto por SSH.

* Repositorio Git inicializado.

* Hyprland funcional.

* Terminal gráfica integrada.

* Configuración base versionada.

* Workspaces y atajos esenciales configurados.

A partir de aquí el sistema ya es utilizable y las siguientes fases se centrarán en añadir componentes del escritorio (barra, launcher, notificaciones, wallpaper, bloqueo, etc.) evaluando cada uno antes de integrarlo definitivamente en la guía.
