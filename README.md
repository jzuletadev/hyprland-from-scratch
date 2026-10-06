# Hyprland From Scratch

[#hyprland-from-scratch](#hyprland-from-scratch)

A step-by-step guide to build a clean, minimal and fully understood Hyprland desktop on Arch Linux.
The goal is not to install a desktop as fast as possible, but to understand every component that is installed and keep the entire configuration under version control.

---

# Project Goals

[#project-goals](#project-goals)

- Build everything from scratch.
- Install only what is needed.
- Understand every package before installing it.
- Version all dotfiles.
- Reproduce the entire desktop from this repository.

---

# Repository Structure

[#repository-structure](#repository-structure)

```
hyprland-from-scratch/
├── guide/
├── dotfiles/
├── scripts/
├── assets/
├── .gitignore
└── README.md
```

---

# Phase 0 — Arch Linux

[#phase-0--arch-linux](#phase-0--arch-linux)

Boot the official Arch ISO and start the installer.

```
archinstall
```

Use the following configuration:

| Setting             | Value                                           |
| ------------------- | ----------------------------------------------- |
| Locale              | en\_US.UTF-8                                    |
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

```
reboot
```

Expected state:

- Arch boots into a TTY.
- Internet works.
- No desktop environment installed.
- No display manager installed.

---

# Phase 0.5 — Initial System Setup

[#phase-05--initial-system-setup](#phase-05--initial-system-setup)

Update the system.

```
sudo pacman -Syu
```

Enable SSH.

```
sudo pacman -S openssh
sudo systemctl enable --now sshd
```

Get the machine IP.

```
ip a
```

Connect from the main computer.

```
ssh user@<ip>
```

Configure Git.

```
git config --global user.name "Your Name"
git config --global user.email "you@example.com"
git config --global init.defaultBranch main
```

Generate an SSH key.

```
ssh-keygen -t ed25519 -C "you@example.com"
```

Display the public key.

```
cat ~/.ssh/id_ed25519.pub
```

Add it to GitHub. Test the connection.

```
ssh -T git@github.com
```

Clone the repository.

```
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

[#phase-1--minimal-hyprland](#phase-1--minimal-hyprland)

Install only the required packages.

```
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

```
fc-cache -fv
fc-match monospace
```

Create the configuration directory.

```
mkdir -p ~/.config/hypr
nano ~/.config/hypr/hyprland.conf
```

Set minimal and basic configuration.

```
monitor = ,preferred,auto,1
bind = SUPER, T, exec, kitty
bind = SUPER, M, exit
```

Save (`Ctrl+O`, `Enter`, `Ctrl+X`).

### Before launching: make sure no stale session is running

[#before-launching-make-sure-no-stale-session-is-running](#before-launching-make-sure-no-stale-session-is-running)

If you've tried launching Hyprland before and it crashed, closed abruptly, or you switched
TTYs without exiting it properly, a leftover process or lockfile can silently block the new
session (you'll see `Unable to lock lockfile ... maybe another compositor is running` in the
log, and keybinds like opening a terminal simply won't do anything, even though Hyprland
appears to be running).

```
ps aux | grep -i hypr
```

If a `Hyprland` process shows up, kill it:

```
killall -9 Hyprland
```

Check for a leftover lock:

```
ls -la /run/user/$(id -u)/ | grep wayland
```

If a `wayland-*.lock` file exists with no live process behind it, remove it:

```
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

[#start-hyprland](#start-hyprland)

Launch it wrapped in a D-Bus session (plain `Hyprland` triggers a
"launched without start-hyprland" warning and D-Bus/portal activation failures):

```
dbus-run-session Hyprland
```

### Verify the config actually loaded

[#verify-the-config-actually-loaded](#verify-the-config-actually-loaded)

```
hyprctl binds | grep -A2 kitty
```

You should see the `SUPER, T, exec, kitty` bind listed. If it's missing, the config wasn't
read — recheck the file path and that it was saved.

If a keybind is present but still doesn't do anything, confirm kitty is actually installed
and reachable:

```
which kitty
```

Verify:

```
echo $XDG_SESSION_TYPE
```

Expected output:

```
wayland
```

If you already have Hyprland running and just changed the config, you don't need to restart
the whole session — reload it live:

```
hyprctl reload
```

Expected state:

- Hyprland starts successfully.
- `hyprctl binds` shows the kitty bind.
- Kitty opens with `SUPER+T`.
- Session runs on Wayland.

---

# Phase 2 — Repository Integration

[#phase-2--repository-integration](#phase-2--repository-integration)

Create the dotfiles structure.

```
dotfiles/
└── hypr/
```

Move the configuration into the repository.

```
mkdir -p ~/hyprland-from-scratch/dotfiles
mv ~/.config/hypr \
   ~/hyprland-from-scratch/dotfiles/
```

Create a symbolic link.

```
ln -s \
~/hyprland-from-scratch/dotfiles/hypr \
~/.config/hypr
```

Verify.

```
ls -l ~/.config
```

Expected output:

```
hypr -> ~/hyprland-from-scratch/dotfiles/hypr
```

Commit the current state.

```
git add .
git commit -m "Initial Hyprland setup"
```

Expected state:

- Hyprland works.
- Configuration is stored inside the repository.
- `~/.config` only contains a symbolic link.
- Git detects configuration changes automatically.

---

# Phase 3 — Waybar

[#phase-3--waybar](#phase-3--waybar)

Waybar is the status bar. Install empty/minimal first, confirm it renders, then add modules
one at a time.

```
sudo pacman -S waybar
mkdir -p ~/hyprland-from-scratch/dotfiles/waybar
```

Place `config.jsonc` (defines **which** modules appear and where — `modules-left`, `modules-center`, `modules-right`) and `style.css` (defines **how** it looks — same selector
model as normal CSS) at `~/hyprland-from-scratch/dotfiles/waybar/`. Baseline: only `hyprland/workspaces` on the left and `clock` centered — no network, battery, or tray yet.

Symlink:

```
ln -s ~/hyprland-from-scratch/dotfiles/waybar ~/.config/waybar
```

Test manually first:

```
waybar &
```

Once confirmed working, autostart it in `hyprland.conf`:

```
exec-once = waybar
```

`exec-once` (vs `bind ... exec`) runs once at session start only — using plain `exec` here
would spawn a new Waybar instance on every `hyprctl reload`.

Expected state:

- Waybar appears at the top of the screen with workspaces + clock.
- It starts automatically with the session, without manual `waybar &`.

---

# Phase 4 — Application Launcher (Rofi)

[#phase-4--application-launcher-rofi](#phase-4--application-launcher-rofi)

Rofi vs Wofi: Wofi is more minimal and Wayland-native from the start; Rofi is more mature,
far more feature-rich (emoji picker, clipboard manager, extensions via `rofi-calc` etc.), and
is what most Hyprland community configs and documentation reference. Rofi is used here for
that ceiling — the goal is deep customization later, not just "an app launcher."

```
sudo pacman -S rofi
mkdir -p ~/hyprland-from-scratch/dotfiles/rofi
```

Place `config.rasi` at `~/hyprland-from-scratch/dotfiles/rofi/config.rasi`. Key points:

- `modi: "drun"` — "desktop run" mode: reads installed `.desktop` files to build the app list.
Other modes exist (`run`, `window`, `ssh`) and can be added later.
- `@theme "/dev/null"` — disables any system default theme so every color rendered comes
explicitly from this file, not a hidden default.

Symlink:

```
ln -s ~/hyprland-from-scratch/dotfiles/rofi ~/.config/rofi
```

Add a bind in `hyprland.conf`:

```
bind = $mainMod, R, exec, rofi -show drun
```

Reload and test:

```
hyprctl reload
```

`SUPER+R` should open the launcher; typing an app name (e.g. "kitty") should find and launch it.

---

# Phase 5 — Notifications (dunst)

[#phase-5--notifications-dunst](#phase-5--notifications-dunst)

Two main options exist: **mako** (native Wayland, minimal, key=value config) or **dunst** (more mature, more configurable — per-app rules, urgency levels, scripting, history). Dunst is
used here: the goal is deep future customization, and dunst's extra surface area is exactly
the kind of thing worth learning early rather than working around later.

Install:

```
sudo pacman -S dunst libnotify
```

Create the config directory in the repo:

```
mkdir -p ~/hyprland-from-scratch/dotfiles/dunst
```

Place a `dunstrc` file at `~/hyprland-from-scratch/dotfiles/dunst/dunstrc` defining `[global]` plus `[urgency_low]`, `[urgency_normal]`, and `[urgency_critical]` sections — the urgency
levels are dunst's key advantage over mako: `urgency_critical` can be set to `timeout = 0` so
critical notifications never auto-dismiss.

Symlink:

```
ln -s ~/hyprland-from-scratch/dotfiles/dunst ~/.config/dunst
```

Autostart in `hyprland.conf`:

```
exec-once = dunst
```

Reload and test all three urgency levels:

```
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
> ```
> killall dunst
> dunst &
> ```

Expected state:

- dunst starts automatically with the session.
- Notifications match the rest of the setup's color scheme.
- Critical notifications stay on screen until dismissed.

---

# Phase 5.5 — Brightness control

[#phase-55--brightness-control](#phase-55--brightness-control)

Hyprland doesn't handle brightness on its own — brightness keys just send a keycode; actually
changing the panel's backlight needs dedicated tooling.

Install:

```
sudo pacman -S brightnessctl
```

Confirm the backlight device is detected:

```
brightnessctl
```

Test manually before binding to keys:

```
brightnessctl set 10%-
brightnessctl set 10%+
```

Find the actual key names your keyboard sends:

```
sudo libinput debug-events
```

(press the brightness keys, look for `KEY_BRIGHTNESSUP` / `KEY_BRIGHTNESSDOWN`, `Ctrl+C` to exit)

Add binds in `hyprland.conf`:

```
bindel = ,XF86MonBrightnessUp, exec, brightnessctl set 5%+
bindel = ,XF86MonBrightnessDown, exec, brightnessctl set 5%-
```

`bindel` (vs plain `bind`) allows key-repeat while held (`e`) and works even when the screen
is locked later on (`l`), once hyprlock is in place.

Reload and test with the physical keys:

```
hyprctl reload
```

---

# Phase 5.6 — Touchpad scroll direction

[#phase-56--touchpad-scroll-direction](#phase-56--touchpad-scroll-direction)

If two-finger scroll feels inverted, it's controlled by `natural_scroll` in the `touchpad` block of `hyprland.conf`. `true` = content follows finger direction (like mobile/macOS); `false` = classic inverted-wheel behavior.

```
input {
    touchpad {
        natural_scroll = true
    }
}
```

Reload to apply:

```
hyprctl reload
```

> This setting only affects the touchpad. An external mouse's scroll direction is controlled
> separately via `input { natural_scroll = ... }` at the top level of the `input` block (outside `touchpad`), and is usually left `false` since inverted scroll feels unnatural on a mouse.

---

# Phase 6 — Wallpaper (awww)

[#phase-6--wallpaper-awww](#phase-6--wallpaper-awww)

> **Naming note:** the tool referenced across most community guides as `swww` was renamed by
> its creator to **awww** after the original `swww` project was archived. Arch's official
> repos now only ship `awww` — `pacman -S swww` resolves to the `awww` package automatically,
> but the binaries are `awww` and `awww-daemon`, not `swww`/`swww-daemon`. All commands below
> use the current names.

Install:

```
sudo pacman -S awww
```

Create the repo folders (this is the first real use of the `assets/` folder reserved from the
start of the repo structure):

```
mkdir -p ~/hyprland-from-scratch/dotfiles/awww
mkdir -p ~/hyprland-from-scratch/assets/wallpapers
```

Place a wallpaper image at `~/hyprland-from-scratch/assets/wallpapers/`. No image handy? Generate
a solid-color placeholder locally (no internet needed beyond the Arch mirrors already used for
pacman):

```
sudo pacman -S imagemagick
convert -size 1920x1080 xc:'#1e1e2e' ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

awww needs its daemon running before it can set a wallpaper:

```
awww-daemon &
awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

Test an animated transition (this is the actual reason to prefer this tool over hyprpaper):

```
awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg --transition-type wipe --transition-duration 1.5
```

> **How to actually see the transition:** the transition is a property of the `awww img` command itself, not a separate toggle — but it's only visible when switching *between two
> different images*. Running it with the same image that's already set technically still runs
> the transition, but with no visual difference between "before" and "after" there's nothing to
> see. Keep at least two test images in `assets/wallpapers/` and alternate between them:
>
> ```
> awww img ~/hyprland-from-scratch/assets/wallpapers/2.png --transition-type wipe --transition-duration 1.5
> awww img ~/hyprland-from-scratch/assets/wallpapers/3.jpeg --transition-type grow --transition-duration 1.5
> ```
>
> Some `--transition-type` values worth comparing: `simple` (fade), `wipe`, `wave`, `grow`, `outer`, `random` (picks one at random each time).

Autostart in `hyprland.conf`:

```
exec-once = awww-daemon
exec-once = awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

Reload:

```
hyprctl reload
```

For a full test, restart the whole Hyprland session (not just reload) and confirm the
wallpaper appears on its own, without running any command by hand.

---

# Phase 7 — Lock Screen (hyprlock)

[#phase-7--lock-screen-hyprlock](#phase-7--lock-screen-hyprlock)

```
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

```
bind = $mainMod, L, exec, hyprlock
```

Reload and test:

```
hyprctl reload
```

`SUPER+L` should show the lock screen; typing your user password should unlock it.

> Keep an SSH session open while testing — `killall hyprlock` from there is the escape hatch
> if the lock screen ever gets stuck.

---

# Phase 8 — Idle Management (hypridle)

[#phase-8--idle-management-hypridle](#phase-8--idle-management-hypridle)

hypridle watches for inactivity and fires staged actions — dim, lock, screen off, suspend —
integrating directly with hyprlock from Phase 7.

```
sudo pacman -S hypridle
```

Baseline `hypridle.conf` structure:

- `general` block — `lock_cmd` (what locks the session), `before_sleep_cmd`/`after_sleep_cmd` (run right before/after suspend).
- One `listener { }` block per stage, each independent: its own `timeout` (seconds) and
`on-timeout` action, with `on-resume` reverting it once activity resumes. Baseline staging:
dim at 150s, lock at 300s, screen off at 330s, suspend at 900s.

Place the file alongside the others (no new symlink needed) and autostart it:

```
exec-once = hypridle
```

```
hyprctl reload
```

> **Testing tip:** minute-long timeouts are slow to verify. Temporarily lower every `timeout` to a handful of seconds, restart the daemon (`pkill hypridle && hypridle &`), confirm each
> stage fires in order, then restore the real values (150/300/330/900) once confirmed.

---

# Phase 9 — SDDM (login screen)

[#phase-9--sddm-login-screen](#phase-9--sddm-login-screen)

SDDM provides a traditional graphical login (username + password) instead of TTY autologin,
and — as a side benefit — launches Hyprland correctly on its own, resolving the earlier
"launched without start-hyprland" warning without needing `dbus-run-session Hyprland` by hand.

Install:

```
sudo pacman -S sddm
sudo systemctl enable sddm
```

Confirm Hyprland exposes its session file (installed automatically by the `hyprland` package):

```
ls /usr/share/wayland-sessions/
```

You should see both `hyprland.desktop` and `hyprland-uwsm.desktop` — prefer the **uwsm** variant in SDDM's session picker. `uwsm` (Universal Wayland Session Manager) is the modern
recommended launch method: it handles D-Bus, environment variables, and session lifecycle
correctly, replacing both the old `start-hyprland` script and a manual `dbus-run-session Hyprland` invocation.

> **Before rebooting**, make sure no TTY autologin override is left configured — it can
> conflict with SDDM taking over the login:
>
> ```
> cat /etc/systemd/system/getty@tty1.service.d/override.conf 2>/dev/null
> ```
>
> If this returns nothing, there's nothing to clean up.

Reboot:

```
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

[#phase-10--theming-tokyo-night](#phase-10--theming-tokyo-night)

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

[#hyprland-borders-blur-rounding](#hyprland-borders-blur-rounding)

`col.active_border` becomes a blue-to-purple gradient, `col.inactive_border` a translucent
dark surface color, corner `rounding` goes from 6 to 10, and window blur gets enabled (small `size`/`passes` — this machine's internal panel is driven by the AMD iGPU, not the NVIDIA GPU,
see Phase 12 below, so a light blur has headroom to spare). `active_opacity` / `inactive_opacity` add a subtle transparency to unfocused windows.

`layerrule = blur, waybar` and `layerrule = blur, rofi` extend that same blur to the bar and
launcher — without this, layer-shell surfaces stay flat even if their own CSS/rasi sets an
alpha-transparent background.

## Waybar: from 2 modules to a real bar

[#waybar-from-2-modules-to-a-real-bar](#waybar-from-2-modules-to-a-real-bar)

The Phase 3 baseline only had workspaces + a clock. This phase adds:

- `hyprland/window` (center) — the focused window's title.
- `network`, `wireplumber`, `battery` (right, before the clock) — all backed by daemons already
running on this system (NetworkManager, wireplumber, upower), so no new packages needed.

Verify the modules Waybar was actually built with, since a distro package can be compiled
without some of them:

```
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
> ```
> waybar --log-level debug 2>&1 | grep "Found config file"
> ```

## Rofi, dunst, hyprlock

[#rofi-dunst-hyprlock](#rofi-dunst-hyprlock)

Same palette swapped into `rofi/config.rasi`, `dunst/dunstrc`, and `hyprlock.conf` — background,
borders/frame, text, and the critical-urgency/red accent all point at the same hex values as
Waybar and Hyprland now.

## Media keys

[#media-keys](#media-keys)

Since Waybar now shows volume, the media keys to actually change it were missing. Added to `hyprland.conf`, next to the existing brightness keys:

```
bindel = ,XF86AudioRaiseVolume, exec, wpctl set-volume -l 1.0 @DEFAULT_AUDIO_SINK@ 5%+
bindel = ,XF86AudioLowerVolume, exec, wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-
bindel = ,XF86AudioMute, exec, wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle
```

`wpctl` (wireplumber's CLI) is used instead of `pamixer` since wireplumber is already the
running audio session manager — no extra package.

## Polish pass

[#polish-pass](#polish-pass)

A few loose ends closed once the base theme was actually verified on screen (see the
"Bug found" note above — that verification is what surfaced these):

- **`layerrule = blur, <namespace>` was dropped.** This is the documented syntax across
Hyprland community configs, but on this install (0.56.1) `hyprctl keyword layerrule "blur, waybar"` reliably returns `invalid field blur: missing a value`, and trying variants
(`blur 1, namespace ...`, colon syntax, etc.) only ever produced the same generic
"unrecognized field" error — not enough signal to reverse-engineer the real syntax for this
version. Window blur (`decoration { blur { ... } }`) works fine on its own; Waybar/Rofi keep
their alpha-transparent backgrounds without the extra blur-through effect.
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

```
hyprctl reload
killall waybar && waybar &
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

[#phase-11--desktop-pass-waybar-pills-kitty-fastfetch-btop-rofi-control-center](#phase-11--desktop-pass-waybar-pills-kitty-fastfetch-btop-rofi-control-center)

A second, more ambitious pass driven directly by reference screenshots (r/unixporn-style
setups) instead of just filling in a minimal baseline. Goal: a cohesive "everything is one
theme" feel without adopting a heavy all-in-one framework (evaluated and deliberately skipped:
Caelestia's `quickshell`-based stack — liked the look, not the weight).

## A real bug: Nerd Font icons were never actually rendering

[#a-real-bug-nerd-font-icons-were-never-actually-rendering](#a-real-bug-nerd-font-icons-were-never-actually-rendering)

Every icon glyph added across Phase 10 and this phase (idle inhibitor, backlight, network,
volume, battery, the new Arch logo) silently failed to save — typing a Nerd Font Private Use
Area character directly into a config file produced an empty string on disk, with zero
indication anything was wrong (no parse error; the format string is still valid, just iconless).
Confirmed by scanning the written file in Python for characters above `0x7f`: none found, despite
every module "having" an icon. Fix: generate the file with a small `python3` script using `\uXXXX` escapes instead of the literal character, e.g.:

```
icon = ""  # Arch logo, nf-linux-archlinux
```

Verify a codepoint is actually in the installed font before using it:

```
fc-query -f "%{charset}\n" /usr/share/fonts/TTF/JetBrainsMonoNerdFontMono-Regular.ttf
```

## Waybar: pill clusters, Arch logo, music, Bluetooth

[#waybar-pill-clusters-arch-logo-music-bluetooth](#waybar-pill-clusters-arch-logo-music-bluetooth)

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
Bluetooth picker script below.
- Full module list now: `custom/arch`, `hyprland/workspaces`, `mpris` (left) — `hyprland/window` (center) — one `group/status` pill (idle_inhibitor, backlight, network, wireplumber,
custom/bluetooth, battery), tray, clock (right).

## kitty — first real theme since Phase 1

[#kitty--first-real-theme-since-phase-1](#kitty--first-real-theme-since-phase-1)

`dotfiles/kitty/kitty.conf` (new file, new `~/.config/kitty` symlink — it was an empty directory
before this). Tokyo Night ANSI palette, JetBrainsMono Nerd Font, subtle `background_opacity`.
Window rounding/blur come from Hyprland's compositor-level `decoration` block, not kitty itself.

## fastfetch and btop — not runnable yet

[#fastfetch-and-btop--not-runnable-yet](#fastfetch-and-btop--not-runnable-yet)

Both configs are written and ready but **need packages this session couldn't install** (`sudo pacman -S fastfetch btop` requires an interactive sudo password, which an agent session
can't supply):

- `dotfiles/fastfetch/config.jsonc` — curated module list (not fastfetch's full default set),
Arch ASCII logo. Colors are named ANSI keywords (`blue`, `magenta`, ...) rather than hex —
they inherit Tokyo Night automatically from kitty's ANSI remap above, so the config doesn't
need to duplicate hex values.
- `dotfiles/btop/btop.conf` + `dotfiles/btop/themes/tokyonight.theme` — full Tokyo Night theme
covering CPU/mem/net/proc boxes and gradients.
- Keybind already wired in `hyprland.conf`: `$mainMod, Escape` opens btop in a floating,
centered kitty window. This exposes a second Hyprland gotcha (see below).

Once installed, symlinks aren't needed for a manual first run — just run `fastfetch` or `btop`.

## A second Hyprland gotcha: `windowrulev2` looks like it works but doesn't

[#a-second-hyprland-gotcha-windowrulev2-looks-like-it-works-but-doesnt](#a-second-hyprland-gotcha-windowrulev2-looks-like-it-works-but-doesnt)

The documented way to make one specific window float/size/center
(`windowrulev2 = float, class:^(name)$`) prints a "deprecated, see wiki" notice but — confirmed
by checking `hyprctl clients -j` afterward — the rule is silently ignored; the window opens
tiled, at default size. The replacement unified `windowrule` keyword hits the exact same
unresolvable `invalid field type X` wall as `layerrule` (see Phase 10's blur note) — no way to
tell a wrong-syntax error from an unrecognized-field error through trial and error.

**What actually works:** inline bracket rules on the `exec` dispatcher itself, verified with `hyprctl dispatch exec` then checking `hyprctl clients -j` for the resulting window geometry:

```
bind = $mainMod, Escape, exec, [float;size 800 500;center] kitty --class btop-floating -e btop
```

## Rofi: mode-switcher + a lightweight control center

[#rofi-mode-switcher--a-lightweight-control-center](#rofi-mode-switcher--a-lightweight-control-center)

- `configuration.modi` now lists `drun,run,filebrowser,window`, and `mode-switcher` was added
to `mainbox`'s children — Rofi draws a row of mode buttons at the bottom, all built in, no
new packages.
- Three new scripts under `scripts/`, each a self-contained Rofi `-dmenu` menu instead of
installing a dedicated control-panel app:
  * `rofi-wifi.sh` — lists networks via `nmcli`, prompts for a password with `rofi -password` if the network is secured, connects.
  * `rofi-bluetooth.sh` — lists paired devices via `bluetoothctl`, connect/disconnect toggle,
plus a power on/off entry.
  * `rofi-audio.sh` — lists audio sinks by parsing `wpctl status` (piped through a small Python
regex — more reliable than awk/sed against `wpctl`'s tree-drawing output), switches default
sink, toggles mute.
- Wired into Waybar: click the network icon → `rofi-wifi.sh`; click the Bluetooth icon →
`rofi-bluetooth.sh`; left-click volume → mute toggle (unchanged), right-click volume →
`rofi-audio.sh`.

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

```
hyprctl reload
killall waybar && waybar &
```

Expected state:

- Waybar reads as distinct rounded pill groups, not one flat bar; an Arch logo sits at the far
left and opens Rofi when clicked.
- A music widget appears in the bar only when something is actually playing.
- Bluetooth has a status icon in the bar; clicking network/Bluetooth/right-clicking volume opens
a themed Rofi menu that actually changes the setting.
- Opening a new kitty window shows the Tokyo Night theme, not kitty's defaults.
- `$mainMod, Escape` opens btop floating and centered (once btop is installed).
- Rofi's launcher shows a row of mode buttons (apps/run/files/windows) at the bottom.

---

# Phase 12 — NVIDIA Hybrid GPU (new approach)

[#phase-12--nvidia-hybrid-gpu-new-approach](#phase-12--nvidia-hybrid-gpu-new-approach)

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

[#install-the-driver-stack](#install-the-driver-stack)

```
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

```
dkms status
```

Expected output (version numbers will vary):

```
nvidia-open/615.71.09, 7.2.8-arch1-2, x86_64: installed
```

If this comes back empty, headers are missing — install `linux-headers` and
run `sudo dkms autoinstall`.

## Load the modules early

[#load-the-modules-early](#load-the-modules-early)

Edit `/etc/mkinitcpio.conf` and set `MODULES` to:

```
MODULES=(amdgpu nvidia nvidia_modeset nvidia_uvm nvidia_drm)
```

Then regenerate the initramfs:

```
sudo mkinitcpio -P
```

## Module options

[#module-options](#module-options)

Create `/etc/modprobe.d/nvidia.conf`:

```
options nvidia NVreg_PreserveVideoMemoryAllocations=1 NVreg_TemporaryFilePath=/var/tmp
options nvidia_drm modeset=1 fbdev=1
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

[#stable-device-paths-via-udev](#stable-device-paths-via-udev)

`/dev/dri/card*` numbering is not stable across boots, and `AQ_DRM_DEVICES`
uses `:` as a separator so the raw `/dev/dri/by-path/pci-...` paths can't be
used directly either. Fix by creating named symlinks pinned to PCI addresses.

Confirm the PCI addresses on this machine:

```
lspci | grep -E 'VGA|3D'
```

Create `/etc/udev/rules.d/99-gpu-symlinks.rules` (adjust the PCI addresses to
match the output above):

```
KERNEL=="card*", KERNELS=="0000:05:00.0", SUBSYSTEM=="drm", SUBSYSTEMS=="pci", SYMLINK+="dri/amd-igpu"
KERNEL=="card*", KERNELS=="0000:01:00.0", SUBSYSTEM=="drm", SUBSYSTEMS=="pci", SYMLINK+="dri/nvidia-dgpu"
```

Reload and apply:

```
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Verify:

```
ls -l /dev/dri/ | grep -E 'amd-igpu|nvidia-dgpu'
```

Expected: two symlinks pointing at the correct `card*` for each GPU.

## Hyprland GPU selection via uwsm

[#hyprland-gpu-selection-via-uwsm](#hyprland-gpu-selection-via-uwsm)

This is the step the historical appendix got wrong. For uwsm-launched Hyprland
sessions (Phase 9), `AQ_*` variables must live in `~/.config/uwsm/env-hyprland`
— not in `hyprland.conf` and not in `~/.config/uwsm/env`. Setting them anywhere
else means they're silently absent from the Hyprland process environment at
runtime, which is exactly how the old setup failed.

```
mkdir -p ~/.config/uwsm
```

Create `~/.config/uwsm/env-hyprland`:

```
export AQ_DRM_DEVICES=/dev/dri/amd-igpu:/dev/dri/nvidia-dgpu
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

[#reboot-and-verify](#reboot-and-verify)

```
sudo reboot
```

At the SDDM login (Phase 9), select the **Hyprland (uwsm)** session, log in,
then from a kitty terminal:

```
printenv | grep AQ_DRM
```

Expected:

```
AQ_DRM_DEVICES=/dev/dri/amd-igpu:/dev/dri/nvidia-dgpu
```

If this comes back empty, verify the file lives at `~/.config/uwsm/env-hyprland`
exactly (common mistake: placing it in `~/.config/uwsm/env` instead, which
loads for every uwsm-managed session but not with the `AQ_*` scope Hyprland
reads at startup).

```
nvidia-smi
```

Should list the dGPU (model, driver version). Processes should be minimal —
Hyprland/Xwayland may appear with a few MiB of memory, but no significant
compute load.

```
hyprctl monitors
```

Both `eDP-1` (internal, AMD) and any connected external (`HDMI-A-1` or `DP-1`,
NVIDIA) should appear. If only `eDP-1` shows up with no external connected,
that's expected; plugging one in should surface it immediately via Hyprland's
hotplug.

```
glxinfo | grep "OpenGL renderer"
```

Should name the AMD GPU — this confirms ordinary apps are not landing on the
dGPU by accident. (If `glxinfo` is missing: `sudo pacman -S --needed mesa-utils`.)

## Per-app offload to the dGPU

[#per-app-offload-to-the-dgpu](#per-app-offload-to-the-dgpu)

For individual apps that should run on the NVIDIA GPU (games, Blender, GPU
compute), prefix with `prime-run`:

```
prime-run glxinfo | grep "OpenGL renderer"
```

Should name the NVIDIA RTX 3060. Same pattern for Steam game launch options:

```
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

[#validation-worth-repeating](#validation-worth-repeating)

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

[#critical-hardware-caveat](#critical-hardware-caveat)

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

[#useful-diagnostic-commands](#useful-diagnostic-commands)

```
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

# Appendix — NVIDIA Hybrid GPU (historical note)

[#appendix--nvidia-hybrid-gpu-historical-note](#appendix--nvidia-hybrid-gpu-historical-note)

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

# Next Phases

[#next-phases](#next-phases)

The remaining phases will be developed incrementally.

- Utility Scripts
- Boot menu (Visor, github.com/IO-ZetZor/Visor-BootManager) — deliberately deferred, not part
of the Phase 11 desktop pass. Unlike everything above, this replaces the bootloader itself
(compiles from source, writes to the EFI System Partition) — real risk of an unbootable
machine if misconfigured. Do this as its own phase, with a rescue USB on hand, only when
explicitly asked.
