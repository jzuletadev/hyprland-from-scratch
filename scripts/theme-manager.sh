#!/usr/bin/env bash
set -e

C="$HOME/.config"
THEME_TRACKER="$C/hypr/current_theme"
MODE_TRACKER="$C/hypr/current_mode"

mkdir -p "$C/hypr" "$C/kitty" "$HOME/.local/state/hyprland-from-scratch"

# Leer estados actuales o fijar valores por defecto
PALETTE=$(cat "$THEME_TRACKER" 2>/dev/null || echo "nord")
MODE=$(cat "$MODE_TRACKER" 2>/dev/null || echo "dark")

# Manejo de argumentos
if [ "$1" = "--mode" ]; then
    if [ "$2" = "toggle" ]; then
        [ "$MODE" = "dark" ] && MODE="light" || MODE="dark"
    elif [ "$2" = "light" ] || [ "$2" = "dark" ]; then
        MODE="$2"
    fi
elif [ -n "$1" ]; then
    case "$1" in
        catppuccin*|tokyo*|nord*|gruvbox*|arch-cyan*|arch-purple*)
            PALETTE="$1"
            ;;
        dark)
            MODE="dark"
            ;;
        light)
            MODE="light"
            ;;
        *)
            PALETTE="$1"
            ;;
    esac
fi

# Guardar persistencia
echo "$PALETTE" > "$THEME_TRACKER"
echo "$MODE" > "$MODE_TRACKER"
echo "$PALETTE" > "$HOME/.local/state/hyprland-from-scratch/theme"
echo "$MODE" > "$HOME/.local/state/hyprland-from-scratch/mode"

# 1. Definir colores de la paleta (Acentos)
case "$PALETTE" in
    catppuccin*)
        PALETTE="catppuccin"
        NAME="Catppuccin"
        ACCENT="#cba6f7"
        LIGHT_ACCENT="#dcb8ff"
        ;;
    tokyo*)
        PALETTE="tokyo-night"
        NAME="Tokyo Night"
        ACCENT="#7aa2f7"
        LIGHT_ACCENT="#b4f9f8"
        ;;
    gruvbox*)
        PALETTE="gruvbox"
        NAME="Gruvbox"
        ACCENT="#fabd2f"
        LIGHT_ACCENT="#fe8019"
        ;;
    arch-cyan*)
        PALETTE="arch-cyan"
        NAME="Arch Cyan"
        ACCENT="#1793d1"
        LIGHT_ACCENT="#5db3df"
        ;;
    arch-purple*)
        PALETTE="arch-purple"
        NAME="Arch Purple"
        ACCENT="#9564dd"
        LIGHT_ACCENT="#b58ef0"
        ;;
    nord*|*)
        PALETTE="nord"
        NAME="Nord Ice"
        ACCENT="#88c0d0"
        LIGHT_ACCENT="#81a1c1"
        ;;
esac

# 2. Definir fondos y textos según el Modo (Dark / Light)
if [ "$MODE" = "light" ]; then
    BG="#f5f7fa"
    FG="#2e3440"
    BORDER_ACTIVE="$ACCENT"
    BORDER_INACTIVE="#d8dee9"
    GTK_PREF="prefer-light"
else
    BG="#141414"
    FG="#e0e0e0"
    BORDER_ACTIVE="$ACCENT"
    BORDER_INACTIVE="#2d2d2d"
    GTK_PREF="prefer-dark"
fi

# 3. Aplicar a GTK y XDG Desktop Portal (Navegadores, Electron, Apps del sistema)
gsettings set org.gnome.desktop.interface color-scheme "$GTK_PREF" 2>/dev/null || true

# 4. Actualizar bordes en Hyprland
cat << INI > "$C/hypr/appearance.conf"
# Archivo autogenerado por theme-manager.sh
\$border_active = rgb(${BORDER_ACTIVE#\#})
\$border_inactive = rgb(${BORDER_INACTIVE#\#})
\$border_size = 2
INI
hyprctl reload 2>/dev/null || true

# 5. Actualizar colores de Kitty en caliente
cat << CONF > "$C/kitty/theme.conf"
# Tema dinámico Kitty ($NAME - $MODE)
foreground $FG
background $BG
selection_foreground $BG
selection_background $ACCENT
cursor $ACCENT
CONF

# Notificar a todas las ventanas Kitty abiertas para cambiar fondo y texto al instante
for sock in /tmp/kitty-*; do
    [ -S "$sock" ] && kitty @ --to "unix:$sock" set-colors "$C/kitty/theme.conf" 2>/dev/null || true
done

# 6. Sincronizar tema de Yazi
if [ -f "$HOME/hyprland-from-scratch/dotfiles/yazi/themes/$MODE.toml" ]; then
    ln -sfn "$HOME/hyprland-from-scratch/dotfiles/yazi/themes/$MODE.toml" "$C/yazi/theme.toml"
elif [ -f "$HOME/hyprland-from-scratch/dotfiles/yazi/themes/$PALETTE.toml" ]; then
    ln -sfn "$HOME/hyprland-from-scratch/dotfiles/yazi/themes/$PALETTE.toml" "$C/yazi/theme.toml"
fi

# 7. Recargar Waybar
pkill -SIGUSR2 waybar 2>/dev/null || true

# 8. Notificación visual
notify-send -a "Theme Engine" "Tema: $NAME ($MODE)" "Acento: $ACCENT | Fondo: $MODE"
