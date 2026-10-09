#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-click-outside.sh
#
# Phase 17. Bound to every left click in hyprland.conf with `bindn`
# (non-consuming: the click still reaches whatever is under the cursor).
# Closes an open Rofi menu when the click lands outside it, and (Phase 41) the
# system info window of scripts/sysinfo.sh the same way.
#
# Rofi's own click-to-exit can't work on Wayland: its layer surface only covers
# the menu itself, so clicks anywhere else never reach it. The info window is a
# plain kitty window: nothing closes it on its own either.

rofi_open=$(pgrep -x rofi)
# hyprctl clients: "Window 55d4... -> title:", then tab-indented "at: 9,45",
# "size: 820,460", "class: sysinfo". Prints "x y w h" if the info window is open.
sysinfo=$(hyprctl clients | awk '
    /^Window /  { at = size = "" }
    /^\tat: /   { at = $2 }
    /^\tsize: / { size = $2 }
    $0 == "\tclass: sysinfo" { gsub(",", " ", at); gsub(",", " ", size); print at, size; exit }')
[ -n "$rofi_open$sysinfo" ] || exit 0   # nothing open: the usual case, ends here

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

# bar icons open/switch/close menus themselves (rofi-common.sh), and the Arch
# logo toggles the info window: closing it here too would let sysinfo.sh find
# it gone and open it again
over waybar && exit 0

if [ -n "$rofi_open" ] && ! over rofi; then
    pkill -x rofi
fi
if [ -n "$sysinfo" ]; then
    read -r x y w h <<< "$sysinfo"
    (( cx >= x && cx < x + w && cy >= y && cy < y + h )) ||
        hyprctl dispatch closewindow class:sysinfo >/dev/null
fi
