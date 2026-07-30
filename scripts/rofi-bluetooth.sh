#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-bluetooth.sh
#
# Phase 11. Bluetooth device picker for Rofi's dmenu mode, driven by
# bluetoothctl (bluez, already installed and running).

set -euo pipefail

THEME="$HOME/hyprland-from-scratch/dotfiles/rofi/config.rasi"

declare -A mac_by_label
labels=()

while read -r _ mac name; do
    [ -z "$mac" ] && continue
    if bluetoothctl info "$mac" | grep -q "Connected: yes"; then
        label="[connected]  $name"
    else
        label="[paired]     $name"
    fi
    labels+=("$label")
    mac_by_label["$label"]="$mac"
done < <(bluetoothctl devices Paired)

labels+=("Power: toggle")

choice=$(printf '%s\n' "${labels[@]}" | rofi -dmenu -i -p "Bluetooth" -theme "$THEME")
[ -z "$choice" ] && exit 0

if [ "$choice" = "Power: toggle" ]; then
    if bluetoothctl show | grep -q "Powered: yes"; then
        bluetoothctl power off && notify-send "Bluetooth" "Powered off"
    else
        bluetoothctl power on && notify-send "Bluetooth" "Powered on"
    fi
    exit 0
fi

mac="${mac_by_label[$choice]:-}"
[ -z "$mac" ] && exit 0

if bluetoothctl info "$mac" | grep -q "Connected: yes"; then
    bluetoothctl disconnect "$mac" && notify-send "Bluetooth" "Disconnected"
else
    bluetoothctl connect "$mac" && notify-send "Bluetooth" "Connected"
fi
