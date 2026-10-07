#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/volume.sh
#
# Phase 34. Volume change with an on-screen indicator: a notification with a
# progress bar that replaces itself (dunst stack tag), so every step shows at
# once instead of piling up.
#
#   volume.sh up [step]     louder by step % (default 5), capped at 100%
#   volume.sh down [step]   quieter by step %
#   volume.sh mute          toggle mute
#
# The 4-finger touchpad gestures use a 10% step: a gesture fires once per
# swipe, so 5% made it take several swipes to hear a difference.

step=${2:-5}
sink=@DEFAULT_AUDIO_SINK@

case "${1:-}" in
    up)   wpctl set-volume -l 1.0 "$sink" "${step}%+" ;;
    down) wpctl set-volume "$sink" "${step}%-" ;;
    mute) wpctl set-mute "$sink" toggle ;;
    *)    echo "usage: volume.sh up|down [step] | mute" >&2; exit 2 ;;
esac

# "Volume: 0.45" or "Volume: 0.45 [MUTED]"
state=$(wpctl get-volume "$sink")
percent=$(awk '{ printf "%d", $2 * 100 + 0.5 }' <<<"$state")
label="$percent%"
[[ "$state" == *MUTED* ]] && label="Muted ($percent%)"

notify-send -a Volume -t 1200 \
    -h string:x-dunst-stack-tag:volume \
    -h "int:value:$percent" \
    "Volume" "$label"
