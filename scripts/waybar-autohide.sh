#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/waybar-autohide.sh

LOCKFILE="/tmp/waybar-autohide.lock"
exec 9>"$LOCKFILE"
if ! flock -n 9; then
    exit 0
fi

# Umbral para MOSTRAR (pegar el mouse al borde superior: 0 a 5px)
REVEAL_Y=5
# Umbral para OCULTAR (mientras estés entre 0 y 55px la barra se mantiene abierta)
HIDE_Y=55

BASE_CONFIG="$HOME/.config/waybar/config.jsonc"
STYLE_CSS="$HOME/.config/waybar/style.css"

declare -A PIDS
declare -A HIDDEN

# Esperar a que Hyprland cargue los monitores al iniciar sesión
for _ in {1..20}; do
    MONITORS=$(hyprctl monitors -j 2>/dev/null | jq -r '.[].name' 2>/dev/null)
    [ -n "$MONITORS" ] && break
    sleep 0.2
done

start_bars() {
    killall -q waybar
    sleep 0.3
    PIDS=()
    HIDDEN=()

    MONITORS=$(hyprctl monitors -j | jq -r '.[].name')

    for mon in $MONITORS; do
        TMP_CFG="/tmp/waybar-${mon}.jsonc"
        # Usar "include" nativo de Waybar para respetar comentarios y formato JSONC
        cat <<EOF > "$TMP_CFG"
{
    "include": ["$BASE_CONFIG"],
    "output": "$mon"
}
EOF
        waybar -c "$TMP_CFG" -s "$STYLE_CSS" >/dev/null 2>&1 &
        PIDS["$mon"]=$!
        HIDDEN["$mon"]=false
    done

    # Esperar a que Waybar dibuje las barras y ocultarlas inicialmente
    sleep 1.0
    for mon in $MONITORS; do
        kill -SIGUSR1 "${PIDS[$mon]}" 2>/dev/null
        HIDDEN["$mon"]=true
    done
}

# Si theme-manager.sh envía SIGUSR2, reiniciamos las barras con el nuevo tema
trap 'start_bars' USR2

start_bars

while true; do
    sleep 0.15

    pos=$(hyprctl cursorpos -j 2>/dev/null)
    [ -n "$pos" ] || continue
    cx=$(echo "$pos" | jq -r '.x')
    cy=$(echo "$pos" | jq -r '.y')

    while IFS=$'\t' read -r m_name m_x m_y m_w m_h; do
        pid="${PIDS[$m_name]}"
        [ -n "$pid" ] && kill -0 "$pid" 2>/dev/null || continue

        if (( cx >= m_x && cx < m_x + m_w && cy >= m_y && cy < m_y + m_h )); then
            rel_y=$(( cy - m_y ))
            if [ "${HIDDEN[$m_name]}" = true ]; then
                want_hidden=true
                (( rel_y <= REVEAL_Y )) && want_hidden=false
            else
                want_hidden=false
                (( rel_y > HIDE_Y )) && want_hidden=true
            fi
        else
            want_hidden=true
        fi

        if [ "$want_hidden" != "${HIDDEN[$m_name]}" ]; then
            kill -SIGUSR1 "$pid" 2>/dev/null
            HIDDEN["$m_name"]=$want_hidden
        fi
    done < <(hyprctl monitors -j | jq -r '.[] | "\(.name)\t\(.x)\t\(.y)\t\(.width)\t\(.height)"' 2>/dev/null)
done