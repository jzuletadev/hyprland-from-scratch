# Build log

> The phase-by-phase story of how this desktop was built: every decision, bug and fix, in
> order. The install guide is the [README](../README.md); this file is for the *why*.
> Section links and "Phase N" references below point within this file.

# Hyprland From Scratch

A step-by-step guide to build a clean, minimal and fully understood Hyprland desktop on Arch Linux.
The goal is not to install a desktop as fast as possible, but to understand every component that is installed and keep the entire configuration under version control.

---

# Project Goals

- Build everything from scratch.
- Install only what is needed.
- Understand every package before installing it.
- Version all dotfiles.
- Reproduce the entire desktop from this repository.

---

# Repository Structure

```text
hyprland-from-scratch/
├── dotfiles/          # one folder per app, each symlinked as a whole into ~/.config/
├── scripts/           # Rofi menus used by Waybar + link-dotfiles.sh (restore helper)
├── assets/wallpapers/ # wallpapers referenced by hyprland.conf and hyprlock.conf
├── packages/          # package lists per layer (base, nvidia-hybrid, apps, aur)
├── system/            # files that live outside ~/.config (copied with sudo, never symlinked)
│   ├── nvidia-hybrid/ # Phase 12 only — hybrid iGPU + NVIDIA laptops
│   └── sddm/          # Phase 24 — login screen theme + SDDM config
└── README.md
```

> **The repo must be cloned at `~/hyprland-from-scratch`.** Waybar, Hyprland, hyprlock and the
> Rofi scripts reference files by that absolute path.

---

# How to Use This Guide

There are two different ways to use this repository. Pick one:

