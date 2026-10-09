# Hyprland From Scratch

A complete, minimal Hyprland desktop for Arch Linux, built piece by piece and kept entirely in this
repository so it can be installed again on any machine.

- **Look:** "Arch Neutral" — neutral greys with one accent, the Arch logo blue. Adwaita Sans for
  the interface, JetBrains Mono for the terminal.
- **Bar (Waybar):** workspaces, music, window title, screen recording, brightness and volume
  sliders, network / Bluetooth / audio menus, battery, power profiles, calendar.
- **Launchers and menus (Rofi):** quick app search on SUPER, full launcher, settings, wallpaper
  picker, keybinding help.
- **Lock, idle and login:** hyprlock, hypridle and an SDDM theme that all match.
- **Extras:** screenshots straight to the clipboard with annotation (Satty), screen recording,
  touchpad gestures, minimize / show desktop.

The full story of how it was built — every decision and bug — is in
[`docs/build-log.md`](docs/build-log.md). This README is the install guide.

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
git clone https://github.com/jzuletadev/hyprland-from-scratch.git ~/hyprland-from-scratch
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

1. installs the packages in `packages/base.txt` (asks for your sudo password);
2. copies or links every folder of `dotfiles/` into `~/.config/` — anything already there is kept
   as `<name>.bak-<date>`;
3. enables SDDM, Bluetooth and power profiles, and Waybar as a user service;
4. sets the default apps (images → Loupe, video and music → Celluloid);
5. installs the login screen theme.

