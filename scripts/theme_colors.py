"""Theme-aware color helpers for the curses-based TUI menus.

Reads the active flavor (palette) and maps its accent to a curses color, so the
TUI highlight/help colors follow the active theme instead of hardcoded colors.
"""
import colorsys
import os
import re

CONFIG_DIR = os.path.expanduser("~/.config")
FLAVOR_DIR = os.path.expanduser("~/hyprland-from-scratch/dotfiles/yazi/flavors")

ANSI_NAMES = (
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
)


def get_mode():
    try:
        with open(os.path.join(CONFIG_DIR, "hypr", "current_mode")) as f:
            return f.read().strip()
    except Exception:
        return "dark"


def get_flavor():
    try:
        with open(os.path.join(CONFIG_DIR, "hypr", "current_theme")) as f:
            return f.read().strip()
    except Exception:
        return "dracula"


def get_accent_hex():
    flavor = get_flavor()
    path = os.path.join(FLAVOR_DIR, flavor + ".yazi", "flavor.toml")
    try:
        with open(path, encoding="utf-8", errors="ignore") as f:
            toml_text = f.read()
        m = re.search(r'normal_main\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, re.IGNORECASE | re.DOTALL)
        if m:
            return m.group(1).lower()
    except Exception:
        pass
    return "#cba6f7"


def ansi_accent(hex_color):
    """Map a hex accent color to the nearest hue-appropriate curses ANSI color (0-7)."""
    h = hex_color.lstrip("#")
    r = int(h[0:2], 16) / 255.0
    g = int(h[2:4], 16) / 255.0
    b = int(h[4:6], 16) / 255.0
    hue_deg = colorsys.rgb_to_hsv(r, g, b)[0] * 360.0
    if hue_deg < 40:
        return 1    # red
    if hue_deg < 75:
        return 3    # yellow
    if hue_deg < 165:
        return 2    # green
    if hue_deg < 195:
        return 6    # cyan
    if hue_deg < 255:
        return 4    # blue
    return 5        # magenta


def accent_luminance(hex_color):
    h = hex_color.lstrip("#")
    r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
    return (0.299 * r + 0.587 * g + 0.114 * b) / 255.0


def hover_colors(is_light):
    """Return (fg, bg) curses color numbers for the hover highlight bar.

    bg is the accent color itself; fg is a readable text color for that bar.
    """
    bg = ansi_accent(get_accent_hex())
    if is_light:
        return (0, bg)  # black text on the (dark) accent bar
    fg = 0 if accent_luminance(get_accent_hex()) > 0.7 else 7  # black on light accents, white on dark
    return (fg, bg)
