#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/accent.sh
#
# Phase 45. The desktop's accent color, everywhere at once. The settings menu
# (scripts/rofi-settings.sh -> Accent color) calls it; by hand:
#
#   accent.sh <accent> <lighter> <frame> [name]
#   accent.sh '#a57be0' '#be9cec' '#a57be0' Violet
#
#   <accent>   selected rows, prompts, active workspace, Arch logo, volume fill
#   <lighter>  hover, song title, letters matching a menu's filter
#   <frame>    borders: menus, notifications, lock screen ring ("transparent" = none)
#   [name]     shown in a confirmation notification, drawn with the new frame
#
# Writes, all under ~/.config (so copy mode works too):
#   rofi/appearance.rasi              accent, accent-highlight, menu-border-color
#   waybar/accent.css                 @accent, @accent_light (style.css imports it)
#   waybar/config.jsonc               clock tooltip's weekdays and today (Pango markup, no CSS)
#   dunst/dunstrc.d/50-accent.conf    notification frame, progress bars
#   hypr/hyprlock.conf                password ring
# then reloads dunst and restarts Waybar (a restart, never SIGUSR2: Phase 22).
# Rofi and hyprlock read their files every time they open.

set -euo pipefail

hex='^#[0-9a-fA-F]{6}$'
if [ $# -lt 3 ] || ! [[ $1 =~ $hex && $2 =~ $hex ]] || ! [[ $3 =~ $hex || $3 = transparent ]]; then
    sed -n '7,8p' "$0" | sed 's/^# *//' >&2
    exit 2
fi
accent=$1 light=$2 frame=$3 name=${4:-}
C="$HOME/.config"

sed -i -e "s|^\(    accent: *\)[^;]*;|\1$accent;|" \
       -e "s|^\(    accent-highlight: *\)[^;]*;|\1bold $light;|" \
       -e "s|^\(    menu-border-color: *\)[^;]*;|\1$frame;|" "$C/rofi/appearance.rasi"

cat > "$C/waybar/accent.css" <<EOF
/* ~/hyprland-from-scratch/dotfiles/waybar/accent.css
 * Phase 45. Written by scripts/accent.sh (settings menu -> Accent color);
 * imported by style.css. */
@define-color accent $accent;
@define-color accent_light $light;
EOF
sed -i -e "/\"weekdays\":/s/color='#[0-9a-fA-F]\{6\}'/color='$accent'/" \
       -e "/\"today\":/s/background='#[0-9a-fA-F]\{6\}'/background='$accent'/" "$C/waybar/config.jsonc"

dunst_frame=$frame
[ "$frame" = transparent ] && dunst_frame="#00000000"
mkdir -p "$C/dunst/dunstrc.d"
cat > "$C/dunst/dunstrc.d/50-accent.conf" <<EOF
# ~/hyprland-from-scratch/dotfiles/dunst/dunstrc.d/50-accent.conf
# Phase 45. Written by scripts/accent.sh (settings menu -> Accent color); dunst
# reads it after dunstrc. Critical notifications keep dunstrc's red frame.
[global]
    frame_color = "$dunst_frame"
[urgency_normal]
    highlight = "$accent"   # progress bars (volume indicator, Phase 34)
EOF

if [ "$frame" = transparent ]; then
    ring="rgba(0, 0, 0, 0.0)"
else
    ring="rgba($((16#${frame:1:2})), $((16#${frame:3:2})), $((16#${frame:5:2})), 0.8)"
fi
sed -i "s|^\(    outer_color = \)rgba([^)]*)|\1$ring|" "$C/hypr/hyprlock.conf"

dunstctl reload >/dev/null 2>&1 || true
[ -n "$name" ] && notify-send -a Settings "Settings" "Accent: $name"
# last: when this runs from a Waybar click, the restart ends this script too
systemctl --user restart --no-block waybar.service
