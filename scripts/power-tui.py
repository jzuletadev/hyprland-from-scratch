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

    actions = [
        ("lock",      "l", "Lock Screen",   "Bloquear pantalla con hyprlock"),
        ("sleep",     "s", "Sleep / Suspend","Suspender equipo (RAM)"),
        ("logout",    "e", "Logout Session","Cerrar sesión de Hyprland"),
        ("reboot",    "r", "Reboot System", "Reiniciar el equipo"),
        ("shutdown",  "p", "Power Off",     "Apagar el sistema por completo"),
        ("hibernate", "h", "Hibernate",     "Guardar estado en swap y apagar"),
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

        # Opciones
        for i, (action_id, key_hint, label, desc) in enumerate(actions):
            y = box_top + 2 + i
            if y >= box_top + box_h - 2:
                break
            is_hovered = (i == selected_idx)

            line_str = f" [{key_hint}] {label:<16} {desc}"
            if is_hovered:
                safe_addstr(stdscr, y, box_x + 2, line_str.ljust(box_w - 4), curses.color_pair(3) | curses.A_BOLD)
            else:
                # Resaltar la tecla de acceso rápido
                prefix = f" [{key_hint}] "
                safe_addstr(stdscr, y, box_x + 2, prefix, curses.color_pair(2) | curses.A_BOLD)
                safe_addstr(stdscr, y, box_x + 2 + len(prefix), f"{label:<16}", curses.color_pair(4) if action_id in ["reboot", "shutdown"] else curses.color_pair(5))
                safe_addstr(stdscr, y, box_x + 2 + len(prefix) + 17, desc, curses.color_pair(5))

        # Footer inferior
        help_text = "↵ Execute | Hotkeys (l/s/e/r/p) | j/k Nav | q Cancel"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(actions)
        elif key in [curses.KEY_DOWN, ord('j')]:
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
