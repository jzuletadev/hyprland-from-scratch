# Hyprland From Scratch

A complete, minimal Hyprland desktop for Arch Linux, built piece by piece and kept entirely in this
repository so it can be installed again on any machine.

- **Dynamic Theme Engine:** 7 curated palettes (**Catppuccin**, **Nord Ice**, **Tokyo Night**,
  **Gruvbox**, **Rosé Pine**, **Dracula**, **Monokai Pro**) with live **Dark / Light** mode switching
  across Hyprland borders, Kitty, Waybar, Rofi, Dunst, Yazi, and GTK apps.
- **Typography:** Adwaita Sans for the interface, JetBrainsMono Nerd Font for the terminal and TUIs.
- **Bar (Waybar):** system info & theme launcher, workspaces, active media (`mpris`), window title,
  live recording timer, brightness, Wi-Fi, volume, Bluetooth, battery, and clock.
- **Custom Python TUIs:** lightweight `curses` floating menus in Kitty for Wi-Fi, Bluetooth, audio
  volume, screen brightness, power profiles, session/power actions, theme switching, wallpaper
  picking, screen recording, calendar, and live keybinding search.
- **File Manager & Monitor:** `yazi` (with dynamic flavor sync and hidden-files mode) and `btop`.
- **Lock, Idle & Login:** `hyprlock`, `hypridle` (battery-aware), and a matching SDDM theme that
  automatically syncs with your current wallpaper.
- **Screenshots & Recording:** screenshots straight to the clipboard with annotation (`satty`) and
  hardware-friendly screen/region recording (`wf-recorder`).

---

## 1. Install Arch Linux

Boot the Arch ISO and run `archinstall`:

| Setting | Value |
|---------|-------|
| Bootloader | systemd-boot |
| Profile | **Minimal** (no desktop: this repo provides it) |
| Audio | PipeWire |
| Network | NetworkManager |
| User account | Your user, in the `wheel` group (sudo) |
| Additional packages | `git base-devel` (+ `linux-headers` on hybrid NVIDIA laptops, annex A) |

Everything else (locale, disk, timezone…) is up to you. Reboot into the TTY and log in.

## 2. Install the desktop

Clone the repo **to this exact path** — configs and scripts refer to it:

```bash
git clone [https://github.com/jzuletadev/hyprland-from-scratch.git](https://github.com/jzuletadev/hyprland-from-scratch.git) ~/hyprland-from-scratch
cd ~/hyprland-from-scratch
```

Then choose how you want it:

| Mode | Command | What you get |
|------|---------|--------------|
| **Use it** | `./install.sh` | Configs are **copied** into `~/.config`. The repo can change without touching your desktop; run the installer again to update. |
| **Keep customizing** | `./install.sh --link` | `~/.config/<app>` is a **symlink** to the repo: edit a file in the repo and the desktop changes live. The way this setup is developed. |

Options: `--apps` also installs the personal apps in `packages/apps.txt` (Docker, Node, Steam —
Steam needs `[multilib]` enabled in `/etc/pacman.conf`), `--dry-run` prints every step without
doing anything.

The installer:

1. installs the packages in `packages/base.txt` (asks for your sudo password) and refreshes fonts;
2. copies or links every folder of `dotfiles/` into `~/.config/` — anything already there is kept
   as `<name>.bak-<date>`;
3. configures `~/.bashrc` to source `dotfiles/bash/bashrc` (history, `fzf`, `ble.sh` support);
4. sets `iwd` as the Wi-Fi backend for NetworkManager and enables system/user services (`sddm`,
   `bluetooth`, `power-profiles-daemon`, `iwd`, and `waybar`);
5. sets default apps (images → `imv`, video/audio → `mpv`);
6. installs the SDDM login screen theme and prepares `~/Pictures/wallpapers` with live SDDM background sync.

