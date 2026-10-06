#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/wallpaper.sh
#
# Phase 17. Wallpaper control on top of awww, which has no UI of its own:
#
#   wallpaper.sh            pick one in Rofi (thumbnail grid) — SUPER+W, or
#                           right-click the Arch logo in Waybar
#   wallpaper.sh set FILE   apply FILE — Thunar's "Set as wallpaper" action
#   wallpaper.sh restore    re-apply the last choice — exec-once at login
#
# The choice is kept as a symlink, ~/.local/state/hyprland-from-scratch/wallpaper,
# which hyprlock.conf also uses as its background, so the lock screen always
# matches the desktop. The picker lists assets/wallpapers/ (versioned) and
# ~/Pictures/wallpapers/ (personal, outside the repo).

set -uo pipefail

REPO="$HOME/hyprland-from-scratch"
DIRS=("$REPO/assets/wallpapers" "$HOME/Pictures/wallpapers")
DEFAULT="$REPO/assets/wallpapers/3.jpeg"
CURRENT="$HOME/.local/state/hyprland-from-scratch/wallpaper"
THUMBS="$HOME/.cache/hyprland-from-scratch/wallpaper-thumbs"

source "$REPO/scripts/rofi-common.sh"

notify()   { notify-send -a Wallpaper "Wallpaper" "$1"; }
is_image() { [ -f "$1" ] && [[ "${1,,}" =~ \.(jpe?g|png|webp|gif|bmp)$ ]]; }

wait_daemon() {   # at login awww-daemon starts in parallel with this script
    local i
    for ((i = 0; i < 50; i++)); do
        awww query >/dev/null 2>&1 && return 0
        sleep 0.1
    done
    return 1
}

apply() {   # apply <file> <transition-type>
    wait_daemon || { notify "awww-daemon isn't running"; return 1; }
    awww img "$1" --transition-type "$2" --transition-duration 1.2 --transition-fps 60 || return 1
    mkdir -p "$(dirname "$CURRENT")"
    ln -sfn "$1" "$CURRENT"
}

thumb() {   # thumb <image>: print the path of a cached 320x180 thumbnail
    local t
    t="$THUMBS/$(printf '%s' "$1" | sha1sum | cut -c1-16).png"
    if [ ! -f "$t" ] || [ "$1" -nt "$t" ]; then
        mkdir -p "$THUMBS"
        magick "$1[0]" -define jpeg:size=640x360 -thumbnail '320x180^' \
               -gravity center -extent 320x180 "$t" 2>/dev/null
    fi
    printf '%s' "$t"
}

pick() {
    rofi_toggle "-p Wallpaper "
    local current dir img name choice
    local images=()
    current=$(readlink -e "$CURRENT" 2>/dev/null)
    for dir in "${DIRS[@]}"; do
        [ -d "$dir" ] || continue
        while IFS= read -r -d '' img; do
            is_image "$img" && images+=("$img")
        done < <(find "$dir" -maxdepth 1 -type f -print0 | sort -z)
    done
    if [ ${#images[@]} -eq 0 ]; then
        notify "No images in assets/wallpapers or ~/Pictures/wallpapers"
        return
    fi
    # rofi dmenu: "label\0icon\x1f<path>" shows <path> as the entry's icon;
    # -format i returns the chosen index instead of its label
    choice=$(for img in "${images[@]}"; do
                 name=$(basename "$img")
                 [ "$img" = "$current" ] && name+="  (current)"
                 printf '%s\0icon\x1f%s\n' "$name" "$(thumb "$img")"
             done | rofi -dmenu -i -no-custom -show-icons -format i -p "Wallpaper" \
                         -theme "$MENU_THEME" -theme-str '
                 window       { width: 640px; }
                 listview     { columns: 3; lines: 2; fixed-height: true; spacing: 10px; }
                 element      { orientation: vertical; padding: 10px; spacing: 6px; }
                 element-icon { size: 170px; }
                 element-text { horizontal-align: 0.5; }') || return 0
    apply "${images[$choice]}" grow && notify "$(basename "${images[$choice]}")"
}

set_file() {
    local file
    file=$(realpath -e -- "${1:-}" 2>/dev/null) || { notify "File not found: ${1:-}"; exit 1; }
    is_image "$file" || { notify "Not an image: $(basename "$file")"; exit 1; }
    apply "$file" grow && notify "$(basename "$file")"
}

restore() {
    local file
    file=$(readlink -e "$CURRENT" 2>/dev/null) || file=$DEFAULT
    apply "$file" fade
}

case "${1:-pick}" in
    pick)    pick ;;
    set)     set_file "${2:-}" ;;
    restore) restore ;;
    *)       echo "usage: wallpaper.sh [pick | set FILE | restore]" >&2; exit 2 ;;
esac
