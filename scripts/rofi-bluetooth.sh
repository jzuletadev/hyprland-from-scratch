#!/usr/bin/env bash
# ~/hyprland-from-scratch/scripts/rofi-bluetooth.sh
#
# Phase 11, rewritten in Phase 16. Bluetooth menu for Rofi's dmenu mode,
# driven by bluetoothctl (bluez). Connects/disconnects known devices, scans
# for and pairs new ones (pair + trust + connect, so they reconnect on their
# own after a reboot), forgets devices, and powers the adapter on/off.
#
# Pairing runs inside an interactive bluetoothctl that registers a
# NoInputNoOutput agent: right for mice, headsets, speakers and controllers.
# A device that asks for a PIN (some keyboards) needs a manual
# `bluetoothctl` session instead.
#
# Icons are $'\uXXXX' escapes: literal Nerd Font glyphs get silently
# stripped when written into files here (see docs/build-log.md Phase 11).

set -uo pipefail

source "$HOME/hyprland-from-scratch/scripts/rofi-common.sh"
rofi_toggle "-p Bluetooth "   # same icon clicked again = close; another menu open = switch

THEME="$HOME/.config/rofi/menu.rasi"
SCAN_SECONDS=10

I_CONNECTED=$'\uf00c'
I_PAIRED=$'\uf0c1'
I_NEW=$'\uf067'
I_SCAN=$'\uf002'
I_POWER=$'\uf011'
I_FORGET=$'\uf1f8'

menu()   { rofi -dmenu -i -no-custom -p "$1" -theme "$THEME"; }
notify() { notify-send -a Bluetooth "Bluetooth" "$1"; }

declare -A NAME ACTION
ENTRIES=()
SCAN_PID= AGENT_PID=
# background bluetoothctl processes (discovery, pairing agent) die with the menu
trap 'kill ${SCAN_PID:-} ${AGENT_PID:-} 2>/dev/null' EXIT
add()        { ENTRIES+=("$1"); ACTION["$1"]=$2; }
reset_menu() { ENTRIES=(); ACTION=(); }

load_names() {   # NAME[mac] for every device bluez knows (paired or just seen)
    NAME=()
    local mac name
    while read -r _ mac name; do
        [ -n "$mac" ] && NAME[$mac]=$name
    done < <(bluetoothctl devices)
}
macs()    { bluetoothctl devices "$@" | awk '{print $2}'; }   # optional filter: Paired, Connected
powered() { [[ "$(bluetoothctl show)" == *"Powered: yes"* ]]; }
is()      { [[ "$(bluetoothctl info "$2")" == *"$1: yes"* ]]; }  # is Paired|Connected <mac>

