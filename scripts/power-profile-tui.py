#!/usr/bin/env python3
import curses
import subprocess
import sys
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

def main(stdscr):
    curses.curs_set(0)
    curses.use_default_colors()
    
    mode = get_current_mode()
    is_light = (mode == "light")

    if is_light:
        # Colores contrastados para fondo claro
        curses.init_pair(1, curses.COLOR_GREEN, -1)   # Marco
        curses.init_pair(2, curses.COLOR_BLUE, -1)    # Ayuda / Conectado
        curses.init_pair(3, curses.COLOR_WHITE, curses.COLOR_MAGENTA) # Cursor activo
        curses.init_pair(4, curses.COLOR_BLUE, -1)    # Encabezados (Azul oscuro legible)
        curses.init_pair(5, curses.COLOR_BLACK, -1)   # Texto normal (NEGRO)
    else:
        # Colores para fondo oscuro
        curses.init_pair(1, curses.COLOR_GREEN, -1)   # Marco
        curses.init_pair(2, curses.COLOR_CYAN, -1)    # Ayuda / Conectado
        curses.init_pair(3, curses.COLOR_WHITE, curses.COLOR_MAGENTA) # Cursor activo
        curses.init_pair(4, curses.COLOR_YELLOW, -1)  # Encabezados
        curses.init_pair(5, curses.COLOR_WHITE, -1)   # Texto normal (BLANCO)

    profiles = [
        ("performance", "Performance ", "Máximo rendimiento / Mayor consumo"),
        ("balanced",    "Balanced    ", "Equilibrado / Recomendado"),
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
        
        # 1. Caja principal superior: "Power Management"
        box_top = 1
        box_h = 9
        box_w = min(w - 2, 54)
        box_x = max(1, (w - box_w) // 2)

        # Dibujar marco de caja
        stdscr.attron(curses.color_pair(1))
        for x in range(box_x, box_x + box_w):
            stdscr.addch(box_top, x, curses.ACS_HLINE)
            stdscr.addch(box_top + box_h - 1, x, curses.ACS_HLINE)
        for y in range(box_top, box_top + box_h):
            stdscr.addch(y, box_x, curses.ACS_VLINE)
            stdscr.addch(y, box_x + box_w - 1, curses.ACS_VLINE)
        stdscr.addch(box_top, box_x, curses.ACS_ULCORNER)
        stdscr.addch(box_top, box_x + box_w - 1, curses.ACS_URCORNER)
        stdscr.addch(box_top + box_h - 1, box_x, curses.ACS_LLCORNER)
        stdscr.addch(box_top + box_h - 1, box_x + box_w - 1, curses.ACS_LRCORNER)

        # Título en el borde superior
        title = " Power Profiles "
        stdscr.addstr(box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)
        stdscr.attroff(curses.color_pair(1))

        # Encabezado de columnas
        stdscr.addstr(box_top + 1, box_x + 4, "Mode", curses.color_pair(4) | curses.A_BOLD)
        stdscr.addstr(box_top + 1, box_x + 20, "Status / Description", curses.color_pair(4) | curses.A_BOLD)

        # Opciones
        for i, (p_id, name, desc) in enumerate(profiles):
            y = box_top + 3 + i
            is_active = (p_id == current)
            mark = "●" if is_active else " "
            
            line_str = f" {mark} {name}  {desc[:26]:<26} "
            if i == selected_idx:
                stdscr.addstr(y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                stdscr.addstr(y, box_x + 2, f" {mark} ", curses.color_pair(2) if is_active else curses.color_pair(5))
                stdscr.addstr(y, box_x + 5, f"{name}  {desc[:26]}", curses.color_pair(5))

        # Barra inferior de atajos (estilo impala)
        footer_y = box_top + box_h + 1
        if footer_y < h:
            help_text = "↵ Apply | q/Esc Close | j,↓ Down | k,↑ Up"
            stdscr.addstr(footer_y, max(1, (w - len(help_text)) // 2), help_text, curses.color_pair(2))

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
