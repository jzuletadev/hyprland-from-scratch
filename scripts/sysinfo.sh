#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/sysinfo.sh
#
# Phase 23. System information (fastfetch, Phase 11) in a floating kitty window,
# opened by clicking the Arch logo in Waybar. Clicking the logo again, or
# pressing any key in the window, closes it. Size and position come from the
# "sysinfo" window rules in hyprland.conf.

if hyprctl clients -j | grep -q '"class": "sysinfo"'; then
    hyprctl dispatch closewindow class:sysinfo >/dev/null
    exit 0
fi

exec kitty --class sysinfo -e bash -c 'fastfetch; printf "\n  Press any key to close"; read -rsn1'
