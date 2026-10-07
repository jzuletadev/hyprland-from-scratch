#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/minimize.sh
#
# Phase 31. "Minimize" for Hyprland, which has no such concept: windows are
# parked on a hidden special workspace, special:minimized, and brought back to
# whatever workspace is active when restoring.
#
#   minimize.sh window    the focused window             SUPER+N
#   minimize.sh all       every window on this workspace  3 fingers down
#   minimize.sh restore   all minimized windows, here     3 fingers up, SUPER+SHIFT+N
#   minimize.sh toggle    all if there's something here, restore otherwise (SUPER+D)

PARKING="special:minimized"

active_ws() { hyprctl activeworkspace -j | python3 -c 'import json,sys; print(json.load(sys.stdin)["id"])'; }

addresses() {   # addresses <workspace name or id>: the windows on it
    hyprctl clients -j | python3 -c '
import json, sys
target = sys.argv[1]
for c in json.load(sys.stdin):
    if target in (str(c["workspace"]["id"]), c["workspace"]["name"]):
        print(c["address"])' "$1"
}

move_all() {   # move_all <from> <to>: one batch, so it's a single redraw
    local batch="" address
    for address in $(addresses "$1"); do
        batch+="dispatch movetoworkspacesilent $2,address:$address;"
    done
    [ -n "$batch" ] && hyprctl --batch "$batch" >/dev/null
}

case "${1:-}" in
    window)  hyprctl dispatch movetoworkspacesilent "$PARKING" >/dev/null ;;
    all)     move_all "$(active_ws)" "$PARKING" ;;
    restore) move_all "$PARKING" "$(active_ws)" ;;
    toggle)
        here=$(active_ws)
        if [ -n "$(addresses "$here")" ]; then move_all "$here" "$PARKING"; else move_all "$PARKING" "$here"; fi ;;
    *)       echo "usage: minimize.sh window|all|restore|toggle" >&2; exit 2 ;;
esac
