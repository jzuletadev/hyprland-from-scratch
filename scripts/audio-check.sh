#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/audio-check.sh
#
# Red de seguridad contra una carrera de arranque de PipeWire.
#
#   EL SINTOMA: el portatil enciende sin sonido y sin microfono. El kernel ve la
#   tarjeta interna perfectamente, pero PipeWire no le asigna NINGUN perfil, asi que
#   no crea ni sink ni source para ella. Google Meet dice «microfono no encontrado» y
#   subir el volumen no hace nada.
#
#   LA CAUSA: WirePlumber arranca antes de que el sistema este listo. Medido en esta
#   maquina: WirePlumber a las 18:32:02 y UPower a las 18:32:05, tres segundos
#   despues, con WirePlumber quejandose de que UPower no existia. En esa ventana se
#   salta la tarjeta. Reiniciarlo despues la coge siempre.
#
#   LO QUE NO ERA: el error «SPA handle 'api.alsa.acp.device' could not be loaded» que
#   sale en el log es ruido — aparece tambien en los arranques que funcionan. Y
#   tampoco es el ahorro de energia de snd_hda_intel (`power_save=10`): se probó a
#   arrancar WirePlumber con la tarjeta dormida y la coge igual.
#
# Esto NO adivina la dependencia que falta: comprueba el RESULTADO. Y comprueba lo
# correcto — mirar «hay algun sink» no vale, porque en el arranque roto los sinks del
# HDMI si estaban; la que faltaba era la interna. Lo que delata el fallo es una
# tarjeta sin perfil activo.

set -uo pipefail

ESPERA=${1:-10}
notify() { command -v notify-send >/dev/null && notify-send -a "Audio" "$@"; }

# Tarjetas de ALSA que PipeWire conoce pero no ha llegado a configurar.
sin_perfil() {
    local tarjetas perfiles
    tarjetas=$(pactl list cards 2>/dev/null | grep -c 'Name: alsa_card') || return 1
    perfiles=$(pactl list cards 2>/dev/null | grep -c 'Active Profile:')
    [ "$tarjetas" -gt 0 ] && [ "$perfiles" -lt "$tarjetas" ]
}

sleep "$ESPERA"

# Sin PipeWire no hay nada que arreglar aqui.
pactl info >/dev/null 2>&1 || exit 0
sin_perfil || exit 0

echo "audio-check: una tarjeta se quedo sin perfil; reinicio wireplumber" >&2
systemctl --user restart wireplumber || exit 1
sleep 4

if sin_perfil; then
    notify -u critical "Sigue habiendo una tarjeta sin configurar"
    exit 1
fi
echo "audio-check: resuelto" >&2
