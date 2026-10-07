#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-volume.sh
#
# Phase 20. Volume control for Rofi's *script* mode, opened from the audio
# menu's "Volume" entry:
#
#   rofi -show volume -modi "volume:<this script>"
#
# Unlike the dmenu menus, script mode keeps Rofi open between clicks: Rofi
# runs this script again after each pick (ROFI_RETV=1, the row's hidden
# "info" in ROFI_INFO) and redraws with its new output. So Louder/Quieter can
# be clicked repeatedly and the prompt shows the level live; a preset applies
# and closes (no output = Rofi exits).
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

I_UP=$'\uf028'
I_DOWN=$'\uf027'
I_LEVEL=$'\uf026'
I_ACTIVE=$'\uf00c'

volume() {   # current output volume as an integer percent ("Volume: 0.40" -> 40)
    wpctl get-volume @DEFAULT_AUDIO_SINK@ | awk '{ printf "%d", $2 * 100 + 0.5 }'
}

if [ "${ROFI_RETV:-0}" = 1 ]; then
    case "${ROFI_INFO:-}" in
        up)    wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+ ;;
        down)  wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%- ;;
        set:*) wpctl set-volume @DEFAULT_AUDIO_SINK@ "${ROFI_INFO#set:}%"; exit 0 ;;
    esac
fi

v=$(volume)
muted=""
[[ "$(wpctl get-volume @DEFAULT_AUDIO_SINK@)" == *MUTED* ]] && muted=" (muted)"

printf '\0prompt\x1fVolume %s%%%s\n' "$v" "$muted"
printf '\0keep-selection\x1ftrue\n'   # stay on Louder/Quieter after each click
printf '%s  Louder (+5%%)\0info\x1fup\n' "$I_UP"
printf '%s  Quieter (-5%%)\0info\x1fdown\n' "$I_DOWN"
for p in 100 75 50 25 10 0; do
    icon=$I_LEVEL
    [ "$p" = "$v" ] && icon=$I_ACTIVE
    printf '%s  %s%%\0info\x1fset:%s\n' "$icon" "$p" "$p"
done
