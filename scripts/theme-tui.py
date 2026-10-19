#!/usr/bin/env python3
import curses
import os
import subprocess

THEME_SCRIPT = os.path.expanduser("~/hyprland-from-scratch/scripts/theme-manager.sh")
PALETTE_FILE = os.path.expanduser("~/.config/hypr/current_theme")

MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

def get_current_mode():
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return "dark"

PALETTES = [
    ("nord",         "Nord Ice",       "Azul ártico / Frío"),
    ("catppuccin",   "Catppuccin",     "Mocha pastel / Suave"),
    ("tokyo-night",  "Tokyo Night",    "Azul neón / Nocturno"),
    ("gruvbox",      "Gruvbox",        "Tierra retro / Cálido"),
    ("arch-cyan",    "Arch Cyan",      "Azul Arch oficial"),
    ("arch-purple",  "Arch Purple",    "Violeta clásico / Neutra"),
]

def get_state():
    palette = "nord"
    mode = "dark"
    if os.path.exists(PALETTE_FILE):
        try:
            with open(PALETTE_FILE, "r") as f:
                palette = f.read().strip() or "nord"
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
    subprocess.run([THEME_SCRIPT, palette_id], check=False)

def toggle_mode():
    subprocess.run([THEME_SCRIPT, "--mode", "toggle"], check=False)

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

    selected_idx = 0

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()
        current_palette, current_mode = get_state()

        # Generar lista de opciones dinámicas (Fila 0 = Switch Modo; Filas siguientes = Paletas)
        mode_label = f"Modo: {'☀ CLARO' if current_mode == 'light' else '🌙 OSCURO'}"
        mode_desc = f"Cambiar a {'Modo Oscuro' if current_mode == 'light' else 'Modo Claro'} [Tecla m]"
        
        menu_items = [("__toggle_mode__", mode_label, mode_desc)] + PALETTES

        if selected_idx >= len(menu_items):
            selected_idx = len(menu_items) - 1

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
        title = f" System Appearance [{current_palette.capitalize()} · {current_mode.upper()}] "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Opciones
        for i, (item_id, name, desc) in enumerate(menu_items):
            y = box_top + 2 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            
            if item_id == "__toggle_mode__":
                col1 = f" ⏻ {name:<14}"
                col2 = f" {desc}"
            else:
                is_active = (item_id == current_palette)
                mark = "●" if is_active else " "
                col1 = f" {mark} {name:<14}"
                col2 = f" {desc}"

            row_str = (col1 + col2).ljust(box_w - 4)

            if is_hovered:
                safe_addstr(stdscr, y, box_x + 2, row_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                if item_id == "__toggle_mode__":
                    safe_addstr(stdscr, y, box_x + 2, col1, curses.color_pair(4) | curses.A_BOLD)
                    safe_addstr(stdscr, y, box_x + 2 + len(col1), col2, curses.color_pair(5))
                else:
                    is_active = (item_id == current_palette)
                    attr = curses.color_pair(2) if is_active else curses.color_pair(5)
                    safe_addstr(stdscr, y, box_x + 2, col1, attr)
                    safe_addstr(stdscr, y, box_x + 2 + len(col1), col2, curses.color_pair(5))

        # Atajos
        help_text = "↵ Apply | m Toggle Mode | j/k Nav | q Close"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(menu_items)
        elif key in [curses.KEY_DOWN, ord('j')]:
            selected_idx = (selected_idx + 1) % len(menu_items)
        elif key in [ord('m'), ord('M')]:
            toggle_mode()
        elif key in [10, 13, curses.KEY_ENTER]:
            item_id = menu_items[selected_idx][0]
            if item_id == "__toggle_mode__":
                toggle_mode()
            else:
                apply_palette(item_id)
                break
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)
