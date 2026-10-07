#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/default-apps.sh
#
# Phase 33. Default applications, written to ~/.config/mimeapps.list with
# xdg-mime (used by Thunar, the browser's "open file", xdg-open...):
#   images          -> Loupe
#   video and audio -> Celluloid
# Each app gets every MIME type its .desktop file declares under those
# families, so no format is missed. Safe to re-run; apps not installed are skipped.
# Before this, images and MP4 videos opened in Brave.

set -uo pipefail

assign() {   # assign <desktop-id> <mime-family...>
    local desktop=$1 file types pattern
    shift
    file="/usr/share/applications/$desktop"
    [ -f "$file" ] || { echo "skip: $desktop (not installed)"; return; }
    pattern=$(IFS='|'; echo "$*")
    types=$(grep -m1 '^MimeType=' "$file" | cut -d= -f2 | tr ';' '\n' | grep -E "^($pattern)/")
    # shellcheck disable=SC2086  # one argument per MIME type
    xdg-mime default "$desktop" $types
    echo "$desktop: $(wc -l <<<"$types") types ($*)"
}

assign org.gnome.Loupe.desktop image
assign io.github.celluloid_player.Celluloid.desktop video audio
