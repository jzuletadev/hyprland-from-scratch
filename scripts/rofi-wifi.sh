#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-wifi.sh
#
# Phase 11. Wi-Fi picker for Rofi's dmenu mode, driven by nmcli (already
# installed as part of NetworkManager). No dedicated network-manager applet.

set -euo pipefail

THEME="$HOME/hyprland-from-scratch/dotfiles/rofi/config.rasi"

mapfile -t networks < <(
    nmcli -t -f IN-USE,SIGNAL,SSID device wifi list |
    awk -F: '$3 != "" {printf "%s %3s%%  %s\n", ($1=="*" ? "*" : " "), $2, $3}' |
    awk '!seen[$0]++'
)

choice=$(printf '%s\n' "${networks[@]}" | rofi -dmenu -i -p "Wi-Fi" -theme "$THEME")
[ -z "$choice" ] && exit 0

ssid=$(echo "$choice" | sed -E 's/^.{8}//')

if nmcli -t -f NAME connection show | grep -qx "$ssid"; then
    nmcli connection up "$ssid" && notify-send "Wi-Fi" "Connected to $ssid"
    exit 0
fi

security=$(nmcli -t -f SSID,SECURITY device wifi list | awk -F: -v s="$ssid" '$1==s {print $2; exit}')

if [ -z "$security" ] || [ "$security" = "--" ]; then
    nmcli device wifi connect "$ssid" && notify-send "Wi-Fi" "Connected to $ssid"
else
    password=$(rofi -dmenu -password -p "Password for $ssid" -theme "$THEME")
    [ -z "$password" ] && exit 0
    if nmcli device wifi connect "$ssid" password "$password"; then
        notify-send "Wi-Fi" "Connected to $ssid"
    else
        notify-send -u critical "Wi-Fi" "Failed to connect to $ssid"
    fi
fi
