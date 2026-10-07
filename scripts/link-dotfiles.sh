#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/link-dotfiles.sh
#
# Puts every folder in dotfiles/ into ~/.config/, in one of two ways:
#
#   link-dotfiles.sh           link: ~/.config/<app> -> repo. Editing the repo
#                              edits the running desktop (to keep customizing).
#   link-dotfiles.sh --copy    copy: ~/.config/<app> is a plain copy. The repo
#                              can change without touching the desktop; run it
#                              again to update (to just use it).
#   ... --dry-run              print what would happen
#
# Safe to re-run and to switch modes. Anything that isn't ours is moved to
# <name>.bak-<timestamp>, never deleted; copies carry a marker file so an
# update overwrites them in place instead of piling up backups.

set -euo pipefail

REPO="$HOME/hyprland-from-scratch"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
STAMP="$(date +%Y%m%d-%H%M%S)"
MARKER=".hyprland-from-scratch"   # inside copied folders
MODE=link
DRY=0
for arg in "$@"; do
    case "$arg" in
        --copy)    MODE=copy ;;
        --dry-run) DRY=1 ;;
        *) echo "usage: link-dotfiles.sh [--copy] [--dry-run]" >&2; exit 2 ;;
    esac
done

run() {
    if (( DRY )); then echo "  would run: $*"; else "$@"; fi
}

# Scripts, wallpapers and Waybar's click handlers reference ~/hyprland-from-scratch.
if [[ "$(cd "$(dirname "$0")/.." && pwd)" != "$REPO" ]]; then
    echo "error: the repo must be cloned at $REPO (configs and scripts reference that path)" >&2
    exit 1
fi

mkdir -p "$CONFIG"

for src in "$REPO"/dotfiles/*/; do
    src="${src%/}"
    name="$(basename "$src")"
    dst="$CONFIG/$name"
    ours_link=0; ours_copy=0
    [[ -L "$dst" && "$(readlink -f "$dst")" == "$src" ]] && ours_link=1
    [[ -d "$dst" && ! -L "$dst" && -e "$dst/$MARKER" ]] && ours_copy=1

    if [[ $MODE == link ]]; then
        if (( ours_link )); then echo "ok      $name"; continue; fi
        if [[ -e "$dst" || -L "$dst" ]]; then
            echo "backup  $name -> $name.bak-$STAMP"
            run mv "$dst" "$dst.bak-$STAMP"
        fi
        echo "link    $name"
        run ln -s "$src" "$dst"
    else
        if (( ours_link )); then
            run rm "$dst"            # our own symlink (switching from link mode)
        elif (( ! ours_copy )) && [[ -e "$dst" || -L "$dst" ]]; then
            echo "backup  $name -> $name.bak-$STAMP"
            run mv "$dst" "$dst.bak-$STAMP"
        fi
        if (( ours_copy )); then echo "update  $name"; else echo "copy    $name"; fi
        run mkdir -p "$dst"
        run cp -rT "$src" "$dst"
        run touch "$dst/$MARKER"
    fi
done

# Target folder for the screenshot binds in hyprland.conf (grim fails if it's missing).
run mkdir -p "$HOME/Pictures/screenshots"

echo "done ($MODE). Log out and back in (or reboot) to load everything."
