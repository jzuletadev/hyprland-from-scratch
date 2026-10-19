#!/usr/bin/env python3
import curses
import json
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

MODS = [(64, "SUPER"), (4, "CTRL"), (8, "ALT"), (1, "SHIFT")]
KEY_NAMES = {
    "slash": "/", "Escape": "Esc", "left": "Left", "right": "Right",
    "up": "Up", "down": "Down", "mouse:272": "left-drag", "mouse:273": "right-drag"
}

EXTRA = [
    ("SUPER + 1...9, 0", "Go to workspace 1-10", None, None),
    ("SUPER + SHIFT + 1...9, 0", "Move window to workspace 1-10 (follow)", None, None),
    ("SUPER + ALT + 1...9, 0", "Send window to workspace 1-10 (stay)", None, None),
    ("3 fingers down / up", "Minimize all windows here / restore them", None, None),
    ("3 fingers left / right", "Focus window on the left / right", None, None),
    ("4 fingers down / up", "Volume down / up, 10% per swipe", None, None),
    ("4 fingers left / right", "Previous / next workspace", None, None),
    ("3 fingers spread / pinch", "Fullscreen on / off", None, None),
    ("Brightness / volume keys", "Laptop brightness, volume, mute", None, None),
    ("Click outside a menu", "Close the menu", None, None),
    ("Bar: Arch logo", "Click: system info / right-click: settings", None, None),
]

def parse_bind_key(bind):
    if bind.get("key") == "Super_L" and bind.get("release"):
        return "SUPER (tap)"
    mods = [name for bit, name in MODS if bind.get("modmask", 0) & bit]
    key_raw = bind.get("key", "")
    key = KEY_NAMES.get(key_raw, key_raw.upper() if len(key_raw) == 1 else key_raw)
    return " + ".join(mods + [key])

def get_binds():
    items = []
    try:
        res = subprocess.run(["hyprctl", "binds", "-j"], capture_output=True, text=True, check=True)
        raw_binds = json.loads(res.stdout)
        for b in raw_binds:
            if b.get("has_description"):
                combo = parse_bind_key(b)
                desc = b.get("description", "")
                dispatcher = b.get("dispatcher", "") if not b.get("mouse") else None
                arg = b.get("arg", "")
                items.append((combo, desc, dispatcher, arg))
    except Exception:
        pass

    for combo, desc, disp, arg in EXTRA:
        items.append((combo, desc, disp, arg))
    return items

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

    all_binds = get_binds()
    filter_text = ""
    selected_idx = 0
    scroll_offset = 0

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        if h < 8 or w < 35:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            key = stdscr.getch()
            if key in [27, ord('q'), ord('Q')]:
                break
            continue

        if filter_text:
            query = filter_text.lower()
            filtered = [b for b in all_binds if query in b[0].lower() or query in b[1].lower()]
        else:
            filtered = all_binds

        if not filtered:
            selected_idx = 0
        elif selected_idx >= len(filtered):
            selected_idx = len(filtered) - 1

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
        title = " Hyprland Keybindings "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Barra de búsqueda
        search_prompt = f"Filter: {filter_text}_" if filter_text else "Type to filter..."
        safe_addstr(stdscr, box_top + 1, box_x + 3, search_prompt, curses.color_pair(4) | curses.A_BOLD)

        visible_lines = box_h - 3
        if selected_idx < scroll_offset:
            scroll_offset = selected_idx
        elif selected_idx >= scroll_offset + visible_lines:
            scroll_offset = selected_idx - visible_lines + 1

        col1_w = min(28, max(18, box_w // 3))

        for i in range(visible_lines):
            item_idx = scroll_offset + i
            if item_idx >= len(filtered):
                break
            combo, desc, disp, _ = filtered[item_idx]
            y = box_top + 2 + i
            is_hovered = (item_idx == selected_idx)

            col1_str = f" {combo:<{col1_w - 2}}"
            col2_str = f" {desc}"

            if is_hovered:
                full_row = (col1_str + col2_str).ljust(box_w - 4)
                safe_addstr(stdscr, y, box_x + 2, full_row, curses.color_pair(3) | curses.A_BOLD)
            else:
                key_color = curses.color_pair(2) | curses.A_BOLD if disp else curses.color_pair(5)
                desc_color = curses.color_pair(5) if disp else curses.color_pair(5) | curses.A_DIM

                safe_addstr(stdscr, y, box_x + 2, col1_str[:col1_w], key_color)
                safe_addstr(stdscr, y, box_x + 2 + col1_w, col2_str, desc_color)

        # Footer con atajos
        help_text = "↵ Run bind | j/k Nav | Backspace Clear | q Close"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            if filtered:
                selected_idx = (selected_idx - 1) % len(filtered)
        elif key in [curses.KEY_DOWN, ord('j')]:
            if filtered:
                selected_idx = (selected_idx + 1) % len(filtered)
        elif key in [10, 13, curses.KEY_ENTER]:
            if filtered:
                _, _, disp, arg = filtered[selected_idx]
                if disp:
                    subprocess.Popen(["hyprctl", "dispatch", disp, arg])
                break
        elif key in [curses.KEY_BACKSPACE, 127, 8]:
            if filter_text:
                filter_text = filter_text[:-1]
                selected_idx = 0
        elif key in [27, ord('q'), ord('Q')]:
            break
        elif 32 <= key <= 126:
            filter_text += chr(key)
            selected_idx = 0

if __name__ == "__main__":
    curses.wrapper(main)
