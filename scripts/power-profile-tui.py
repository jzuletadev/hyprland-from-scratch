#!/usr/bin/env python3
import curses
import subprocess
import os

MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

def get_current_mode():
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return "dark"

def get_current_profile():
    try:
        res = subprocess.run(["powerprofilesctl", "get"], capture_output=True, text=True)
        return res.stdout.strip() or "balanced"
    except Exception:
        return "balanced"

def set_profile(profile):
    try:
        subprocess.run(["powerprofilesctl", "set", profile], check=False)
    except Exception:
        pass

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

    mode = get_current_mode()
    is_light = (mode == "light")
    from theme_colors import get_accent_hex, ansi_accent, hover_colors
    ACCENT_COLOR = ansi_accent(get_accent_hex())
    HOVER_FG, HOVER_BG = hover_colors(is_light)

    if is_light:
        curses.init_pair(1, curses.COLOR_GREEN, -1)                   # Marco
        curses.init_pair(2, ACCENT_COLOR, -1)                    # Ayuda / Conectado
        curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
        curses.init_pair(4, curses.COLOR_BLUE, -1)                    # Encabezados
        curses.init_pair(5, curses.COLOR_BLACK, -1)                   # Texto normal
    else:
        curses.init_pair(1, curses.COLOR_GREEN, -1)                   # Marco
        curses.init_pair(2, ACCENT_COLOR, -1)                    # Ayuda / Conectado
        curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
        curses.init_pair(4, curses.COLOR_YELLOW, -1)                  # Encabezados
        curses.init_pair(5, curses.COLOR_WHITE, -1)                   # Texto normal

    profiles = [
        ("performance", "Performance ", "Máximo rendimiento"),
        ("balanced",    "Balanced    ", "Equilibrado"),
        ("power-saver", "Power-saver ", "Ahorro de batería"),
    ]

    current = get_current_profile()
    selected_idx = 1
    for idx, (p_id, _, _) in enumerate(profiles):
        if p_id == current:
            selected_idx = idx
            break

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

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

        # Dibujar marco de caja protegido
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

        # Título en el borde superior
        title = " Power Profiles "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Encabezado de columnas
        safe_addstr(stdscr, box_top + 1, box_x + 4, "Mode", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 1, box_x + 20, "Status / Description", curses.color_pair(4) | curses.A_BOLD)

        # Opciones
        for i, (p_id, name, desc) in enumerate(profiles):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_active = (p_id == current)
            mark = "●" if is_active else " "

            line_str = f" {mark} {name}  {desc[:26]:<26} "
            if i == selected_idx:
                _bar_end = min(box_x + box_w - 1, box_x + 2 + len(line_str))
                for _bx in range(box_x + 2, _bar_end):
                    stdscr.addstr(y, _bx, ' ', curses.color_pair(3))
                safe_addstr(stdscr, y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                safe_addstr(stdscr, y, box_x + 2, f" {mark} ", curses.color_pair(2) if is_active else curses.color_pair(5))
                safe_addstr(stdscr, y, box_x + 5, f"{name}  {desc[:26]}", curses.color_pair(5))

        # Barra inferior de atajos
        help_text = "↵ apply | j/k nav | q quit"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(profiles)
        elif key in [curses.KEY_DOWN, ord('j')]:
            selected_idx = (selected_idx + 1) % len(profiles)
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            target_profile = profiles[selected_idx][0]
            set_profile(target_profile)
            current = target_profile
            break
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)