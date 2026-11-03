#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/audio-check.sh

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
    notify -u critical "A sound card is still unconfigured"
    exit 1
fi
echo "audio-check: resuelto" >&2
