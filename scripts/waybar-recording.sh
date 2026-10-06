#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/waybar-recording.sh
#
# Phase 19 (button since Phase 22). Status for Waybar's custom/recording module
# (polled every second, and refreshed at once by screen-record.sh through
# signal 9): a camera button while idle (click = recording menu), a red
# "REC mm:ss" pill while wf-recorder runs (click = stop and save).
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see README Phase 11).

I_REC=$'\uf111'
I_CAMERA=$'\uf03d'

pid=$(pgrep -x -o wf-recorder) || {
    printf '{"text":"%s","class":"idle","tooltip":"Screen recording\\nClick or SUPER+SHIFT+R: start"}\n' "$I_CAMERA"
    exit 0
}
secs=$(ps -o etimes= -p "$pid" | tr -d ' ')
printf '{"text":"%s  REC %02d:%02d","class":"recording","tooltip":"Recording\\nClick or SUPER+SHIFT+R: stop and save"}\n' \
    "$I_REC" $((secs / 60)) $((secs % 60))
