#!/usr/bin/env python3
# ~/hyprland-from-scratch/scripts/rofi-keybinds.py
#
# Phase 27. Keybinding help (SUPER + /). Lists every bind declared with
# `bindd` in hyprland.conf — the description is part of the bind — read live
# from `hyprctl binds -j`, so the list can't drift from the config. A few
# extra rows cover groups of binds and mouse actions. Picking a bind runs it.
#
# Opening it again while it's open closes it (like rofi_toggle in rofi-common.sh).

import html
import json
import os
import re
import subprocess
import time

THEME = os.path.expanduser("~/.config/rofi/menu.rasi")
LAYOUT = "window { width: 680px; } listview { lines: 14; }"
PROMPT = "Keybindings"

MODS = [(64, "SUPER"), (4, "CTRL"), (8, "ALT"), (1, "SHIFT")]
KEY_NAMES = {"slash": "/", "Escape": "Esc", "left": "Left", "right": "Right",
             "up": "Up", "down": "Down", "mouse:272": "left-drag", "mouse:273": "right-drag"}

# shown for reference only (bound without a description, or not keys at all)
EXTRA = [
    ("SUPER + 1...9, 0", "Go to workspace 1-10"),
    ("SUPER + SHIFT + 1...9, 0", "Move the window to workspace 1-10 (and follow it)"),
    ("SUPER + ALT + 1...9, 0", "Send the window to workspace 1-10 (stay here)"),
    ("3 fingers down / up", "Minimize all windows here / restore them"),
    ("3 fingers left / right", "Focus the window on the left / right"),
    ("4 fingers down / up", "Volume down / up, 10% per swipe"),
    ("4 fingers left / right", "Previous / next workspace"),
    ("3 fingers spread / pinch", "Fullscreen on / off"),
    ("Brightness / volume keys", "Laptop brightness, volume, mute"),
    ("Click outside a menu", "Close the menu"),
    ("Bar: Arch logo", "Click: system info / right-click: settings"),
]


def menu_accent():
    """The menus' accent color, from ~/.config/rofi/appearance.rasi (Phase 43)."""
    try:
        with open(os.path.expanduser("~/.config/rofi/appearance.rasi")) as rasi:
            match = re.search(r"^\s*accent:\s*(#[0-9a-fA-F]{6})", rasi.read(), re.M)
    except OSError:
        match = None
    return match.group(1) if match else "#1793d1"


ACCENT = menu_accent()


def toggle_running():
    running = subprocess.run(["pgrep", "-ax", "rofi"], capture_output=True, text=True).stdout
    if not running:
        return
    subprocess.run(["pkill", "-x", "rofi"])
    while subprocess.run(["pgrep", "-x", "rofi"], capture_output=True).returncode == 0:
        time.sleep(0.05)
    if f"-p {PROMPT}" in running:
        raise SystemExit(0)   # it was open: this keypress closes it


def keys(bind):
    if bind["key"] == "Super_L" and bind.get("release"):
        return "SUPER (tap)"
    mods = [name for bit, name in MODS if bind["modmask"] & bit]
    key = KEY_NAMES.get(bind["key"], bind["key"].upper() if len(bind["key"]) == 1 else bind["key"])
    return " + ".join(mods + [key])


def row(combo, description, dim=False):
    color = "#6e6e6e" if dim else ACCENT
    return (f"<span font_family='JetBrainsMono Nerd Font' foreground='{color}'>"
            f"{html.escape(combo.ljust(26))}</span> {html.escape(description)}")


def main():
    toggle_running()
    binds = [b for b in json.loads(subprocess.run(["hyprctl", "binds", "-j"],
                                                  capture_output=True, text=True).stdout)
             if b.get("has_description")]
    rows = [row(keys(b), b["description"]) for b in binds]
    rows += [row(combo, description, dim=True) for combo, description in EXTRA]

    picked = subprocess.run(
        ["rofi", "-dmenu", "-i", "-no-custom", "-markup-rows", "-format", "i",
         "-p", PROMPT, "-theme", THEME, "-theme-str", LAYOUT],
        input="\n".join(rows), capture_output=True, text=True).stdout.strip()
    if not picked.isdigit() or int(picked) >= len(binds):
        return   # cancelled, or one of the reference rows
    bind = binds[int(picked)]
    if bind["mouse"]:
        return   # drag binds need the mouse; nothing to run from a menu
    subprocess.run(["hyprctl", "dispatch", bind["dispatcher"], bind["arg"]],
                   capture_output=True)


main()
