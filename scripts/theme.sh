#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/theme.sh
#
# Phase 27. Light or dark theme for applications: GTK3 (Thunar, file pickers),
# GTK4/libadwaita, and every website that follows the system (Brave with its
# theme set to "Device"). The desktop itself — bar, menus, notifications, lock
# and login screens — keeps the Arch Neutral dark palette either way.
#
#   theme.sh dark | light   switch and remember the choice
#   theme.sh toggle         switch to the other one
#   theme.sh restore        re-apply the remembered choice (exec-once at login)
#   theme.sh get            print the current choice
#
# The choice lives in ~/.local/state/hyprland-from-scratch/theme (default: dark).
#
# GTK3 (Thunar) gets its dark variant from gtk-application-prefer-dark-theme in
# ~/.config/gtk-3.0/settings.ini, written here: the theme *name* "Adwaita-dark"
# only exists with the gnome-themes-extra package, so without it GTK3 silently
# fell back to light Adwaita (Phase 29). GTK4/libadwaita follows color-scheme.
# Icons: Papirus (Papirus-Dark / Papirus) when installed, Adwaita otherwise.

STATE="$HOME/.local/state/hyprland-from-scratch/theme"
GTK3_SETTINGS="$HOME/.config/gtk-3.0/settings.ini"

current() { cat "$STATE" 2>/dev/null || echo dark; }

icon_theme() {   # icon_theme dark|light
    if [ -d /usr/share/icons/Papirus ]; then
        if [ "$1" = light ]; then echo Papirus; else echo Papirus-Dark; fi
    else
        echo Adwaita
    fi
}

apply() {
    local dark=1 scheme=prefer-dark icons
    [ "$1" = light ] && { dark=0; scheme=prefer-light; }
    icons=$(icon_theme "$1")
    gsettings set org.gnome.desktop.interface color-scheme "$scheme"
    gsettings set org.gnome.desktop.interface gtk-theme 'Adwaita'
    gsettings set org.gnome.desktop.interface icon-theme "$icons"
    mkdir -p "$(dirname "$GTK3_SETTINGS")"
    printf '%s\n' "# written by ~/hyprland-from-scratch/scripts/theme.sh — changes here are overwritten" \
        "[Settings]" \
        "gtk-application-prefer-dark-theme=$dark" \
        "gtk-icon-theme-name=$icons" > "$GTK3_SETTINGS"
    # Thunar runs as a long-lived service (thunar.service) that read the old
    # settings when it started; restart it so the next window uses the new ones —
    # but only when none of its windows is open, so nothing closes under you.
    if ! hyprctl clients -j 2>/dev/null | grep -q '"class": "Thunar"'; then
        systemctl --user try-restart thunar.service 2>/dev/null
    fi
}

remember() {
    mkdir -p "$(dirname "$STATE")"
    echo "$1" > "$STATE"
}

case "${1:-get}" in
    dark|light) apply "$1"; remember "$1" ;;
    toggle)     if [ "$(current)" = dark ]; then next=light; else next=dark; fi
                apply "$next"; remember "$next" ;;
    restore)    apply "$(current)" ;;
    get)        current ;;
    *)          echo "usage: theme.sh dark|light|toggle|restore|get" >&2; exit 2 ;;
esac
