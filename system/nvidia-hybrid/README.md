# Portátil híbrido (iGPU AMD/Intel + NVIDIA)

Tres ficheros, y ninguno lo instala `install.sh`: se copian a mano.

| fichero | destino | para qué |
|---|---|---|
| `env-hyprland` | `~/.config/uwsm/env-hyprland` | qué tarjetas usa el compositor y en qué orden |
| `99-gpu-symlinks.rules` | `/etc/udev/rules.d/` | nombres estables para `DRI_PRIME` y para lanzar algo en la dedicada |
| `nvidia.conf` | `/etc/modprobe.d/` | opciones del módulo |

```bash
cp system/nvidia-hybrid/env-hyprland ~/.config/uwsm/env-hyprland
sudo cp system/nvidia-hybrid/99-gpu-symlinks.rules /etc/udev/rules.d/
sudo udevadm control --reload && sudo udevadm trigger --subsystem-match=drm
```

El de uwsm lo **sourcea** `prepare-env.sh` antes de arrancar Hyprland, así que es shell
de verdad, no una lista de `CLAVE=valor`.

## El fallo que hay que no repetir

Durante un tiempo las dos piezas identificaban las tarjetas por su **dirección PCI**
(`KERNELS=="0000:06:00.0"` para la AMD). Esa dirección **cambia entre arranques**.

Un día la integrada apareció en `05:00.0`. La regla dejó de casar, el enlace
`/dev/dri/amd-igpu` no se creó, y `AQ_DRM_DEVICES` quedó apuntando a un fichero que no
existía. Hyprland abrió **solo** la NVIDIA y, como el panel del portátil cuelga de la
AMD, esa pantalla **desapareció del escritorio**: se quedó congelada con el último
frame de SDDM —el «verificando»— mientras la sesión arrancaba en el monitor externo.

El síntoma engaña: parece que el login se ha colgado, y lo que pasa es que nadie está
dibujando en esa salida. Un `hyprctl monitors` que solo lista una pantalla, con
`/sys/class/drm/cardN-eDP-1/status` diciendo `connected`, es exactamente esto.

Ahora `env-hyprland` resuelve las tarjetas por su **DRIVER** al iniciar la sesión
—`amdgpu`/`i915`/`xe` contra `nvidia`—, que sí es estable, y la regla de udev busca por
**fabricante** (`0x1002`, `0x10de`) en vez de por dirección. La integrada va primero
porque es la que mueve el panel.

## Diagnóstico rápido

```bash
hyprctl monitors | grep ^Monitor          # ¿están todas las pantallas?
for c in /sys/class/drm/card*-*/; do echo "$(basename $c) $(cat $c/status)"; done
tr '\0' '\n' < /proc/$(pgrep -x Hyprland)/environ | grep AQ_DRM_DEVICES
ls -l /dev/dri/                            # ¿existen los enlaces a los que apunta?
```

Si `AQ_DRM_DEVICES` nombra una ruta que no está en `/dev/dri/`, esa tarjeta —y todas
sus pantallas— no existen para el compositor.
