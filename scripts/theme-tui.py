#!/usr/bin/env python3
import curses
import os
import subprocess

THEME_SCRIPT = os.path.expanduser("~/hyprland-from-scratch/scripts/theme-manager.sh")
PALETTE_FILE = os.path.expanduser("~/.config/hypr/current_theme")
MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

PALETTES = [
    ("monokai",           "Monokai Pro",    "Warm Charcoal"),
    ("dracula",           "Dracula",        "Púrpura vampire"),
    ("nord",              "Nord Ice",       "Arctic Blue"),
    ("rose-pine-moon",    "Rose Pine",      "Rose Moon"),
    ("tokyo-night",       "Tokyo Night",    "Arctic Night"),
    ("everforest-medium", "Everforest",     "Medium Green"),
]

def get_state():
    palette = "monokai"
    mode = "dark"
    if os.path.exists(PALETTE_FILE):
        try:
            with open(PALETTE_FILE, "r") as f:
                palette = f.read().strip() or "monokai"
        except Exception:
            pass
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                mode = f.read().strip() or "dark"
        except Exception:
            pass
    return palette, mode

def apply_palette(palette_id):
    subprocess.Popen(["bash", THEME_SCRIPT, palette_id], start_new_session=True)

def safe_addstr(win, y, x, text, attr=0):
    try:
        max_y, max_x = win.getmaxyx()
        if y < 0 or y >= max_y or x < 0 or x >= max_x:
            return
        available = max_x - x - 1
        if available > 0:
            win.addstr(y, x, text[:available], attr)
    except curses.error:
        pass

def main(stdscr):
    curses.curs_set(0)
    curses.use_default_colors()
    stdscr.keypad(True)

    current_palette, current_mode = get_state()
    selected_idx = 0
    for idx, (p_id, _, _) in enumerate(PALETTES):
        if p_id == current_palette:
            selected_idx = idx
            break

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()
        current_palette, current_mode = get_state()
        is_light = (current_mode == "light")
        from theme_colors import get_accent_hex, ansi_accent, hover_colors
        ACCENT_COLOR = ansi_accent(get_accent_hex())
        HOVER_FG, HOVER_BG = hover_colors(is_light)

        if is_light:
            curses.init_pair(1, curses.COLOR_GREEN, -1)                   # Marco
            curses.init_pair(2, ACCENT_COLOR, -1)                    # Atajos / Activo
            curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
            curses.init_pair(4, curses.COLOR_BLUE, -1)                    # Encabezados
            curses.init_pair(5, curses.COLOR_BLACK, -1)                   # Texto normal
        else:
            curses.init_pair(1, curses.COLOR_GREEN, -1)                   # Marco
            curses.init_pair(2, ACCENT_COLOR, -1)                    # Atajos / Activo
            curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
            curses.init_pair(4, curses.COLOR_YELLOW, -1)                  # Encabezados
            curses.init_pair(5, curses.COLOR_WHITE, -1)                   # Texto normal

        if h < 8 or w < 30:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            if stdscr.getch() in [27, ord('q'), ord('Q')]:
                break
            continue

        box_top = 0
        box_h = max(7, h - 1)
        box_w = max(28, w - 2)
        box_x = 1

        # Marco exterior protegido
        stdscr.attron(curses.color_pair(1))
        for x in range(box_x, box_x + box_w):
            try:
                stdscr.addch(box_top, x, curses.ACS_HLINE)
                stdscr.addch(box_top + box_h - 1, x, curses.ACS_HLINE)
            except curses.error:
                pass
        for y in range(box_top, box_top + box_h):
            try:
                stdscr.addch(y, box_x, curses.ACS_VLINE)
                stdscr.addch(y, box_x + box_w - 1, curses.ACS_VLINE)
            except curses.error:
                pass
        try:
            stdscr.addch(box_top, box_x, curses.ACS_ULCORNER)
            stdscr.addch(box_top, box_x + box_w - 1, curses.ACS_URCORNER)
            stdscr.addch(box_top + box_h - 1, box_x, curses.ACS_LLCORNER)
            stdscr.addch(box_top + box_h - 1, box_x + box_w - 1, curses.ACS_LRCORNER)
        except curses.error:
            pass
        stdscr.attroff(curses.color_pair(1))

        # Título y encabezados
        title = f" System Theme [{current_palette.capitalize()}] "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 2, box_x + 5, "Theme", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 2, box_x + 20, "Palette Style", curses.color_pair(4) | curses.A_BOLD)

        # Lista de temas
        for i, (item_id, name, desc) in enumerate(PALETTES):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            is_active = (item_id == current_palette)
            mark = "●" if is_active else " "
            col1 = f" {mark} {name:<14}"
            col2 = f" {desc}"
            row_str = (col1 + col2).ljust(box_w - 4)

            if is_hovered:
                _bar_end = min(box_x + box_w - 1, box_x + 2 + len(row_str))
                for _bx in range(box_x + 2, _bar_end):
                    stdscr.addstr(y, _bx, ' ', curses.color_pair(3))
                safe_addstr(stdscr, y, box_x + 2, row_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                attr = (curses.color_pair(2) | curses.A_BOLD) if is_active else curses.color_pair(5)
                safe_addstr(stdscr, y, box_x + 2, col1, attr)
                safe_addstr(stdscr, y, box_x + 2 + len(col1), col2, curses.color_pair(5))

        # Atajos inferiores
        help_text = "↵ set | i/k nav | q quit"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('i')]:
            selected_idx = (selected_idx - 1) % len(PALETTES)
        elif key in [curses.KEY_DOWN, ord('k')]:
            selected_idx = (selected_idx + 1) % len(PALETTES)
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            item_id = PALETTES[selected_idx][0]
            apply_palette(item_id)
            break
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)