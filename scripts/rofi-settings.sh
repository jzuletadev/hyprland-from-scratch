#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-settings.sh
#
# Phase 27. Settings menu (right-click the Arch logo in Waybar):
#   Apps theme        dark / light (scripts/theme.sh)
#   Window border     color and width, written into dotfiles/hypr/appearance.conf
#                     (Hyprland reloads by itself when the file changes)
#   Wallpaper...      scripts/wallpaper.sh
#   Keybindings...    scripts/rofi-keybinds.py
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"
rofi_toggle "-p Settings " "-p Border color " "-p Border width "

SCRIPTS="$HOME/hyprland-from-scratch/scripts"
APPEARANCE="$HOME/.config/hypr/appearance.conf"

I_THEME=$'\uf186'
I_COLOR=$'\uf1fc'
I_WIDTH=$'\uf0c8'
I_WALLPAPER=$'\uf03e'
I_KEYS=$'\uf11c'
I_ACTIVE=$'\uf00c'
I_OPTION=$'\uf111'

menu()   { rofi -dmenu -i -no-custom -p "$1" -theme "$MENU_THEME"; }
notify() { notify-send -a Settings "Settings" "$1"; }

# border color presets (name|value); the last two hex digits are the opacity
COLORS=(
    "Arch blue|rgba(1793d1cc)"
    "Light blue|rgba(5db3dfcc)"
    "Soft white|rgba(e0e0e080)"
    "Grey|rgba(6e6e6ecc)"
    "Amber|rgba(e0a84ecc)"
    "Invisible|rgba(00000000)"
)

value_of() { sed -n "s/^\$$1 = //p" "$APPEARANCE"; }          # value_of border_size
set_value() { sed -i "s|^\$$1 = .*|\$$1 = $2|" "$APPEARANCE"; } # set_value border_size 2

color_name() {   # the preset name for the current active-border color, or the raw value
    local entry current
    current=$(value_of border_active)
    for entry in "${COLORS[@]}"; do
        [ "${entry#*|}" = "$current" ] && { echo "${entry%%|*}"; return; }
    done
    echo "$current"
}

pick_color() {
    local entry current choice entries=()
    current=$(value_of border_active)
    for entry in "${COLORS[@]}"; do
        if [ "${entry#*|}" = "$current" ]; then
            entries+=("$I_ACTIVE  ${entry%%|*}")
        else
            entries+=("$I_OPTION  ${entry%%|*}")
        fi
    done
    choice=$(printf '%s\n' "${entries[@]}" | menu "Border color") || return
    for entry in "${COLORS[@]}"; do
        if [ "${entry%%|*}" = "${choice#*  }" ]; then
            set_value border_active "${entry#*|}"
            notify "Window border: ${entry%%|*}"
        fi
    done
}

pick_width() {
    local px current choice entries=()
    current=$(value_of border_size)
    for px in 0 1 2 3; do
        if [ "$px" = "$current" ]; then entries+=("$I_ACTIVE  $px px"); else entries+=("$I_OPTION  $px px"); fi
    done
    choice=$(printf '%s\n' "${entries[@]}" | menu "Border width") || return
    px=${choice#*  }
    set_value border_size "${px% px}"
    notify "Window border: ${px}"
}

theme=$("$SCRIPTS/theme.sh" get)
declare -A ACTION
ENTRIES=()
add() { ENTRIES+=("$1"); ACTION["$1"]=$2; }
add "$I_THEME  Apps theme: ${theme^} (switch)"                  theme
add "$I_COLOR  Window border color: $(color_name)"               color
add "$I_WIDTH  Window border width: $(value_of border_size) px"  width
add "$I_WALLPAPER  Wallpaper..."                                  wallpaper
add "$I_KEYS  Keybindings..."                                     keys

choice=$(printf '%s\n' "${ENTRIES[@]}" | menu "Settings") || exit 0
case "${ACTION[$choice]}" in
    theme)     "$SCRIPTS/theme.sh" toggle && notify "Apps theme: $("$SCRIPTS/theme.sh" get)" ;;
    color)     pick_color ;;
    width)     pick_width ;;
    wallpaper) exec "$SCRIPTS/wallpaper.sh" ;;
    keys)      exec "$SCRIPTS/rofi-keybinds.py" ;;
esac
exit 0
