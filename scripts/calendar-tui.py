#!/usr/bin/env python3
import calendar
import curses
import datetime
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

    today = datetime.date.today()
    view_year = today.year
    view_month = today.month

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        if h < 11 or w < 32:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            if stdscr.getch() in [27, ord('q'), ord('Q')]:
                break
            continue

        # Ocupar todo el espacio de la ventana (460x310) como el resto de tus TUIs
        box_top = 0
        box_h = max(11, h - 1)
        box_w = max(38, w - 2)
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

        # Título con Mes y Año centrado en el borde superior
        month_name = calendar.month_name[view_month]
        title = f" {month_name} {view_year} "
        safe_addstr(stdscr, box_top, box_x + max(2, (box_w - len(title)) // 2), title, curses.color_pair(1) | curses.A_BOLD)

        # Matriz del mes y cálculo de centrado vertical
        cal_matrix = calendar.monthcalendar(view_year, view_month)
        num_weeks = len(cal_matrix)
        # Altura del contenido: 1 (reloj) + 1 (espacio) + 1 (cabecera días) + num_weeks
        content_h = 3 + num_weeks
        inner_h = box_h - 2
        start_y = box_top + 1 + max(0, (inner_h - content_h) // 2)

        # Horario actual local centrado
        now_str = f"── {datetime.datetime.now().strftime('%H:%M:%S')} ──"
        safe_addstr(stdscr, start_y, box_x + max(2, (box_w - len(now_str)) // 2), now_str, curses.color_pair(4) | curses.A_BOLD)

        # Días de la semana espaciados (5 columnas por día -> 35 caracteres en total)
        col_w = 5 if box_w >= 40 else 4
        days_names = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
        days_header = "".join(f"{d:^{col_w}}" for d in days_names)
        grid_w = len(days_header)
        hx = box_x + max(2, (box_w - grid_w) // 2)

        header_y = start_y + 2
        safe_addstr(stdscr, header_y, hx, days_header, curses.color_pair(2) | curses.A_BOLD)

        # Dibujar números de los días
        for r_idx, week in enumerate(cal_matrix):
            row_y = header_y + 1 + r_idx
            if row_y >= box_top + box_h - 1:
                break
            for d_idx, day in enumerate(week):
                if day == 0:
                    continue
                col_x = hx + (d_idx * col_w)
                day_cell = f"{day:^{col_w}}"
                is_today = (view_year == today.year and view_month == today.month and day == today.day)
                if is_today:
                    safe_addstr(stdscr, row_y, col_x, day_cell, curses.color_pair(3) | curses.A_BOLD)
                else:
                    safe_addstr(stdscr, row_y, col_x, day_cell, curses.color_pair(5))

        # Footer inferior centrado en el marco
        help_text = "j/l prev/next | t today | q quit"
        footer_str = f" {help_text} "
        footer_x = box_x + max(2, (box_w - len(footer_str)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, footer_str, curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_LEFT, ord('j')]:
            view_month -= 1
            if view_month < 1:
                view_month = 12
                view_year -= 1
        elif key in [curses.KEY_RIGHT, ord('l')]:
            view_month += 1
            if view_month > 12:
                view_month = 1
                view_year += 1
        elif key in [ord('t'), ord('T')]:
            view_year = today.year
            view_month = today.month
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)
