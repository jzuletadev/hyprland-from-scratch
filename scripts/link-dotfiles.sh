#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/link-dotfiles.sh
#
# Links every folder in dotfiles/ into ~/.config/ (Phase 2 pattern, applied to
# all apps at once). Safe to re-run: correct links are skipped, and anything
# already sitting at the target is moved to <name>.bak-<timestamp>, never deleted.
#
# Usage: link-dotfiles.sh [--dry-run]

set -euo pipefail

REPO="$HOME/hyprland-from-scratch"
CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}"
STAMP="$(date +%Y%m%d-%H%M%S)"
DRY=0
[[ "${1:-}" == "--dry-run" ]] && DRY=1

run() {
    if (( DRY )); then echo "  would run: $*"; else "$@"; fi
}

# Every path in the configs is hardcoded to ~/hyprland-from-scratch.
if [[ "$(cd "$(dirname "$0")/.." && pwd)" != "$REPO" ]]; then
    echo "error: repo must be cloned at $REPO (configs and scripts reference that path)" >&2
    exit 1
fi

mkdir -p "$CONFIG"

for src in "$REPO"/dotfiles/*/; do
    src="${src%/}"
    name="$(basename "$src")"
    dst="$CONFIG/$name"

    if [[ -L "$dst" && "$(readlink -f "$dst")" == "$src" ]]; then
        echo "ok      $name"
        continue
    fi
    if [[ -e "$dst" || -L "$dst" ]]; then
        echo "backup  $name -> $name.bak-$STAMP"
        run mv "$dst" "$dst.bak-$STAMP"
    fi
    echo "link    $name"
    run ln -s "$src" "$dst"
done

# Target folder for the screenshot binds in hyprland.conf (grim fails if it's missing).
run mkdir -p "$HOME/Pictures/screenshots"

echo "done. Restart Hyprland (or log out/in) to load everything."
