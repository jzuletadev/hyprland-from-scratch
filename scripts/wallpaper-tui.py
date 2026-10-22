#!/usr/bin/env python3
import curses
import os
import subprocess

WALLPAPER_DIR = os.path.expanduser("~/Pictures/wallpapers")
CURRENT_LINK = os.path.expanduser("~/.local/state/hyprland-from-scratch/wallpaper")
WALLPAPER_SCRIPT = os.path.expanduser("~/hyprland-from-scratch/scripts/wallpaper.sh")

MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

def get_current_mode():
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return "dark"

IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".gif", ".bmp"}

def get_images():
    if not os.path.isdir(WALLPAPER_DIR):
        return []
    items = []
    try:
        for f in sorted(os.listdir(WALLPAPER_DIR)):
            ext = os.path.splitext(f)[1].lower()
            full_path = os.path.join(WALLPAPER_DIR, f)
            if ext in IMAGE_EXTS and os.path.isfile(full_path):
                items.append((f, full_path))
    except Exception:
        pass
    return items

def get_current_wallpaper():
    if os.path.islink(CURRENT_LINK) or os.path.exists(CURRENT_LINK):
        try:
            return os.path.realpath(CURRENT_LINK)
        except Exception:
            pass
    return ""

def apply_wallpaper(path):
    subprocess.run([WALLPAPER_SCRIPT, "set", path], check=False)

def safe_addstr(win, y, x, text, attr=0):
    """Escribe texto en pantalla asegurando no desbordar las dimensiones."""
    try:
        max_y, max_x = win.getmaxyx()
        if y < 0 or y >= max_y or x < 0 or x >= max_x:
            return
        # Reservar al menos 1 celda al final para evitar desbordamiento en curses
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

    all_images = get_images()
    current_path = get_current_wallpaper()

    if not all_images:
        safe_addstr(stdscr, 1, 2, "No se encontraron imagenes en ~/Pictures/wallpapers", curses.color_pair(4))
        safe_addstr(stdscr, 3, 2, "Presiona cualquier tecla para salir...", curses.color_pair(2))
        stdscr.getch()
        return

    selected_idx = 0
    for idx, (_, path) in enumerate(all_images):
        if path == current_path:
            selected_idx = idx
            break

    filter_text = ""
    scroll_offset = 0

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        # Si la ventana es demasiado chica, evitar dibujar
        if h < 6 or w < 20:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequena", curses.color_pair(4))
            stdscr.refresh()
            key = stdscr.getch()
            if key in [27, ord('q'), ord('Q')]:
                break
            continue

        if filter_text:
            filtered = [(name, path) for (name, path) in all_images if filter_text.lower() in name.lower()]
        else:
            filtered = all_images

        if not filtered:
            selected_idx = 0
        elif selected_idx >= len(filtered):
            selected_idx = len(filtered) - 1

        box_top = 0
        box_h = max(5, h - 1)
        box_w = max(20, w - 2)
        box_x = 1

        # Marco exterior
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
        current_name = os.path.basename(current_path) if current_path else "none"
        title = f" Wallpapers [{current_name[:16]}] "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        # Barra de filtro
        filter_display = f"Filter: {filter_text}_" if filter_text else "Type to filter..."
        safe_addstr(stdscr, box_top + 1, box_x + 3, filter_display, curses.color_pair(4) | curses.A_BOLD)

        visible_lines = box_h - 3
        if selected_idx < scroll_offset:
            scroll_offset = selected_idx
        elif selected_idx >= scroll_offset + visible_lines:
            scroll_offset = selected_idx - visible_lines + 1

        # Lista de fondos
        for i in range(visible_lines):
            item_idx = scroll_offset + i
            if item_idx >= len(filtered):
                break
            name, path = filtered[item_idx]
            y = box_top + 2 + i
            is_hovered = (item_idx == selected_idx)
            is_active = (path == current_path)

            mark = "●" if is_active else " "
            display_name = f" {mark} {name}"

            if is_hovered:
                _bar_end = min(box_x + box_w - 1, box_x + 2 + len(display_name.ljust(box_w - 4)))
                for _bx in range(box_x + 2, _bar_end):
                    stdscr.addstr(y, _bx, ' ', curses.color_pair(3))
                safe_addstr(stdscr, y, box_x + 2, display_name.ljust(box_w - 4), curses.color_pair(3) | curses.A_BOLD)
            else:
                attr = curses.color_pair(2) if is_active else curses.color_pair(5)
                safe_addstr(stdscr, y, box_x + 2, display_name.ljust(box_w - 4), attr)

        # Barra de ayuda segura
        help_text = "↵ Set | p Preview | j/k Nav | q Close"
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
                chosen_path = filtered[selected_idx][1]
                apply_wallpaper(chosen_path)
                break
        elif key in [ord('p'), ord('P')]:
            if filtered:
                chosen_path = filtered[selected_idx][1]
                apply_wallpaper(chosen_path)
                current_path = chosen_path
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
