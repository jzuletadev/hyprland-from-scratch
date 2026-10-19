#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/default-apps.sh

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

xdg-mime default imv.desktop image/png image/jpeg image/jpg image/webp image/gif
xdg-mime default mpv.desktop video/mp4 video/mkv video/webm video/x-matroska video/quicktime video/x-msvideo