| Situation | Path | What you do |
|-----------|------|-------------|
| Learning / building it for the first time | **Path A — From scratch** | Follow Phase 0 → Phase 37 in order. Every package and config is explained. |
| New machine, reinstall, or disaster recovery | **Path B — Restore** | Skip the explanations: install packages, clone, run one script. See [Restore on a New Machine](#restore-on-a-new-machine). |

The phases explain **why** each piece exists and the bugs found along the way; Path B relies on
that work already being done and committed. When something breaks after a restore, the matching
phase is where to look.

## Hardware this was built on

Most of the guide is hardware-independent. The parts that are **not**:

| Phase | Applies to | On other hardware |
|-------|-----------|-------------------|
| 5.5 — Brightness | Laptops (internal panel with a backlight) | Skip. Remove `backlight` from `waybar/config.jsonc`. |
| 5.6 — Touchpad | Laptops | Harmless on desktops (the block is ignored without a touchpad). |
| 31 — Touchpad gestures | Laptops / touchpads | Harmless without a touchpad. |
| Waybar `battery` module | Laptops | Remove `battery` from `waybar/config.jsonc` on desktops. |
| 13 — `monitor = desc:AOC 27G2G4 ...` | That exact external monitor (144 Hz) | Harmless — other monitors fall back to `preferred`. Add a `desc:` rule for any monitor whose preferred mode isn't its fastest one. |
| 30 — Steam shader pre-caching off | Hybrid iGPU + NVIDIA laptops (games run with `prime-run`) | Keep pre-caching on. |
| 12 — NVIDIA hybrid GPU | **Only** laptops with an AMD/Intel iGPU **plus** an NVIDIA dGPU (Turing/RTX 20 or newer, open kernel module) | **Skip entirely.** AMD-only or Intel-only machines need nothing extra. A desktop with a single NVIDIA GPU needs a different setup that this guide does not cover — see the [Hyprland NVIDIA wiki page](https://wiki.hypr.land/Nvidia/). |

Reference machine: Dell G15 Ryzen Edition (5000 series) — AMD Radeon Vega iGPU (Cezanne) +
NVIDIA GeForce RTX 3060 Mobile (GA106M, Ampere), internal panel on the AMD, HDMI/DP on the NVIDIA.

---

# Restore on a New Machine

Path B. Use this when the repo is already complete and you just need the same desktop on another
(or a freshly reinstalled) machine. Expect ~30 minutes, most of it downloads.

### 1. Base system

Do [Phase 0](#phase-0--arch-linux) (archinstall with the same table) and the Git/SSH parts of
[Phase 0.5](#phase-05--initial-system-setup). `linux-headers` in the archinstall packages only
matters for Phase 12, but it doesn't hurt elsewhere.

Clone **to the exact path**:

```bash
git clone git@github.com:<user>/hyprland-from-scratch.git ~/hyprland-from-scratch
cd ~/hyprland-from-scratch
```

### 2. Packages

```bash
grep -v '^#' packages/base.txt | sudo pacman -S --needed -
fc-cache -f
~/hyprland-from-scratch/scripts/default-apps.sh   # images -> Loupe, video/audio -> Celluloid (Phase 33)
```

Optional layers:

```bash
# Personal apps (Docker, Node, Steam, ...). Steam needs [multilib] enabled in /etc/pacman.conf first.
grep -v '^#' packages/apps.txt | sudo pacman -S --needed -

# AUR helper, then AUR apps (Brave, VS Code)
git clone https://aur.archlinux.org/paru.git /tmp/paru && (cd /tmp/paru && makepkg -si)
grep -v '^#' packages/aur.txt | paru -S --needed -
```

### 3. Dotfiles

```bash
~/hyprland-from-scratch/scripts/link-dotfiles.sh --dry-run   # preview
~/hyprland-from-scratch/scripts/link-dotfiles.sh
```

The script symlinks every folder in `dotfiles/` into `~/.config/` (moving any existing folder to
`<name>.bak-<timestamp>` instead of deleting it) and creates `~/Pictures/screenshots`. It's
idempotent — re-run it after adding a new `dotfiles/<app>/` folder.

### 4. Services

```bash
sudo systemctl enable sddm
sudo systemctl enable --now bluetooth
systemctl --user enable waybar.service   # the bar (Phase 22); no sudo — it's a user service

# login screen theme (Phases 24/37), with your current wallpaper as background
sudo ~/hyprland-from-scratch/scripts/install-sddm-theme.sh
```

(NetworkManager is already enabled by archinstall. If you installed `apps.txt`:
`sudo systemctl enable --now docker power-profiles-daemon`.)

### 5. Hardware-specific steps

- **Hybrid iGPU + NVIDIA laptop only:** do [Phase 12](#phase-12--nvidia-hybrid-gpu) now, using the
  ready-made files in `system/nvidia-hybrid/`. **Check the PCI addresses** in
  `99-gpu-symlinks.rules` with `lspci` — they differ between laptop models.
- **Desktop (no battery/backlight):** remove `backlight` and `battery` from `waybar/config.jsonc`.
- **Any machine:** check `kb_layout` in `hyprland.conf`, and run `hyprctl monitors all` — if a
  monitor runs below its fastest refresh rate, add a `desc:` rule for it (Phase 13).

### 6. Reboot and verify

```bash
sudo reboot
```

At SDDM pick **Hyprland (uwsm)**. Then confirm:

- Wallpaper, Waybar (pills + Arch logo), and dunst start on their own.
- `SUPER+T` opens kitty in the Arch Neutral colors (Phase 15); tapping `SUPER` opens Rofi.
- `SUPER+L` locks; volume keys move the Waybar volume level.
- `ls -l ~/.config` shows every app pointing into `~/hyprland-from-scratch/dotfiles/`.

**Not restored by this repo** (re-create by hand): SSH keys and `git config`, browser/VS Code
profiles (use their own account sync), Wi-Fi passwords, and anything in `~/Documents`.

---

# Phase 0 — Arch Linux

Boot the official Arch ISO and start the installer.

```bash
archinstall
```

Use the following configuration:

| Setting             | Value                                           |
| ------------------- | ----------------------------------------------- |
| Locale              | en_US.UTF-8                                    |
| Keyboard            | us *(or latam if preferred)*                    |
| Mirrors             | Automatic                                       |
| Disk                | Use entire disk                                 |
| Bootloader          | systemd-boot (default)                          |
| Swap                | Default                                         |
| Hostname            | Your choice                                     |
| Root Password       | Configure                                       |
| User Account        | Create user + wheel                             |
| Profile             | Minimal / None                                  |
| Audio               | PipeWire                                        |
| Network             | NetworkManager                                  |
| Timezone            | Your timezone                                   |
| Additional packages | `git curl base-devel openssh linux-headers`     |

> **Why linux-headers:** required for any DKMS-built kernel module (notably
> `nvidia-open-dkms` in Phase 12). Without it, DKMS silently installs no module —
> `dkms status` comes back empty and `nvidia-smi` fails with
> "couldn't communicate with the NVIDIA driver" even though the package is
> installed. Easier to include it at install time than to debug it later.

Install the system and reboot.

```bash
reboot
```

Expected state:

- Arch boots into a TTY.
- Internet works.
- No desktop environment installed.
- No display manager installed.

---

# Phase 0.5 — Initial System Setup

Update the system.

```bash
sudo pacman -Syu
```

Enable SSH.

```bash
sudo pacman -S openssh
sudo systemctl enable --now sshd
```

Get the machine IP.

```bash
ip a
```

Connect from the main computer.

```bash
ssh user@<ip>
```

Configure Git.

```bash
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git config --global init.defaultBranch main
```

Generate an SSH key.

```bash
ssh-keygen -t ed25519 -C "you@example.com"
```

Display the public key.

```bash
cat ~/.ssh/id_ed25519.pub
```

Add it to GitHub. Test the connection.

```bash
ssh -T git@github.com
```

Clone the repository.

```bash
cd ~
git clone git@github.com:<user>/hyprland-from-scratch.git
cd hyprland-from-scratch
```

Expected state:

- SSH access works.
- Git is configured.
- GitHub authentication works.
- Repository cloned locally.

> **Note:** from this point on, SSH is only useful for editing files and reviewing logs.
> Hyprland itself must be launched from the physical TTY (not over SSH), since it needs
> direct access to the DRM/seat device.

---

# Phase 1 — Minimal Hyprland

Install only the required packages.

```bash
sudo pacman -S \
    hyprland \
    kitty \
    xdg-desktop-portal \
    xdg-desktop-portal-hyprland \
    ttf-jetbrains-mono-nerd
```

> **Why a font package here:** a minimal Arch install ships with zero fonts. Kitty (and most
> GUI apps) resolve their default font through fontconfig at startup — with no monospace font
> available, kitty fails with `FcFontMatch() failed` and crashes silently before ever creating
> a window. Hyprland itself starts fine and the compositor stays up, so the failure is easy to
> misread as an input/keybind problem instead of a missing font. A Nerd Font variant is used
> here because Waybar and other later pieces of this setup rely on the icon glyphs it bundles.

Refresh the font cache and confirm it resolves:

```bash
fc-cache -fv
fc-match monospace
```

Create the configuration directory.

```bash
mkdir -p ~/.config/hypr
nano ~/.config/hypr/hyprland.conf
```

Set minimal and basic configuration.

```text
monitor = ,preferred,auto,1
bind = SUPER, T, exec, kitty
bind = SUPER, M, exit
```

Save (`Ctrl+O`, `Enter`, `Ctrl+X`).

### Before launching: make sure no stale session is running

If you've tried launching Hyprland before and it crashed, closed abruptly, or you switched
TTYs without exiting it properly, a leftover process or lockfile can silently block the new
session (you'll see `Unable to lock lockfile ... maybe another compositor is running` in the
log, and keybinds like opening a terminal simply won't do anything, even though Hyprland
appears to be running).

```bash
ps aux | grep -i hypr
```

If a `Hyprland` process shows up, kill it:

```bash
killall -9 Hyprland
```

Check for a leftover lock:

```bash
ls -la /run/user/$(id -u)/ | grep wayland
```

If a `wayland-*.lock` file exists with no live process behind it, remove it:

```bash
rm -f /run/user/$(id -u)/wayland-1.lock
```

> **Watch out for other open TTY sessions.** `logind` only keeps one session active per seat.
> If you have a login open on another TTY (e.g. you switched to tty2 to test something, or an
> old SSH-triggered login session is still sitting there), it can silently steal the seat from
> the TTY running Hyprland — the compositor keeps rendering, but stops receiving keyboard/mouse
> input entirely, with no visible error. Check with `loginctl list-sessions` and make sure the
> session on the TTY running Hyprland shows `State: active` before troubleshooting anything
> else. Reactivate it with `sudo loginctl activate <session-id>` if needed.

### Start Hyprland

Launch it wrapped in a D-Bus session (plain `Hyprland` triggers a
"launched without start-hyprland" warning and D-Bus/portal activation failures):

```bash
dbus-run-session Hyprland
```

### Verify the config actually loaded

```bash
hyprctl binds | grep -A2 kitty
```

You should see the `SUPER, T, exec, kitty` bind listed. If it's missing, the config wasn't
read — recheck the file path and that it was saved.

If a keybind is present but still doesn't do anything, confirm kitty is actually installed
and reachable:

```bash
which kitty
```

Verify:

```bash
echo $XDG_SESSION_TYPE
```

Expected output:

```text
wayland
```

If you already have Hyprland running and just changed the config, you don't need to restart
the whole session — reload it live:

```bash
hyprctl reload
```

Expected state:

- Hyprland starts successfully.
- `hyprctl binds` shows the kitty bind.
- Kitty opens with `SUPER+T`.
- Session runs on Wayland.

---

# Phase 2 — Repository Integration

Create the dotfiles structure.

```text
dotfiles/
└── hypr/
```

Move the configuration into the repository.

```bash
mkdir -p ~/hyprland-from-scratch/dotfiles
mv ~/.config/hypr \
   ~/hyprland-from-scratch/dotfiles/
```

Create a symbolic link.

```bash
ln -s \
~/hyprland-from-scratch/dotfiles/hypr \
~/.config/hypr
```

Verify.

```bash
ls -l ~/.config
```

Expected output:

```text
hypr -> ~/hyprland-from-scratch/dotfiles/hypr
```

Commit the current state.

```bash
git add .
git commit -m "Initial Hyprland setup"
```

Expected state:

- Hyprland works.
- Configuration is stored inside the repository.
- `~/.config` only contains a symbolic link.
- Git detects configuration changes automatically.

> Every later phase repeats this same pattern (`dotfiles/<app>/` + `ln -s` into `~/.config/`).
> `scripts/link-dotfiles.sh` does it for all folders at once — that's what Path B uses.

---

# Phase 3 — Waybar

Waybar is the status bar. Install empty/minimal first, confirm it renders, then add modules
one at a time.

```bash
sudo pacman -S waybar
mkdir -p ~/hyprland-from-scratch/dotfiles/waybar
```

Place `config.jsonc` (defines **which** modules appear and where — `modules-left`, `modules-center`, `modules-right`) and `style.css` (defines **how** it looks — same selector
model as normal CSS) at `~/hyprland-from-scratch/dotfiles/waybar/`. Baseline: only `hyprland/workspaces` on the left and `clock` centered — no network, battery, or tray yet.

Symlink:

```bash
ln -s ~/hyprland-from-scratch/dotfiles/waybar ~/.config/waybar
```

Test manually first:

```bash
waybar &
```

Once confirmed working, autostart it in `hyprland.conf`:

```text
exec-once = waybar
```

`exec-once` (vs `bind ... exec`) runs once at session start only — using plain `exec` here
would spawn a new Waybar instance on every `hyprctl reload`.

> **Phase 22 replaces this `exec-once` with a systemd user service** that restarts Waybar if it
> crashes. From then on, restart it with `systemctl --user restart waybar`.

Expected state:

- Waybar appears at the top of the screen with workspaces + clock.
- It starts automatically with the session, without manual `waybar &`.

---

# Phase 4 — Application Launcher (Rofi)

Rofi vs Wofi: Wofi is more minimal and Wayland-native from the start; Rofi is more mature,
far more feature-rich (emoji picker, clipboard manager, extensions via `rofi-calc` etc.), and
is what most Hyprland community configs and documentation reference. Rofi is used here for
that ceiling — the goal is deep customization later, not just "an app launcher."

```bash
sudo pacman -S rofi
mkdir -p ~/hyprland-from-scratch/dotfiles/rofi
```

Place `config.rasi` at `~/hyprland-from-scratch/dotfiles/rofi/config.rasi`. Key points:

- `modi: "drun"` — "desktop run" mode: reads installed `.desktop` files to build the app list.
  Other modes exist (`run`, `window`, `ssh`) and can be added later.
- `@theme "/dev/null"` — disables any system default theme so every color rendered comes
  explicitly from this file, not a hidden default.

Symlink:

```bash
ln -s ~/hyprland-from-scratch/dotfiles/rofi ~/.config/rofi
```

Add a bind in `hyprland.conf`:

```text
bind = $mainMod, R, exec, rofi -show drun
```

Reload and test:

```bash
hyprctl reload
```

`SUPER+R` should open the launcher; typing an app name (e.g. "kitty") should find and launch it.

---

# Phase 5 — Notifications (dunst)

Two main options exist: **mako** (native Wayland, minimal, key=value config) or **dunst** (more mature, more configurable — per-app rules, urgency levels, scripting, history). Dunst is
used here: the goal is deep future customization, and dunst's extra surface area is exactly
the kind of thing worth learning early rather than working around later.

Install:

```bash
sudo pacman -S dunst libnotify
```

Create the config directory in the repo:

```bash
mkdir -p ~/hyprland-from-scratch/dotfiles/dunst
```

Place a `dunstrc` file at `~/hyprland-from-scratch/dotfiles/dunst/dunstrc` defining `[global]` plus `[urgency_low]`, `[urgency_normal]`, and `[urgency_critical]` sections — the urgency
levels are dunst's key advantage over mako: `urgency_critical` can be set to `timeout = 0` so
critical notifications never auto-dismiss.

Symlink:

```bash
ln -s ~/hyprland-from-scratch/dotfiles/dunst ~/.config/dunst
```

Autostart in `hyprland.conf`:

```text
exec-once = dunst
```

Reload and test all three urgency levels:

```bash
hyprctl reload
notify-send "Normal" "Standard notification"
notify-send -u low "Low" "Low priority"
notify-send -u critical "Critical" "Should not auto-dismiss"
```

> **If notifications already worked before applying this config:** that's expected — dunst
> ships with a built-in default config and runs fine without a `dunstrc` present. Without the
> file (and its symlink) applied, you get generic default styling and no custom urgency
> behavior, not an error. After symlinking, restart the daemon to pick up the new config:
>
> ```bash
> killall dunst
> dunst &
> ```

Expected state:

- dunst starts automatically with the session.
- Notifications match the rest of the setup's color scheme.
- Critical notifications stay on screen until dismissed.

---

# Phase 5.5 — Brightness control

> **Laptops only.** External desktop monitors don't expose a kernel backlight; skip this phase
> and remove the `backlight` module from Waybar.

Hyprland doesn't handle brightness on its own — brightness keys just send a keycode; actually
changing the panel's backlight needs dedicated tooling.

Install:

```bash
sudo pacman -S brightnessctl
```

Confirm the backlight device is detected:

```bash
brightnessctl
```

Test manually before binding to keys:

```bash
brightnessctl set 10%-
brightnessctl set 10%+
```

Find the actual key names your keyboard sends:

```bash
sudo libinput debug-events
```

(press the brightness keys, look for `KEY_BRIGHTNESSUP` / `KEY_BRIGHTNESSDOWN`, `Ctrl+C` to exit)

Add binds in `hyprland.conf`:

```text
bindel = ,XF86MonBrightnessUp, exec, brightnessctl set 5%+
bindel = ,XF86MonBrightnessDown, exec, brightnessctl set 5%-
```

`bindel` (vs plain `bind`) allows key-repeat while held (`e`) and works even when the screen
is locked later on (`l`), once hyprlock is in place.

Reload and test with the physical keys:

```bash
hyprctl reload
```

---

# Phase 5.6 — Touchpad scroll direction

If two-finger scroll feels inverted, it's controlled by `natural_scroll` in the `touchpad` block of `hyprland.conf`. `true` = content follows finger direction (like mobile/macOS); `false` = classic inverted-wheel behavior.

```text
input {
    touchpad {
        natural_scroll = true
    }
}
```

Reload to apply:

```bash
hyprctl reload
```

> This setting only affects the touchpad. An external mouse's scroll direction is controlled
> separately via `input { natural_scroll = ... }` at the top level of the `input` block (outside `touchpad`), and is usually left `false` since inverted scroll feels unnatural on a mouse.

---

# Phase 6 — Wallpaper (awww)

> Phase 17 adds a picker (`SUPER+W`) on top of what's set up here, and the last choice is restored
> at login instead of a fixed file.

> **Naming note:** the tool referenced across most community guides as `swww` was renamed by
> its creator to **awww** after the original `swww` project was archived. Arch's official
> repos now only ship `awww` — `pacman -S swww` resolves to the `awww` package automatically,
> but the binaries are `awww` and `awww-daemon`, not `swww`/`swww-daemon`. All commands below
> use the current names.

Install:

```bash
sudo pacman -S awww
```

awww has no config file — it's driven entirely by command-line flags — so there's no
`dotfiles/awww/` folder and no symlink. The only repo folder it needs is for the images:

```bash
mkdir -p ~/hyprland-from-scratch/assets/wallpapers
```

Place a wallpaper image at `~/hyprland-from-scratch/assets/wallpapers/`. This repo ships three
(`1.jpg`, `2.png`, `3.jpeg`); `1.jpg` is the one set at startup and reused by hyprlock (Phase 7). No image handy? Generate
a solid-color placeholder locally (no internet needed beyond the Arch mirrors already used for
pacman):

```bash
sudo pacman -S imagemagick
magick -size 1920x1080 xc:'#1a1b26' ~/hyprland-from-scratch/assets/wallpapers/1.jpg
```

awww needs its daemon running before it can set a wallpaper:

```bash
awww-daemon &
awww img ~/hyprland-from-scratch/assets/wallpapers/1.jpg
```

Test an animated transition (this is the actual reason to prefer this tool over hyprpaper):

```bash
awww img ~/hyprland-from-scratch/assets/wallpapers/1.jpg --transition-type wipe --transition-duration 1.5
```

> **How to actually see the transition:** the transition is a property of the `awww img` command itself, not a separate toggle — but it's only visible when switching *between two
> different images*. Running it with the same image that's already set technically still runs
> the transition, but with no visual difference between "before" and "after" there's nothing to
> see. Keep at least two test images in `assets/wallpapers/` and alternate between them:
>
> ```bash
> awww img ~/hyprland-from-scratch/assets/wallpapers/2.png --transition-type wipe --transition-duration 1.5
> awww img ~/hyprland-from-scratch/assets/wallpapers/3.jpeg --transition-type grow --transition-duration 1.5
> ```
>
> Some `--transition-type` values worth comparing: `simple` (fade), `wipe`, `wave`, `grow`, `outer`, `random` (picks one at random each time).

Autostart in `hyprland.conf`:

```text
exec-once = awww-daemon
exec-once = awww img ~/hyprland-from-scratch/assets/wallpapers/1.jpg
```

Reload:

```bash
hyprctl reload
```

For a full test, restart the whole Hyprland session (not just reload) and confirm the
wallpaper appears on its own, without running any command by hand.

---

# Phase 7 — Lock Screen (hyprlock)

> Phase 24 redesigns this lock screen and adds a crash safety net.

```bash
sudo pacman -S hyprlock
```

hyprlock's config lives alongside `hyprland.conf`, in the same `hypr/` folder — as its own
file, `hyprlock.conf`. No new symlink is needed since that whole folder is already linked from
Phase 2; any file added there shows up under `~/.config/hypr/` automatically.

Baseline `hyprlock.conf` has three blocks:

- `background` — the blurred wallpaper shown behind the password field.
- `input-field` — the password entry box.
- `label` — free text, e.g. a live clock via `cmd[update:1000]` (re-runs `date` every second).

> **Common mistake:** it's easy to accidentally paste this block into `hyprland.conf` instead
> of a separate `hyprlock.conf` file — the error will look like `config option <background:path> does not exist` in the Hyprland log. The file path in the error message is
> the giveaway: if it points at `hyprland.conf` instead of `hyprlock.conf`, the block landed in
> the wrong file. Cut it out and move it to its own file to fix it.

Add a bind in `hyprland.conf`:

```text
bind = $mainMod, L, exec, hyprlock
```

Reload and test:

```bash
hyprctl reload
```

`SUPER+L` should show the lock screen; typing your user password should unlock it.

> Keep an SSH session open while testing — `killall hyprlock` from there is the escape hatch
> if the lock screen ever gets stuck.

---

# Phase 8 — Idle Management (hypridle)

> Phase 24 fixes the dim stage and makes suspend battery-only.

hypridle watches for inactivity and fires staged actions — dim, lock, screen off, suspend —
integrating directly with hyprlock from Phase 7.

```bash
sudo pacman -S hypridle
```

Baseline `hypridle.conf` structure:

- `general` block — `lock_cmd` (what locks the session), `before_sleep_cmd`/`after_sleep_cmd` (run right before/after suspend).
- One `listener { }` block per stage, each independent: its own `timeout` (seconds) and
  `on-timeout` action, with `on-resume` reverting it once activity resumes. Current staging:
  dim at 15 min (`900`), lock at 30 min (`1800`), screen off at 35 min (`2100`), suspend at
  75 min (`4500`). (The first baseline was 150/300/330/900 seconds — too aggressive for daily use.)

Place the file alongside the others (no new symlink needed) and autostart it:

```text
exec-once = hypridle
```

```bash
hyprctl reload
```

> **Testing tip:** minute-long timeouts are slow to verify. Temporarily lower every `timeout` to a handful of seconds, restart the daemon (`pkill hypridle && hypridle &`), confirm each
> stage fires in order, then restore the real values (900/1800/2100/4500) once confirmed.

---

# Phase 9 — SDDM (login screen)

> Phase 24 adds a login theme matching the lock screen.

SDDM provides a traditional graphical login (username + password) instead of TTY autologin,
and — as a side benefit — launches Hyprland correctly on its own, resolving the earlier
"launched without start-hyprland" warning without needing `dbus-run-session Hyprland` by hand.

Install:

```bash
sudo pacman -S sddm uwsm
sudo systemctl enable sddm
```

> `uwsm` is a separate package. The `hyprland-uwsm.desktop` session file below ships with
> `hyprland`, but selecting it without `uwsm` installed just drops you back at the login screen.

Confirm Hyprland exposes its session files (installed automatically by the `hyprland` package):

```bash
ls /usr/share/wayland-sessions/
```

You should see both `hyprland.desktop` and `hyprland-uwsm.desktop` — prefer the **uwsm** variant in SDDM's session picker. `uwsm` (Universal Wayland Session Manager) is the modern
recommended launch method: it handles D-Bus, environment variables, and session lifecycle
correctly, replacing both the old `start-hyprland` script and a manual `dbus-run-session Hyprland` invocation.

> **Before rebooting**, make sure no TTY autologin override is left configured — it can
> conflict with SDDM taking over the login:
>
> ```bash
> cat /etc/systemd/system/getty@tty1.service.d/override.conf 2>/dev/null
> ```
>
> If this returns nothing, there's nothing to clean up.

Reboot:

```bash
sudo reboot
```

At the login screen: pick your user, select **Hyprland (uwsm)** from the session picker
(usually a gear icon or dropdown near the password field), enter your password.

Expected state:

- SDDM shows a graphical login instead of a TTY prompt.
- Hyprland starts via uwsm.
- The "launched without start-hyprland" warning no longer appears in the log.

---

# Phase 10 — Theming (Tokyo Night)

> **Superseded by [Phase 15](#phase-15--arch-neutral-theme-replaces-tokyo-night) (Arch Neutral).**
> The palette below is kept as history; the approach — one palette applied to every component —
> is still how the theme works.

Everything up to now used ad hoc colors picked per file (a stray Nord cyan border here, a
Catppuccin-ish background there). This phase formalizes one palette and applies it consistently
across every piece already built: Hyprland borders/blur, Waybar, Rofi, dunst, and hyprlock.
Nothing new gets installed — this is config only.

**Palette (Tokyo Night):**

| Role                           | Hex       |
| ------------------------------ | --------- |
| Background                     | `#1a1b26` |
| Surface (panels, input fields) | `#24283b` |
| Highlight (hover/selected)     | `#292e42` |
| Foreground                     | `#c0caf5` |
| Muted foreground               | `#565f89` |
| Accent — blue                  | `#7aa2f7` |
| Accent — purple                | `#bb9af7` |
| Warning                        | `#e0af68` |
| Critical                       | `#f7768e` |

## Hyprland: borders, blur, rounding

`col.active_border` becomes a blue-to-purple gradient, `col.inactive_border` a translucent
dark surface color, corner `rounding` goes from 6 to 10, and window blur gets enabled (small `size`/`passes` — this machine's internal panel is driven by the AMD iGPU, not the NVIDIA GPU,
see Phase 12 below, so a light blur has headroom to spare). `active_opacity` / `inactive_opacity` add a subtle transparency to unfocused windows.

`layerrule = blur, waybar` and `layerrule = blur, rofi` extend that same blur to the bar and
launcher — without this, layer-shell surfaces stay flat even if their own CSS/rasi sets an
alpha-transparent background.

## Waybar: from 2 modules to a real bar

The Phase 3 baseline only had workspaces + a clock. This phase adds:

- `hyprland/window` (center) — the focused window's title.
- `network`, `wireplumber`, `battery` (right, before the clock) — all backed by daemons already
  running on this system (NetworkManager, wireplumber, upower), so no new packages needed.

Verify the modules Waybar was actually built with, since a distro package can be compiled
without some of them:

```bash
pacman -Qi waybar | grep Depends
```

`libwireplumber` and `libupower-glib` should be listed — confirmed present on this install.

> **Bug found while verifying this phase:** the files had been named `Config.jsonc` / `Style.css` (capital first letter) since Phase 3. Waybar's config search path only ever
> checks lowercase `config`/`config.jsonc` and `style.css` — confirmed with `waybar --log-level debug`, which showed `Found config file: /etc/xdg/waybar/config.jsonc`.
> The capitalized filenames meant every "Phase 3 baseline"-onward screenshot/description was
> silently describing a config that was never actually loaded; Waybar had been falling back to
> the distro's example config (cpu/memory/temperature/pulseaudio/tray and more) the whole time.
> Renamed to `config.jsonc` / `style.css` — verify with the same debug flag after any future
> Waybar change:
>
> ```bash
> waybar --log-level debug 2>&1 | grep "Found config file"
> ```

## Rofi, dunst, hyprlock

Same palette swapped into `rofi/config.rasi`, `dunst/dunstrc`, and `hyprlock.conf` — background,
borders/frame, text, and the critical-urgency/red accent all point at the same hex values as
Waybar and Hyprland now.

## Media keys

Since Waybar now shows volume, the media keys to actually change it were missing. Added to `hyprland.conf`, next to the existing brightness keys:

```text
bindel = ,XF86AudioRaiseVolume, exec, wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+
bindel = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindel = ,XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
```

`wpctl` (wireplumber's CLI) is used instead of `pamixer` since wireplumber is already the
running audio session manager — no extra package.

## Polish pass

A few loose ends closed once the base theme was actually verified on screen (see the
"Bug found" note above — that verification is what surfaced these):

- **`layerrule = blur, <namespace>` was dropped.** This is the documented syntax across
  Hyprland community configs, but on this install (0.56.1) `hyprctl keyword layerrule "blur, waybar"` reliably returns `invalid field blur: missing a value`, and trying variants
  (`blur 1, namespace ...`, colon syntax, etc.) only ever produced the same generic
  "unrecognized field" error — not enough signal to reverse-engineer the real syntax for this
  version. Window blur (`decoration { blur { ... } }`) works fine on its own; Waybar/Rofi keep
  their alpha-transparent backgrounds without the extra blur-through effect.
  **Resolved in Phase 13:** Hyprland 0.53 changed the rule syntax to
  `<effect> <value>, match:<prop> <value>` — the working form is
  `layerrule = blur on, match:namespace waybar`.
- **`hypridle` is now actually started.** `exec-once = hypridle` had been commented out in
  `hyprland.conf` since Phase 8 — the staged dim/lock/dpms/suspend timeouts in
  `hypridle.conf` were fully written but never running. Uncommented it.
- **`animations` block added** (`hyprland.conf`) — a single `tokyoNight` bezier
  (`0.16, 1, 0.3, 1`, a standard "ease-out-expo"-ish curve) reused across windows/border/fade/
  workspace animations instead of Hyprland's built-in defaults.
- **Waybar gained `idle_inhibitor`, `backlight`, and `tray`** — idle_inhibitor toggles whether
  hypridle's timeouts apply (useful now that hypridle actually runs), backlight mirrors the
  existing `brightnessctl` keybinds (scroll on it to adjust), tray holds icons for any
  future tray-registering app (empty today — nothing currently registers one).
- **Rofi spacing/icons polished** — larger `element-icon` (26px), row spacing, a search
  placeholder, and padding around the whole window instead of edge-to-edge elements.

Reload and restart the bar/idle daemon to pick everything up:

```bash
hyprctl reload
systemctl --user restart waybar   # Phase 22+ (before: killall waybar && waybar &)
killall hypridle && hypridle &
killall dunst && dunst &
```

Expected state:

- Active window border shows a blue→purple gradient; inactive windows have a muted dark border.
- Windows have a soft blur; opening/closing/switching windows and workspaces animate with the
  same easing curve instead of Hyprland's defaults.
- Waybar shows the focused window's title in the center; idle/brightness/network/volume/
  battery/tray/clock on the right, all in Tokyo Night colors.
- Volume keys change the level shown in Waybar; scrolling on the backlight icon changes
  brightness.
- The system actually dims/locks/suspends per `hypridle.conf`'s staged timeouts now.
- Rofi, dunst notifications, and the hyprlock screen all share the same palette.

---

# Phase 11 — Desktop pass (Waybar pills, kitty, fastfetch, btop, Rofi control center)

A second, more ambitious pass driven directly by reference screenshots (r/unixporn-style
setups) instead of just filling in a minimal baseline. Goal: a cohesive "everything is one
theme" feel without adopting a heavy all-in-one framework (evaluated and deliberately skipped:
Caelestia's `quickshell`-based stack — liked the look, not the weight).

## A real bug: Nerd Font icons were never actually rendering

Every icon glyph added across Phase 10 and this phase (idle inhibitor, backlight, network,
volume, battery, the new Arch logo) silently failed to save — typing a Nerd Font Private Use
Area character directly into a config file produced an empty string on disk, with zero
indication anything was wrong (no parse error; the format string is still valid, just iconless).
Confirmed by scanning the written file in Python for characters above `0x7f`: none found, despite
every module "having" an icon. Fix: generate the file with a small `python3` script using `\uXXXX` escapes instead of the literal character, e.g.:

```python
icon = "\uf303"  # Arch logo, nf-linux-archlinux — escape, not the literal glyph
```

Verify a codepoint is actually in the installed font before using it:

```bash
fc-query -f "%{charset}\n" /usr/share/fonts/TTF/JetBrainsMonoNerdFontMono-Regular.ttf
```

## Waybar: pill clusters, Arch logo, music, Bluetooth

- Modules now render as separate rounded "pill" groups instead of one flat strip, using
  Waybar's native `"group/name"` module type to cluster related items under one background.
- New leftmost module: an Arch logo (`custom/arch`) that opens Rofi on click — a "start button".
- New `mpris` module (music widget) — free, since `playerctl` was already a Waybar dependency.
  One quirk: Brave registers an MPRIS interface even with nothing playing, so the module can't
  just be hidden via GTK CSS `:empty` (GTK's CSS engine doesn't support that pseudo-class at
  all — using it crashes Waybar outright with `Invalid name of pseudo-class`, confirmed live).
  Fix: no background/pill on `#mpris` at all, just colored text — an empty label is then
  genuinely invisible instead of a visible empty box.
- New `custom/bluetooth` status icon (on/off color via `bluetoothctl show`), click opens the
  Bluetooth picker script below. Needs the Bluetooth stack installed and its service running —
  without it `bluetoothctl show` finds no controller and the icon stays permanently "off":

  ```bash
  sudo pacman -S bluez bluez-utils
  sudo systemctl enable --now bluetooth
  ```
- Full module list now: `custom/arch`, `hyprland/workspaces`, `mpris` (left) — `hyprland/window` (center) — one `group/status` pill (idle_inhibitor, backlight, network, wireplumber,
  custom/bluetooth, battery), tray, clock (right).

## kitty — first real theme since Phase 1

`dotfiles/kitty/kitty.conf` — Tokyo Night ANSI palette, JetBrainsMono Nerd Font, subtle
`background_opacity`. Window rounding/blur come from Hyprland's compositor-level `decoration`
block, not kitty itself. `~/.config/kitty` was an empty directory before this, so remove it
before linking:

```bash
rmdir ~/.config/kitty 2>/dev/null
ln -s ~/hyprland-from-scratch/dotfiles/kitty ~/.config/kitty
```

## fastfetch and btop

```bash
sudo pacman -S fastfetch btop
ln -s ~/hyprland-from-scratch/dotfiles/fastfetch ~/.config/fastfetch
ln -s ~/hyprland-from-scratch/dotfiles/btop ~/.config/btop
```

The symlinks matter: without them both tools run fine but with their stock config, so it's easy
to think the theme is applied when it isn't.

- `dotfiles/fastfetch/config.jsonc` — curated module list (not fastfetch's full default set),
  Arch ASCII logo. Colors are named ANSI keywords (`blue`, `magenta`, ...) rather than hex —
  they inherit Tokyo Night automatically from kitty's ANSI remap above, so the config doesn't
  need to duplicate hex values.
- `dotfiles/btop/btop.conf` + `dotfiles/btop/themes/tokyonight.theme` — full Tokyo Night theme
  covering CPU/mem/net/proc boxes and gradients.
- Keybind in `hyprland.conf`: `$mainMod, Escape` opens btop in a floating, centered kitty
  window. This exposes a second Hyprland gotcha (see below).

## A second Hyprland gotcha: `windowrulev2` looks like it works but doesn't

The documented way to make one specific window float/size/center
(`windowrulev2 = float, class:^(name)$`) prints a "deprecated, see wiki" notice but — confirmed
by checking `hyprctl clients -j` afterward — the rule is silently ignored; the window opens
tiled, at default size. The replacement unified `windowrule` keyword hits the exact same
unresolvable `invalid field type X` wall as `layerrule` (see Phase 10's blur note) — no way to
tell a wrong-syntax error from an unrecognized-field error through trial and error.

**What actually works:** inline bracket rules on the `exec` dispatcher itself, verified with `hyprctl dispatch exec` then checking `hyprctl clients -j` for the resulting window geometry:

```text
bind = $mainMod, Escape, exec, [float;size 800 500;center] kitty --class btop-floating -e btop
```

> **Update (Phase 13): both problems above had different root causes.**
>
> - `windowrulev2` is gone and `windowrule` uses a new syntax since Hyprland 0.53:
>   `<effect> <value>, match:<prop> <value>`. Verified on 0.56.2:
>
>   ```text
>   windowrule = float on, match:class btop-floating
>   windowrule = size 800 500, match:class btop-floating
>   windowrule = center on, match:class btop-floating
>   ```
>
> - The window kept opening at full size — with *both* the inline rule and `windowrule` —
>   because **kitty remembers its last window size** (`remember_window_size yes` by default) and
>   overrides the compositor's size. `remember_window_size no` in `kitty.conf` fixes it; the
>   inline bind above then gives a centered 800x500 window. Checked with test windows:
>   `float`/`center` always applied, `size` only applied once kitty stopped restoring its size.

## Rofi: mode-switcher + a lightweight control center

- `configuration.modi` now lists `drun,run,filebrowser,window`, and `mode-switcher` was added
  to `mainbox`'s children — Rofi draws a row of mode buttons at the bottom, all built in, no
  new packages.
- Three new scripts under `scripts/`, each a self-contained Rofi `-dmenu` menu instead of
  installing a dedicated control-panel app:
  * `rofi-wifi.sh` (renamed `rofi-network.sh` in Phase 16) — lists networks via `nmcli`, prompts for a password with `rofi -password` if the network is secured, connects.
  * `rofi-bluetooth.sh` — lists paired devices via `bluetoothctl`, connect/disconnect toggle,
  plus a power on/off entry.
  * `rofi-audio.sh` — lists audio sinks by parsing `wpctl status` (piped through a small Python
  regex — more reliable than awk/sed against `wpctl`'s tree-drawing output), switches default
  sink, toggles mute.
- Wired into Waybar: click the network icon → `rofi-wifi.sh`; click the Bluetooth icon →
  `rofi-bluetooth.sh`; left-click volume → mute toggle (unchanged), right-click volume →
  `rofi-audio.sh`. (Phase 16 reworks all three menus and the click map.)

> **Bug found after this section first shipped:** every app row rendered as a white card with
> a drop shadow — theme-looking-unapplied, even though `rofi -show drun -dump-theme` echoed
> back all the correct dark colors. Root cause: on Rofi 2.0.0, `element-icon` and `element-text` paint an opaque white background by default and don't inherit transparency
> from the parent `element {}`. Fixed by giving both an explicit `background-color: transparent;` in `config.rasi` (confirmed live, screenshot-compared
> before/after).
>
> **Second bug, same root cause, different widget:** the mode-switcher buttons still showed a
> white separator line between them, and their text was barely legible. `button` needed the
> same treatment — an explicit `border: 0px; border-color: transparent;` (no separator to
> inherit from) and an explicit `text-color` (its default rendered near-black on the dark
> button background). Both confirmed with a before/after screenshot.

Reload and restart the bar to pick everything up:

```bash
hyprctl reload
systemctl --user restart waybar   # Phase 22+ (before: killall waybar && waybar &)
```

Expected state:

- Waybar reads as distinct rounded pill groups, not one flat bar; an Arch logo sits at the far
  left and opens Rofi when clicked.
- A music widget appears in the bar only when something is actually playing.
- Bluetooth has a status icon in the bar; clicking network/Bluetooth/right-clicking volume opens
  a themed Rofi menu that actually changes the setting.
- Opening a new kitty window shows the Tokyo Night theme, not kitty's defaults.
- `$mainMod, Escape` opens btop floating and centered; `fastfetch` shows the Arch logo in Tokyo Night colors.
- Rofi's launcher shows a row of mode buttons (apps/run/files/windows) at the bottom.

---

# Phase 11.5 — Screenshots and file manager

Two everyday tools whose binds already live in `hyprland.conf`.

## Screenshots (grim + slurp)

`grim` captures the screen, `slurp` lets you drag a region and prints its geometry, and
`wl-clipboard` (`wl-copy`) puts the image on the clipboard — three small single-purpose tools
instead of one screenshot app.

```bash
sudo pacman -S grim slurp wl-clipboard
mkdir -p ~/Pictures/screenshots
```

The folder must exist: grim doesn't create parent directories, so without it the binds fail
silently.

| Bind | Action |
|------|--------|
| `SUPER+S` | Full screen → `~/Pictures/screenshots/<timestamp>.png` |
| `SUPER+SHIFT+S` | Selected region → same folder |
| `SUPER+CTRL+S` | Selected region → clipboard only (paste with `Ctrl+V`) |

> Phase 23 replaces these binds: every screenshot is saved **and** copied, and `SUPER+CTRL+S`
> captures the focused window instead.

## File manager (Thunar)

```bash
sudo pacman -S thunar gvfs tumbler
```

- `gvfs` — trash, USB auto-mount, and network locations inside Thunar.
- `tumbler` — image/video thumbnails.

Bound to `SUPER+E`.

Expected state:

- All three screenshot binds produce an image (file or clipboard).
- `SUPER+E` opens Thunar; USB drives show up in its sidebar.

---

# Phase 12 — NVIDIA Hybrid GPU

> **Applies only to hybrid laptops: an AMD or Intel iGPU plus an NVIDIA dGPU from the Turing
> generation (RTX 20 / GTX 16) or newer.** Check with `lspci | grep -E 'VGA|3D'` — you need to see
> **two** lines, one of them NVIDIA. If you see only AMD or only Intel, skip this phase
> entirely: Mesa drives those GPUs out of the box and nothing here is needed. A single-GPU NVIDIA
> desktop is a different setup (NVIDIA drives everything, global NVIDIA env vars *are* needed) and
> is not covered by this guide.
>
> **Applying any of this on the wrong hardware breaks the session**: `AQ_DRM_DEVICES` would point
> at `/dev/dri/*` symlinks that don't exist and Hyprland fails to start. That's why these files
> live in `system/nvidia-hybrid/` and are never symlinked automatically by
> `scripts/link-dotfiles.sh`.

Ready-made copies of every file this phase creates are in `system/nvidia-hybrid/`
(`nvidia.conf`, `99-gpu-symlinks.rules`, `env-hyprland`). The steps below show their content and
the matching `sudo cp` command.

Hardware context on this build: Dell G15 Ryzen Edition, AMD Radeon Vega iGPU
(`0000:05:00.0`) + NVIDIA RTX 3060 dGPU (`0000:01:00.0`). The internal panel
`eDP-1` hangs off the AMD, while the external `HDMI-A-1` and `DP-1` connectors
are physically wired to the NVIDIA — a point worth verifying up front on any
hybrid laptop with `readlink -f /sys/class/drm/card*/device`, since getting
this backwards sends the troubleshooting in the wrong direction (see the
historical appendix at the end).

The goal of this phase is to run the Hyprland compositor and ordinary apps on
the AMD iGPU (lower power, more stable Wayland path), while keeping the NVIDIA
dGPU available for two things: driving the external monitors, and running
individual GPU-heavy apps via `prime-run` on demand. **No NVIDIA session env
vars (`GBM_BACKEND`, `__GLX_VENDOR_LIBRARY_NAME`, `LIBVA_DRIVER_NAME`) are
exported globally** — that was the common failure mode in older
hybrid-NVIDIA guides and is deliberately avoided here.

## Install the driver stack

```bash
sudo pacman -S --needed nvidia-open-dkms nvidia-utils nvidia-settings nvidia-prime libva-nvidia-driver
```

- `nvidia-open-dkms` — the only officially-supported NVIDIA driver on Arch for
  Ampere and newer (`nvidia-dkms`, the closed one, was discontinued for these
  GPUs in Dec 2025).
- `nvidia-prime` — provides the `prime-run` wrapper used below for per-app
  offload to the dGPU.
- `linux-headers` must already be installed (see Phase 0) — DKMS compiles the
  module against the running kernel's headers at install time.

Verify DKMS actually built the module:

```bash
dkms status
```

Expected output (version numbers will vary):

```text
nvidia/615.71.09, 7.2.8-arch1-2, x86_64: installed
```

If this comes back empty, headers are missing — install `linux-headers` and
run `sudo dkms autoinstall`.

## Load the modules early

Edit `/etc/mkinitcpio.conf` and set `MODULES` to:

```text
MODULES=(amdgpu nvidia nvidia_modeset nvidia_uvm nvidia_drm)
```

Then regenerate the initramfs:

```bash
sudo mkinitcpio -P
```

## Module options

Create `/etc/modprobe.d/nvidia.conf`:

```text
options nvidia NVreg_PreserveVideoMemoryAllocations=1 NVreg_TemporaryFilePath=/var/tmp
options nvidia_drm modeset=1 fbdev=1
```

```bash
sudo cp ~/hyprland-from-scratch/system/nvidia-hybrid/nvidia.conf /etc/modprobe.d/nvidia.conf
```

> **Why these and not the older set:**
>
> - `NVreg_PreserveVideoMemoryAllocations=1` + `NVreg_TemporaryFilePath=/var/tmp`
>   keep VRAM allocations through suspend/resume, which this specific hardware
>   historically needed. Kept.
> - `NVreg_EnableGpuFirmware=0` is **removed** on purpose: `nvidia-open-dkms` 615+
>   requires GSP firmware to be enabled to assign CRTCs. With it set to 0 the
>   driver loads but `[drm] Cannot find any crtc or sizes` shows up in `dmesg`
>   and no external output ever lights up.
> - `nvidia_drm.modeset=1` is required for Wayland compositors to drive
>   NVIDIA outputs. `fbdev=1` lets the NVIDIA DRM device expose a framebuffer
>   consistently.

## Stable device paths via udev

`/dev/dri/card*` numbering is not stable across boots, and `AQ_DRM_DEVICES`
uses `:` as a separator so the raw `/dev/dri/by-path/pci-...` paths can't be
used directly either. Fix by creating named symlinks pinned to PCI addresses.

Confirm the PCI addresses on this machine:

```bash
lspci | grep -E 'VGA|3D'
```

Create `/etc/udev/rules.d/99-gpu-symlinks.rules`. **The PCI addresses below are this
laptop's** — on another model they will differ, so edit them to match the `lspci` output
(`05:00.0` becomes `0000:05:00.0`):

```text
KERNEL=="card*", KERNELS=="0000:05:00.0", SUBSYSTEM=="drm", SUBSYSTEMS=="pci", SYMLINK+="dri/amd-igpu"
KERNEL=="card*", KERNELS=="0000:01:00.0", SUBSYSTEM=="drm", SUBSYSTEMS=="pci", SYMLINK+="dri/nvidia-dgpu"
```

```bash
sudo cp ~/hyprland-from-scratch/system/nvidia-hybrid/99-gpu-symlinks.rules /etc/udev/rules.d/
sudoedit /etc/udev/rules.d/99-gpu-symlinks.rules   # only if your PCI addresses differ
```

Reload and apply:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Verify:

```bash
ls -l /dev/dri/ | grep -E 'amd-igpu|nvidia-dgpu'
```

Expected: two symlinks pointing at the correct `card*` for each GPU.

## Hyprland GPU selection via uwsm

This is the step the historical appendix got wrong. For uwsm-launched Hyprland
sessions (Phase 9), `AQ_*` variables must live in `~/.config/uwsm/env-hyprland`
— not in `hyprland.conf` and not in `~/.config/uwsm/env`. Setting them anywhere
else means they're silently absent from the Hyprland process environment at
runtime, which is exactly how the old setup failed.

```bash
mkdir -p ~/.config/uwsm
```

Create `~/.config/uwsm/env-hyprland`:

```bash
export AQ_DRM_DEVICES=/dev/dri/amd-igpu:/dev/dri/nvidia-dgpu
```

```bash
cp ~/hyprland-from-scratch/system/nvidia-hybrid/env-hyprland ~/.config/uwsm/env-hyprland
```

AMD first = compositor and ordinary apps render on the iGPU. NVIDIA listed as
a secondary backend = Aquamarine still initializes it, so its outputs (HDMI /
DisplayPort) are available for external monitors.

**Do not** add `GBM_BACKEND`, `__GLX_VENDOR_LIBRARY_NAME`, `LIBVA_DRIVER_NAME`
or the `__NV_PRIME_RENDER_OFFLOAD` family to this file. The whole point of this
phase is to keep NVIDIA out of the global session; those vars force every app
onto the dGPU and bring back the stability/battery issues this setup is
designed to avoid.

## Reboot and verify

```bash
sudo reboot
```

At the SDDM login (Phase 9), select the **Hyprland (uwsm)** session, log in,
then from a kitty terminal:

```bash
printenv | grep AQ_DRM
```

Expected:

```text
AQ_DRM_DEVICES=/dev/dri/amd-igpu:/dev/dri/nvidia-dgpu
```

If this comes back empty, verify the file lives at `~/.config/uwsm/env-hyprland`
exactly (common mistake: placing it in `~/.config/uwsm/env` instead, which
loads for every uwsm-managed session but not with the `AQ_*` scope Hyprland
reads at startup).

```bash
nvidia-smi
```

Should list the dGPU (model, driver version). Processes should be minimal —
Hyprland/Xwayland may appear with a few MiB of memory, but no significant
compute load.

```bash
hyprctl monitors
```

Both `eDP-1` (internal, AMD) and any connected external (`HDMI-A-1` or `DP-1`,
NVIDIA) should appear. If only `eDP-1` shows up with no external connected,
that's expected; plugging one in should surface it immediately via Hyprland's
hotplug.

```bash
glxinfo | grep "OpenGL renderer"
```

Should name the AMD GPU — this confirms ordinary apps are not landing on the
dGPU by accident. (If `glxinfo` is missing: `sudo pacman -S --needed mesa-utils`.)

## Per-app offload to the dGPU

For individual apps that should run on the NVIDIA GPU (games, Blender, GPU
compute), prefix with `prime-run`:

```bash
prime-run glxinfo | grep "OpenGL renderer"
```

Should name the NVIDIA RTX 3060. Same pattern for Steam game launch options:

```bash
prime-run %command%
```

Expected state at the end of this phase:

- Hyprland session starts via SDDM → uwsm with no manual intervention.
- `AQ_DRM_DEVICES` is set in the Hyprland process environment (not just in
    the login shell).
- Internal panel and external monitors both work.
- `nvidia-smi` runs; the dGPU is listed but idle most of the time.
- `prime-run <app>` routes a single app to the dGPU; everything else stays on
    the iGPU.

## Validation worth repeating

The underlying NVIDIA-on-hybrid bug is intermittent, so a single clean boot
isn't proof. Before considering this phase closed:

- Cold-reboot 2–3 times. `journalctl -b -1 | grep -i nvidia` should stay clean
    across all of them.
- Suspend (close lid) and resume — this is where
    `NVreg_PreserveVideoMemoryAllocations` is actually exercised.
- Run `prime-run` with a real workload once (e.g. `prime-run glmark2`) and
    confirm `nvidia-smi` shows utilization during the run and drops back to idle
    after.

## Critical hardware caveat

On this specific model (Dell G15 Ryzen Edition, which shares its
motherboard/platform with the Alienware line — hence `alienware_wmi` kernel
modules loading despite the G15 badging), running sustained CPU+GPU load
(e.g. `glmark2`, a game) **on battery power** caused an instant total power
cutoff on this hardware — no crash/freeze behavior, no trace in `journalctl`,
consistent with a firmware-level power protection cutting supply before the
kernel can log anything.

**Always connect the charger before running any GPU benchmark, game, or
sustained load test.** Unrelated to the driver/compositor stack; a hardware
caveat worth keeping in the guide.

## Useful diagnostic commands

```bash
# confirm which GPU a process is actually using
nvidia-smi

# confirm which /dev/dri/card maps to which PCI address
readlink -f /sys/class/drm/card0/device
readlink -f /sys/class/drm/card1/device

# confirm the Hyprland process actually received AQ_DRM_DEVICES
cat /proc/$(pgrep -u $(id -u) Hyprland | head -1)/environ | tr '\0' '\n' | grep AQ_DRM

# force a command onto the NVIDIA GPU (short form of prime-run's env setup)
__NV_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia __VK_LAYER_NV_optimus=NVIDIA_only <command>

# check the previous boot's log for NVIDIA-related errors
journalctl -b -1 | grep -i nvidia

# list all recorded boots with start/end timestamps
journalctl --list-boots

# check for thermal/power/panic events in the previous boot
journalctl -b -1 | grep -i -E "thermal|temperature|power|panic|shutdown|emergency"

# lightweight, no-download GPU stress test
sudo pacman -S glmark2
```

---

# Phase 13 — Look & feel pass

A second visual pass, checked on screen this time (cropped screenshots of the bar, a Rofi capture,
and A/B captures with a setting toggled) instead of only reading configs. Nothing new to install.

## External monitor at its real refresh rate

`hyprctl monitors all` showed the external AOC 27G2G4 running at **60 Hz** while listing
`1920x1080@144.00Hz` among its modes. `preferred` means "the monitor's EDID preferred mode", which
on many gaming monitors is 60 Hz. Fix: a dedicated rule, matched by description instead of port
name so it follows the monitor to any connector (HDMI today, DisplayPort tomorrow):

```text
monitor = desc:AOC 27G2G4 GYGM6HA425243, 1920x1080@144, auto, 1
monitor = ,preferred,auto,1
```

For another monitor, take the `description` and the modes from `hyprctl monitors all` and add
its own line above the catch-all. VRR (`misc:vrr`) is left off for now — the external outputs are
driven by the NVIDIA dGPU (Phase 12), so it belongs in the gaming phase where it can be tested.

## Opacity: opaque apps, glass terminal

Before: `active_opacity = 0.97` / `inactive_opacity = 0.90` on every window, and kitty's own
`background_opacity 0.92` on top of that — calls, videos and games were slightly see-through, and
kitty's text faded along with its background. Now Hyprland keeps every window at `1.0` and kitty
alone is translucent through `background_opacity`, which only affects the background (text stays
crisp); Hyprland's `decoration:blur` frosts what shows through.

## Blur on Waybar and Rofi

The `layerrule` dropped in Phase 10, now with the 0.53+ syntax:

```text
layerrule = blur on, match:namespace waybar
layerrule = ignore_alpha 0.5, match:namespace waybar
layerrule = blur on, match:namespace rofi
layerrule = ignore_alpha 0.5, match:namespace rofi
```

`ignore_alpha` matters for Waybar: its surface spans the whole top strip but is transparent
outside the pills, and without it the entire strip would be frosted. Verified with an A/B capture
(`hyprctl keyword decoration:blur:enabled false`, capture, re-enable, compare): differences appear
only inside the pills. Rofi's window background became `#1a1b26e0` so the blur has something to
show through. The effect is subtle over this wallpaper (dark at the top, pills ~85% opaque) —
lower the alpha in `style.css`/`config.rasi` for a more visible frosted look.

## No default wallpaper flash at login

Hyprland draws its own wallpaper/logo until awww loads. The `misc` block turns that off and paints
the Tokyo Night background color instead:

```text
misc {
    disable_hyprland_logo = true
    disable_splash_rendering = true
    force_default_wallpaper = 0
    background_color = rgb(1a1b26)
}
```

## Dark mode for GTK apps

`gsettings` showed `color-scheme 'default'` and `gtk-theme 'Adwaita'` — Thunar, file pickers and
GTK4 apps were light, and websites that follow the system theme too. Two `exec-once` lines in
`hyprland.conf` set `prefer-dark` + `Adwaita-dark` at every login. They live there instead of only
in dconf (`~/.config/dconf` isn't in the repo), so a restore needs no extra step. Apps that ask the
desktop portal (Brave with its theme set to "Device") get the setting through
`xdg-desktop-portal-gtk`.

## Waybar

- **Visible vs focused workspace.** Waybar's `.active` class only marks the *focused* workspace,
  so the laptop panel's bar showed its own workspace as if it were empty. A `.visible` style
  (lighter pill, grey text) marks the workspace on screen on each monitor; `.active` (blue, bold)
  still marks the focused one.
- **GTK button reset** on workspace buttons (`background`, `border`, `box-shadow`, `text-shadow`),
  so the GTK theme's own hover effect doesn't leak into the pill.
- **Bluetooth on/off colors.** Phase 11 described them, but `style.css` never styled the `on`/`off`
  classes the module emits. Now blue when powered, muted when off.
- **Themed tooltips** — GTK's default tooltip bubble replaced with the pill colors.
- **Clock icons.** The format had three spaces between time and date: a glyph lost to the
  Nerd Font bug from Phase 11. Clock and calendar icons were written back with `\uXXXX` escapes.
- **Music widget only while playing.** Empty `format-paused`/`format-stopped`: a paused or idle
  browser tab no longer leaves its title in the bar.

## Rofi

- Row text follows the row color (`element-text { text-color: inherit; }`), so the selected row is
  blue like the active workspace instead of plain white; matched characters are bold purple.
- Mode buttons named consistently: Apps / Run / Files / Windows.

## kitty: `remember_window_size no`

The floating btop window (`SUPER+Escape`) kept opening at full size. kitty restores its last window
size by default and overrides the compositor's size rule — see the update in Phase 11's
`windowrulev2` note. With it off, the bind gives a centered 800x500 window.

Reload everything:

```bash
hyprctl reload
systemctl --user restart waybar   # Phase 22+; don't use SIGUSR2 reloads (see Phase 22)
```

Expected state:

- `hyprctl monitors` shows the AOC at `@144`.
- Brave, VS Code and Steam are fully opaque; kitty's background is translucent and blurred.
- Waybar pills and Rofi have a frosted background; each monitor's bar highlights its visible
  workspace.
- Thunar and GTK dialogs are dark.
- `SUPER+Escape` opens btop floating, centered, 800x500.

---

# Phase 14 — Keybinds: 10 workspaces and SUPER for the launcher

## Workspaces 6–10

`SUPER+6` did nothing, as if workspaces stopped at 5. Hyprland has no such limit — workspaces are
created on demand and numeric IDs go far beyond 10. The config simply had binds for 1–5 only.
Added `SUPER+6…9` and `SUPER+0` (→ workspace 10), plus the matching `SUPER+SHIFT` binds to move the
focused window. Waybar lists them on its own as soon as they exist.

```text
bind = $mainMod, 0, workspace, 10
bind = $mainMod SHIFT, 0, movetoworkspace, 10
```

## Tap SUPER to open the launcher

The same behavior as Caelestia (or Windows/GNOME): pressing and releasing SUPER on its own opens
the app launcher; tapping it again closes it.

```text
bindr = $mainMod, Super_L, exec, pkill -x rofi || rofi -show drun
```

- `bindr` is a **release** bind: it fires when the key is released, not when it's pressed — that's
  what lets SUPER still work as a modifier for every other bind.
- SUPER+`<key>` combos should not open the launcher: the release only counts when SUPER was the
  last key pressed. Check it by pressing `SUPER+T` — only kitty should open.
- `pkill -x rofi ||` turns it into a toggle. `Escape` also closes Rofi.
- `SUPER+R` keeps working as before.

Expected state:

- `SUPER+6` … `SUPER+0` switch to workspaces 6–10; `SUPER+SHIFT+<n>` moves the window there.
- Tapping `SUPER` opens Rofi; tapping it again closes it.
- `SUPER+T`, `SUPER+E`, etc. do their own action without also opening Rofi.

---

# Phase 15 — Arch Neutral theme (replaces Tokyo Night)

After living with Tokyo Night, its blue + purple mix felt too saturated. The new direction is the
Arch logo itself: neutral greys and black, with **one accent — the Arch blue `#1793D1`**. Purple is
gone from the palette entirely.

| Role | Hex | Used for |
|------|-----|----------|
| Background | `#141414` | Waybar pills, Rofi, dunst, kitty, btop, hyprlock field |
| Surface | `#1f1f1f` | Rofi input bar and buttons, visible (unfocused) workspace |
| Highlight | `#2b2b2b` | Selected row, focused workspace, hover |
| Border | `#333333` | Inactive window border, kitty splits |
| Foreground | `#e0e0e0` | Main text |
| Dimmed | `#a8a8a8` | Secondary text (window title, lock-screen placeholder) |
| Muted | `#6e6e6e` | Inactive workspaces, placeholders, low-urgency notifications |
| Accent — Arch blue | `#1793d1` | Active window border, Arch logo, focused workspace, selection, frames |
| Accent — light blue | `#5db3df` | Hover, border gradient end, music widget, search-match highlight |
| Warning | `#e0a84e` | Low battery, idle inhibitor on |
| Critical | `#e05f65` | Critical battery, network down, critical notifications |

The greys are pure neutrals (no blue tint), close to VS Code's Dark Modern, so the editor and the
desktop read as one theme.

## Where the colors live

There's no shared variable file — Hyprland, GTK CSS (Waybar), rasi (Rofi), INI (dunst), kitty and
btop each have their own syntax, so every file holds its own copy of the hex values and **the table
above is the source of truth**. Changing a color means replacing it everywhere:

```bash
grep -rn "#1793d1" ~/hyprland-from-scratch/dotfiles/
```

| File | What changed |
|------|--------------|
| `hypr/hyprland.conf` | Active border gradient `#1793d1 → #5db3df`, inactive border, `misc:background_color` |
| `hypr/hyprlock.conf` | Input field ring/fill/text, clock color, background wallpaper |
| `waybar/style.css` | Pills, workspaces, status colors, tooltips (the music widget moved from green to light blue) |
| `rofi/config.rasi` | Window, input bar, selected row, match highlight, mode buttons |
| `dunst/dunstrc` | Frame and backgrounds per urgency level |
| `kitty/kitty.conf` | Full 16-color ANSI palette: `blue` is the Arch blue, magenta became a muted rose |
| `btop/themes/arch.theme` | New theme (`color_theme = "arch"` in `btop.conf`); gradients go blue (calm) → amber/red (busy) |
| `fastfetch/config.jsonc` | Logo forced to `blue` (= Arch blue through kitty's ANSI palette), title in `cyan` |

`btop/themes/tokyonight.theme` is still in the repo and selectable from btop's options menu.

## Wallpaper

*(Since Phase 17 the wallpaper is picked with `SUPER+W` and remembered; `3.jpeg` is the fallback.)*
The default wallpaper moved from `1.jpg` to **`3.jpeg`** (astronaut: black and grey with a blue
glow — the same palette) in both `hyprland.conf` (`exec-once = awww img ...`) and `hyprlock.conf`.
`1.jpg` and `2.png` stay in `assets/wallpapers/`; switching back is a one-line change in each file.

Apply everything without logging out:

```bash
hyprctl reload                                    # borders, background color
systemctl --user restart waybar                   # bar (Phase 22+)
killall dunst && hyprctl dispatch exec dunst      # notifications
awww img ~/hyprland-from-scratch/assets/wallpapers/3.jpeg --transition-type grow
kill -SIGUSR1 $(pidof kitty)                      # reload already-open kitty windows
```

Rofi, btop, fastfetch and hyprlock read their config on every launch.

Expected state:

- No purple anywhere: borders, bar, launcher and notifications use greys + Arch blue.
- The Arch logo in Waybar and in `fastfetch` is the same blue as the active window border.
- The lock screen (`SUPER+L`) shows the astronaut wallpaper with a blue input ring.

---

# Phase 16 — Controls: Bluetooth pairing, network, audio, brightness

The status icons in Waybar showed the right values, but most clicks did nothing useful:

- **Bluetooth** — the menu only listed devices that were *already* paired, plus a power toggle.
  There was no way to scan for and pair a new device, so a Bluetooth mouse couldn't be added at all.
- **Brightness** — only reacted to scrolling (no click action), and only the laptop panel has a
  backlight, so working on the external monitor it looked like it did nothing.
- **Volume** — left-click silently toggled mute: easy to trigger without noticing.
- **Network** — the Wi-Fi list repeated a network once per access point/band, and there was no
  adapter status or Wi-Fi on/off.
- All menus reused the launcher theme ("Search apps…" plus the Apps/Run/Files buttons), so they
  looked like the app launcher.

`brightnessctl` and `wpctl` themselves worked from the session (checked by writing the current
value back): the controls were missing, not broken.

## Click map

| Module | Click | Right-click | Scroll |
|--------|-------|-------------|--------|
| Brightness | Presets menu (laptop panel) — *replaced by a hover slider in Phase 21* | — | ±5% |
| Network | Network menu | — | — |
| Volume | Audio menu | Mute | ±5% |
| Bluetooth | Bluetooth menu | — | — |

Every module's tooltip repeats its click map.

## Pairing a Bluetooth device (mouse, headset, speaker)

1. Put the device in pairing mode — usually by holding its pairing/connect button until the LED
   blinks fast.
2. Click the Bluetooth icon → **Scan for new devices**. A notification marks the start of a
   10-second search.
3. Pick the device from the list. The script pairs it, **trusts** it (so it reconnects on its own
   after sleep, reboot or power cycling) and connects it.
4. The icon turns blue while something is connected; its tooltip lists the device.

How `scripts/rofi-bluetooth.sh` does it:

- Discovery stays on while the result list is open. bluez forgets unpaired devices ~30 s after
  discovery stops, so a device picked after that would already be gone. It's turned off right
  before pairing, which is more reliable without discovery running.
- Nameless advertisers (beacons, nearby phones) are hidden; two devices with the same name get
  the end of their MAC address appended.
- **Pairing needs a Bluetooth agent.** The first version ran `bluetoothctl --agent NoInputNoOutput
  pair <MAC>` as a one-shot command and every attempt failed. `journalctl -u bluetooth` showed why:

  ```text
  src/device.c:new_auth() No agent available for request type 2
  device_confirm_passkey: Operation not permitted
  ```

  bluez refuses to pair while no agent is registered — even for a mouse that asks nothing — and
  bluetoothctl silently skips registering its agent in one-shot mode (`--agent` is ignored there).
  The pairing now runs inside an *interactive* bluetoothctl fed through a bash `coproc`, which does
  register the agent (the adapter also switches to `Pairable: yes`). With a `NoInputNoOutput`
  agent the pairing is "Just Works" and bluez accepts it on its own because we started it.
  Verified with a Logitech M196: paired, bonded, trusted, connected, and listed by
  `hyprctl devices` as `logi-m196-mouse`.
- That agent is right for mice, headsets, speakers and controllers. A keyboard that asks you to
  type a PIN needs a manual session instead:

  ```bash
  bluetoothctl
  # inside: scan on → pair <MAC> (type the PIN on the keyboard) → trust <MAC> → connect <MAC>
  ```

- Also in the menu: connect/disconnect paired devices, **Forget a device…**, Bluetooth on/off
  (clears a soft `rfkill` block if needed).

The icon's state comes from `scripts/waybar-bluetooth.sh` (polled every 5 s): CSS classes `off`
(muted), `on` with nothing connected (dimmed) and `connected` (Arch blue).

## Network menu

`scripts/rofi-wifi.sh` became `scripts/rofi-network.sh`:

- One status row per adapter, e.g. `Wired (enp3s0): connected - Wired connection 1`. Selecting an
  adapter opens **nmtui** in a floating kitty — NetworkManager's own text UI for everything else
  (static IPs, editing/forgetting saved networks, VPN, hotspot).
- Wi-Fi networks: one row per name with its strongest signal, a lock icon when it needs a
  password, sorted by signal. Selecting the connected one disconnects it.
- A failed password attempt deletes the half-saved profile, so the next try asks again instead of
  silently reusing the wrong password.
- **Rescan Wi-Fi**, **Turn Wi-Fi on/off**, **Advanced settings (nmtui)**.

## Audio menu

Output devices (names shortened: `GA106 High Definition Audio Controller HDMI / DisplayPort 1
Output [27G2G4] (Stereo)` → `GA106 HDMI / DisplayPort 1 Output [27G2G4]`), **Mute/Unmute
output**, **Mute/Unmute microphone** (handy during calls), and a **Mixer** entry that appears once
`pavucontrol` is installed (per-app volumes, input devices):

```bash
sudo pacman -S pavucontrol   # optional
```

## Brightness

`scripts/rofi-brightness.sh` offered 100/75/50/25/10% for the laptop panel; scrolling keeps
stepping by 5%. *(Phase 21 replaced this menu with a slider like the volume one, and removed the
script.)* External monitors have no kernel backlight — their brightness goes over DDC/CI
with `ddcutil`, which isn't set up yet (see Next Phases).

## A theme for menus

The menus use `dotfiles/rofi/menu.rasi`, which imports `config.rasi` (same look as the launcher)
and changes only what a menu needs: the prompt is shown (Network, Bluetooth, Audio, …), no
mode-switcher row, a generic "Filter..." placeholder, and the list shrinks to its entries.

## Notes on the scripts

- Icons are written as `$'\uf293'`-style escapes that bash expands at runtime, so the scripts are
  plain ASCII and immune to the glyph-stripping problem from Phase 11. Check with
  `grep -n "^I_" scripts/*.sh`.
- To see what a menu would show without opening Rofi, put a fake `rofi` first in `PATH`:

  ```bash
  mkdir -p /tmp/rofi-stub && printf '#!/bin/sh\ncat\nexit 1\n' > /tmp/rofi-stub/rofi
  chmod +x /tmp/rofi-stub/rofi
  PATH=/tmp/rofi-stub:$PATH ~/hyprland-from-scratch/scripts/rofi-network.sh
  ```

  The entries print to the terminal and `exit 1` behaves like pressing Escape, so nothing runs.

Restart the bar:

```bash
systemctl --user restart waybar   # Phase 22+; don't use SIGUSR2 reloads (see Phase 22)
```

Expected state:

- Clicking the Bluetooth icon → **Scan for new devices** finds a device in pairing mode; after
  picking it, the icon turns blue and the device works.
- After a reboot, a paired mouse reconnects by itself once it's switched on.
- If pairing fails, `journalctl -u bluetooth -n 20` shows bluez's reason.
- Clicking network/volume opens their menus; right-click on volume mutes. (Brightness: Phase 21.)
- The menus show their own prompt and no Apps/Run/Files buttons.

---

# Phase 17 — Popup animations, click-outside to close, wallpaper picker

## Animations for popups

Rofi menus and notifications are layer-shell surfaces, and the `layers*` animations were never
configured: they appeared and vanished instantly. Now:

```text
bezier = smoothIn, 0.32, 0, 0.67, 0          # gentle start, fast end: things leaving
animation = layersIn, 1, 3, smoothOut, fade  # 300 ms in
animation = layersOut, 1, 2, smoothIn, fade  # 200 ms out
animation = fadeLayersIn, 1, 3, smoothOut
animation = fadeLayersOut, 1, 2, smoothIn
layerrule = animation popin 90%, match:namespace rofi              # menus grow in / shrink out
layerrule = animation slide right, match:namespace notifications   # dunst slides in from the edge
```

The default layer style is a plain fade on purpose: `popin` for every layer would also zoom the
bar and the wallpaper at login. Opening is a bit slower than closing (300 vs 200 ms) with mirrored
curves — popups arrive softly and get out of the way fast. `hyprctl animations -j` lists every
animation with its current values.

## Click outside a menu to close it

Rofi has `click-to-exit` (on by default), but it can't work on Wayland: Rofi's layer surface only
covers the menu itself (`hyprctl layers` shows it at e.g. 576x492), so clicks anywhere else never
reach it. The compositor sees every click, though:

```text
bindn = , mouse:272, exec, $scripts/rofi-click-outside.sh
```

- `bindn` = **non-consuming**: the click still reaches the window under the cursor.
- `scripts/rofi-click-outside.sh` exits immediately when no Rofi is running (the usual case).
  Otherwise it compares `hyprctl cursorpos` with the Rofi layer's geometry from `hyprctl layers`
  and closes the menu if the click landed outside it.
- Clicks on Waybar are left alone: the bar icons handle menus themselves through
  `scripts/rofi-common.sh` — clicking the icon of the open menu closes it, clicking another icon
  switches menus (only one Rofi can run at a time, so the open one is closed first).

## Changing the wallpaper

It couldn't be changed: awww is command-line only, and Thunar's built-in "Set as wallpaper" needs
xfdesktop (XFCE's desktop), which isn't installed. `scripts/wallpaper.sh` adds the missing pieces:

| How | What |
|-----|------|
| `SUPER+W`, or right-click the Arch logo | Picker: a grid of thumbnails, the current one marked |
| Thunar → right-click an image → **Set as wallpaper** | Applies that image (custom action in `dotfiles/Thunar/uca.xml`) |
| `wallpaper.sh set FILE` | Same, from a terminal |
| `exec-once = $scripts/wallpaper.sh restore` | Re-applies the last choice at login |

- The picker lists `assets/wallpapers/` (versioned with the repo) and `~/Pictures/wallpapers/`
  (personal images that stay out of git). Thumbnails are cached in
  `~/.cache/hyprland-from-scratch/wallpaper-thumbs/` (needs `imagemagick`).
- The choice is a symlink, `~/.local/state/hyprland-from-scratch/wallpaper`. `hyprlock.conf`
  uses that same path as its background, so the lock screen always matches the desktop. On a fresh
  install the symlink doesn't exist yet and `3.jpeg` is used.
- Changes animate with awww's `grow` transition; the login restore uses a `fade`.

Thunar's config folder became a dotfile for this (`dotfiles/Thunar/`, linked by
`scripts/link-dotfiles.sh`; the previous folder was kept as `~/.config/Thunar.bak-<date>`). Its
"Open Terminal Here" action now opens kitty directly instead of going through `exo-open`.

Expected state:

- Rofi menus grow in and shrink out; notifications slide in from the right.
- With a menu open, clicking anywhere outside it closes it — and the click still works on whatever
  was under the cursor.
- Clicking the same Waybar icon again closes its menu; clicking another icon switches menus.
- `SUPER+W` shows the wallpaper thumbnails; picking one changes the desktop with an animation,
  it survives a reboot, and `SUPER+L` shows it on the lock screen.
- In Thunar, right-clicking an image shows **Set as wallpaper**.

---

# Phase 18 — Hover to select in menus

In every Rofi menu (launcher, network, Bluetooth, audio, brightness, wallpaper) the highlighted
row stayed put while the mouse moved over the list: it only moved on click or scroll, and running
an entry took a double-click. Those are Rofi's defaults:

```text
hover-select: false;                 # highlight ignores the pointer
me-select-entry: "MousePrimary";     # click = select
me-accept-entry: "MouseDPrimary";    # double-click = run
```

Changed in the `configuration` block of `dotfiles/rofi/config.rasi` — the combination Rofi's own
help recommends for hover selection:

```text
hover-select: true;                  # the highlight follows the pointer
me-select-entry: "";                 # selecting by click is no longer needed
me-accept-entry: "MousePrimary";     # one click runs the highlighted row
```

Rofi reads `config.rasi` on every launch (the menus get it through `menu.rasi`'s `@import`), so it
applies everywhere without reloading anything. Check the effective values with:

```bash
rofi -dump-config | grep -E "hover-select|me-select-entry|me-accept-entry"
```

The keyboard works as before: arrows/typing move the highlight, `Enter` runs it, `Escape` closes.

Expected state:

- Moving the mouse over any menu moves the highlight row by row.
- A single click runs the entry under the cursor (launches the app, picks the network, sets the
  brightness, applies the wallpaper).

---

# Phase 19 — Calendar, battery details, power profiles, screen recording

## Calendar on the clock

The clock had only a one-line date tooltip. Waybar's clock module can render a month calendar in
the tooltip, styled with Pango markup in the same palette (today = Arch-blue block):

```text
"tooltip-format": "<b>{:%A, %d %B %Y}</b>\n\n<tt>{calendar}</tt>",
"calendar": { "mode": "month", "mode-mon-col": 3, "format": { "today": "<span background='#1793d1' ...>{}</span>", ... } },
"actions":  { "on-click": "shift_reset", "on-click-right": "mode",
              "on-scroll-up": "shift_up", "on-scroll-down": "shift_down" }
```

Hover shows the current month; scroll moves between months; right-click switches to a full-year
view (3 months per row); left-click jumps back to today.

## Battery details and power profiles

- The battery tooltip now shows time to full/empty, current power draw and **health** — `upower`
  reports this battery holds ~57% of its design capacity, which explains shorter runtimes.
- A **power profile** icon sits next to the battery: leaf = power-saver, scale = balanced,
  bolt = performance (amber when active). Left-click goes to the next profile, right-click to the
  previous one. It's Waybar's native `power-profiles-daemon` module, which talks to the daemon over
  D-Bus. On this laptop the profiles drive `amd_pstate` and the firmware's `platform_profile`:
  performance for games, power-saver on battery.

> `powerprofilesctl` (the daemon's CLI) is broken here: it's a Python script that needs
> `python-gobject`, which isn't installed. Nothing in this setup uses it. To read or switch the
> profile from a terminal without it:
>
> ```bash
> busctl --system get-property org.freedesktop.UPower.PowerProfiles /org/freedesktop/UPower/PowerProfiles org.freedesktop.UPower.PowerProfiles ActiveProfile
> busctl --system set-property org.freedesktop.UPower.PowerProfiles /org/freedesktop/UPower/PowerProfiles org.freedesktop.UPower.PowerProfiles ActiveProfile s performance
> ```

## Screen recording

```bash
sudo pacman -S wf-recorder
```

`wf-recorder` records through the compositor's screencopy protocol, which Hyprland provides.
`scripts/screen-record.sh` wraps it:

- `SUPER+SHIFT+R` opens a menu: **Record screen** (the focused monitor), **Record region** (drag a
  box with `slurp`), each with or without **desktop audio** (the "monitor" of the default output,
  so it captures what you hear, not the microphone).
- While recording, a red **REC mm:ss** pill appears at the right of the bar
  (`scripts/waybar-recording.sh`, refreshed every second and instantly through Waybar signal 9).
  Click it, or press `SUPER+SHIFT+R` again, to stop: wf-recorder gets `SIGINT` so it finishes
  the MP4 properly, and a notification shows where it was saved.
- Files go to `~/Videos/recordings/<date>_<time>.mp4`, encoded with x264 `preset=veryfast`,
  `crf=23` on the CPU: plenty for 1080p on this Ryzen, and it avoids choosing between the AMD and
  NVIDIA GPUs for hardware encoding.

Expected state:

- Hovering the clock shows a calendar with today highlighted; scrolling changes the month.
- Hovering the battery shows time left, power draw and health; the icon next to it switches
  power profiles.
- `SUPER+SHIFT+R` → **Record screen** shows the red REC pill; stopping it leaves a playable MP4
  in `~/Videos/recordings/`.

---

# Phase 20 — Volume control: slider in the bar, stepper in the audio menu

The audio menu could switch the output device but not change the volume (only scrolling on the
icon and the media keys could). Two additions:

## Slider on hover

Hovering the volume icon slides out a volume slider: Waybar's native `pulseaudio/slider` module
(through `pipewire-pulse`) inside a **drawer** group, whose first module stays visible and whose
other modules are revealed while the pointer is over it:

```text
"group/volume": {
    "orientation": "horizontal",
    "drawer": { "transition-duration": 300, "children-class": "volume-drawer", "transition-left-to-right": true },
    "modules": ["wireplumber", "pulseaudio/slider"]
},
"pulseaudio/slider": { "min": 0, "max": 100, "orientation": "horizontal" }
```

`group/volume` takes the volume icon's place inside the status pill (groups nest). The slider is a
GTK scale styled in `style.css`: `trough` = the track (`#2b2b2b`), `highlight` = the filled part
(Arch blue), and the knob (`slider`) hidden — click or drag anywhere on the track.

## Volume in the audio menu

The audio menu now starts with **Volume: N%**, which opens `scripts/rofi-volume.sh`: **Louder
(+5%)**, **Quieter (−5%)** and presets 100/75/50/25/10/0%. It runs in Rofi's **script mode**
instead of dmenu mode, the difference that makes a stepper usable:

- dmenu mode (all the other menus): Rofi prints the choice and exits; the script acts afterwards.
- Script mode (`rofi -show volume -modi "volume:<script>"`): Rofi stays open and runs the script
  again after every pick (`ROFI_RETV=1`, the row's hidden value in `ROFI_INFO`), redrawing with
  whatever it prints. Louder/Quieter can be clicked repeatedly, the prompt shows the level live
  (`\0prompt\x1f...`), and `\0keep-selection\x1ftrue` keeps the highlight on the same row. A preset
  prints nothing, and an empty list makes Rofi close.

`rofi_toggle` in `scripts/rofi-common.sh` now takes several patterns, so clicking the volume icon
while either the audio menu or the volume control is open closes it.

Expected state:

- Hovering the volume icon reveals a slider; dragging it changes the volume, and the percentage
  next to the icon follows.
- Audio menu → **Volume: N%** → clicking **Louder** several times raises the volume 5% per click
  without the menu closing; a preset sets it and closes the menu.

---

# Phase 21 — Brightness slider

The volume slider from Phase 20 worked well enough to replace the brightness presets menu: hovering
the brightness icon now slides out Waybar's native `backlight/slider`, in the same kind of drawer
group (`group/brightness`, in place of `backlight` inside the status pill).

```text
"group/brightness": {
    "orientation": "horizontal",
    "drawer": { "transition-duration": 300, "children-class": "brightness-drawer", "transition-left-to-right": true },
    "modules": ["backlight", "backlight/slider"]
},
"backlight/slider": { "min": 5, "max": 100, "orientation": "horizontal" }
```

- **Gold fill** (`#e0a84e`, the palette's amber) instead of the volume slider's blue, so the two
  read differently at a glance. Track and hidden knob share the volume slider's CSS rules.
- **`min: 5`, not 0**: on some laptop panels a backlight of 0 switches the screen off completely,
  which is easy to do by accident with a slider.
- Scrolling on the icon and the brightness keys still step by 5%. The click no longer opens a menu,
  and `scripts/rofi-brightness.sh` was removed.
- As before, this is the laptop panel only; the external monitor needs DDC/CI (Next Phases).

Expected state:

- Hovering the brightness icon reveals a gold slider; dragging it changes the laptop panel's
  brightness and the percentage next to the icon follows.
- It never goes below 5%.

---

# Phase 22 — Waybar as a service, recording button

## The bar disappeared: a crash, and why

Waybar vanished a few minutes after Phase 21. `coredumpctl list waybar` showed a segfault, and
`coredumpctl info <PID>` the backtrace:

```text
#0  Glib::DispatchNotifier::send_notification   (libglibmm)
...
#6  ...                                         (libplayerctl.so.2)
```

Not the new brightness slider: the crash came from **libplayerctl**, behind the music (`mpris`)
module, delivering a player event (a browser tab starting or stopping media) to a callback whose
object no longer existed. The likely trigger: the many `killall -SIGUSR2 waybar` reloads during
Phases 13–21. A SIGUSR2 reload rebuilds every module inside the *same* process, but the old mpris
module's playerctl subscriptions survive it; the next player event calls into freed memory. The
original Waybar process (from login) had been reloaded about seven times.

Two changes:

1. **Restart, never reload.** `systemctl --user restart waybar` starts a fresh process. Every
   `SIGUSR2` instruction in this guide was replaced. (Waybar's own unit maps `systemctl reload`
   to SIGUSR2, so don't use `reload` either.)
2. **Run Waybar as a systemd user service** instead of `exec-once`, so a crash no longer means
   losing the bar until the next login. Waybar ships the unit, and the uwsm session (Phase 9)
   provides the `graphical-session.target` it hooks into:

   ```bash
   systemctl --user enable --now waybar.service
   ```

   `/usr/lib/systemd/user/waybar.service` has `Restart=on-failure` and is `PartOf` the graphical
   session, so it starts and stops with Hyprland. `exec-once = waybar` was removed from
   `hyprland.conf` — keeping both would start two bars. uwsm exports `WAYLAND_DISPLAY` and
   `HYPRLAND_INSTANCE_SIGNATURE` into the systemd user environment
   (`systemctl --user show-environment`), so the Hyprland modules and the `hyprctl` calls in the
   scripts work from the service too. Verified by killing it on purpose
   (`systemctl --user kill -s KILL waybar.service`): a new process was up within a second
   (`systemctl --user show waybar.service -p NRestarts` → `NRestarts=1`).

## Recording button

The screen-recording module used to exist only while recording. It's now always visible: a camera
button left of the status pill. Clicking it opens the recording menu; while recording, it turns
into the red **REC mm:ss** pill, and clicking it stops and saves. (`scripts/screen-record.sh`
already toggled: menu when idle, stop when recording — the button just calls it.)

## Testing screen recording

```bash
sudo pacman -S wf-recorder
```

1. Click the camera button (or press `SUPER+SHIFT+R`) → **Record screen**. For **Record region**,
   drag a box; `Escape` cancels.
2. The button turns into the red REC pill and counts up.
3. Click the pill (or `SUPER+SHIFT+R`): a notification shows where the file was saved.
4. Check the file:

   ```bash
   ls -lh ~/Videos/recordings/
   ffprobe -hide_banner ~/Videos/recordings/<file>.mp4   # ffprobe comes with ffmpeg, a wf-recorder dependency
   brave ~/Videos/recordings/<file>.mp4                  # play it
   ```

Without `wf-recorder`, the button still opens the menu, and picking an option shows a notification
with the install command.

Expected state:

- `systemctl --user status waybar` is active; after a crash the bar comes back by itself.
- A camera button sits left of the status pill; a test recording plays back.

---

# Phase 23 — Screenshots to the clipboard, system info, calendar clicks

## Screenshots: saved and copied

Every screenshot is now saved **and** copied to the clipboard, ready to paste with `Ctrl+V`.
`scripts/screenshot.sh` replaces the three inline `grim` binds:

| Bind | Captures |
|------|----------|
| `SUPER+S` | The focused monitor (before: both monitors as one 3840-wide image) |
| `SUPER+SHIFT+S` | A region dragged with `slurp` (`Escape` cancels) |
| `SUPER+CTRL+S` | The focused window (before: region to clipboard only, now redundant) |

- Files go to `~/Pictures/screenshots/<date>_<time>.png`.
- `wl-copy --type image/png < file` puts the PNG on the Wayland clipboard. `wl-copy` forks into
  the background and keeps serving it until something else is copied, so pasting works after
  the script has exited.
- The notification shows a thumbnail of the capture (`notify-send -i <file>`). `dunstrc` now caps
  icons with `max_icon_size = 96` — without a cap, an image icon could show at full size.
- The focused window's geometry comes from `hyprctl activewindow -j` (`at` + `size`), the focused
  monitor from `hyprctl monitors -j`.

## System info from the Arch logo

Clicking the Arch logo opens **fastfetch** (Phase 11) in a floating kitty window
(`scripts/sysinfo.sh`); clicking the logo again, or pressing any key in the window, closes it.
Its size and position come from window rules — the 0.53+ syntax from Phase 13, which works for
kitty since `remember_window_size no`:

```text
windowrule = float on, match:class sysinfo
windowrule = size 820 460, match:class sysinfo
windowrule = center on, match:class sysinfo
```

The launcher moved off the logo's click: tapping `SUPER` (Phase 14) or `SUPER+R` opens it. The
logo's right-click still opens the wallpaper picker.

## Calendar clicks

The calendar from Phase 19 works (checked with a capture while hovering the clock), but it only
shows on hover, and the left click — "back to today" — did nothing visible while already on the
current month, so the clock looked dead when clicked. The click now switches between the month and
year views; right-click goes back to today:

```text
"actions": { "on-click": "mode", "on-click-right": "shift_reset",
             "on-scroll-up": "shift_up", "on-scroll-down": "shift_down" }
```

## Small fixes

- `"height": 36` in Waybar's config: the sliders made the modules 36 px tall, and Waybar warned
  at every start that the requested 34 was too small.

Expected state:

- After `SUPER+SHIFT+S` and a drag, `Ctrl+V` in a chat pastes the image; the file is also in
  `~/Pictures/screenshots/` and the notification shows a small thumbnail.
- Clicking the Arch logo shows fastfetch in a centered floating window; clicking it again closes it.
- Hovering the clock shows the calendar; clicking switches to the year view.

---

# Phase 24 — Interactive calendar; lock, login and idle screens

> To install the login theme, use `scripts/install-sddm-theme.sh` (Phase 37); the manual
> `cp` steps below are what it automates.

## Interactive calendar

The hover calendar (Phase 19) can't be browsed: GTK tooltips aren't interactive. Clicking the clock
now opens `scripts/rofi-calendar.py`, a calendar in Rofi's **script mode** (the same mechanism as
the volume control, Phase 20), so it stays open between clicks:

- Top row: `«` `‹` **Today** `›` `»` — previous/next year and month, back to the current month.
- A 7-column grid of days (week starts on Sunday, like the tooltip), today in an Arch-blue block.
- The month on screen travels between runs in `ROFI_DATA`; the clicked row's action arrives in
  `ROFI_INFO`.

Three Rofi details, all found by looking at the first render:

- `flow: horizontal` on the listview, or a multi-column list fills **column by column**.
- `\0markup-rows\x1ftrue` from the script: in script mode the `-markup-rows` flag is ignored and the
  Pango markup (weekday colors, today's block) showed up as raw `<span…>` text.
- `-no-show-icons`: `config.rasi` turns icons on, and the empty icon slot ate half of each cell
  ("Today" showed as "T…").

## Lock screen (hyprlock 0.9.6)

Redesigned with a big centered clock, the date, "Hi, `$USER`", and the password field, over the
current wallpaper blurred and dimmed (`brightness = 0.6`). The ring is Arch blue, turns **amber** while
the password is checked or Caps Lock is on, and **red** after a failure (`$PAMFAIL ($ATTEMPTS)`).
`hide_cursor` and `ignore_empty_input` (an Enter on an empty field isn't a failed attempt).
Fade in/out with the same bezier as `hyprland.conf`. Every option was checked against the example
hyprlock ships, `/usr/share/hypr/hyprlock.conf`.

**Safety net first.** If hyprlock crashes while locked, Hyprland shows a red "lockscreen died"
screen and the session is stuck — unless this is set in `hyprland.conf`:

```text
misc { allow_session_lock_restore = true }
```

Then from a TTY (`Ctrl+Alt+F3`, log in): `hyprctl --instance 0 dispatch exec hyprlock`, and unlock
normally. Test the lock screen with `SUPER+L`.

## Idle (hypridle 0.1.8)

- **The "dim" stage made the panel black.** It used `brightnessctl -s set 10`, copied from the
  upstream example — a *raw* value, which on this panel (max 65535) is 0.015%. `scripts/idle.sh dim`
  now sets 40% of the current brightness (saved first; any input restores it).
- **Suspend only on battery.** The 75-minute stage runs `scripts/idle.sh suspend`, which suspends
  only if a battery exists and no `Mains` power supply in `/sys/class/power_supply` is online.
  Plugged in, downloads and game updates keep running with the screens off. A desktop (no battery)
  never suspends.
- Both actions accept `--dry-run` to print what they would do.
- Timeline: dim 15 min → lock 30 → screens off 35 → suspend 75 (battery only). hypridle doesn't
  reload its config: `pkill -x hypridle; hyprctl dispatch exec hypridle`.

> The example in `/usr/share/hypr/hypridle.conf` turns screens off with
> `hyprctl dispatch 'hl.dsp.dpms({action = "off"})'` — the syntax of Hyprland's newer **Lua** config.
> With `hyprland.conf` that answers `Invalid dispatcher`; keep `hyprctl dispatch dpms off`.

## Login screen (SDDM)

SDDM had no config at all, so it showed its built-in fallback theme. `system/sddm/` adds a theme
written for this setup, **Arch Neutral** (`Main.qml`, Qt 6): the same layout as the lock screen —
blurred, dimmed wallpaper (`MultiEffect`), big clock, date, user, password field with the same
blue/amber/red ring — plus a session picker and Reboot / Power off.

- `metadata.desktop` says `QtVersion=6`, so SDDM runs it with `sddm-greeter-qt6`
  (`qt6-declarative` is already an SDDM dependency).
- `background.jpg` in the repo is a relative **symlink** to `assets/wallpapers/3.jpeg`, so the 6 MB
  image isn't stored twice; `cp -L` installs the real file. SDDM runs as its own user before anyone
  logs in, so it can't read the wallpaper symlink in your home — to change the login background,
  replace `/usr/share/sddm/themes/arch-neutral/background.jpg`.
- Like Phase 12's files, these are **copied** into the system, not symlinked.

Preview it without logging out (a window opens; close it with `SUPER+Q`):

```bash
sddm-greeter-qt6 --test-mode --theme ~/hyprland-from-scratch/system/sddm/themes/arch-neutral
```

Install:

```bash
sudo cp -rL ~/hyprland-from-scratch/system/sddm/themes/arch-neutral /usr/share/sddm/themes/
sudo mkdir -p /etc/sddm.conf.d
sudo cp ~/hyprland-from-scratch/system/sddm/sddm.conf.d/10-arch-neutral.conf /etc/sddm.conf.d/
```

`10-arch-neutral.conf` selects the theme and the Adwaita cursor. If a theme fails to load, SDDM falls
back to its built-in one, so a broken theme can't lock you out. The greeter logs QML errors to the
**journal**, not the terminal — that's how the theme was checked without opening anything:
`QT_QPA_PLATFORM=offscreen sddm-greeter-qt6 --test-mode --theme <dir>`, then
`journalctl --since -1min | grep Main.qml` (a deliberately broken copy did show its `TypeError`
there; the real theme loads with no errors).

Expected state:

- Clicking the clock opens the calendar; `‹ ›` and `« »` move through months and years.
- `SUPER+L`: big clock, date, greeting, blue ring; a wrong password turns it red.
- After 15 idle minutes the laptop panel dims (still readable); on AC the laptop never suspends by
  itself.
- The SDDM preview shows the same design; after installing, the real login screen does too.

---

# Phase 25 — Fonts: emoji and CJK on the web

Some web text and every emoji showed as empty boxes. `fc-match` showed why — the generic families
browsers ask for had nothing behind them:

```text
emoji      -> Noto Znamenny Musical Notation   # no emoji font installed at all
fc-list :lang=ja / zh-cn / ko                  # 0 fonts: Japanese, Chinese, Korean
```

Only `noto-fonts` (Latin, Greek, Cyrillic, Arabic, Devanagari, ...) and the Nerd Font were installed.

```bash
sudo pacman -S noto-fonts-emoji noto-fonts-cjk   # CJK is a ~190 MB download
fc-cache -f
```

Plus a fontconfig file, `dotfiles/fontconfig/fonts.conf` (linked to `~/.config/fontconfig` by
`scripts/link-dotfiles.sh`): `emoji` means Noto Color Emoji, and each generic family tries color
emoji right after its main font, so a monochrome emoji-like glyph from some text font doesn't win.

## A nicer default sans-serif, with nothing new installed

Web pages felt like "Times New Roman everywhere", for two reasons:

- **Brave's own default.** With no fonts set in its settings, Chromium's *Standard font* — used by
  every page that doesn't pick its own — is **Times New Roman**, which fontconfig resolves to Noto
  Serif. Fix it in `brave://settings/fonts`: **Standard font** and **Sans-serif font** →
  `Adwaita Sans`, **Fixed-width font** → `JetBrainsMono Nerd Font`. Serif can stay Noto Serif.
- **The system sans-serif was Noto Sans**, a fairly technical design. `fonts.conf` now puts
  **Adwaita Sans** first for `sans-serif` (and `system-ui`, which modern sites use and which follows
  it). It's GNOME's UI font, derived from **Inter**: open, soft letterforms made for screens. It's
  already installed as a GTK dependency (`adwaita-fonts`), and GTK apps like Thunar already used it
  (`gsettings get org.gnome.desktop.interface font-name` → `'Adwaita Sans 11'`). Noto Sans stays
  right behind it for the scripts Adwaita Sans doesn't cover.

```bash
fc-match sans-serif   # Adwaita Sans
fc-match system-ui    # Adwaita Sans
fc-match serif        # Noto Serif (unchanged)
```

Browsers and Electron apps read fontconfig at startup: restart them to see the change.

**Why the desktop's own font wasn't changed** *(at first — Phase 26 did change it, with tabular figures and the Propo icon font solving the problems below)*. Caelestia (`github.com/caelestia-dots/shell`) uses Rubik
(clock, workspaces), Google Sans Flex (body text), CaskaydiaCove Nerd Font (monospace) and Material
Symbols (icons). Here the bar, menus, notifications, lock and login screens all use JetBrains Mono.
Swapping it for a proportional font would change more than letter shapes: pills would change width
as numbers change, and the calendar grid and percentages would stop lining up. The current look was
kept on purpose; Rubik is in AUR (`ttf-rubik-vf`) and Inter in the official repos (`inter-font`) if
that's ever wanted.

Restart the browser after installing (it caches fonts), then check:

```bash
fc-match emoji                 # Noto Color Emoji
fc-list :lang=ja family | wc -l   # more than 0
```

Expected state:

- Emoji show in color in Brave, chats and notifications; Japanese/Chinese/Korean text renders.
- Web pages and apps use Adwaita Sans instead of a serif or Noto Sans (after setting Brave's
  standard font and restarting it).
- The bar, menus, notifications, lock and login screens look exactly as before (JetBrains Mono).

---

# Phase 26 — One UI font: Adwaita Sans everywhere

After Phase 25 the apps and web pages used Adwaita Sans while the bar, menus, notifications, lock
and login screens still used JetBrains Mono (each sets its font itself, so fontconfig didn't touch
them). Unified on **Adwaita Sans**, keeping JetBrains Mono only where monospace belongs: kitty,
btop, fastfetch, and code (`monospace` in fontconfig now prefers JetBrains Mono, so code blocks on
web pages and the `<tt>` calendar tooltip match the terminal).

| Where | Setting |
|-------|---------|
| Waybar `style.css` | `font-family: "Adwaita Sans", "JetBrainsMono Nerd Font Propo"; font-weight: 500; font-feature-settings: "tnum";` |
| Rofi `config.rasi` | `font: "Adwaita Sans, JetBrainsMono Nerd Font Propo 11";` |
| dunst `dunstrc` | `font = Adwaita Sans 11` |
| hyprlock | `$font = Adwaita Sans`; clock `<span font_features="tnum">` |
| SDDM theme | `fontFamily: "Adwaita Sans"`; clock `font.features: { "tnum": 1 }` |

Two details make a proportional font work in a bar full of numbers and icons:

- **Tabular figures (`tnum`).** In a proportional font "1" is narrower than "8", so the clock and
  the percentages would change their pill's width every time they tick. Adwaita Sans (like Inter)
  has an OpenType `tnum` feature that gives all digits the same width; GTK CSS, Pango markup and
  Qt 6.6+ can all turn it on.
- **The "Propo" Nerd Font for icons.** The first try used `"JetBrainsMono Nerd Font"` as the icon
  fallback and every icon touched its text ("⚙5%"): that variant draws icons wider than their
  advance, counting on the monospace space after them. `JetBrainsMono Nerd Font Propo` (installed
  with the same package) gives each icon its real width.

The SDDM theme is a copied file: after changing it, install it again (`sudo cp -rL ...`, Phase 24).

Expected state:

- The bar, menus, notifications and lock screen use the same font as the apps; icons keep a gap
  before their text; the clock's pill keeps its width as minutes change.
- kitty, btop and code blocks stay in JetBrains Mono.

---

# Phase 27 — Settings menu, keybinding help, quick search, thinner border

A round of usability notes, all built on what earlier phases set up.

## Light or dark: a place to choose

The theme was dark, forced by two `gsettings` lines at every login. `scripts/theme.sh` replaces
them: `dark`, `light`, `toggle`, and `restore` (the `exec-once` at login), remembering the choice in
`~/.local/state/hyprland-from-scratch/theme`. It switches what *applications* use — GTK3 (Thunar,
file pickers), GTK4/libadwaita, and websites that follow the system (Brave with its theme set to
"Device"). The desktop itself (bar, menus, notifications, lock, login) keeps the Arch Neutral dark
palette either way, like GNOME's always-dark top bar. Switch it from the settings menu.

## Settings menu

Right-click the Arch logo → `scripts/rofi-settings.sh`:

- **Apps theme** — dark / light.
- **Window border color** — Arch blue, Light blue, Soft white, Grey, Amber, Invisible.
- **Window border width** — 0–3 px.
- **Wallpaper...** and **Keybindings...**

Border values live in their own file, `dotfiles/hypr/appearance.conf`, which `hyprland.conf` pulls
in with `source =` and uses through variables:

```text
$border_size = 1
$border_active = rgba(1793d1cc)      # rgba(RRGGBBAA): the last two digits are the opacity
$border_inactive = rgba(333333aa)
```

The menu rewrites those lines with `sed`; Hyprland reloads by itself when the file changes. They
can also be edited by hand.

## A thinner, quieter border

The active border was 2 px with an opaque blue-to-light-blue gradient — the "neon" look. Now it's
**1 px of solid Arch blue at 80% opacity**, with no gradient.

## SUPER = quick search, SUPER+R = full launcher

Tapping SUPER opens `dotfiles/rofi/search.rasi`: the launcher's look without the Apps / Run / Files /
Windows row, six results — type, Enter, done. `SUPER+R` keeps the full launcher with every mode.

## Keybinding help (SUPER + /)

Every bind meant for people is now declared with **`bindd`** — Hyprland's bind with a description:

```text
bindd = $mainMod, T, Terminal (kitty), exec, $terminal
bindrd = $mainMod, Super_L, App search, exec, ...     # flags combine: r (release) + d
```

`scripts/rofi-keybinds.py` reads them live from `hyprctl binds -j` (`has_description`,
`description`, `modmask`, `key`), so the help can't fall out of date: a new `bindd` shows up by
itself. Workspace, media-key and mouse binds stay plain `bind` and appear as a few summary rows.
Keys are aligned in JetBrains Mono, descriptions in Adwaita Sans. Picking a row runs it
(`hyprctl dispatch <dispatcher> <args>`), so it doubles as a command palette.

## Fullscreen (SUPER+F)

`fullscreen, 0` — real fullscreen, which also hides the bar. Same key to come back.

## Five workspaces always in the bar

> **Removed in Phase 36** (back to Hyprland's default workspaces); kept here as history.

Workspaces 1–5 are **persistent** in `hyprland.conf` (`workspace = N, persistent:true`): they exist
even when empty, and Waybar (`"all-outputs": true`) lists them on every monitor's bar — the focused
one in blue, the one visible on the other monitor on a grey pill.

Two attempts that didn't work:

- Waybar's own `"persistent-workspaces": {"*": 5}` makes five **per monitor**: 1–5 on the laptop
  bar and 6–10 on the external one.
- Persistent workspaces with no monitor all landed on the focused (external) monitor; workspace 1
  left the laptop panel, which got a new empty workspace 6, and a browser window jumped to the
  external screen. Workspace 1 is now pinned: `workspace = 1, monitor:eDP-1, persistent:true`.
- *Not enough:* 2–5 still had no monitor, which broke after the next cold boot — Phase 35 pins
  them to the external monitor.

Expected state:

- Tapping SUPER shows a small search box; `SUPER+R` the full launcher.
- `SUPER + /` lists every keybinding; picking one runs it.
- Right-click on the Arch logo opens Settings; changing the border color applies at once.
- The active window has a thin, solid border.
- `SUPER+F` makes the window fullscreen over the bar, and back.
- The bar shows 1–5 even when they're empty.

---

# Phase 28 — A search box that shows nothing until you type

The quick search (tap SUPER) still listed six apps as soon as it opened. Now it opens as **just the
search box**, and matches appear as you type — more minimal, and nothing on screen until asked.
`SUPER+R` keeps the full launcher with its list and modes. All in `dotfiles/rofi/search.rasi`:

```text
listview { require-input: true; fixed-height: false; }      # hidden until there's input; as tall as the matches
window   { location: north; anchor: north; y-offset: 28%; } # hangs from a fixed point
configuration {
    drun-match-fields: "name,generic,keywords";              # not categories or the command
    sort: true;
    sorting-method: "fzf";                                   # best match first
}
```

- **Anchored at the top, not centered.** A centered window is re-centered every time the list
  grows, so the box jumped as you typed. Hanging from a point 28% down the screen, the box stays put
  and the list grows below it (checked: same position empty and with results).
- **Name matching.** By default Rofi also matches categories and the command: typing "ste" listed
  Thunar and kitty (category "Sy**ste**m") above Steam. Matching the name, generic name and
  keywords, sorted fzf-style, puts Steam first.
- The `configuration` block inside `search.rasi` only affects this search: `SUPER+R` (plain
  `config.rasi`) keeps Rofi's defaults (`rofi -show drun -dump-config` shows both).

Expected state:

- Tapping SUPER shows only a search box; typing "ste" shows Steam first; Enter launches it.
- `SUPER+R` still shows the full list with the Apps / Run / Files / Windows buttons.

---

# Phase 29 — Thunar actually dark, and better icons

Thunar stayed light although `gsettings` said `gtk-theme 'Adwaita-dark'` and
`color-scheme 'prefer-dark'` since Phase 13. Two separate causes:

1. **`Adwaita-dark` doesn't exist here.** GTK3 only ships "Adwaita" built in; its dark variant is
   requested with the `gtk-application-prefer-dark-theme` setting. A theme literally *named*
   `Adwaita-dark` comes from the `gnome-themes-extra` package (`/usr/share/themes` only had
   `Default` and `Emacs`), so GTK3 silently fell back to light Adwaita. GTK4/libadwaita apps were
   fine: they follow `color-scheme`. `scripts/theme.sh` now sets `gtk-theme 'Adwaita'` and writes
   `~/.config/gtk-3.0/settings.ini` with `gtk-application-prefer-dark-theme=1` (or `0` for light).
   It's a generated file, not a dotfile: the script rewrites it at every login and theme switch.
2. **Thunar is a long-lived service.** Its D-Bus activation starts `thunar.service`
   (`/usr/bin/Thunar --daemon`), which keeps running between windows and only reads settings at
   start — this one had been up since the morning, so every new window came from the old process.
   The process is `Thunar` with a capital T (`pgrep -x Thunar`). `theme.sh` now runs
   `systemctl --user try-restart thunar.service` after switching, unless a Thunar window is open.

`gtk-query-settings` (part of GTK3) prints the settings exactly as a GTK3 app sees them, without
opening a window — that's how the settings file was checked:

```bash
gtk-query-settings | grep -E "prefer-dark|theme-name"
```

## Icons: Papirus

The default Adwaita icon theme mixes styles in Thunar's sidebar, and some of its icons are dark
symbols that nearly vanish on a dark background. **Papirus** is a complete, consistent set whose
blue folders match the Arch blue:

```bash
sudo pacman -S papirus-icon-theme
~/hyprland-from-scratch/scripts/theme.sh restore
```

`theme.sh` uses **Papirus-Dark** for the dark theme and **Papirus** for the light one when the
package is installed (Adwaita otherwise), through both `gsettings icon-theme` (GTK4) and
`settings.ini` (GTK3). Rofi's `icon-theme` is `Papirus-Dark` too, so the launcher shows the same
icons (it falls back to `hicolor` until Papirus is installed).

Expected state:

- Thunar opens dark; with Papirus installed, its sidebar and folders use Papirus icons.
- The launcher's app icons match Thunar's.

---

# Phase 30 — Steam on a hybrid laptop: shader pre-caching

> **Hybrid iGPU + NVIDIA laptops only** (Phase 12). On a single-GPU machine pre-caching works as
> intended and should stay on.

Every launch of Dota 2 stopped at **"Processing Vulkan shaders (n%)"**. Dota runs on the NVIDIA dGPU
(launch options `prime-run %command%`, Phase 12), but Steam's shader workers don't:

```bash
pgrep -a fossilize                    # fossilize_replay = Steam's shader pre-compiler
nvidia-smi                            # the workers aren't listed: they use the AMD iGPU
tr '\0' '\n' < /proc/<pid>/environ | grep -E '__NV_PRIME|__VK_LAYER_NV'   # no prime-run variables
```

Steam starts them with its own environment, without `prime-run`'s variables, so they compile for
the AMD iGPU (RADV). Dota's shader cache folder
(`~/.local/share/Steam/steamapps/shadercache/570`) holds both `mesa_shader_cache_sf` /
`radv_builtin_shaders` (AMD) and `nvidiav1` (NVIDIA). The loop:

1. Steam pre-compiles the recorded shaders — for the wrong GPU.
2. Dota runs on NVIDIA and compiles what it needs into the NVIDIA driver's own cache.
3. During play, Steam records the shaders that were used.
4. Next launch, Steam "processes" them again for the AMD iGPU.

**Fix:** Steam → Settings → Downloads → Shader Pre-Caching → turn off **Enable Shader
Pre-caching** (change it from Steam's UI: Steam rewrites its config files on exit). The NVIDIA
driver keeps its own shader cache, so at most there's a short hitch the first time a new effect
appears, then it's cached. Until then, **Skip** on that dialog is safe — it only skips work for the
wrong GPU.

Not recommended: starting all of Steam with `prime-run` makes the workers compile for NVIDIA, but
keeps the dGPU awake for as long as Steam is open (battery, heat).

Expected state:

- Launching Dota 2 goes straight to the game, without the shader dialog.

---

# Phase 31 — Moving windows, touchpad gestures, minimal lock and login

## Moving and arranging windows

Already there since Phase 14: `SUPER+SHIFT+<n>` moves the focused window to workspace *n* and
follows it. Added:

| Keys | Action |
|------|--------|
| `SUPER+ALT+1…9, 0` | Send the window to workspace 1–10 but **stay** where you are (`movetoworkspacesilent`) |
| `SUPER+SHIFT+arrows` | Move the window left/right/up/down — swaps places with its neighbor (`movewindow`) |
| `SUPER+J` | Side by side ↔ stacked: flips how the window shares space with its neighbor |
| `SUPER+CTRL+arrows` | Resize the window, 40 px per press; hold to repeat (`binde` + `resizeactive`) |
| `SUPER` + left drag / right drag | Move / resize with the mouse (`bindm`) |
| `SUPER+N` | Minimize the window |
| `SUPER+SHIFT+N` | Restore minimized windows onto the current workspace |
| `SUPER+D` | Show desktop: minimize everything here, or restore if it's already empty |

- In Hyprland 0.56 the dwindle layout's split toggle is `layoutmsg, togglesplit`; the old
  standalone `togglesplit` dispatcher is gone (`hyprctl configerrors`: "Invalid dispatcher").
- Hyprland has no "minimize". `scripts/minimize.sh` parks windows on a hidden special workspace,
  `special:minimized`, and brings them back to whatever workspace is active (one `hyprctl --batch`
  call, so it's a single redraw).
- All of these are `bindd`, so they appear in the help menu (`SUPER + /`). Flags combine:
  `binded` = repeat + description, `bindmd` = mouse + description.

## Touchpad gestures

Hyprland 0.51+ syntax — `gesture = fingers, direction, action` — with `dispatcher` to run any
dispatcher once per swipe:

```text
gesture = 3, down,  dispatcher, exec, ~/hyprland-from-scratch/scripts/minimize.sh all
gesture = 3, up,    dispatcher, exec, ~/hyprland-from-scratch/scripts/minimize.sh restore
gesture = 3, left,  dispatcher, movefocus, l
gesture = 3, right, dispatcher, movefocus, r
gesture = 4, down,  dispatcher, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
gesture = 4, up,    dispatcher, exec, wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+
```

Directions can also be `horizontal`, `vertical`, `swipe`, `pinchin` and `pinchout`, and besides
`dispatcher` there are built-in actions such as `workspace` (an animated swipe between workspaces)
and `move` (all accepted by `hyprctl keyword gesture` on this version). Phase 32 adds two of them.

## Lock screen: minimal

Clock 110 → 64 px, date 22 → 14 px, no greeting. The password field is **hidden until you start
typing**: `fade_on_empty = true` keeps it invisible while empty, the first key brings it up, and it
fades again 2 s after being emptied. hyprlock can't react to a click, only to keys — the same as
macOS. Pill-shaped (`rounding = -1`), 1 px ring.

## Login screen: minimal, form on demand

The SDDM theme now shows only a smaller clock (84 px) and date, plus a faint "Press any key or click
to sign in". A click or any key fades in user, password, session and power buttons, and that first
key counts as the first character of the password. Esc, or 30 s without input, hides the form again.
Install it again for the change to reach the real login screen (`sudo cp -rL ...`, Phase 24).

## Quick search lower on screen

`search.rasi`: `y-offset` 28% → **60%**, between the middle and the bottom. With six results the
window ends at about 94% of a 1080 px screen (checked: box at y = 662, list bottom at 1012).

Expected state:

- `SUPER+ALT+4` sends a window to workspace 4 while you stay put; `SUPER+SHIFT+Left` swaps it with
  its left neighbor; `SUPER+J` stacks two side-by-side windows.
- Three fingers down hides every window on the workspace, up brings them back; four fingers change
  the volume.
- The lock screen shows a small clock; typing makes the password field appear.
- The login screen shows the clock until a click or a key.

---

# Phase 32 — Workspace swipe and pinch to fullscreen

Two more gestures, on top of Phase 31's:

```text
gesture = 4, horizontal, workspace                          # swipe between workspaces
gesture = 3, pinchout, dispatcher, fullscreenstate, 2 -1    # spread 3 fingers: fullscreen
gesture = 3, pinchin,  dispatcher, fullscreenstate, 0 -1    # pinch them: back
```

- `workspace` is a built-in gesture action: the workspaces slide under your fingers, using the
  `workspaces` animation, instead of jumping at the end of the swipe.
- The pinch uses `fullscreenstate <internal> <client>` (2 = fullscreen, 0 = none, -1 = leave the
  app's own state alone) rather than the `fullscreen` toggle, so spreading always means "fullscreen"
  and pinching always means "back" — an extra pinch can't flip it the wrong way. Confirmed the
  dispatcher exists by binding it to an unused key (a made-up dispatcher name is rejected).
- Only one action per fingers + direction: Hyprland answers "Previous PINCH_OUT shadows new
  PINCH_OUT" if a direction is reused. None of these overlap with the 3-finger swipes or the
  4-finger up/down volume.

Expected state:

- Four fingers left/right slide to the previous/next workspace.
- Spreading three fingers makes the focused window fullscreen; pinching them restores it.

---

# Phase 33 — Image viewer, screenshot annotation, video player

```bash
sudo pacman -S loupe satty celluloid
```

| App | Role |
|-----|------|
| **Loupe** (GNOME Image Viewer) | Default for every image type: simple, follows the dark theme, zoom / rotate / browse the folder |
| **Satty** | Annotate screenshots: arrows, text, boxes, blur, highlight |
| **Celluloid** (GTK front end for mpv) | Default for every video and audio type; shows up in the bar's music widget (MPRIS) |

## Default applications

Images and MP4 videos opened in **Brave** (`xdg-mime query default image/png`), and other video or
audio types had no default. `scripts/default-apps.sh` assigns every MIME type each app's `.desktop`
file declares — `image/*` to Loupe, `video/*` and `audio/*` to Celluloid (26 and 100 types) — with
`xdg-mime default`, which writes `~/.config/mimeapps.list` (used by Thunar, `xdg-open`, browsers).
Satty's own `.desktop` file also claims PNG and JPEG; the explicit assignment keeps Loupe as the
viewer. The script is idempotent; on a new machine run it after installing the packages (restore
step 2).

## Satty and screenshots

- Every screenshot's notification (`SUPER+S`, `SUPER+SHIFT+S`, `SUPER+CTRL+S`) now offers
  **Annotate**: click the notification and the capture opens in Satty. `notify-send --action`
  waits for the answer, so `scripts/screenshot.sh` does that in a background subshell.
- `SUPER+ALT+S` drags a region straight into Satty (`grim ... -t ppm - | satty --filename -`).
- `dotfiles/satty/config.toml` (keys from Satty's README): starts with the arrow tool, **Enter
  copies to the clipboard and saves** `~/Pictures/screenshots/<date>_<time>-annotated.png`, Esc
  closes, palette in Arch Neutral colors (keys 1–6), Adwaita Sans for text.
- **dunst:** left click now runs a notification's action and closes it (right click only closes;
  middle click closes all). Before, left click only closed it and actions were out of reach — this
  also makes clicking a browser notification open its page.

Tested with stand-ins for `grim`, `notify-send` (answering "annotate") and `satty`: the click path
opens the saved file, and `SUPER+ALT+S` pipes the region into Satty.

Expected state:

- Double-clicking an image in Thunar opens Loupe; a video or song opens Celluloid.
- After a screenshot, clicking its notification opens it in Satty; Enter copies and saves the
  annotated version.

---

# Phase 34 — More responsive volume gesture

Raising the volume with four fingers took several swipes. A `dispatcher` gesture runs its command
**once per swipe** — `hyprctl descriptions` lists no threshold or repeat option for them, only for
the workspace swipe — and each run changed the volume by 5%, which is hard to hear.

`scripts/volume.sh up|down [step] | mute` replaces the bare `wpctl` calls in the gestures:

- **10% per swipe** for the gestures (`volume.sh up 10`); two quick swipes are +20%.
- **An on-screen level**: a notification with a progress bar (`-h int:value:<percent>`) that
  replaces itself (`-h string:x-dunst-stack-tag:volume`) instead of stacking, gone after 1.2 s.
  dunst's `highlight = "#1793d1"` (normal urgency) colors the bar Arch blue.
- `volume.sh up 0` shows the current level without changing it — that's how it was checked.

The volume keys keep their 5% steps with no indicator; they could call `volume.sh` too.

Expected state:

- One quick four-finger swipe up raises the volume 10% and shows "Volume NN%" with a bar.

---

# Phase 35 — Workspaces pinned to their monitors

> **Superseded by Phase 36**, which removed persistent workspaces altogether.

After a cold boot the bar showed **six** workspaces, and `SUPER+2` switched the *laptop* screen to
workspace 2 instead of going to the external monitor:

```text
workspace 1  on eDP-1     (Brave)
workspace 2-5 on eDP-1    (empty)
workspace 6  on HDMI-A-1  (VS Code)
```

Phase 27 made workspaces 1–5 persistent but only pinned 1 to a monitor. Persistent workspaces are
created at login, and with no monitor rule all of 2–5 landed on the laptop panel; the external
monitor then needed a workspace of its own and got a new one, 6. It hadn't shown up earlier because
2–4 had been created by hand on the external monitor and the session wasn't restarted until the
next day.

Every persistent workspace now has a monitor, and each monitor a default:

```text
$external = desc:AOC 27G2G4 GYGM6HA425243
workspace = 1, monitor:eDP-1, default:true, persistent:true
workspace = 2, monitor:$external, default:true, persistent:true
workspace = 3, monitor:$external, persistent:true
...
```

- `default:true` is what a monitor shows at login, so the external one opens on 2 and no extra
  workspace is created.
- The external monitor is matched by description, like its 144 Hz rule (Phase 13), so it follows
  the monitor to any port. With the laptop alone, 2–5 fall back to its panel.
- Reloading the config moved 2–5 to the external monitor by itself (`hyprctl workspaces`), which
  also means they go back there when the monitor is reconnected; VS Code was then moved from 6 to 2.
- `hyprctl workspacerules -j` lists the rules as Hyprland understood them.

A different external monitor won't match `$external` — change the variable to its description
(`hyprctl monitors all`).

Expected state:

- After a reboot: laptop on workspace 1, external monitor on 2, bars show 1–5 only.
- `SUPER+2`…`SUPER+5` change the external monitor; `SUPER+1` goes to the laptop panel.

---

# Phase 36 — Back to default workspaces

Phases 27 and 35 kept workspaces 1–5 always visible in the bar. That requires them to *exist*
always — `persistent:true` — and a persistent workspace is created at login, so each one must be
pinned to a monitor. That broke after a cold boot (Phase 35) and left edge cases: the laptop alone
(2–5 not created until used), or an external monitor other than the AOC (starts on workspace 6).
Showing five numbers wasn't worth it, so it's gone:

- `hyprland.conf`: no `workspace = ...` rules at all (`hyprctl workspacerules -j` → `[]`).
- Waybar: back to its default — each bar lists the workspaces of its own monitor (no
  `all-outputs`, no `persistent-workspaces`).

Hyprland's default behavior works the same with one, two or more monitors of any model: each
monitor starts with its own workspace (laptop 1, external 2), `SUPER+<n>` creates workspace *n* on
the focused monitor, or moves focus to the monitor that already has it; empty workspaces disappear
when you leave them. Applied live without moving any window: the empty persistent workspaces 4 and
5 were dropped, 1–3 stayed where they were.

Expected state:

- After a reboot: laptop on 1, external monitor on 2; each bar shows its own workspaces.
- `SUPER+3` on the external monitor creates 3 there; `SUPER+1` goes to the laptop panel.

---

# Phase 37 — Installing the login theme (for real)

The login screen still looked like SDDM's default. `journalctl -b -u sddm` said why — `Loaded empty
theme configuration` — and `/usr/share/sddm/themes/` had only SDDM's own themes, with no
`/etc/sddm.conf.d/`: the theme from Phases 24/26/31 lived only in the repo. Its `sudo cp` install
steps were never run, and they'd need running again after every theme change.

Two more things had broken meanwhile: the wallpapers in `assets/wallpapers/` were renamed (`3.jpeg`
became part of a new `1.png`…`6.png` set), so the theme's `background.jpg` symlink to `3.jpeg`
dangled, and `wallpaper.sh restore` fell back to that same missing file.

**`scripts/install-sddm-theme.sh`** does the whole install in one step:

```bash
sudo ~/hyprland-from-scratch/scripts/install-sddm-theme.sh            # or: ... install-sddm-theme.sh /path/to/image
```

- Copies the theme to `/usr/share/sddm/themes/arch-neutral` and `10-arch-neutral.conf` to
  `/etc/sddm.conf.d/`, then checks that every file SDDM reads is there.
- **The login background is your current wallpaper** (the symlink `wallpaper.sh` keeps, found
  through `$SUDO_USER`'s home), copied as `background.<ext>` with `theme.conf` rewritten to match —
  so the login, lock screen and desktop show the same image. An image given as an argument wins;
  with neither, the first image in `assets/wallpapers/`. The repo no longer keeps an image or a
  symlink inside the theme.
- Run it again after changing the theme or the wallpaper. It changes the *next* login screen.
- Tested without root: the same script pointed at a temporary folder, then
  `QT_QPA_PLATFORM=offscreen sddm-greeter-qt6 --test-mode --theme <that folder>` loaded it with
  no errors in the journal.

`wallpaper.sh restore` no longer names a file: with nothing picked yet (or the picked file gone) it
uses the first image in `assets/wallpapers/`.

Expected state:

- After running the installer and logging out (or rebooting), the login screen shows the Arch
  Neutral theme over your current wallpaper.
- `journalctl -b -u sddm` no longer says `Loaded empty theme configuration`.

---

# Appendix — NVIDIA Hybrid GPU (historical note)

An earlier version of this guide documented a workaround built around
`NVreg_PreserveVideoMemoryAllocations=1` + `NVreg_TemporaryFilePath=/var/tmp` +
`NVreg_EnableGpuFirmware=0`, together with `nvidia_drm.modeset=0` on the kernel
command line, trying to force the AMD GPU to drive the display while NVIDIA
stayed out of the way.

That setup never held up under real use on this hardware. Two root causes:

- `AQ_DRM_DEVICES` was being set in `~/.config/uwsm/env` (or in `hyprland.conf`
    directly) instead of `~/.config/uwsm/env-hyprland`, so it never reached the
    Hyprland process environment under uwsm — `printenv | grep AQ_DRM` came back
    empty inside the session even though the file on disk looked right.
- `NVreg_EnableGpuFirmware=0` is actively wrong for `nvidia-open-dkms` 615+,
    which requires GSP firmware enabled to assign CRTCs to the dGPU. With it
    disabled, the driver loaded but could not light up any NVIDIA-attached
    output — exactly the HDMI/DisplayPort externals on this laptop.

The replacement is Phase 12 above, which uses the current Hyprland + uwsm
recommended path: stable `/dev/dri` symlinks via udev, `AQ_DRM_DEVICES` in
`env-hyprland`, NVIDIA session env vars kept out of the global session, and
`prime-run` for per-app dGPU offload. It was validated end-to-end on Oct 2026:
Hyprland session arrives via SDDM + uwsm with `AQ_DRM_DEVICES` present in the
process environment, both monitors active, NVIDIA idle at the desktop,
`prime-run` routing single apps to the dGPU on demand.

---

# Keybind Reference

Everything bound in `hyprland.conf` (`SUPER` = the Windows key) plus the Waybar clicks.

| Keys | Action |
|------|--------|
| `SUPER` (tap) | Quick app search: just a box, matches appear as you type; tap again to close |
| `SUPER+R` | Full launcher (apps / run / files / windows) |
| `SUPER+/` | Keybinding help (picking a row runs it) |
| `SUPER+F` | Fullscreen on/off (hides the bar) |
| `SUPER+SHIFT+arrows` | Move the window (swap with its neighbor) |
| `SUPER+CTRL+arrows` | Resize the window (hold to repeat) |
| `SUPER+J` | Side by side ↔ stacked |
| `SUPER` + drag | Left button: move · right button: resize |
| `SUPER+N` / `SUPER+SHIFT+N` | Minimize window / restore minimized |
| `SUPER+D` | Show desktop (minimize all / restore) |
| `SUPER+ALT+1…9, 0` | Send the window to a workspace without following it |
| Touchpad, 3 fingers | Down: minimize all · up: restore · left/right: focus window |
| Touchpad, 4 fingers | Down / up: volume ±10% (with an on-screen level) · left / right: switch workspace |
| Touchpad, 3-finger pinch | Spread: fullscreen · pinch: back |
| `SUPER+W` | Wallpaper picker |
| Click outside an open menu | Closes it |
| Mouse over a menu row / click | Highlights it / runs it |
| `SUPER+T` | Terminal (kitty) |
| `SUPER+E` | File manager (Thunar) |
| `SUPER+Q` | Close the focused window |
| `SUPER+V` | Toggle floating |
| `SUPER+←/→/↑/↓` | Move focus |
| `SUPER+1…9`, `SUPER+0` | Go to workspace 1–10 |
| `SUPER+SHIFT+1…9`, `SUPER+SHIFT+0` | Move the focused window to workspace 1–10 |
| `SUPER+L` | Lock screen (hyprlock) |
| `SUPER+Escape` | System monitor (btop, floating) |
| `SUPER+S` | Screenshot of the focused monitor → saved + copied to the clipboard |
| `SUPER+SHIFT+S` | Screenshot of a selected region → saved + copied |
| `SUPER+CTRL+S` | Screenshot of the focused window → saved + copied |
| `SUPER+ALT+S` | Screenshot of a region, opened in Satty to annotate |
| Click a screenshot's notification | Annotate it in Satty |
| `SUPER+SHIFT+R` | Screen recording: menu; again = stop and save (same as the camera button) |
| `SUPER+M` | Exit Hyprland (back to SDDM) |
| Brightness / volume / mute keys | Brightness ±5%, volume ±5%, mute |

| Waybar | Click |
|--------|-------|
| Arch logo | Left: system info (again: close) · Right: settings (apps theme, border, wallpaper, keybindings) |
| Network | Network menu: adapters, Wi-Fi, nmtui (`scripts/rofi-network.sh`) |
| Bluetooth | Bluetooth menu: connect, scan & pair, forget, power (`scripts/rofi-bluetooth.sh`) |
| Volume | Hover: volume slider · Left: audio menu (`scripts/rofi-audio.sh`) · Right: mute · Scroll: ±5% |
| Brightness | Hover: gold slider · Scroll: ±5% (laptop panel only) |
| Workspace number | Switch to it |
| Clock | Hover: calendar · Click: interactive calendar (browse months/years) · Scroll: month · Right: month/year · Middle: back to today |
| Battery | Hover: time left, power draw, health |
| Power profile (next to battery) | Left: next profile · Right: previous |
| Camera button (left of the status pill) | Recording menu; while recording it's a red REC pill: stop and save |

Clicking the icon of an open menu closes it; clicking another icon switches to that menu.

---

# Keeping the Repo Restorable

Path B only works if the repo matches the machine. Whenever something changes:

- **New package** → add it to the right list in `packages/` (`base.txt` if any dotfile or script
  depends on it, `apps.txt` for personal apps, `aur.txt` for AUR, `nvidia-hybrid.txt` for Phase 12).
- **New app config** → create `dotfiles/<app>/`, then run `scripts/link-dotfiles.sh` (it moves
  the existing `~/.config/<app>` to a backup and links the repo copy).
- **New file outside `~/.config`** (`/etc/...`) → keep a copy under `system/<topic>/` and document
  the `sudo cp` step in its phase.
- **New service** → add its `systemctl enable` to step 4 of the restore section.

Quick drift check — explicitly installed packages that aren't in any list:

```bash
comm -23 <(pacman -Qqe | sort) <(grep -hv '^#' ~/hyprland-from-scratch/packages/*.txt | grep . | sort)
```

(Expect a few archinstall base packages in the output — `base`, `linux`, `linux-firmware`, `sudo`, ...)

---

# Next Phases

The remaining phases will be developed incrementally.

- Gaming — Steam is installed (`packages/apps.txt`); still to document: `prime-run %command%`
  launch options per game (Phase 12), `gamemode`, and `mangohud` for an FPS/temperature overlay.
- External monitor brightness — the AOC has no kernel backlight; it needs DDC/CI through
  `ddcutil` (plus the `i2c-dev` module). Its HDMI port is driven by the NVIDIA dGPU (Phase 12), so
  this needs testing before it goes in the guide.
- Utility Scripts
- Boot menu (Visor, github.com/IO-ZetZor/Visor-BootManager) — deliberately deferred, not part
  of the Phase 11 desktop pass. Unlike everything above, this replaces the bootloader itself
  (compiles from source, writes to the EFI System Partition) — real risk of an unbootable
  machine if misconfigured. Do this as its own phase, with a rescue USB on hand, only when
  explicitly asked.

---

# Phase 38 — Short README, installer, copy or link mode

The README had grown into this build log (3,100+ lines, 37 phases): good history, poor install
guide — nobody reinstalls by following 37 phases by hand. Split in two:

- **`README.md`** (~230 lines): install Arch, run the installer, keybindings, where to customize,
  troubleshooting, and **annex A** for hybrid NVIDIA laptops (Phase 12 condensed, plus the Steam
  notes of Phase 30).
- **`docs/build-log.md`**: this file — the old README, unchanged, with this phase added. Code comments
  that said "README Phase N" now point here.

**`install.sh`** replaces the manual restore steps: packages (`base.txt`, `--apps` for
`apps.txt`), configs into `~/.config`, services (SDDM, Bluetooth, power profiles, Waybar user
service), default apps, login theme. `--dry-run` prints every step. It notices hybrid graphics and
points to annex A, but never applies it.

**Two ways to install.** `./install.sh` *copies* the configs (use the desktop; the repo can change
without touching it; re-run to update). `./install.sh --link` *symlinks* them — the way this machine
runs, editing the repo edits the desktop live. `scripts/link-dotfiles.sh --copy` does the copying:
copied folders carry a `.hyprland-from-scratch` marker so a re-run updates them in place instead of
backing them up again, and the two modes can be switched back and forth (tested against a throwaway
`XDG_CONFIG_HOME`: foreign config backed up, second run updates without backups, link → copy
replaces only our own symlinks).

Fixes found on the way:

- Scripts that read their config from the repo path (`dotfiles/rofi/menu.rasi`,
  `dotfiles/hypr/appearance.conf`) now read `~/.config/...` — in copy mode the repo isn't the live
  config, and the settings menu would have edited the wrong file.
- `packages/base.txt` has comments at the end of some lines; the old restore command
  (`grep -v '^#' … | pacman -S -`) would have passed those words to pacman as package names. The
  installer strips everything after `#` (all 45 names checked with `pacman -Si`).
- `power-profiles-daemon` moved from `apps.txt` to `base.txt`: the bar's power profile icon needs it.
- `.gitignore` for the `__pycache__/` that running the Python helpers creates.
