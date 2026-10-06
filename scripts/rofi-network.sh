#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-network.sh
#
# Phase 11 (as rofi-wifi.sh), extended in Phase 16. Network menu for Rofi's
# dmenu mode, driven by nmcli (NetworkManager): status of each adapter, Wi-Fi
# on/off and rescan, one row per Wi-Fi network (strongest signal wins),
# connect/disconnect with a password prompt, and nmtui for everything else
# (wired settings, static IPs, VPN, forgetting saved networks).
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see README Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"
rofi_toggle "-p Network "   # same icon clicked again = close; another menu open = switch

THEME="$HOME/hyprland-from-scratch/dotfiles/rofi/menu.rasi"

I_WIRED=$'\uf108'
I_WIFI=$'\uf1eb'
I_LOCK=$'\uf023'
I_ACTIVE=$'\uf00c'
I_POWER=$'\uf011'
I_RESCAN=$'\uf021'
I_SETTINGS=$'\uf013'

menu()   { rofi -dmenu -i -no-custom -p "$1" -theme "$THEME"; }
notify() { notify-send -a Network "Network" "$1"; }

declare -A CMD ARG SIGNAL SECURE INUSE
ENTRIES=()
add() { ENTRIES+=("$1"); CMD["$1"]=$2; ARG["$1"]=${3:-}; }   # add <label> <command> [ssid]

settings() {   # nmtui in a floating kitty, same inline-rule style as the btop bind
    hyprctl dispatch exec "[float;size 900 600;center] kitty --class nmtui-floating -e nmtui" >/dev/null
}

connect() {
    local ssid=$1 saved pw
    saved=$(nmcli -t -f NAME connection show)
    if grep -qxF -- "$ssid" <<<"$saved"; then
        if nmcli connection up id "$ssid" >/dev/null 2>&1; then
            notify "Connected to $ssid"
        else
            notify "Couldn't connect to $ssid"
        fi
        return
    fi
    if [ -n "${SECURE[$ssid]:-}" ]; then
        pw=$(rofi -dmenu -password -p "$ssid" -theme "$THEME" \
                  -theme-str 'entry { placeholder: "Password"; }' </dev/null) || return
        [ -z "$pw" ] && return
        if nmcli device wifi connect "$ssid" password "$pw" >/dev/null 2>&1; then
            notify "Connected to $ssid"
        else
            # a failed attempt leaves a saved profile with the wrong password,
            # which the next attempt would reuse instead of asking again
            nmcli connection delete id "$ssid" >/dev/null 2>&1
            notify "Couldn't connect to $ssid. Wrong password?"
        fi
    else
        if nmcli device wifi connect "$ssid" >/dev/null 2>&1; then
            notify "Connected to $ssid"
        else
            notify "Couldn't connect to $ssid"
        fi
    fi
}

run() {
    local cmd=$1 ssid=$2
    case "$cmd" in
        connect)    connect "$ssid" ;;
        disconnect) nmcli connection down id "$ssid" >/dev/null 2>&1 && notify "Disconnected from $ssid" ;;
        wifi-on)    nmcli radio wifi on && notify "Wi-Fi on" ;;
        wifi-off)   nmcli radio wifi off && notify "Wi-Fi off" ;;
        rescan)     nmcli device wifi rescan >/dev/null 2>&1; sleep 2; exec "$0" ;;
        settings)   settings ;;
    esac
}

# ---- adapters (selecting one opens nmtui) ----
while IFS=: read -r dev type state conn; do
    case "$type" in
        ethernet) add "$I_WIRED  Wired ($dev): $state${conn:+ - $conn}" settings ;;
        wifi)     add "$I_WIFI  Wi-Fi ($dev): $state${conn:+ - $conn}" settings ;;
    esac
done < <(nmcli -t -f DEVICE,TYPE,STATE,CONNECTION device status)

# ---- Wi-Fi networks ----
wifi_on=false
[ "$(nmcli radio wifi)" = "enabled" ] && wifi_on=true

if $wifi_on; then
    while IFS=: read -r inuse signal security ssid; do
        ssid=${ssid//\\:/:}   # nmcli -t escapes ":" inside values
        [ -z "$ssid" ] && continue
        if [ -z "${SIGNAL[$ssid]:-}" ] || [ "$signal" -gt "${SIGNAL[$ssid]}" ]; then
            SIGNAL[$ssid]=$signal
        fi
        [ "$inuse" = "*" ] && INUSE[$ssid]=1
        [ -n "$security" ] && [ "$security" != "--" ] && SECURE[$ssid]=1
    done < <(nmcli -t -f IN-USE,SIGNAL,SECURITY,SSID device wifi list)

    while IFS=$'\t' read -r signal ssid; do
        if [ -n "${INUSE[$ssid]:-}" ]; then
            add "$I_ACTIVE  $ssid  $signal%  (connected)" disconnect "$ssid"
        elif [ -n "${SECURE[$ssid]:-}" ]; then
            add "$I_LOCK  $ssid  $signal%" connect "$ssid"
        else
            add "$I_WIFI  $ssid  $signal%" connect "$ssid"
        fi
    done < <(for s in "${!SIGNAL[@]}"; do printf '%s\t%s\n' "${SIGNAL[$s]}" "$s"; done | sort -rn)

    add "$I_RESCAN  Rescan Wi-Fi" rescan
    add "$I_POWER  Turn Wi-Fi off" wifi-off
else
    add "$I_POWER  Turn Wi-Fi on" wifi-on
fi
add "$I_SETTINGS  Advanced settings (nmtui)" settings

choice=$(printf '%s\n' "${ENTRIES[@]}" | menu "Network") || exit 0
run "${CMD[$choice]}" "${ARG[$choice]}"