choose() {   # show ENTRIES, run the chosen action
    local choice
    [ ${#ENTRIES[@]} -eq 0 ] && return 1
    choice=$(printf '%s\n' "${ENTRIES[@]}" | menu "$1") || return 1
    run ${ACTION[$choice]}
}

connect() {
    local mac=$1
    timeout 20 bluetoothctl connect "$mac" >/dev/null 2>&1
    if is Connected "$mac"; then
        notify "Connected: ${NAME[$mac]}"
    else
        notify "Couldn't connect to ${NAME[$mac]} — is it on and in range?"
    fi
}

pair() {
    local mac=$1 i
    notify "Pairing with ${NAME[$mac]}..."
    # bluez refuses to pair while no agent is registered ("No agent available
    # for request type 2" in `journalctl -u bluetooth`), and one-shot
    # `bluetoothctl --agent X pair <mac>` never registers one: bluetoothctl
    # skips agent registration in non-interactive mode. So the pairing runs in
    # an interactive session fed through a coprocess. With a NoInputNoOutput
    # agent the pairing is "Just Works", which bluez accepts on its own since
    # we started it — no prompt to answer.
    coproc BTCTL { bluetoothctl --agent NoInputNoOutput >/dev/null 2>&1; }
    AGENT_PID=$BTCTL_PID
    sleep 1   # let it load the device list and register the agent first
    printf 'default-agent\npair %s\n' "$mac" >&"${BTCTL[1]}"
    for ((i = 0; i < 30; i++)); do
        is Paired "$mac" && break
        sleep 1
    done
    printf 'quit\n' >&"${BTCTL[1]}" 2>/dev/null
    if ! is Paired "$mac"; then
        notify "Pairing with ${NAME[$mac]} failed. Put it back in pairing mode and scan again."
        return 1
    fi
    # trusted = allowed to reconnect by itself (after sleep, reboot, power cycle)
    bluetoothctl trust "$mac" >/dev/null 2>&1
    connect "$mac"
}

scan() {
    notify "Searching for ${SCAN_SECONDS}s. Put the device in pairing mode."
    # Discovery stays on while the result list is open: bluez drops unpaired
    # devices ~30s after discovery stops, so they'd vanish before being picked.
    bluetoothctl --timeout 120 scan on >/dev/null 2>&1 &
    SCAN_PID=$!   # global: the EXIT trap runs after this function has returned
    sleep "$SCAN_SECONDS"

    load_names
    reset_menu
    local paired mac label
    local -A seen=()
    paired=" $(macs Paired | tr '\n' ' ') "
    for mac in "${!NAME[@]}"; do
        [[ "$paired" == *" $mac "* ]] && continue
        # nameless advertisers (beacons, phones nearby) show their MAC as name
        [[ "${NAME[$mac]}" =~ ^([0-9A-F]{2}-){5}[0-9A-F]{2}$ ]] && continue
        label="$I_NEW  ${NAME[$mac]}"
        # same name twice (two routers, two identical mice): add the MAC's tail
        [[ -n "${seen[${NAME[$mac]}]:-}" ]] && label+="  (${mac: -5})"
        seen[${NAME[$mac]}]=1
        add "$label" "pair $mac"
    done
    if [ ${#ENTRIES[@]} -eq 0 ]; then
        notify "No new devices found. Is it in pairing mode?"
        return
    fi
    local choice
    choice=$(printf '%s\n' "${ENTRIES[@]}" | sort | menu "Pair") || return
    kill "$SCAN_PID" 2>/dev/null   # pairing is more reliable with discovery off
    run ${ACTION[$choice]}
}

forget() {
    reset_menu
    local mac
    for mac in $(macs Paired); do
        add "$I_FORGET  ${NAME[$mac]}" "remove $mac"
    done
    choose "Forget"
}

run() {
    local cmd=$1 mac=${2:-}
    case "$cmd" in
        connect)    connect "$mac" ;;
        disconnect) bluetoothctl disconnect "$mac" >/dev/null 2>&1 && notify "Disconnected: ${NAME[$mac]}" ;;
        pair)       pair "$mac" ;;
        remove)     bluetoothctl remove "$mac" >/dev/null 2>&1 && notify "Forgot ${NAME[$mac]}" ;;
        scan)       scan ;;
        forget)     forget ;;
        power-on)
            # a soft rfkill block makes "power on" fail; clear it and retry once
            bluetoothctl power on >/dev/null 2>&1 \
                || { rfkill unblock bluetooth 2>/dev/null; sleep 1; bluetoothctl power on >/dev/null 2>&1; }
            if powered; then notify "On"; else notify "Couldn't turn Bluetooth on"; fi ;;
        power-off)  bluetoothctl power off >/dev/null 2>&1 && notify "Off" ;;
    esac
}

load_names
reset_menu
if ! powered; then
    add "$I_POWER  Turn Bluetooth on" "power-on"
else
    connected=" $(macs Connected | tr '\n' ' ') "
    for mac in $(macs Paired); do
        if [[ "$connected" == *" $mac "* ]]; then
            add "$I_CONNECTED  ${NAME[$mac]}  (connected)" "disconnect $mac"
        else
            add "$I_PAIRED  ${NAME[$mac]}" "connect $mac"
        fi
    done
    add "$I_SCAN  Scan for new devices" "scan"
    [ -n "$(macs Paired)" ] && add "$I_FORGET  Forget a device..." "forget"
    add "$I_POWER  Turn Bluetooth off" "power-off"
fi
choose "Bluetooth"
exit 0
