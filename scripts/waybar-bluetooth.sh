#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/waybar-bluetooth.sh
#
# Phase 16. Status for Waybar's custom/bluetooth module (polled every 5s).
# Prints one JSON line: class "off" / "on" (nothing connected) / "connected",
# with the connected device names in the tooltip.
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see README Phase 11).

I_ON=$'\uf293'
I_OFF=$'\uf127'

json() {   # json <text> <class> <tooltip>
    local t=${3//\\/\\\\}
    t=${t//\"/\\\"}
    t=${t//$'\n'/\\n}
    printf '{"text":"%s","class":"%s","tooltip":"%s"}\n' "$1" "$2" "$t"
}

if [[ "$(bluetoothctl show 2>/dev/null)" != *"Powered: yes"* ]]; then
    json "$I_OFF" off "Bluetooth off"$'\n'"Click: Bluetooth menu"
    exit 0
fi

names=$(bluetoothctl devices Connected 2>/dev/null | cut -d' ' -f3-)
if [ -n "$names" ]; then
    json "$I_ON" connected "Connected:"$'\n'"$names"$'\n'"Click: Bluetooth menu"
else
    json "$I_ON" on "Bluetooth on, nothing connected"$'\n'"Click: Bluetooth menu"
fi
