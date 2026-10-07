#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/screen-record.sh
#
# Phase 19. Screen recording with wf-recorder (captures through the
# compositor's screencopy protocol, which Hyprland provides).
#
#   screen-record.sh         not recording: menu (screen or region, with or
#                            without desktop audio); recording: stop and save
#   screen-record.sh stop    stop and save (Waybar's red REC button)
#
# SUPER+SHIFT+R runs it. Files go to ~/Videos/recordings/<date>_<time>.mp4.
# Encoding is CPU x264 "veryfast": fine for 1080p on this Ryzen, and it avoids
# picking between the AMD and NVIDIA GPUs for hardware encoding.
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"

DIR="$HOME/Videos/recordings"
CURRENT="${XDG_RUNTIME_DIR:-/tmp}/screen-record.path"   # file being written

I_SCREEN=$'\uf108'
I_REGION=$'\uf125'
I_AUDIO=$'\uf028'

notify()    { notify-send -a "Screen recording" "Screen recording" "$1"; }
recording() { pgrep -x wf-recorder >/dev/null; }
refresh()   { pkill -RTMIN+9 -x waybar; }   # redraw the REC indicator now (signal 9)

stop() {
    recording || return 0
    pkill -INT -x wf-recorder   # SIGINT = finish writing the file properly
    while recording; do sleep 0.1; done
    refresh
    notify "Saved to ~/${CURRENT_FILE#"$HOME"/}"
}

start() {   # start <screen|region> <audio: yes|no>
    local file geo args=()
    command -v wf-recorder >/dev/null || {
        notify "wf-recorder isn't installed: sudo pacman -S wf-recorder"
        exit 1
    }
    mkdir -p "$DIR"
    file="$DIR/$(date +%Y-%m-%d_%H-%M-%S).mp4"
    args=(-f "$file" -p preset=veryfast -p crf=23)
    if [ "$1" = region ]; then
        geo=$(slurp) || exit 0   # Escape in slurp = cancel
        args+=(-g "$geo")
    else
        args+=(-o "$(hyprctl monitors -j | python3 -c '
import json, sys
print(next(m["name"] for m in json.load(sys.stdin) if m["focused"]))')")
    fi
    # desktop sound = the "monitor" of the default output device
    [ "$2" = yes ] && args+=(--audio="$(pactl get-default-sink).monitor")

    printf '%s' "$file" > "$CURRENT"
    setsid -f wf-recorder "${args[@]}" >/dev/null 2>&1   # detached: outlives this script
    sleep 0.5
    if recording; then
        refresh
    else
        notify "Recording didn't start (wf-recorder exited)"
    fi
}

CURRENT_FILE=$(cat "$CURRENT" 2>/dev/null)

if [ "${1:-}" = stop ] || recording; then
    stop
    exit 0
fi

rofi_toggle "-p Record "
declare -A CMD
ENTRIES=()
add() { ENTRIES+=("$1"); CMD["$1"]=$2; }
add "$I_SCREEN  Record screen"               "screen no"
add "$I_AUDIO  Record screen with audio"     "screen yes"
add "$I_REGION  Record region"               "region no"
add "$I_AUDIO  Record region with audio"     "region yes"

choice=$(printf '%s\n' "${ENTRIES[@]}" | rofi -dmenu -i -no-custom -p "Record" -theme "$MENU_THEME") || exit 0
start ${CMD[$choice]}
