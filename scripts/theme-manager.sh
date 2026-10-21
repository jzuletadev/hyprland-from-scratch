#!/usr/bin/env bash
set -u

C="$HOME/.config"
DOTFILES="$HOME/hyprland-from-scratch/dotfiles"
FLAVORS_DIR="$DOTFILES/yazi/flavors"

THEME_TRACKER="$C/hypr/current_theme"

mkdir -p "$C/hypr" "$C/kitty" "$C/waybar" "$C/dunst/dunstrc.d" "$C/rofi" "$C/yazi" "$C/yazi-hidden" "$HOME/.local/state/hyprland-from-scratch"

# 1. Determinar el flavor solicitado o leer el actual
FLAVOR="${1:-}"
if [ -z "$FLAVOR" ]; then
    FLAVOR=$(cat "$THEME_TRACKER" 2>/dev/null || echo "catppuccin-mocha")
fi
FLAVOR="${FLAVOR%.yazi}"

# Si no existe el flavor exacto, buscar coincidencia parcial en el directorio de flavors de Yazi
if [ ! -d "$FLAVORS_DIR/${FLAVOR}.yazi" ]; then
    MATCH=$(find "$FLAVORS_DIR" -maxdepth 1 -type d -name "${FLAVOR}*.yazi" | sort | head -n 1)
    if [ -n "$MATCH" ]; then
        FLAVOR=$(basename "$MATCH" .yazi)
    else
        FLAVOR="catppuccin-mocha"
    fi
fi

FLAVOR_DIR="$FLAVORS_DIR/${FLAVOR}.yazi"
FLAVOR_TOML="$FLAVOR_DIR/flavor.toml"
TMTHEME_XML="$FLAVOR_DIR/tmtheme.xml"

if [ ! -f "$FLAVOR_TOML" ]; then
    echo "Error: No se encontró $FLAVOR_TOML" >&2
    exit 1
fi

# 2. Extraer toda la paleta directamente desde flavor.toml y tmtheme.xml (sin archivos .conf)
eval "$(python3 - "$FLAVOR" "$FLAVOR_TOML" "$TMTHEME_XML" << 'PY'
import sys, re, os

flavor, toml_path, xml_path = sys.argv[1], sys.argv[2], sys.argv[3]

toml_text = open(toml_path, encoding="utf-8", errors="ignore").read() if os.path.exists(toml_path) else ""
xml_text = open(xml_path, encoding="utf-8", errors="ignore").read() if os.path.exists(xml_path) else ""

def find_hex(pattern, text, default=None):
    m = re.search(pattern, text, re.IGNORECASE | re.DOTALL)
    return m.group(1).lower() if m else default

bg = find_hex(r'<key>background</key>\s*<string>(#[0-9a-fA-F]{6})', xml_text)
fg = find_hex(r'<key>foreground</key>\s*<string>(#[0-9a-fA-F]{6})', xml_text)

