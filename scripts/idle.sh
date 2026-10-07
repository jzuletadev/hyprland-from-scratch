#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/idle.sh
#
# Phase 24. Actions behind hypridle's listeners (dotfiles/hypr/hypridle.conf):
#
#   idle.sh dim       laptop panel to 40% of its current brightness (saved
#                     first, so `brightnessctl -r` restores it on activity)
#   idle.sh suspend   suspend, but only when running on battery
#
# --dry-run (as the last argument) prints the decision instead of acting.

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
        # The upstream example uses `brightnessctl -s set 10`: a raw value, which
        # on this panel (max 65535) is ~0% — practically black, not dimmed.
        current=$(brightnessctl get)
        run brightnessctl -q -s set $(( current * 40 / 100 ))
        ;;
    suspend)
        if on_battery; then
            run systemctl suspend
        else
            $dry && echo "plugged in: not suspending"
        fi
        ;;
    *)
        echo "usage: idle.sh dim|suspend [--dry-run]" >&2
        exit 2
        ;;
esac
