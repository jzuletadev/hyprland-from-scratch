#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-click-outside.sh
#
# Phase 17. Bound to every left click in hyprland.conf with `bindn`
# (non-consuming: the click still reaches whatever is under the cursor).
# Closes an open Rofi menu when the click lands outside it.
#
# Rofi's own click-to-exit can't work on Wayland: its layer surface only covers
# the menu itself, so clicks anywhere else never reach it.

pgrep -x rofi >/dev/null || exit 0   # nothing open: the usual case, ends here

IFS=', ' read -r cx cy < <(hyprctl cursorpos)   # "2710, 400"

over() {   # over <namespace>: is the cursor on a layer surface with that namespace?
    # hyprctl layers: "Layer 55d4...: xywh: 2592 312 576 492, a: 1, namespace: rofi, pid: 1234"
    hyprctl layers | awk -v ns="$1" -v cx="$cx" -v cy="$cy" '
        index($0, "namespace: " ns ",") {
            split($0, part, "xywh: ")
            split(part[2], g, /[ ,]+/)
            if (cx >= g[1] && cx < g[1] + g[3] && cy >= g[2] && cy < g[2] + g[4]) hit = 1
        }
        END { exit !hit }'
}

over rofi && exit 0     # a click inside the menu
over waybar && exit 0   # bar icons open/switch/close menus themselves (rofi-common.sh)
pkill -x rofi