if not bg:
    bg = find_hex(r'overall\s*=\s*\{\s*bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
         find_hex(r'normal_main\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#1e1e2e")

if not fg:
    fg = find_hex(r'url\s*=\s*"\*"\s*,\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
         find_hex(r'footer\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#cdd6f4")

r, g, b = int(bg[1:3], 16), int(bg[3:5], 16), int(bg[5:7], 16)
luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255.0
mode = "light" if luminance > 0.55 else "dark"

accent = find_hex(r'normal_main\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
         find_hex(r'active\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
         find_hex(r'cwd\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#cba6f7")

light_accent = find_hex(r'cwd\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
               find_hex(r'select_main\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, accent)

border_inactive = find_hex(r'border_style\s*=.*?fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
                  find_hex(r'normal_alt\s*=.*?bg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#403e41")

c_red     = find_hex(r'title_error\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
            find_hex(r'perm_write\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#f38ba8")
c_green   = find_hex(r'title_info\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
            find_hex(r'marker_copied\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#a6e3a1")
c_yellow  = find_hex(r'title_warn\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
            find_hex(r'perm_read\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#f9e2af")
c_blue    = find_hex(r'url\s*=\s*"\*/"\s*,\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or accent
c_magenta = find_hex(r'desc\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or \
            find_hex(r'find_position\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text, "#cba6f7")
c_cyan    = find_hex(r'cwd\s*=\s*\{\s*fg\s*=\s*"(#[0-9a-fA-F]{6})"', toml_text) or light_accent

name = flavor.replace("-", " ").title()

print(f'NAME="{name}"')
print(f'MODE="{mode}"')
print(f'BG="{bg}"')
print(f'FG="{fg}"')
print(f'ACCENT="{accent}"')
print(f'LIGHT_ACCENT="{light_accent}"')
print(f'BORDER_ACTIVE="{accent}"')
print(f'BORDER_INACTIVE="{border_inactive}"')
print(f'C0="{border_inactive}"')
print(f'C1="{c_red}"')
print(f'C2="{c_green}"')
print(f'C3="{c_yellow}"')
print(f'C4="{c_blue}"')
print(f'C5="{c_magenta}"')
print(f'C6="{c_cyan}"')
print(f'C7="{fg}"')
PY
)"

GTK_PREF="prefer-dark"
[ "$MODE" = "light" ] && GTK_PREF="prefer-light"

# 3. Guardar estado actual
echo "$FLAVOR" > "$THEME_TRACKER"
echo "$MODE" > "$HOME/.config/hypr/current_mode"
echo "$FLAVOR" > "$HOME/.local/state/hyprland-from-scratch/theme"
echo "$MODE" > "$HOME/.local/state/hyprland-from-scratch/mode"

# 4. Aplicar esquema a GTK / Portals según la luminancia extraída
gsettings set org.gnome.desktop.interface color-scheme "$GTK_PREF" 2>/dev/null || true

# 5. Hyprland: Bordes activos e inactivos
cat << INI > "$C/hypr/appearance.conf"
# Autogenerado desde $FLAVOR.yazi ($NAME)
\$border_active = rgb(${BORDER_ACTIVE#\#})
\$border_inactive = rgb(${BORDER_INACTIVE#\#})
\$border_size = 2
INI
hyprctl reload 2>/dev/null || true

# 6. Kitty: Paleta completa extraída de Yazi
cat << CONF > "$C/kitty/theme.conf"
# Autogenerado desde $FLAVOR.yazi ($NAME)
foreground              $FG
background              $BG
selection_foreground    $BG
selection_background    $ACCENT
cursor                  $ACCENT
cursor_text_color       $BG
url_color               $LIGHT_ACCENT

color0  $C0
color1  $C1
color2  $C2
color3  $C3
color4  $C4
color5  $C5
color6  $C6
color7  $C7
color8  $C0
color9  $C1
color10 $C2
color11 $C3
color12 $C4
color13 $C5
color14 $C6
color15 $FG
CONF

for sock in /tmp/kitty /tmp/kitty-*; do
    [ -S "$sock" ] && kitty @ --to "unix:$sock" set-colors --all "$C/kitty/theme.conf" 2>/dev/null || true
done

# 7. Yazi: Generar theme.toml inyectando sección maestra [icon] usando los colores del flavor
generate_yazi_theme() {
    local target_file="$1"
    cat << EOF > "$target_file"
# Autogenerado desde $FLAVOR.yazi ($NAME)
[flavor]
use = "$FLAVOR"
dark = "$FLAVOR"
light = "$FLAVOR"

[icon]
prepend_dirs = [
  { name = ".config", text = "", fg = "$C1" },
  { name = ".git", text = "", fg = "$C4" },
  { name = "Desktop", text = "", fg = "$C6" },
  { name = "Downloads", text = "", fg = "$C3" },
  { name = "Documents", text = "", fg = "$C2" },
  { name = "Music", text = "", fg = "$C6" },
  { name = "Pictures", text = "", fg = "$C5" },
  { name = "Fotos", text = "", fg = "$C5" },
  { name = "Videos", text = "", fg = "$C1" },
  { name = "Screenshots", text = "", fg = "$C5" },
  { name = "Public", text = "", fg = "$C2" },
  { name = "Templates", text = "", fg = "$C4" },
  { name = "dev", text = "", fg = "$C6" },
  { name = "hyprland-from-scratch", text = "󰣇", fg = "$C3" },
]

prepend_conds = [
  { if = "dir & hovered", text = "", fg = "$C3" },
  { if = "dir", text = "", fg = "$C4" },
]

prepend_globs = [
  { url = "*.{jpg,jpeg,png,gif,webp,svg,bmp,ico,tiff,heic}", text = "", fg = "$C3" },
  { url = "*.{mp4,mkv,mov,avi,webm,flv,wmv,mpeg}", text = "", fg = "$C2" },
  { url = "*.{mp3,flac,wav,m4a,ogg,opus,aac,wma}", text = "", fg = "$C5" },
  { url = "*.{zip,rar,7z*,tar,gzip,xz,zstd,bzip*,lzma,compress,archive,cpio,arj,xar,ms-cab*}", text = "", fg = "$C1" },
]
EOF
}

generate_yazi_theme "$DOTFILES/yazi/theme.toml"
if [ -d "$DOTFILES/yazi-hidden" ]; then
    generate_yazi_theme "$DOTFILES/yazi-hidden/theme.toml"
fi

# 8. Waybar: Variables CSS de acento
cat << CSS > "$C/waybar/accent.css"
/* Autogenerado desde $FLAVOR.yazi ($NAME) */
@define-color accent $ACCENT;
@define-color accent_light $LIGHT_ACCENT;
CSS
pkill -SIGUSR2 waybar 2>/dev/null || true

# 9. Dunst: Notificaciones con marco de acento
cat << CONF > "$C/dunst/dunstrc.d/50-accent.conf"
[global]
    frame_color = "$ACCENT"
    frame_width = 2
[urgency_normal]
    highlight = "$ACCENT"
CONF
dunstctl reload 2>/dev/null || killall -SIGUSR2 dunst 2>/dev/null || true

# 10. Rofi: Colores y bordes para el app launcher
cat << RASI > "$C/rofi/appearance.rasi"
// Autogenerado desde $FLAVOR.yazi ($NAME)

* {
    accent: $ACCENT;
    accent-highlight: bold $LIGHT_ACCENT;
    menu-border-color: $ACCENT;
    menu-border: 1px;
}
RASI

notify-send -a "Theme Engine" "Tema: $NAME" "Flavor: $FLAVOR.yazi ($MODE)" || true