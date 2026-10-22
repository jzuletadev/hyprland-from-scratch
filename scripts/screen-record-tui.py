#!/usr/bin/env python3
import curses
import subprocess
import os
import sys
import time
from datetime import datetime

MODE_FILE = os.path.expanduser("~/.config/hypr/current_mode")

def get_current_mode():
    if os.path.exists(MODE_FILE):
        try:
            with open(MODE_FILE, "r") as f:
                return f.read().strip()
        except Exception:
            pass
    return "dark"

DIR = os.path.expanduser("~/Videos/recordings")
CURRENT_PATH_FILE = os.path.join(os.environ.get("XDG_RUNTIME_DIR", "/tmp"), "screen-record.path")

def is_recording():
    res = subprocess.run(["pgrep", "-x", "wf-recorder"], capture_output=True)
    return res.returncode == 0

def notify(msg):
    subprocess.run(["notify-send", "-a", "Screen recording", "Screen recording", msg], check=False)

def refresh_waybar():
    subprocess.run(["pkill", "-RTMIN+9", "-x", "waybar"], check=False)

def stop_recording():
    if not is_recording():
        return
    subprocess.run(["pkill", "-INT", "-x", "wf-recorder"], check=False)
    while is_recording():
        time.sleep(0.1)
    refresh_waybar()

    saved_file = ""
    if os.path.exists(CURRENT_PATH_FILE):
        with open(CURRENT_PATH_FILE, "r") as f:
            saved_file = f.read().strip()
    notify(f"Guardado en {saved_file}")

def get_focused_monitor():
    try:
        import json
        res = subprocess.run(["hyprctl", "monitors", "-j"], capture_output=True, text=True)
        monitors = json.loads(res.stdout)
        for m in monitors:
            if m.get("focused", False):
                return m.get("name", "eDP-1")
    except Exception:
        pass
    return "eDP-1"

def get_default_sink():
    try:
        res = subprocess.run(["pactl", "get-default-sink"], capture_output=True, text=True)
        sink = res.stdout.strip()
        if sink:
            return sink
    except Exception:
        pass
    return ""

def run_backend_recording(mode, audio):
    time.sleep(0.25)
    os.makedirs(DIR, exist_ok=True)
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    filepath = os.path.join(DIR, f"{timestamp}.mp4")

    args = ["wf-recorder", "-f", filepath, "-p", "preset=veryfast", "-p", "crf=23"]

    if mode == "region":
        slurp_res = subprocess.run(["slurp"], capture_output=True, text=True)
        geo = slurp_res.stdout.strip()
        if not geo:
            return
        args.extend(["-g", geo])
    else:
        focused = get_focused_monitor()
        args.extend(["-o", focused])

    if audio:
        sink = get_default_sink()
        if sink:
            args.append(f"--audio={sink}.monitor")

    with open(CURRENT_PATH_FILE, "w") as f:
        f.write(filepath)

    subprocess.Popen(
        args,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
        close_fds=True
    )
    time.sleep(0.5)
    if is_recording():
        refresh_waybar()
    else:
        notify("No se pudo iniciar la grabación con wf-recorder")

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

def tui_menu(stdscr):
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

    options = [
        ("screen", False, "Screen",         "Pantalla completa"),
        ("screen", True,  "Screen + Audio", "Pantalla con audio"),
        ("region", False, "Region",         "Área rectangular"),
        ("region", True,  "Region + Audio", "Área con audio"),
    ]

    selected_idx = 0

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()

        if h < 8 or w < 30:
            safe_addstr(stdscr, 0, 0, "Ventana muy pequeña", curses.color_pair(4))
            stdscr.refresh()
            if stdscr.getch() in [27, ord('q'), ord('Q')]:
                return None, None
            continue

        box_top = 0
        box_h = max(7, h - 1)
        box_w = max(28, w - 2)
        box_x = 1

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

        title = " Screen Recording "
        safe_addstr(stdscr, box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)

        safe_addstr(stdscr, box_top + 1, box_x + 4, "Mode", curses.color_pair(4) | curses.A_BOLD)
        safe_addstr(stdscr, box_top + 1, box_x + 20, "Description", curses.color_pair(4) | curses.A_BOLD)

        for i, (m, a, name, desc) in enumerate(options):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            mark = "●" if is_hovered else " "

            line_str = f" {mark} {name:<14}  {desc[:22]:<22} "
            if is_hovered:
                _bar_end = min(box_x + box_w - 1, box_x + 2 + len(line_str))
                for _bx in range(box_x + 2, _bar_end):
                    stdscr.addstr(y, _bx, ' ', curses.color_pair(3))
                safe_addstr(stdscr, y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                safe_addstr(stdscr, y, box_x + 2, f"   {name:<14}  {desc[:22]}", curses.color_pair(5))

        help_text = "↵ Record | j/k Nav | q Close"
        footer_x = max(box_x + 2, (w - len(help_text)) // 2)
        safe_addstr(stdscr, box_top + box_h - 1, footer_x, f" {help_text} ", curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(options)
        elif key in [curses.KEY_DOWN, ord('j')]:
            selected_idx = (selected_idx + 1) % len(options)
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            return options[selected_idx][0], options[selected_idx][1]
        elif key in [27, ord('q'), ord('Q')]:
            return None, None

def main():
    if len(sys.argv) > 1 and sys.argv[1] == "stop":
        stop_recording()
        return

    if len(sys.argv) > 2 and sys.argv[1] == "--start-backend":
        mode = sys.argv[2]
        audio = (sys.argv[3] == "true") if len(sys.argv) > 3 else False
        run_backend_recording(mode, audio)
        return

    if is_recording():
        stop_recording()
        return

    mode, audio = curses.wrapper(tui_menu)

    if mode is not None:
        script_path = os.path.abspath(__file__)
        subprocess.Popen(
            [sys.executable, script_path, "--start-backend", mode, "true" if audio else "false"],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            start_new_session=True,
            close_fds=True
        )

if __name__ == "__main__":
    main()