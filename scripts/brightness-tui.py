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

DEVICE = "amdgpu_bl1"

def get_brightness():
    try:
        res = subprocess.run(["brightnessctl", "-d", DEVICE, "-m"], capture_output=True, text=True, timeout=1)
        parts = res.stdout.strip().split(",")
        if len(parts) >= 4:
            return int(parts[3].replace("%", ""))
    except Exception:
        pass
    return 50

def set_brightness(pct):
    pct = max(1, min(100, pct))
    subprocess.run(["brightnessctl", "-d", DEVICE, "set", f"{pct}%"], capture_output=True, text=True)
    return pct

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
        # Colores contrastados para fondo claro
        curses.init_pair(1, curses.COLOR_GREEN, -1)   # Marco
        curses.init_pair(2, ACCENT_COLOR, -1)    # Ayuda / Conectado
        curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
        curses.init_pair(4, curses.COLOR_BLUE, -1)    # Encabezados (Azul oscuro legible)
        curses.init_pair(5, curses.COLOR_BLACK, -1)   # Texto normal (NEGRO)
    else:
        # Colores para fondo oscuro
        curses.init_pair(1, curses.COLOR_GREEN, -1)   # Marco
        curses.init_pair(2, ACCENT_COLOR, -1)    # Ayuda / Conectado
        curses.init_pair(3, HOVER_FG, HOVER_BG) # Cursor activo
        curses.init_pair(4, curses.COLOR_YELLOW, -1)  # Encabezados
        curses.init_pair(5, curses.COLOR_WHITE, -1)   # Texto normal (BLANCO)
    
    presets = [
        (100, "100%", "Máximo / Exterior"),
        (75,  " 75%", "Alto / Día brillante"),
        (50,  " 50%", "Medio / Oficina"),
        (25,  " 25%", "Bajo / Luz tenue"),
        (10,  " 10%", "Mínimo / Noche"),
    ]

    current_val = get_brightness()
    diffs = [abs(p[0] - current_val) for p in presets]
    selected_idx = diffs.index(min(diffs))

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

        # Marco continuo protegido
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

        # Título
        title = f" Brightness Control [{current_val}%] "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Encabezados alineados con las columnas
        safe_addstr(stdscr, box_top + 1, box_x + 5, "Level", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 1, box_x + 16, "Preset / Context", curses.color_pair(4) | curses.A_BOLD)

        # Opciones con barra hover de extremo a extremo (con 1 espacio de margen)
        inner_w = box_w - 4
        for i, (pct, name, desc) in enumerate(presets):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            mark = "●" if is_hovered else " "
            
            # 3 chars (" ● ") + 11 chars (name:<11) = 14 chars -> empieza en box_x + 2 + 14 = box_x + 16
            line_str = f" {mark} {name:<11}{desc}".ljust(inner_w)
            if is_hovered:
                safe_addstr(stdscr, y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                safe_addstr(stdscr, y, box_x + 2, line_str, curses.color_pair(5))

        # Barra de progreso visual en el espacio inferior si hay altura suficiente
        bar_y = box_top + 3 + len(presets) + 1
        if bar_y < box_top + box_h - 1:
            bar_max_w = inner_w - 8  # Espacio para la barra descontando etiqueta
            if bar_max_w > 5:
                filled = int(round((current_val / 100.0) * bar_max_w))
                empty = bar_max_w - filled
                bar_visual = f" [{'█' * filled}{'░' * empty}] {current_val:>3}%"
                safe_addstr(stdscr, bar_y, box_x + 2, bar_visual, curses.color_pair(2) | curses.A_BOLD)

        # Barra de atajos segura
        help_text = "↵ Set | ←/h -5% | →/l +5% | q Close"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))
        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(presets)
        elif key in [curses.KEY_DOWN, ord('j')]:
            selected_idx = (selected_idx + 1) % len(presets)
        elif key in [curses.KEY_LEFT, ord('h')]:
            current_val = set_brightness(current_val - 5)
        elif key in [curses.KEY_RIGHT, ord('l')]:
            current_val = set_brightness(current_val + 5)
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            set_brightness(presets[selected_idx][0])
            break
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)
