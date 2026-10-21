#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/default-apps.sh

set -uo pipefail

xdg-mime default imv.desktop image/png image/jpeg image/jpg image/webp image/gif
xdg-mime default mpv.desktop video/mp4 video/mkv video/webm video/x-matroska video/quicktime video/x-msvideo