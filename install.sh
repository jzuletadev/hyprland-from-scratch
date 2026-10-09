#!/usr/bin/env bash
# ~/hyprland-from-scratch/install.sh
#
# Installs the desktop on a fresh Arch Linux (see README "1. Install Arch").
#
#   ./install.sh            use it:  configs are copied into ~/.config
#   ./install.sh --link     develop: ~/.config/<app> -> this repo, edits are live
#
#   --apps      also the personal apps in packages/apps.txt (Steam needs [multilib])
#   --dry-run   print every step without changing anything
#
# Run it as your user (it asks for sudo when needed). Safe to run again: to
# update after `git pull`, or to switch between --link and copy mode.

set -euo pipefail

REPO="$HOME/hyprland-from-scratch"
MODE=copy
APPS=0
DRY=0
for arg in "$@"; do
    case "$arg" in
        --link)    MODE=link ;;
        --apps)    APPS=1 ;;
        --dry-run) DRY=1 ;;
        -h|--help) sed -n '3,14p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown option: $arg (see --help)" >&2; exit 2 ;;
    esac
done

step() { printf '\n\033[1;34m==>\033[0m \033[1m%s\033[0m\n' "$*"; }
run()  { echo "+ $*"; (( DRY )) || "$@"; }
packages() {   # packages <list>: names only (comments stripped)
    sed 's/#.*//' "$REPO/packages/$1" | tr -s ' \t' '\n' | grep -v '^$'
}

[ "$(id -u)" -ne 0 ] || { echo "run it as your user, not root (it uses sudo itself)" >&2; exit 1; }
[ "$(cd "$(dirname "$0")" && pwd)" = "$REPO" ] || {
    echo "clone the repo to $REPO first: configs and scripts reference that path" >&2; exit 1; }

step "Packages (packages/base.txt)"
mapfile -t pkgs < <(packages base.txt)
if (( APPS )); then
    mapfile -t extra < <(packages apps.txt)
    if printf '%s\n' "${extra[@]}" | grep -qx steam && ! grep -q '^\[multilib\]' /etc/pacman.conf; then
        echo "note: skipping steam — enable [multilib] in /etc/pacman.conf first"
        mapfile -t extra < <(printf '%s\n' "${extra[@]}" | grep -vx steam)
    fi
    pkgs+=("${extra[@]}")
fi
run sudo pacman -S --needed "${pkgs[@]}"
run fc-cache -f

step "Configs into ~/.config ($MODE)"
if [ "$MODE" = copy ]; then
    run "$REPO/scripts/link-dotfiles.sh" --copy
else
    run "$REPO/scripts/link-dotfiles.sh"
fi

step "Shell: history and suggestions (~/.bashrc)"
# ~/.bashrc stays the user's own (personal tokens, paths); it only sources ours
if grep -qF '.config/bash/bashrc' "$HOME/.bashrc" 2>/dev/null; then
    echo "already sourced from ~/.bashrc"
else
    echo "+ append to ~/.bashrc: . ~/.config/bash/bashrc"
    (( DRY )) || printf '\n# hyprland-from-scratch (Phase 46): history, suggestions, fzf\n[ -f ~/.config/bash/bashrc ] && . ~/.config/bash/bashrc\n' >> "$HOME/.bashrc"
fi

step "Services"
run sudo systemctl enable sddm.service bluetooth.service power-profiles-daemon.service
run systemctl --user enable waybar.service   # the bar; restarted by systemd if it crashes

step "Default apps (images -> Loupe, video/audio -> Celluloid)"
run "$REPO/scripts/default-apps.sh"

step "Login screen theme"
run sudo "$REPO/scripts/install-sddm-theme.sh"

step "Done"
echo "Reboot, then pick \"Hyprland (uwsm)\" on the login screen."
echo "Keys: tap SUPER for apps, SUPER + / for every shortcut."
echo "Suggestions while typing in the terminal: install blesh-git from the AUR (README, packages/aur.txt)."
if lspci 2>/dev/null | grep -E 'VGA|3D' | grep -qi nvidia && [ "$(lspci | grep -cE 'VGA|3D')" -gt 1 ]; then
    echo
    echo "Hybrid graphics detected (iGPU + NVIDIA): follow README annex A before rebooting."
fi
