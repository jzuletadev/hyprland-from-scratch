#!/usr/bin/env python3
import curses
import subprocess
import re

def get_volume_info():
    try:
        res = subprocess.run(["wpctl", "get-volume", "@DEFAULT_AUDIO_SINK@"], capture_output=True, text=True, timeout=1)
        out = res.stdout.strip()
        is_muted = "MUTED" in out.upper()
        
        match = re.search(r'(\d+\.\d+)', out)
        if match:
            vol = int(float(match.group(1)) * 100)
            return vol, is_muted
        
        match_pct = re.search(r'(\d+)%', out)
        if match_pct:
            return int(match_pct.group(1)), is_muted
    except Exception:
        pass
    return 50, False

def set_volume(pct):
    pct = max(0, min(100, pct))
    subprocess.run(["wpctl", "set-volume", "@DEFAULT_AUDIO_SINK@", f"{pct/100:.2f}"], check=False)
    subprocess.run(["wpctl", "set-mute", "@DEFAULT_AUDIO_SINK@", "0"], check=False)
    return pct, False

def toggle_mute():
    subprocess.run(["wpctl", "set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"], check=False)
    return get_volume_info()

def main(stdscr):
    curses.curs_set(0)
    curses.use_default_colors()
    stdscr.keypad(True)
    
    curses.init_pair(1, curses.COLOR_GREEN, -1)   # Bordes / Título
    curses.init_pair(2, curses.COLOR_CYAN, -1)    # Ayuda / Atajos
    curses.init_pair(3, curses.COLOR_WHITE, curses.COLOR_MAGENTA) # Cursor
    curses.init_pair(4, curses.COLOR_YELLOW, -1)  # Encabezados
    curses.init_pair(5, curses.COLOR_WHITE, -1)   # Texto
    curses.init_pair(6, curses.COLOR_RED, -1)     # Muted

    presets = [
        (100, "100%", "Volumen máximo"),
        (75,  " 75%", "Alto"),
        (50,  " 50%", "Medio (Normal)"),
        (25,  " 25%", "Bajo"),
        (10,  " 10%", "Mínimo"),
    ]

    vol, is_muted = get_volume_info()
    diffs = [abs(p[0] - vol) for p in presets]
    selected_idx = diffs.index(min(diffs))

    while True:
        stdscr.erase()
        h, w = stdscr.getmaxyx()
        
        box_top = 0
        box_h = max(8, h - 2)
        box_w = max(30, w - 2)
        box_x = 1

        # Dibujar bordes de caja
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

        # Título
        state_label = "MUTED" if is_muted else f"{vol}%"
        title = f" Master Volume [{state_label}] "
        stdscr.addstr(box_top, box_x + 2, title, curses.color_pair(6 if is_muted else 1) | curses.A_BOLD)
        stdscr.attroff(curses.color_pair(1))

        # Encabezados
        stdscr.addstr(box_top + 1, box_x + 4, "Level", curses.color_pair(4) | curses.A_BOLD)
        stdscr.addstr(box_top + 1, box_x + 18, "Output Status", curses.color_pair(4) | curses.A_BOLD)

        # Opciones
        for i, (pct, name, desc) in enumerate(presets):
            y = box_top + 3 + i
            if y >= box_top + box_h - 1:
                break
            is_hovered = (i == selected_idx)
            mark = "●" if is_hovered else " "
            
            line_str = f" {mark} {name:<5}  {desc[:24]:<24} "
            if is_hovered:
                stdscr.addstr(y, box_x + 2, line_str, curses.color_pair(3) | curses.A_BOLD)
            else:
                stdscr.addstr(y, box_x + 2, f"   {name:<5}  {desc[:24]}", curses.color_pair(5))

        # Barra inferior de atajos pegada al fondo
        help_text = "↵ Set | m Mute | ←/h -5% | →/l +5% | q Close"
        footer_y = h - 1
        stdscr.addstr(footer_y, max(1, (w - len(help_text)) // 2), help_text, curses.color_pair(2))

        stdscr.refresh()

        key = stdscr.getch()
        if key in [curses.KEY_UP, ord('k')]:
            selected_idx = (selected_idx - 1) % len(presets)
        elif key in [curses.KEY_DOWN, ord('j')]:
            selected_idx = (selected_idx + 1) % len(presets)
        elif key in [curses.KEY_LEFT, ord('h')]:
            vol, is_muted = set_volume(vol - 5)
        elif key in [curses.KEY_RIGHT, ord('l')]:
            vol, is_muted = set_volume(vol + 5)
        elif key in [ord('m'), ord('M')]:
            vol, is_muted = toggle_mute()
        elif key in [10, 13, curses.KEY_ENTER, ord(' ')]:
            vol, is_muted = set_volume(presets[selected_idx][0])
            break
        elif key in [27, ord('q'), ord('Q')]:
            break

if __name__ == "__main__":
    curses.wrapper(main)
