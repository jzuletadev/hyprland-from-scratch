# Hyprland From Scratch

A complete, minimal Hyprland desktop for Arch Linux, built piece by piece and kept entirely in this
repository so it can be installed again on any machine.

- **No bar, no launcher.** The desktop is a set of panels that are born from the screen edges when
  you brush them, plus a wheel in the middle on `SUPER`. Nothing reserves screen space and nothing
  polls the cursor. It is one process, `hfs`, written in Rust with GTK4 and wlr-layer-shell.
- **The surfaces:** volume and brightness on the right edge; workspaces on the left; clock and
  calendar bottom-right; media and system monitor top-centre; power top-left; notifications
  top-right; clipboard history bottom-left. Inside the wheel: app search, Wi-Fi, Bluetooth,
  screenshots and recording, wallpaper, and theme.
- **Dynamic theme engine:** 6 palettes (**Nord**, **Tokyo Night**, **Everforest**, **Dracula**,
  **Rosé Pine Moon**, **Monokai**) applied live across Hyprland borders, Kitty, Dunst, Yazi and GTK.
  The desktop follows the change without restarting.
- **Typography:** Adwaita Sans for the interface, JetBrainsMono Nerd Font for the terminal.
- **File manager:** `thunar`.
- **Lock, Idle & Login:** `hyprlock`, `hypridle` (battery-aware), and a matching SDDM theme that
  automatically syncs with your current wallpaper.
- **Screenshots & Recording:** screenshots straight to the clipboard with annotation (`satty`) and
  hardware-friendly screen/region recording (`wf-recorder`).

> **v1 and v2.** This desktop was first built with Waybar, Rofi and eleven Python `curses` TUIs.
> Those are gone: see [`V2-PLAN.md`](V2-PLAN.md) for what replaced each one and what is still left.
>
> **Working on it?** [`DESARROLLO.md`](DESARROLLO.md) is the hands-on loop — build, install
> over the running binary, restart, and check nothing broke. [`crates/README.md`](crates/README.md)
> explains the architecture and the traps.

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
   `bluetooth`, `power-profiles-daemon`, and `iwd`);
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

`SUPER` is the Windows key. **`SUPER + /` opens the Atajos panel** (generated live from
`hyprctl binds`), where you can filter every shortcut by name.

> **Wi-Fi, Bluetooth, volume, brightness, power, power profiles and the calendar no longer have
> their own key.** They live on the always-present edge surfaces and inside the wheel: brush the
> right edge for volume and brightness, the top-left corner for power, the bottom-right for the
> calendar, and tap `SUPER` for Wi-Fi and Bluetooth.

### Apps & panels

| Keys | Action |
|------|--------|
| `SUPER` (tap) | The wheel — app search and every settings panel |
| `SUPER + SHIFT + A` | Peek every edge surface at once |
| `SUPER + T` | Terminal (`kitty`) |
| `SUPER + E` | File manager (`thunar`) |
| `SUPER + SHIFT + E` | File manager (`thunar`; hidden files are `Ctrl+H` inside it) |
| `SUPER + A` | Theme picker — the wheel's Tema panel |
| `SUPER + W` | Wallpaper picker — the wheel's Fondo panel |
| `SUPER + M` | Mute / unmute audio |
| `SUPER + R` | Screenshots and screen recording — the wheel's Captura panel |
| `SUPER + /` | Keybindings help and search — the wheel's Atajos panel |
| `SUPER + L` | Lock screen (`hyprlock`) |

### Windows & Workspaces

| Keys | Action |
|------|--------|
| `SUPER + Q` | Close active window |
| `SUPER + F` | Toggle floating window |
| `SUPER + SHIFT + F` | Toggle fullscreen |
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
| `SUPER + R` | The Captura panel — the four screenshots and screen recording |

Click a screenshot's notification to open it in Satty for annotation. Screenshots are saved to
`~/Pictures/screenshots` and recordings to `~/Videos/recordings`.

**Surface interactions:**
- **Arch logo** (top of the left rail): opens the machine's card — model, CPU, both GPUs, kernel,
  packages, disk, uptime and the active palette.
- **Workspaces** (left rail): click a dot to switch · scroll anywhere on the rail to move through
  them, including to one that does not exist yet.
