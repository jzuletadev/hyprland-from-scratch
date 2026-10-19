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
    # Esta función corre fuera de la terminal una vez cerrada la ventana
    time.sleep(0.25)  # Da tiempo a que Hyprland retire la ventana de la pantalla
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

def tui_menu(stdscr):
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

        box_top = 0
        box_h = max(8, h - 2)
        box_w = max(30, w - 2)
        box_x = 1

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

        title = " Screen Recording "
        stdscr.addstr(box_top, box_x + 2, title, curses.color_pair(1) | curses.A_BOLD)
        stdscr.attroff(curses.color_pair(1))

        stdscr.addstr(box_top + 1, box_x + 4, "Mode", curses.color_pair(4) | curses.A_BOLD)
        stdscr.addstr(box_top + 1, box_x + 20, "Description", curses.color_pair(4) | curses.A_BOLD)

        for i, (m, a, name, desc) in enumerate(options):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            mark = "●" if is_hovered else " "

            line_str = f" {mark} {name:<14}  {desc[:22]:<22} "
            if is_hovered:
                stdscr.addstr(y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                stdscr.addstr(y, box_x + 2, f"   {name:<14}  {desc[:22]}", curses.color_pair(5))

        help_text = "↵ Record | j,↓ Down | k,↑ Up | q Close"
        footer_y = h - 1
        stdscr.addstr(footer_y, max(1, (w - len(help_text)) // 2), help_text, curses.color_pair(2))

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
    # 1. Modo Stop (llamado desde Waybar o segundo toque del atajo)
    if len(sys.argv) > 1 and sys.argv[1] == "stop":
        stop_recording()
        return

    # 2. Modo Backend (ejecutado en segundo plano tras cerrar la ventana)
    if len(sys.argv) > 2 and sys.argv[1] == "--start-backend":
        mode = sys.argv[2]
        audio = (sys.argv[3] == "true") if len(sys.argv) > 3 else False
        run_backend_recording(mode, audio)
        return

    # 3. Si ya está grabando, detenerlo inmediatamente
    if is_recording():
        stop_recording()
        return

    # 4. Mostrar TUI modal
    mode, audio = curses.wrapper(tui_menu)

    # Si se seleccionó una opción, cerrar la ventana y disparar la grabación desacoplada
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
