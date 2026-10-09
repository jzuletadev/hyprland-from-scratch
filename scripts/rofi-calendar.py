#!/usr/bin/env python3
# ~/hyprland-from-scratch/scripts/rofi-calendar.py
#
# Phase 24. Interactive calendar in Rofi's script mode, opened by clicking the
# clock in Waybar. The hover calendar from Phase 19 can't be clicked (GTK
# tooltips aren't interactive), so browsing other months happens here:
#
#   «  ‹  Today  ›  »    previous/next year and month, back to the current month
#   a 7-column grid of days, today in a block of the menu accent color
#
# Rofi stays open between clicks (script mode, like rofi-volume.sh): after each
# pick it runs this script again with ROFI_RETV=1 and the row's hidden value
# in ROFI_INFO, and the month on screen travels along in ROFI_DATA ("2026-10").
#
# Run without Rofi's variables it is the launcher: it closes whatever menu is
# open (like rofi_toggle in rofi-common.sh; clicking the clock again just
# closes the calendar) and starts Rofi on itself.

import calendar
import datetime
import os
import re
import subprocess
import time

ME = os.path.abspath(__file__)
THEME = os.path.expanduser("~/.config/rofi/menu.rasi")
# flow: horizontal fills the grid row by row (Rofi's default is column by column)
LAYOUT = """
window       { width: 420px; }
inputbar     { children: [ "prompt" ]; }
listview     { columns: 7; lines: 8; flow: horizontal; fixed-height: true; spacing: 2px; }
element      { padding: 8px 0; spacing: 0; }
element-text { horizontal-align: 0.5; }
"""
WEEK_STARTS = calendar.SUNDAY   # same as the Waybar tooltip


def menu_accent():
    """The menus' accent color, from ~/.config/rofi/appearance.rasi (Phase 43)."""
    try:
        with open(os.path.expanduser("~/.config/rofi/appearance.rasi")) as rasi:
            match = re.search(r"^\s*accent:\s*(#[0-9a-fA-F]{6})", rasi.read(), re.M)
    except OSError:
        match = None
    return match.group(1) if match else "#1793d1"


ACCENT = menu_accent()


def launch():
    running = subprocess.run(["pgrep", "-ax", "rofi"], capture_output=True, text=True).stdout
    if running:
        subprocess.run(["pkill", "-x", "rofi"])
        while subprocess.run(["pgrep", "-x", "rofi"], capture_output=True).returncode == 0:
            time.sleep(0.05)
        if "-show calendar" in running:
            return   # the clock was clicked while the calendar was open: close it
    # -no-show-icons: config.rasi turns icons on, and the empty icon slot would
    # eat half of each 60px cell
    os.execvp("rofi", ["rofi", "-show", "calendar", "-modi", f"calendar:{ME}",
                       "-no-show-icons", "-theme", THEME, "-theme-str", LAYOUT])


def month_to_show():
    today = datetime.date.today()
    year, month = today.year, today.month
    if os.environ.get("ROFI_DATA"):
        year, month = (int(part) for part in os.environ["ROFI_DATA"].split("-"))
    if os.environ.get("ROFI_RETV") == "1":
        action = os.environ.get("ROFI_INFO", "")
        if action == "today":
            year, month = today.year, today.month
        elif action == "prev-year":
            year -= 1
        elif action == "next-year":
            year += 1
        elif action in ("prev-month", "next-month"):
            month += 1 if action == "next-month" else -1
            if month == 0:
                year, month = year - 1, 12
            elif month == 13:
                year, month = year + 1, 1
    return year, month


def show():
    year, month = month_to_show()
    today = datetime.date.today()
    lines = []

    def option(key, value):          # mode options: "\0key\x1fvalue"
        lines.append(f"\0{key}\x1f{value}")

    def cell(label, info=None):      # rows: "label\0info\x1fvalue" or not selectable
        if info is None:
            lines.append(f"{label}\0nonselectable\x1ftrue")
        else:
            lines.append(f"{label}\0info\x1f{info}")

    option("prompt", f"{calendar.month_name[month]} {year}")
    option("data", f"{year}-{month}")
    option("markup-rows", "true")      # script mode ignores -markup-rows on the command line
    option("keep-selection", "true")   # stay on « ‹ › » while clicking through

    for label, info in [("«", "prev-year"), ("‹", "prev-month"), (" ", None),
                        ("Today", "today"), (" ", None),
                        ("›", "next-month"), ("»", "next-year")]:
        cell(label, info)

    weekdays = calendar.Calendar(WEEK_STARTS).iterweekdays()
    for day in weekdays:
        cell(f"<span color='{ACCENT}'><b>{calendar.day_abbr[day][:2]}</b></span>")

    weeks = calendar.Calendar(WEEK_STARTS).monthdayscalendar(year, month)
    weeks += [[0] * 7] * (6 - len(weeks))   # always 6 rows: the window keeps its size
    for week in weeks:
        for day in week:
            if day == 0:
                cell(" ")
            elif datetime.date(year, month, day) == today:
                cell(f"<span background='{ACCENT}' color='#141414'><b> {day} </b></span>", "day")
            else:
                cell(str(day), "day")

    print("\n".join(lines))


if "ROFI_RETV" in os.environ:
    show()
else:
    launch()
