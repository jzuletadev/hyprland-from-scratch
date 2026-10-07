# ~/hyprland-from-scratch/scripts/rofi-common.sh
#
# Phase 17. Sourced (not run) by the rofi-*.sh menus and wallpaper.sh.

MENU_THEME="$HOME/.config/rofi/menu.rasi"

# Only one Rofi can run at a time. Before a menu opens, close whatever menu is
# already open. If it was one of this menu's own (its command line contains
# one of the arguments, e.g. "-p Network "), the click was meant to close it:
# stop there instead of reopening it. Clicking another Waybar icon therefore
# switches menus.
rofi_toggle() {
    local running pattern
    running=$(pgrep -ax rofi) || return 0
    pkill -x rofi
    while pgrep -x rofi >/dev/null; do sleep 0.05; done
    for pattern in "$@"; do
        [[ "$running" == *"$pattern"* ]] && exit 0
    done
    return 0
}
