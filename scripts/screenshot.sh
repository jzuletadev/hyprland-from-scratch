#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/screenshot.sh
#
# Phase 23. Screenshots that are saved AND copied to the clipboard, so they can
# be pasted right away (Ctrl+V in a chat, an issue, a document).
#
#   screenshot.sh screen   the focused monitor        SUPER+S
#   screenshot.sh region   drag a box (Escape cancels) SUPER+SHIFT+S
#   screenshot.sh window   the focused window         SUPER+CTRL+S
#   screenshot.sh annotate drag a box, open it in Satty SUPER+ALT+S (Phase 33)
#
# Files: ~/Pictures/screenshots/<date>_<time>.png. The notification shows a
# thumbnail of the capture (dunst's max_icon_size keeps it small); clicking it
# opens the capture in Satty to annotate (Phase 33, dotfiles/satty/config.toml).

set -uo pipefail

DIR="$HOME/Pictures/screenshots"
file="$DIR/$(date +%Y-%m-%d_%H-%M-%S).png"
mkdir -p "$DIR"

notify() { notify-send -a Screenshot "$@"; }

case "${1:-screen}" in
    annotate)
        # straight into Satty: it copies and saves on Enter (see its config)
        geo=$(slurp) || exit 0
        grim -g "$geo" -t ppm - | satty --filename -
        exit 0 ;;
    screen)
        output=$(hyprctl monitors -j | python3 -c '
import json, sys
print(next(m["name"] for m in json.load(sys.stdin) if m["focused"]))')
        grim -o "$output" "$file" ;;
    region)
        geo=$(slurp) || exit 0
        grim -g "$geo" "$file" ;;
    window)
        geo=$(hyprctl activewindow -j | python3 -c '
import json, sys
w = json.load(sys.stdin)
if w:
    x, y = w["at"]
    width, height = w["size"]
    print(f"{x},{y} {width}x{height}")')
        [ -z "$geo" ] && { notify "Screenshot" "No focused window"; exit 1; }
        grim -g "$geo" "$file" ;;
    *)
        echo "usage: screenshot.sh [screen | region | window | annotate]" >&2
        exit 2 ;;
esac || { notify "Screenshot" "Capture failed"; exit 1; }

# wl-copy forks into the background and keeps serving the image until
# something else is copied, so it can be pasted after this script exits
wl-copy --type image/png < "$file"
# the notification offers "Annotate" (a click on it, see dunstrc); waiting for
# the answer happens in the background so this script returns at once
(
    action=$(notify -i "$file" --action=annotate=Annotate "Screenshot" "Copied to the clipboard
Saved as ~/${file#"$HOME"/}
Click to annotate")
    [ "$action" = annotate ] && satty --filename "$file"
) >/dev/null 2>&1 &
