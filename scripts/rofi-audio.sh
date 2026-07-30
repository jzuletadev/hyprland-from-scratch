#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-audio.sh
#
# Phase 11. Audio output picker + mute toggle for Rofi's dmenu mode, driven
# by wpctl (wireplumber's CLI, already the running audio session manager).

set -euo pipefail

THEME="$HOME/hyprland-from-scratch/dotfiles/rofi/config.rasi"

mapfile -t sinks < <(
    wpctl status | python3 -c "
import sys, re
lines = sys.stdin.read().splitlines()
in_sinks = False
for line in lines:
    if '├─ Sinks:' in line:
        in_sinks = True
        continue
    if in_sinks and ('├─ Sources:' in line or '├─ Filters:' in line):
        break
    if in_sinks:
        m = re.search(r'(\*)?\s*(\d+)\.\s+(.+?)\s*\[vol:', line)
        if m:
            active, sid, name = m.groups()
            print(f\"{sid}\t{'[active]  ' if active else '[ ]       '}{name.strip()}\")
"
)

declare -A id_by_label
labels=()
for entry in "${sinks[@]}"; do
    id="${entry%%$'\t'*}"
    label="${entry#*$'\t'}"
    labels+=("$label")
    id_by_label["$label"]="$id"
done
labels+=("Toggle mute")

choice=$(printf '%s\n' "${labels[@]}" | rofi -dmenu -i -p "Audio output" -theme "$THEME")
[ -z "$choice" ] && exit 0

if [ "$choice" = "Toggle mute" ]; then
    wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
    exit 0
fi

id="${id_by_label[$choice]:-}"
[ -z "$id" ] && exit 0
wpctl set-default "$id"
notify-send "Audio" "Output: ${choice#*  }"