**Hybrid laptop (AMD/Intel + NVIDIA)?** Do [annex A](#annex-a--hybrid-laptops-amdintel--nvidia)
before rebooting. Then reboot and pick **Hyprland (uwsm)** on the login screen.

Optional: AUR apps (Brave, VS Code, and `blesh-git` for suggestions while typing in the terminal) need an AUR helper:

```bash
git clone https://aur.archlinux.org/paru.git /tmp/paru && (cd /tmp/paru && makepkg -si)
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

`SUPER` is the Windows key — or the Menu key, the one left of the right Ctrl, which works as a
second `SUPER` (handy on 60% keyboards whose Fn layer blocks the Windows key). **`SUPER + /` shows this list on screen** (generated from the config,
so it's always current), and picking a row runs it.

| Keys | Action |
|------|--------|
| `SUPER` (tap) | App search — type, Enter |
| `SUPER + R` | Full launcher (apps / run / files / windows) |
| `SUPER + T` | Terminal (kitty) |
| `SUPER + E` | File manager (Thunar) |
| `SUPER + W` | Wallpaper picker |
| `SUPER + Esc` | System monitor (btop) |
| `SUPER + Q` | Close window |
| `SUPER + F` | Fullscreen on/off |
| `SUPER + V` | Floating on/off |
| `SUPER + arrows` | Focus the window in that direction |
| `SUPER + SHIFT + arrows` | Move the window (swap with its neighbor) |
| `SUPER + CTRL + arrows` | Resize the window (hold to repeat) |
| `SUPER + J` | Side by side ↔ stacked |
| `SUPER` + mouse drag | Left button: move · right button: resize |
| Drag a window's edge or corner | Resize it (no keys needed) |
| `SUPER + 1…9, 0` | Go to workspace 1–10 |
| `SUPER + SHIFT + 1…9, 0` | Move the window to that workspace (and follow it) |
| `SUPER + ALT + 1…9, 0` | Send the window to that workspace (stay here) |
| `SUPER + N` / `SUPER + SHIFT + N` | Minimize window / restore minimized |
| `SUPER + D` | Show desktop (minimize all / restore) |
| `SUPER + S` | Screenshot of the monitor — saved and copied to the clipboard |
| `SUPER + SHIFT + S` | Screenshot of a region — saved and copied |
| `SUPER + CTRL + S` | Screenshot of the window — saved and copied |
| `SUPER + ALT + S` | Screenshot of a region, opened in Satty to annotate |
| `SUPER + SHIFT + R` | Screen recording (again: stop and save) |
| `SUPER + L` | Lock |
| `SUPER + M` | Log out |

Click a screenshot's notification to annotate it. Screenshots go to `~/Pictures/screenshots`,
recordings to `~/Videos/recordings`.

**Touchpad:** 3 fingers down / up — minimize all / restore · 3 fingers left / right — focus the
window on that side · 4 fingers left / right — switch workspace · 4 fingers down / up — volume
· spread / pinch 3 fingers — fullscreen on / off.

**Bar:** Arch logo — system info (right-click: settings) · camera — record · brightness and volume —
hover for a slider, scroll to adjust · volume / network / Bluetooth — click for their menus · power
icon — switch profile · clock — hover for the calendar, click to browse months.

---

## Customizing

| What | Where |
|------|-------|
| Wallpaper | `SUPER + W`, or Thunar → right-click an image → *Set as wallpaper*. Add images to `assets/wallpapers/`. |
| Light / dark apps, accent color (menus, bar, notifications, lock screen), window border color, border width (windows and menus) | Right-click the Arch logo → Settings. By hand: `scripts/accent.sh` for the accent; border values live in `dotfiles/hypr/appearance.conf` and `dotfiles/rofi/appearance.rasi` |
| Keybindings, gestures, window rules | `dotfiles/hypr/hyprland.conf` — use `bindd` so the new bind shows in `SUPER + /` |
| Bar | `dotfiles/waybar/config.jsonc`, `style.css` |
| Menus and launcher | `dotfiles/rofi/` |
| Lock screen, idle timeouts | `dotfiles/hypr/hyprlock.conf`, `hypridle.conf` |
| Login screen | `system/sddm/` — then `sudo ./scripts/install-sddm-theme.sh` (it's copied into the system) |
| Notifications | `dotfiles/dunst/dunstrc` |
| Terminal | `dotfiles/kitty/kitty.conf` (`CTRL+SHIFT+A`, then `M` / `L`: more / less transparent, live) |
| Shell (history, suggestions, `Ctrl+R` search) | `dotfiles/bash/bashrc`, sourced from `~/.bashrc` — personal lines (tokens, paths) go in `~/.bashrc` |
| System info logo (Arch logo click) | Put an image at `dotfiles/fastfetch/logo.png` (`.jpg` / `.webp` too) — any shape, it's fitted into a square. Without one: the official Arch logo. |
| Packages | `packages/*.txt` |

The palette (Phase 15 of the build log): background `#141414`, surface `#1f1f1f`, text `#e0e0e0`,
accent `#1793d1`, light accent `#5db3df`, warning `#e0a84e`, critical `#e05f65`.

After editing (in `--link` mode): Hyprland reloads by itself; Waybar needs
`systemctl --user restart waybar`; Rofi and the lock screen read their files on every launch.

## Troubleshooting

| Problem | Fix |
|---------|-----|
| The bar disappeared | `systemctl --user restart waybar` (it also restarts by itself after a crash) |
| A config error banner at the top | `hyprctl configerrors` shows the line |
| Login screen looks like plain SDDM | `sudo ./scripts/install-sddm-theme.sh`, then log out |
| An app or website is light | Settings → Apps theme; Brave also needs its theme set to "Device" |
| Emoji or Asian text show as boxes | Install `noto-fonts-emoji` / `noto-fonts-cjk` (in `packages/base.txt`), `fc-cache -f`, then quit the browser completely (`pkill -x brave`: it keeps running in the background) |
| Stuck on a red "lockscreen died" screen | `Ctrl+Alt+F3`, log in, `hyprctl --instance 0 dispatch exec hyprlock` |
| The external monitor runs at 60 Hz | Add a `monitor = desc:…, <res>@<hz>, auto, 1` line (`hyprctl monitors all` lists modes) |

## Repository layout

```text
install.sh            installer (copy or --link mode)
dotfiles/             one folder per app, copied or linked into ~/.config/
scripts/              menus, bar helpers, screenshots, recording, wallpaper, idle, installers
assets/wallpapers/    wallpapers (any image you add shows in the picker)
packages/             package lists: base, apps, aur, nvidia-hybrid
system/               files copied outside ~/.config: SDDM theme, hybrid-GPU configs
docs/build-log.md     the phase-by-phase build story
```

---

## Annex A — Hybrid laptops (AMD/Intel + NVIDIA)

> **Only** for laptops with an integrated GPU **plus** an NVIDIA GPU of the Turing generation or
> newer (RTX 20/30/40, GTX 16). Check with `lspci | grep -E 'VGA|3D'`: two lines, one NVIDIA.
> AMD-only or Intel-only machines need nothing. A single-GPU NVIDIA desktop is a different setup,
> not covered here. Applying this on other hardware stops Hyprland from starting.

The desktop runs on the integrated GPU (cooler, longer battery); the NVIDIA GPU drives the external
outputs wired to it and runs the apps you start with `prime-run`. Built and tested on a Dell G15
(Ryzen + RTX 3060); details in the build log, Phase 12.

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
