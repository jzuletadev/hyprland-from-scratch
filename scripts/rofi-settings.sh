#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-settings.sh
#
# Phase 27. Settings menu (right-click the Arch logo in Waybar):
#   Apps theme        dark / light (scripts/theme.sh)
#   Accent color      menus, bar, notifications and lock screen: scripts/accent.sh
#                     (Phase 45; menus only in Phase 43)
#   Window border     color, written into ~/.config/hypr/appearance.conf
#                     (Hyprland reloads by itself when the file changes)
#                     Both color pickers offer the same palette (Phase 44).
#   Border width      windows and menus alike: both files above
#   Wallpaper...      scripts/wallpaper.sh
#   Keybindings...    scripts/rofi-keybinds.py
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"
rofi_toggle "-p Settings " "-p Accent color " "-p Border color " "-p Border width "

SCRIPTS="$HOME/hyprland-from-scratch/scripts"
APPEARANCE="$HOME/.config/hypr/appearance.conf"
RASI="$HOME/.config/rofi/appearance.rasi"

I_THEME=$'\uf186'
I_ACCENT=$'\ue22b'
I_COLOR=$'\uf1fc'
I_WIDTH=$'\uf0c8'
I_WALLPAPER=$'\uf03e'
I_KEYS=$'\uf11c'
I_ACTIVE=$'\uf00c'
I_OPTION=$'\uf111'   # filled dot
I_NONE=$'\uf10c'     # hollow dot: Invisible

menu()   { rofi -dmenu -i -no-custom -p "$1" -theme "$MENU_THEME"; }
notify() { notify-send -a Settings "Settings" "$1"; }

value_of() { sed -n "s/^\$$1 = //p" "$APPEARANCE"; }          # value_of border_size
set_value() { sed -i "s|^\$$1 = .*|\$$1 = $2|" "$APPEARANCE"; } # set_value border_size 2
rasi_value() { sed -n "s/^    $1: *\([^;]*\);.*/\1/p" "$RASI"; }   # rasi_value accent
rasi_set() { sed -i "s|^\(    $1: *\)[^;]*;|\1$2;|" "$RASI"; }      # rasi_set menu-border 2px

# One palette for both color pickers (Phase 44). Columns: name | accent: menus'
# border, selected row and prompt | lighter accent: letters matching the filter |
# window border, rgba() with the opacity in the last two digits. kitty.conf's
# normal / bright pairs, plus Orange, Pink and Violet made to match them (same
# saturation and lightness). Invisible: no border on menus (their text goes
# neutral white) or on windows.
PALETTE=(
    "Arch blue|#1793d1|#5db3df|rgba(1793d1cc)"
    "Cyan|#45b8d9|#74cde6|rgba(45b8d9cc)"
    "Green|#8cc265|#a6d986|rgba(8cc265cc)"
    "Amber|#e0a84e|#f0c278|rgba(e0a84ecc)"
    "Orange|#e28a50|#eea472|rgba(e28a50cc)"
    "Red|#e05f65|#f07a80|rgba(e05f65cc)"
    "Pink|#e273a2|#ee96ba|rgba(e273a2cc)"
    "Violet|#a57be0|#be9cec|rgba(a57be0cc)"
    "Soft white|#e0e0e0|#ffffff|rgba(e0e0e080)"
    "Grey|#a8a8a8|#e0e0e0|rgba(6e6e6ecc)"
    "Invisible|#e0e0e0|#ffffff|rgba(00000000)"
)

palette_name() {   # palette_name <index> <raw value>: the row's name, or the raw value if -1
    if [ "$1" -ge 0 ]; then echo "${PALETTE[$1]%%|*}"; else echo "$2"; fi
}

accent_index() {   # the palette row matching the menus' current accent, or -1
    local i name want accent border
    accent=$(rasi_value accent)
    border=$(rasi_value menu-border-color)
    for i in "${!PALETTE[@]}"; do
        IFS='|' read -r name want _ _ <<< "${PALETTE[$i]}"
        [ "$want" = "$accent" ] || continue
        # Soft white and Invisible share the text color; the border tells them apart
        if [ "$name" = Invisible ]; then [ "$border" = transparent ]; else [ "$border" != transparent ]; fi &&
            { echo "$i"; return; }
    done
    echo -1
}

border_index() {   # the palette row matching the window border's current color, or -1
    local i current
    current=$(value_of border_active)
    for i in "${!PALETTE[@]}"; do
        [ "${PALETTE[$i]##*|}" = "$current" ] && { echo "$i"; return; }
    done
    echo -1
}

pick_from_palette() {   # pick_from_palette <prompt> <current index>: prints the chosen index
    local i name accent color icon entries=()
    for i in "${!PALETTE[@]}"; do
        IFS='|' read -r name accent _ _ <<< "${PALETTE[$i]}"
        color=$accent icon=$I_OPTION
        [ "$name" = Invisible ] && color="#6e6e6e" icon=$I_NONE
        [ "$i" = "$2" ] && icon=$I_ACTIVE
        entries+=("<span foreground='$color'>$icon</span>  $name")
    done
    # -markup-rows: each dot in its own color; -format i: the chosen row's index, not its markup
    i=$(printf '%s\n' "${entries[@]}" |
        rofi -dmenu -i -no-custom -markup-rows -format i -p "$1" -theme "$MENU_THEME") || return 1
    [[ "$i" =~ ^[0-9]+$ ]] && echo "$i"
}

pick_accent() {
    local i name accent light border
    i=$(pick_from_palette "Accent color" "$(accent_index)") || return
    IFS='|' read -r name accent light _ <<< "${PALETTE[$i]}"
    border=$accent
    [ "$name" = Invisible ] && border=transparent
    # menus, bar, notifications, lock screen (Phase 45); it also confirms with a
    # notification and restarts Waybar, so it's the last thing this script does
    exec "$SCRIPTS/accent.sh" "$accent" "$light" "$border" "$name"
}

pick_color() {
    local i
    i=$(pick_from_palette "Border color" "$(border_index)") || return
    set_value border_active "${PALETTE[$i]##*|}"
    notify "Window border: ${PALETTE[$i]%%|*}"
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
    rasi_set menu-border "${px% px}px"
    notify "Border width: ${px}"
}

theme=$("$SCRIPTS/theme.sh" get)
declare -A ACTION
ENTRIES=()
add() { ENTRIES+=("$1"); ACTION["$1"]=$2; }
add "$I_THEME  Apps theme: ${theme^} (switch)"                  theme
add "$I_ACCENT  Accent color: $(palette_name "$(accent_index)" "$(rasi_value accent)")"  accent
add "$I_COLOR  Window border color: $(palette_name "$(border_index)" "$(value_of border_active)")"  color
add "$I_WIDTH  Border width: $(value_of border_size) px (windows and menus)"  width
add "$I_WALLPAPER  Wallpaper..."                                  wallpaper
add "$I_KEYS  Keybindings..."                                     keys

choice=$(printf '%s\n' "${ENTRIES[@]}" | menu "Settings") || exit 0
case "${ACTION[$choice]}" in
    theme)     "$SCRIPTS/theme.sh" toggle && notify "Apps theme: $("$SCRIPTS/theme.sh" get)" ;;
    accent)    pick_accent ;;
    color)     pick_color ;;
    width)     pick_width ;;
    wallpaper) exec "$SCRIPTS/wallpaper.sh" ;;
    keys)      exec "$SCRIPTS/rofi-keybinds.py" ;;
esac
exit 0
