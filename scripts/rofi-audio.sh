#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-audio.sh
#
# Phase 11, extended in Phase 16. Audio menu for Rofi's dmenu mode, driven by
# wpctl (wireplumber's CLI, already the running audio session manager):
# set the volume (Phase 20, via rofi-volume.sh), pick the output device,
# mute/unmute the output and the microphone, and open pavucontrol (per-app
# volumes, input devices) when it's installed.
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"
rofi_toggle "-p Audio " "-show volume"   # same icon clicked again = close; another menu open = switch

THEME="$HOME/.config/rofi/menu.rasi"

I_ACTIVE=$'\uf00c'
I_OUTPUT=$'\uf028'
I_MUTED=$'\uf026'
I_MIC=$'\uf130'
I_MIC_OFF=$'\uf131'
I_MIXER=$'\uf1de'
I_VOLUME=$'\uf027'

menu()   { rofi -dmenu -i -no-custom -p "$1" -theme "$THEME"; }
notify() { notify-send -a Audio "Audio" "$1"; }

declare -A CMD ARG
ENTRIES=()
add() { ENTRIES+=("$1"); CMD["$1"]=$2; ARG["$1"]=${3:-}; }

# ---- volume: opens rofi-volume.sh (script mode: stays open while adjusting) ----
add "$I_VOLUME  Volume: $(wpctl get-volume @DEFAULT_AUDIO_SINK@ | awk '{ printf "%d", $2 * 100 + 0.5 }')%" volume

# ---- outputs: "id<TAB>*<TAB>name" per sink, parsed from `wpctl status` ----
while IFS=$'\t' read -r id active name; do
    # drop the chipset noise: "GA106 High Definition Audio Controller HDMI / DisplayPort 1 Output"
    name=${name/ High Definition Audio Controller/}
    name=${name/ HD Audio Controller/}
    name=${name% (Stereo)}
    if [ "$active" = "*" ]; then
        add "$I_ACTIVE  $name" noop
    else
        add "$I_OUTPUT  $name" default "$id"
    fi
done < <(wpctl status | python3 -c '
import sys, re
in_sinks = False
for line in sys.stdin.read().splitlines():
    if "├─ Sinks:" in line:
        in_sinks = True
        continue
    if in_sinks and ("├─ Sources:" in line or "├─ Filters:" in line):
        break
    if in_sinks:
        m = re.search(r"(\*)?\s*(\d+)\.\s+(.+?)\s*\[vol:", line)
        if m:
            active, sid, name = m.groups()
            mark = active or "-"
            print(f"{sid}\t{mark}\t{name.strip()}")
')

# ---- mute toggles (labels show what selecting them will do) ----
if [[ "$(wpctl get-volume @DEFAULT_AUDIO_SINK@)" == *MUTED* ]]; then
    add "$I_OUTPUT  Unmute output" mute-output
else
    add "$I_MUTED  Mute output" mute-output
fi
if [[ "$(wpctl get-volume @DEFAULT_AUDIO_SOURCE@ 2>/dev/null)" == *MUTED* ]]; then
    add "$I_MIC  Unmute microphone" mute-mic
else
    add "$I_MIC_OFF  Mute microphone" mute-mic
fi
command -v pavucontrol >/dev/null && add "$I_MIXER  Mixer (per-app volume)" mixer

choice=$(printf '%s\n' "${ENTRIES[@]}" | menu "Audio") || exit 0
case "${CMD[$choice]}" in
    volume)
        exec rofi -show volume -modi "volume:$HOME/hyprland-from-scratch/scripts/rofi-volume.sh" \
                  -theme "$MENU_THEME" ;;
    default)     wpctl set-default "${ARG[$choice]}" && notify "Output: ${choice#*  }" ;;
    mute-output) wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle ;;
    mute-mic)
        wpctl set-mute @DEFAULT_AUDIO_SOURCE@ toggle
        if [[ "$(wpctl get-volume @DEFAULT_AUDIO_SOURCE@)" == *MUTED* ]]; then
            notify "Microphone muted"
        else
            notify "Microphone on"
        fi ;;
    mixer)       hyprctl dispatch exec "[float;size 900 600;center] pavucontrol" >/dev/null ;;
esac
exit 0
