#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/install-sddm-theme.sh

set -euo pipefail

[ "$(id -u)" -eq 0 ] || { echo "run it with sudo" >&2; exit 1; }

REPO="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$REPO/system/sddm"
THEME_DIR=/usr/share/sddm/themes/arch-neutral
CONF_DIR=/etc/sddm.conf.d

# pick the background
background="${1:-}"
if [ -z "$background" ] && [ -n "${SUDO_USER:-}" ]; then
    user_home=$(getent passwd "$SUDO_USER" | cut -d: -f6)
    background=$(readlink -e "$user_home/.local/state/hyprland-from-scratch/wallpaper" 2>/dev/null || true)
fi
if [ -z "$background" ]; then
    background=$(find "$REPO/assets/wallpapers" -maxdepth 1 -type f \
        \( -iname '*.jpg' -o -iname '*.jpeg' -o -iname '*.png' -o -iname '*.webp' \) | sort | head -1)
fi
[ -f "$background" ] || { echo "no background image found" >&2; exit 1; }
ext="${background##*.}"
ext="${ext,,}"

install -d "$THEME_DIR" "$CONF_DIR"
# -T: copy the folder's contents onto the installed one (updates in place)
cp -rT "$SRC/themes/arch-neutral" "$THEME_DIR"
find "$THEME_DIR" -maxdepth 1 -name 'background.*' -delete   # an old one, maybe another format
install -m 644 "$background" "$THEME_DIR/background.$ext"
sed -i "s|^background=.*|background=background.$ext|" "$THEME_DIR/theme.conf"
install -m 644 "$SRC/sddm.conf.d/10-arch-neutral.conf" "$CONF_DIR/"

# check what SDDM will read
for f in Main.qml metadata.desktop theme.conf "background.$ext"; do
    [ -s "$THEME_DIR/$f" ] || { echo "missing: $THEME_DIR/$f" >&2; exit 1; }
done
grep -q '^Current=arch-neutral' "$CONF_DIR/10-arch-neutral.conf"
echo "Background: $background"

echo "Installed: $THEME_DIR"
echo "Config:    $CONF_DIR/10-arch-neutral.conf"
echo "It shows at the next login screen. Preview now (as your user, not root):"
echo "  sddm-greeter-qt6 --test-mode --theme $THEME_DIR"
