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

def get_devices():
    devices = []
    try:
        # Obtener dispositivos conocidos / emparejados
        res = subprocess.run(["bluetoothctl", "devices"], capture_output=True, text=True, timeout=2)
        for line in res.stdout.splitlines():
            parts = line.strip().split(" ", 2)
            if len(parts) >= 3 and parts[0] == "Device":
                mac = parts[1]
                name = parts[2]
                
                # Comprobar si está conectado
                info = subprocess.run(["bluetoothctl", "info", mac], capture_output=True, text=True, timeout=1).stdout
                connected = "Connected: yes" in info
                paired = "Paired: yes" in info
                devices.append({
                    "mac": mac,
                    "name": name,
                    "connected": connected,
                    "paired": paired
                })
    except Exception:
        pass
    devices.sort(key=lambda d: not d["connected"])
    return devices

def toggle_connection(dev):
    action = "disconnect" if dev["connected"] else "connect"
    subprocess.run(["bluetoothctl", action, dev["mac"]], capture_output=True)

def toggle_power():
    res = subprocess.run(["bluetoothctl", "show"], capture_output=True, text=True).stdout
    if "Powered: yes" in res:
        subprocess.run(["bluetoothctl", "power", "off"], capture_output=True)
    else:
        subprocess.run(["bluetoothctl", "power", "on"], capture_output=True)

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

    selected_idx = 0
    scroll_offset = 0
    devices = get_devices()

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        if h < 8 or w < 32:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            if stdscr.getch() in [27, ord('q'), ord('Q')]:
                break
            continue

        box_top = 0
        box_h = max(6, h - 1)
        box_w = max(30, w - 2)
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
        title = f" Bluetooth Devices ({len(devices)}) "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Encabezado
        safe_addstr(stdscr, box_top + 1, box_x + 4, "Device Name", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 1, box_x + 24, "Status", curses.color_pair(4) | curses.A_BOLD)

        visible_lines = box_h - 3
        if selected_idx < scroll_offset:
            scroll_offset = selected_idx
        elif selected_idx >= scroll_offset + visible_lines:
            scroll_offset = selected_idx - visible_lines + 1

        for i in range(visible_lines):
            item_idx = scroll_offset + i
            if item_idx >= len(devices):
                break
            d = devices[item_idx]
            y = box_top + 2 + i
            is_hovered = (item_idx == selected_idx)
            is_connected = d["connected"]

            mark = "●" if is_connected else " "
            status = "Connected" if is_connected else ("Paired" if d["paired"] else "Available")
            row_str = f" {mark} {d['name'][:18]:<18}  {status:<10}".ljust(box_w - 4)

            if is_hovered:
                _bar_end = min(box_x + box_w - 1, box_x + 2 + len(row_str))
                for _bx in range(box_x + 2, _bar_end):
                    stdscr.addstr(y, _bx, ' ', curses.color_pair(3))
                safe_addstr(stdscr, y, box_x + 2, row_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                attr = curses.color_pair(2) if is_connected else curses.color_pair(5)
                safe_addstr(stdscr, y, box_x + 2, row_str, attr)

        help_text = "↵ Toggle Conn | p Power | r Scan | q Close"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            if devices:
                selected_idx = (selected_idx - 1) % len(devices)
        elif key in [curses.KEY_DOWN, ord('j')]:
            if devices:
                selected_idx = (selected_idx + 1) % len(devices)
        elif key in [ord('p'), ord('P')]:
            toggle_power()
            devices = get_devices()
        elif key in [ord('r'), ord('R')]:
            devices = get_devices()
            selected_idx = 0
        elif key in [10, 13, curses.KEY_ENTER]:
            if devices:
                toggle_connection(devices[selected_idx])
                devices = get_devices()
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)
