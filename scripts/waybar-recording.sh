#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/waybar-recording.sh

I_REC=$'\uf111'
I_CAMERA=$'\uf03d'

pid=$(pgrep -x -o wf-recorder) || {
    # printf '{"text":"%s","class":"idle","tooltip":"Screen recording\\nClick or SUPER+SHIFT+R: start"}\n' "$I_CAMERA"
    exit 0
}

secs=$(ps -o etimes= -p "$pid" | tr -d ' ')
printf '{"text":"%s  REC %02d:%02d","class":"recording","tooltip":"Recording\\nClick or SUPER+SHIFT+R: stop and save"}\n' \
    "$I_REC" $((secs / 60)) $((secs % 60))
