#!/usr/bin/env bash
# Backend de gestión de fondos de pantalla (awww, hyprlock y SDDM)

set -uo pipefail

REPO="$HOME/hyprland-from-scratch"
DEFAULT_DIR="$HOME/Pictures/wallpapers"
CURRENT="$HOME/.local/state/hyprland-from-scratch/wallpaper"
SDDM_WALLPAPER="/usr/share/backgrounds/current_wallpaper.jpg"

notify()   { notify-send -a Wallpaper "Wallpaper" "$1"; }
is_image() { [ -f "$1" ] && [[ "${1,,}" =~ \.(jpe?g|png|webp|gif|bmp)$ ]]; }

wait_daemon() {
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
    
    # Sincronizar con la pantalla de inicio de sesión SDDM si hay permisos
    if [ -w "$SDDM_WALLPAPER" ]; then
        cp -f "$1" "$SDDM_WALLPAPER" 2>/dev/null || true
    fi
}

set_file() {
    local file
    file=$(realpath -e -- "${1:-}" 2>/dev/null) || { notify "File not found: ${1:-}"; exit 1; }
    is_image "$file" || { notify "Not an image: $(basename "$file")"; exit 1; }
    apply "$file" grow && notify "$(basename "$file")"
}

restore() {
    local file
    if ! file=$(readlink -e "$CURRENT" 2>/dev/null); then
        file=$(find "$DEFAULT_DIR" -maxdepth 1 -type f -print0 2>/dev/null | sort -z |
               while IFS= read -r -d '' f; do is_image "$f" && { printf '%s' "$f"; break; }; done)
    fi
    [ -n "$file" ] || { notify "No wallpaper found in ~/Pictures/wallpapers"; return 1; }
    apply "$file" fade
}

case "${1:-restore}" in
    set)     set_file "${2:-}" ;;
    restore) restore ;;
    *)       echo "usage: wallpaper.sh [set FILE | restore]" >&2; exit 2 ;;
esac