**Hybrid laptop (AMD/Intel + NVIDIA)?** Do [annex A](#annex-a--hybrid-laptops-amdintel--nvidia)
before rebooting. Then reboot and pick **Hyprland (uwsm)** on the login screen.

Optional: AUR apps (Brave, VS Code, and `blesh-git` for suggestions while typing in the terminal)
need an AUR helper:

```bash
git clone [https://aur.archlinux.org/paru.git](https://aur.archlinux.org/paru.git) /tmp/paru && (cd /tmp/paru && makepkg -si)
sed 's/#.*//' packages/aur.txt | xargs paru -S --needed
```

### Updating

```bash
cd ~/hyprland-from-scratch && git pull
./install.sh            # or ./install.sh --link: either one is safe to run again
```

In `--link` mode most changes are live already; the installer only matters for new packages,
new config folders or the login theme.

---

## Keybindings

`SUPER` is the Windows key. **`SUPER + /` opens the interactive Keybindings TUI** (generated live
from `hyprctl binds`), where you can filter shortcuts by name and press `Enter` to execute any bind.

### Apps & System TUIs

| Keys | Action |
|------|--------|
| `SUPER` (tap) | App search (Rofi) — type, Enter |
| `SUPER + T` | Terminal (`kitty`) |
| `SUPER + E` | File manager (`yazi`) |
| `SUPER + SHIFT + E` | File manager showing hidden files (`yazi`) |
| `SUPER + A` | Theme & Dark/Light Mode Picker (`theme-tui.py`) |
| `SUPER + W` | Wallpaper Picker (`wallpaper-tui.py`) |
| `SUPER + Z` | Wi-Fi Manager (`wifi-tui.py`) |
| `SUPER + B` | Bluetooth Manager (`bluetooth-tui.py`) |
| `SUPER + V` | Volume Control Menu (`volume-tui.py`) |
| `SUPER + M` | Mute / unmute audio |
| `SUPER + G` | Brightness Control Menu (`brightness-tui.py`) |
| `SUPER + P` | Power Profile Selector (`power-profile-tui.py`) |
| `SUPER + C` | Calendar (`calendar-tui.py`) |
| `SUPER + R` | Screen Record Menu / Stop recording (`screen-record-tui.py`) |
| `SUPER + SHIFT + Esc` | Task Manager (`btop`) |
| `SUPER + /` | Keybindings Help & Search (`keybinds-tui.py`) |
| `SUPER + L` | Lock screen (`hyprlock`) |
| `SUPER + Esc` | Session & Power Menu (`power-tui.py`: Lock, Sleep, Logout, Reboot, Power Off, Hibernate) |

### Windows & Workspaces

| Keys | Action |
|------|--------|
| `SUPER + Q` | Close active window |
| `SUPER + F` | Toggle floating window |
| `SUPER + SHIFT + F` | Toggle fullscreen (hides the bar) |
| `SUPER + D` | Show desktop (minimize all windows on workspace / restore) |
| `SUPER + arrows` | Focus window in that direction |
| `SUPER + SHIFT + arrows` | Move window (swap places with neighbor) |
| `SUPER + CTRL + arrows` | Resize window (hold to repeat) |
| `SUPER` + mouse drag | Left button: move window · Right button: resize window |
| Drag a window's edge or corner | Resize it directly with the mouse |
| `SUPER + 1…9, 0` | Switch to workspace 1–10 (`0` = workspace 10) |
| `SUPER + SHIFT + 1…9, 0` | Move window to workspace 1–10 and follow it |
| `SUPER + ALT + 1…9, 0` | Send window to workspace 1–10 silently (stay here) |

### Screenshots & Recording

| Keys | Action |
|------|--------|
| `SUPER + S` | Screenshot of the focused monitor — saved and copied to clipboard |
| `SUPER + SHIFT + S` | Screenshot of a region — saved and copied |
| `SUPER + CTRL + S` | Screenshot of the active window — saved and copied |
| `SUPER + ALT + S` | Screenshot of a region, opened directly in Satty to annotate |
| `SUPER + R` | Open Screen Record TUI (or stop active recording immediately) |

Click a screenshot's notification to open it in Satty for annotation. Screenshots are saved to
`~/Pictures/screenshots` and recordings to `~/Videos/recordings`.

**Bar interactions:**
- **Arch logo:** Left-click opens system info (`fastfetch`) · Right-click opens the Theme Picker TUI.
- **Recording indicator:** Appears while `wf-recorder` is active · Click to stop and save.
- **Brightness & Volume:** Scroll to adjust ±5% · Click to open their TUI menus (Right-click volume to mute).
- **Wi-Fi, Bluetooth, Battery & Clock:** Click any module to open its respective floating TUI.

---

## Customizing

| What | Where |
|------|-------|
| Theme & Dark/Light Mode | Press `SUPER + A` or right-click the Arch logo in Waybar. Definitions live in `dotfiles/themes/*.conf` and are applied by `scripts/theme-manager.sh`. |
| Wallpaper | Press `SUPER + W`. Put your images in `~/Pictures/wallpapers/` (initial defaults come from `assets/wallpapers/`). |
| Keybindings & Window Rules | `dotfiles/hypr/hyprland.conf` — use `bindd` so new shortcuts automatically appear in `SUPER + /`. |
| Bar | `dotfiles/waybar/config.jsonc` and `dotfiles/waybar/style.css` (accent colors are auto-generated in `accent.css`). |
| App Search | `dotfiles/rofi/config.rasi` and `dotfiles/rofi/search.rasi`. |
| TUI Menus | `scripts/*-tui.py` (built with Python `curses`, automatically adapts to Dark/Light mode). |
| Lock Screen & Idle | `dotfiles/hypr/hyprlock.conf` and `dotfiles/hypr/hypridle.conf` (triggers `scripts/idle.sh` on battery). |
| Login Screen (SDDM) | `system/sddm/` — run `sudo ./scripts/install-sddm-theme.sh` to update system files. |
| Notifications | `dotfiles/dunst/dunstrc` (accent frame auto-generated in `dunstrc.d/50-accent.conf`). |
| Terminal | `dotfiles/kitty/kitty.conf` (dynamic palette written to `dotfiles/kitty/theme.conf` and reloaded live via socket). |
| File Manager | `dotfiles/yazi/` and `dotfiles/yazi-hidden/` (flavors live in `dotfiles/yazi/flavors/`). |
| Shell | `dotfiles/bash/bashrc`, sourced from `~/.bashrc` — keep personal tokens/paths in `~/.bashrc`. |
| System Info Logo | Place a custom image at `dotfiles/fastfetch/logo.png` (`.jpg` / `.webp` supported). Falls back to the official Arch logo. |
| Packages | `packages/*.txt` |

After editing (in `--link` mode): Hyprland reloads automatically; Waybar reloads styles via
`theme-manager.sh` or restarts with `systemctl --user restart waybar`; Kitty updates colors live.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| The bar disappeared | `systemctl --user restart waybar` (systemd also restarts it automatically on crash) |
| A config error banner at the top | `hyprctl configerrors` shows the exact line |
| Login screen looks like plain SDDM | `sudo ./scripts/install-sddm-theme.sh`, then log out |
| Emoji or Asian text show as boxes | Install `noto-fonts-emoji` / `noto-fonts-cjk` (in `packages/base.txt`), run `fc-cache -f`, and restart your browser (`pkill -x brave`) |
| Stuck on a red "lockscreen died" screen | `Ctrl+Alt+F3`, log in, `hyprctl --instance 0 dispatch exec hyprlock` |
| External monitor runs at 60 Hz | Add a `monitor = desc:…, <res>@<hz>, auto, 1` line to `hyprland.conf` (`hyprctl monitors all` lists modes) |

## Repository layout

```text
install.sh            installer (copy or --link mode)
dotfiles/             one folder per app, copied or linked into ~/.config/
  themes/             modular color palettes (*.conf) for Dark and Light modes
  yazi/flavors/       Yazi color flavors synced by theme-manager.sh
scripts/              Python curses TUIs (*-tui.py) and Bash backends (*.sh)
assets/wallpapers/    default wallpapers copied to ~/Pictures/wallpapers/ on install
packages/             package lists: base, apps, aur, nvidia-hybrid
system/               files copied outside ~/.config: SDDM theme, hybrid-GPU configs
```

---

## Annex A — Hybrid laptops (AMD/Intel + NVIDIA)

> **Only** for laptops with an integrated GPU **plus** an NVIDIA GPU of the Turing generation or
> newer (RTX 20/30/40, GTX 16). Check with `lspci | grep -E 'VGA|3D'`: two lines, one NVIDIA.
> AMD-only or Intel-only machines need nothing. A single-GPU NVIDIA desktop is a different setup,
> not covered here. Applying this on other hardware stops Hyprland from starting.

The desktop runs on the integrated GPU (cooler, longer battery); the NVIDIA GPU drives the external
outputs wired to it and runs the apps you start with `prime-run`. Built and tested on a Dell G15
(Ryzen + RTX 3060).

1. **Driver** (needs `linux-headers`):

   ```bash
   sed 's/#.*//' packages/nvidia-hybrid.txt | xargs sudo pacman -S --needed
   dkms status          # must list nvidia/<version> ... installed
   ```

2. **Load the modules early** — in `/etc/mkinitcpio.conf`, put your iGPU driver first
   (`amdgpu` or `i915`), then `sudo mkinitcpio -P`:

   ```text
   MODULES=(amdgpu nvidia nvidia_modeset nvidia_uvm nvidia_drm)
   ```

3. **Module options and stable GPU names.** The udev rule names each GPU by its PCI address, which
   differs between laptop models: compare with `lspci | grep -E 'VGA|3D'` and edit before copying.

   ```bash
   sudo cp system/nvidia-hybrid/nvidia.conf /etc/modprobe.d/
   sudo cp system/nvidia-hybrid/99-gpu-symlinks.rules /etc/udev/rules.d/
   sudoedit /etc/udev/rules.d/99-gpu-symlinks.rules
   sudo udevadm control --reload-rules && sudo udevadm trigger
   ls -l /dev/dri/ | grep -E 'amd-igpu|nvidia-dgpu'   # two symlinks
   ```

4. **Tell Hyprland which GPU renders** (iGPU first):

   ```bash
   mkdir -p ~/.config/uwsm && cp system/nvidia-hybrid/env-hyprland ~/.config/uwsm/
   ```

   Never add global NVIDIA variables (`GBM_BACKEND`, `__GLX_VENDOR_LIBRARY_NAME`…): they would move
   every app to the NVIDIA GPU.

5. Reboot. Check: `printenv | grep AQ_DRM` inside the session, `nvidia-smi` lists the GPU.

**Games:** set each game's launch options in Steam to `prime-run %command%`, and turn off Steam →
Settings → Downloads → *Shader Pre-caching* (Steam would compile shaders for the integrated GPU on
every launch). **Never run games or benchmarks on battery** on the G15: under load it cut power
instantly.