- **Volume & brightness** (right edge): drag the bars, or use the media keys — the panel peeks on
  its own when you do.
- **Notifications** (top-right): click an entry to put it back on screen, which is the only thing
  that brings its actions back.
- **Clipboard** (bottom-left): click an entry to copy it again.

---

## Customizing

| What | Where |
|------|-------|
| Theme | Press `SUPER + A` (the wheel's Tema panel). The v2 palettes live in `palettes/*.toml`; `scripts/theme-manager.sh` applies the matching Yazi flavor to Kitty, Dunst, Yazi, GTK and the Hyprland borders. |
| Wallpaper | Press `SUPER + W`. Put your images in `~/Pictures/wallpapers/` (initial defaults come from `assets/wallpapers/`). |
| Keybindings & Window Rules | `dotfiles/hypr/hyprland.conf` — use `bindd` so new shortcuts get a readable description in `SUPER + /` (undescribed ones still show up, labelled with what they run). |
| The surfaces and the wheel | `crates/` — one binary, `hfs`. See [`crates/README.md`](crates/README.md). |
| Lock Screen & Idle | `dotfiles/hypr/hyprlock.conf` and `dotfiles/hypr/hypridle.conf` (triggers `scripts/idle.sh` on battery). |
| Login Screen (SDDM) | `system/sddm/` — run `sudo ./scripts/install-sddm-theme.sh` to update system files. |
| Notifications | `dotfiles/dunst/dunstrc` (accent frame auto-generated in `dunstrc.d/50-accent.conf`). |
| Terminal | `dotfiles/kitty/kitty.conf` (dynamic palette written to `dotfiles/kitty/theme.conf` and reloaded live via socket). |
| File Manager | `thunar`. Yazi is still installed because its flavors are where `theme-manager.sh` reads its colours from (`dotfiles/yazi/flavors/`). |
| Shell | `dotfiles/bash/bashrc`, sourced from `~/.bashrc` — keep personal tokens/paths in `~/.bashrc`. |
| System Info Logo | Place a custom image at `dotfiles/fastfetch/logo.png` (`.jpg` / `.webp` supported). Falls back to the official Arch logo. |
| Packages | `packages/*.txt` |

After editing (in `--link` mode): Hyprland reloads automatically; Kitty updates colors live; the
desktop watches `~/.config/hypr` and rebuilds its surfaces when the theme changes. After editing
the Rust, `cargo build --release` and restart it with `pkill -x hfs` (Hyprland's `exec-once` does
not bring it back — run `~/.local/bin/hfs shell &` or log out and in).

## Troubleshooting

| Problem | Fix |
|---------|-----|
| The edge panels disappeared | `pkill -x hfs; ~/.local/bin/hfs shell &` · `hyprctl layers` should list one `hfs-*` namespace per surface per monitor |
| A surface opened on the wrong monitor | The compositor decides that at map time; restart `hfs shell` with the monitor already connected |
| A config error banner at the top | `hyprctl configerrors` shows the exact line |
| Login screen looks like plain SDDM | `sudo ./scripts/install-sddm-theme.sh`, then log out |
| Emoji or Asian text show as boxes | Install `noto-fonts-emoji` / `noto-fonts-cjk` (in `packages/base.txt`), run `fc-cache -f`, and restart your browser (`pkill -x brave`) |
| Stuck on a red "lockscreen died" screen | `Ctrl+Alt+F3`, log in, `hyprctl --instance 0 dispatch exec hyprlock` |
| External monitor runs at 60 Hz | Add a `monitor = desc:…, <res>@<hz>, auto, 1` line to `hyprland.conf` (`hyprctl monitors all` lists modes) |

## Repository layout

```text
install.sh            installer (copy or --link mode)
crates/               the desktop: one Rust binary, `hfs` (see crates/README.md)
design/               the navigable mockup the v2 was built from
palettes/             the v2 palettes (*.toml), owned by this repo
dotfiles/             one folder per app, copied or linked into ~/.config/
  yazi/flavors/       Yazi color flavors, still the source for theme-manager.sh
scripts/              Bash glue (*.sh): install, theme engine, screenshots, wallpaper
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