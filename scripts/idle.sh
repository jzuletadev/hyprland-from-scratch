#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/idle.sh

set -uo pipefail

dry=false
[ "${!#}" = "--dry-run" ] && dry=true
run() { if $dry; then echo "would run: $*"; else "$@"; fi; }

on_battery() {   # true if there is a battery and no mains adapter is online
    local supply has_battery=false
    for supply in /sys/class/power_supply/*; do
        case "$(cat "$supply/type" 2>/dev/null)" in
            Mains)   [ "$(cat "$supply/online" 2>/dev/null)" = 1 ] && return 1 ;;
            Battery) has_battery=true ;;
        esac
    done
    $has_battery   # a desktop (no battery) counts as plugged in: never suspend
}

case "${1:-}" in
    dim)
        if on_battery; then
            current=$(brightnessctl get)
            run brightnessctl -q -s set $(( current * 40 / 100 ))
        else
            $dry && echo "plugged in: not dimming"
        fi
        ;;
    lock)
        if on_battery; then
            run loginctl lock-session
        else
            $dry && echo "plugged in: not locking"
        fi
        ;;
    dpms-off)
        if on_battery; then
            run hyprctl dispatch dpms off
        else
            $dry && echo "plugged in: not turning screen off"
        fi
        ;;
    suspend)
        if on_battery; then
            run systemctl suspend
        else
            $dry && echo "plugged in: not suspending"
        fi
        ;;
    *)
        echo "usage: idle.sh dim|lock|dpms-off|suspend [--dry-run]" >&2
        exit 2
        ;;
esac