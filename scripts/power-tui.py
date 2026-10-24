#!/usr/bin/env python3
import curses
import os
import subprocess

MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

def get_current_mode():
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return "dark"

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

def execute_action(action_id):
    if action_id == "lock":
        subprocess.Popen(["hyprlock"])
    elif action_id == "sleep":
        subprocess.Popen(["bash", "-c", "hyprlock & sleep 0.5 && systemctl suspend"])
    elif action_id == "logout":
        subprocess.run(["hyprctl", "dispatch", "exit"])
    elif action_id == "reboot":
        subprocess.run(["systemctl", "reboot"])
    elif action_id == "shutdown":
        subprocess.run(["systemctl", "poweroff"])
    elif action_id == "hibernate":
        subprocess.Popen(["bash", "-c", "hyprlock & sleep 0.5 && systemctl hibernate"])

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

    actions = [
        ("lock",      "l", "Lock Screen",   "Bloquear pantalla"),
        ("sleep",     "s", "Sleep / Suspend","Suspender equipo"),
        ("logout",    "e", "Logout Session","Cerrar sesión"),
        ("reboot",    "r", "Reboot System", "Reiniciar"),
        ("shutdown",  "p", "Power Off",     "Apagar el sistema"),
        ("hibernate", "h", "Hibernate",     "Hibernar"),
    ]

    selected_idx = 0

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        if h < 8 or w < 30:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            key = stdscr.getch()
            if key in [27, ord('q'), ord('Q')]:
                break
            continue

        box_top = 0
        box_h = max(7, h - 1)
        box_w = max(28, w - 2)
        box_x = 1

        # Marco continuo
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
        title = " Session & Power Control "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Encabezado de columnas alineado
        safe_addstr(stdscr, box_top + 2, box_x + 7, "Action", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 2, box_x + 24, "Description", curses.color_pair(4) | curses.A_BOLD)

        # Opciones dentro del marco (con 1 espacio de margen a cada lado)
        inner_w = box_w - 4
        for i, (action_id, key_hint, label, desc) in enumerate(actions):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)

            # 15 caracteres para la etiqueta (ej. "Sleep / Suspend") + 2 espacios
            prefix = f" [{key_hint}] "
            label_col = f"{label:<15}  "
            max_desc = max(0, inner_w - len(prefix) - len(label_col))
            desc_clipped = desc[:max_desc]

            line_str = f"{prefix}{label_col}{desc_clipped}".ljust(inner_w)[:inner_w]

            if is_hovered:
                safe_addstr(stdscr, y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                safe_addstr(stdscr, y, box_x + 2, prefix, curses.color_pair(2) | curses.A_BOLD)
                label_color = curses.color_pair(4) if action_id in ["reboot", "shutdown"] else curses.color_pair(5)
                safe_addstr(stdscr, y, box_x + 2 + len(prefix), label_col, label_color)
                safe_addstr(stdscr, y, box_x + 2 + len(prefix) + len(label_col), desc_clipped, curses.color_pair(5))

        # Footer inferior compacto que respeta las esquinas del marco
        help_text = "↵ run | i/k nav | q quit"
        footer_str = f" {help_text[:inner_w - 2]} "
        footer_x = max(box_x + 2, box_x + (box_w - len(footer_str)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, footer_str, curses.color_pair(2))
        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('i')]:
            selected_idx = (selected_idx - 1) % len(actions)
        elif key in [curses.KEY_DOWN, ord('k')]:
            selected_idx = (selected_idx + 1) % len(actions)
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            execute_action(actions[selected_idx][0])
            break
        elif key in [27, ord('q'), ord('Q')]:
            break
        # Atajos de una sola tecla (Hotkeys)
        else:
            char = chr(key).lower() if 0 <= key <= 255 else ''
            matched = False
            for act_id, hint, _, _ in actions:
                if char == hint:
                    execute_action(act_id)
                    matched = True
                    break
            if matched:
                break

if __name__ == "__main__":
    curses.wrapper(main)
