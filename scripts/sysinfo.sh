#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/sysinfo.sh

if hyprctl clients -j | grep -q '"class": "sysinfo"'; then
    hyprctl dispatch closewindow class:sysinfo >/dev/null
    exit 0
fi

logo=/usr/share/pixmaps/archlinux-logo.png   # 256x256, Arch blue #1793d1
cache=${XDG_CACHE_HOME:-$HOME/.cache}/hyprland-from-scratch
for img in ~/.config/fastfetch/logo.{png,jpg,jpeg,webp}; do
    [ -f "$img" ] || continue
    fitted="$cache/sysinfo-logo-$(stat -Lc '%i-%Y-%s' "$img").png"   # new name when the image changes
    if [ -f "$fitted" ] || { mkdir -p "$cache" &&
        find "$cache" -maxdepth 1 -name 'sysinfo-logo-*.png' -delete &&
        magick "$img" -resize 512x512 -background none -gravity center -extent 512x512 "$fitted"; }; then
        logo=$fitted
    fi
    break
done

# 24 columns x 11 rows: square at kitty.conf's font size (cells are 9 x 20 px)
exec kitty --class sysinfo -e bash -c '
    fastfetch --logo-type kitty-direct --logo "$1" --logo-width 24 --logo-height 11 --logo-padding-right 3
    printf "\n  Press any key to close"; read -rsn1' _ "$logo"
