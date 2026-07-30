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

Boot the official Arch ISO and start the installer.

```bash
archinstall
```

Use the following configuration:

| Setting | Value |
|----------|-------|
| Locale | en_US.UTF-8 |
| Keyboard | us *(or latam if preferred)* |
| Mirrors | Automatic |
| Disk | Use entire disk |
| Bootloader | systemd-boot (default) |
| Swap | Default |
| Hostname | Your choice |
| Root Password | Configure |
| User Account | Create user + wheel |
| Profile | Minimal / None |
| Audio | PipeWire |
| Network | NetworkManager |
| Timezone | Your timezone |
| Additional packages | `git curl base-devel openssh` |

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

```bash
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

---

---

# Phase 3 — Waybar

Waybar is the status bar. Install empty/minimal first, confirm it renders, then add modules
one at a time.

```bash
sudo pacman -S waybar
mkdir -p ~/hyprland-from-scratch/dotfiles/waybar
```

Place `config.jsonc` (defines **which** modules appear and where — `modules-left`,
`modules-center`, `modules-right`) and `style.css` (defines **how** it looks — same selector
model as normal CSS) at `~/hyprland-from-scratch/dotfiles/waybar/`. Baseline: only
`hyprland/workspaces` on the left and `clock` centered — no network, battery, or tray yet.

Symlink:

```bash
ln -s ~/hyprland-from-scratch/dotfiles/waybar ~/.config/waybar
```

Test manually first:

```bash
waybar &
```

Once confirmed working, autostart it in `hyprland.conf`:

```bash
exec-once = waybar
```

`exec-once` (vs `bind ... exec`) runs once at session start only — using plain `exec` here
would spawn a new Waybar instance on every `hyprctl reload`.

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

```bash
bind = $mainMod, R, exec, rofi -show drun
```

Reload and test:

```bash
hyprctl reload
```

`SUPER+R` should open the launcher; typing an app name (e.g. "kitty") should find and launch it.

---

# Phase 5 — Notifications (dunst)

Two main options exist: **mako** (native Wayland, minimal, key=value config) or **dunst**
(more mature, more configurable — per-app rules, urgency levels, scripting, history). Dunst is
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

Place a `dunstrc` file at `~/hyprland-from-scratch/dotfiles/dunst/dunstrc` defining `[global]`
plus `[urgency_low]`, `[urgency_normal]`, and `[urgency_critical]` sections — the urgency
levels are dunst's key advantage over mako: `urgency_critical` can be set to `timeout = 0` so
critical notifications never auto-dismiss.

Symlink:

```bash
ln -s ~/hyprland-from-scratch/dotfiles/dunst ~/.config/dunst
```

Autostart in `hyprland.conf`:

```bash
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

```bash
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

If two-finger scroll feels inverted, it's controlled by `natural_scroll` in the `touchpad`
block of `hyprland.conf`. `true` = content follows finger direction (like mobile/macOS);
`false` = classic inverted-wheel behavior.

```bash
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
> separately via `input { natural_scroll = ... }` at the top level of the `input` block (outside
> `touchpad`), and is usually left `false` since inverted scroll feels unnatural on a mouse.

---

# Phase 6 — Wallpaper (awww)

> **Naming note:** the tool referenced across most community guides as `swww` was renamed by
> its creator to **awww** after the original `swww` project was archived. Arch's official
> repos now only ship `awww` — `pacman -S swww` resolves to the `awww` package automatically,
> but the binaries are `awww` and `awww-daemon`, not `swww`/`swww-daemon`. All commands below
> use the current names.

Install:

```bash
sudo pacman -S awww
```

Create the repo folders (this is the first real use of the `assets/` folder reserved from the
start of the repo structure):

```bash
mkdir -p ~/hyprland-from-scratch/dotfiles/awww
mkdir -p ~/hyprland-from-scratch/assets/wallpapers
```

Place a wallpaper image at `~/hyprland-from-scratch/assets/wallpapers/`. No image handy? Generate
a solid-color placeholder locally (no internet needed beyond the Arch mirrors already used for
pacman):

```bash
sudo pacman -S imagemagick
convert -size 1920x1080 xc:'#1e1e2e' ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

awww needs its daemon running before it can set a wallpaper:

```bash
awww-daemon &
awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

Test an animated transition (this is the actual reason to prefer this tool over hyprpaper):

```bash
awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg --transition-type wipe --transition-duration 1.5
```

> **How to actually see the transition:** the transition is a property of the `awww img`
> command itself, not a separate toggle — but it's only visible when switching *between two
> different images*. Running it with the same image that's already set technically still runs
> the transition, but with no visual difference between "before" and "after" there's nothing to
> see. Keep at least two test images in `assets/wallpapers/` and alternate between them:
> ```bash
> awww img ~/hyprland-from-scratch/assets/wallpapers/2.png --transition-type wipe --transition-duration 1.5
> awww img ~/hyprland-from-scratch/assets/wallpapers/3.jpeg --transition-type grow --transition-duration 1.5
> ```
> Some `--transition-type` values worth comparing: `simple` (fade), `wipe`, `wave`, `grow`,
> `outer`, `random` (picks one at random each time).

Autostart in `hyprland.conf`:

```bash
exec-once = awww-daemon
exec-once = awww img ~/hyprland-from-scratch/assets/wallpapers/wallpaper.jpg
```

Reload:

```bash
hyprctl reload
```

For a full test, restart the whole Hyprland session (not just reload) and confirm the
wallpaper appears on its own, without running any command by hand.

---

# Phase 7 — Lock Screen (hyprlock)

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
> of a separate `hyprlock.conf` file — the error will look like `config option
> <background:path> does not exist` in the Hyprland log. The file path in the error message is
> the giveaway: if it points at `hyprland.conf` instead of `hyprlock.conf`, the block landed in
> the wrong file. Cut it out and move it to its own file to fix it.

Add a bind in `hyprland.conf`:

```bash
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

hypridle watches for inactivity and fires staged actions — dim, lock, screen off, suspend —
integrating directly with hyprlock from Phase 7.

```bash
sudo pacman -S hypridle
```

Baseline `hypridle.conf` structure:
- `general` block — `lock_cmd` (what locks the session), `before_sleep_cmd`/`after_sleep_cmd`
  (run right before/after suspend).
- One `listener { }` block per stage, each independent: its own `timeout` (seconds) and
  `on-timeout` action, with `on-resume` reverting it once activity resumes. Baseline staging:
  dim at 150s, lock at 300s, screen off at 330s, suspend at 900s.

Place the file alongside the others (no new symlink needed) and autostart it:

```bash
exec-once = hypridle
```

```bash
hyprctl reload
```

> **Testing tip:** minute-long timeouts are slow to verify. Temporarily lower every `timeout`
> to a handful of seconds, restart the daemon (`pkill hypridle && hypridle &`), confirm each
> stage fires in order, then restore the real values (150/300/330/900) once confirmed.

---

# Phase 9 — SDDM (login screen)

SDDM provides a traditional graphical login (username + password) instead of TTY autologin,
and — as a side benefit — launches Hyprland correctly on its own, resolving the earlier
"launched without start-hyprland" warning without needing `dbus-run-session Hyprland` by hand.

Install:

```bash
sudo pacman -S sddm
sudo systemctl enable sddm
```

Confirm Hyprland exposes its session file (installed automatically by the `hyprland` package):

```bash
ls /usr/share/wayland-sessions/
```

You should see both `hyprland.desktop` and `hyprland-uwsm.desktop` — prefer the **uwsm**
variant in SDDM's session picker. `uwsm` (Universal Wayland Session Manager) is the modern
recommended launch method: it handles D-Bus, environment variables, and session lifecycle
correctly, replacing both the old `start-hyprland` script and a manual
`dbus-run-session Hyprland` invocation.

> **Before rebooting**, make sure no TTY autologin override is left configured — it can
> conflict with SDDM taking over the login:
> ```bash
> cat /etc/systemd/system/getty@tty1.service.d/override.conf 2>/dev/null
> ```
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

Everything up to now used ad hoc colors picked per file (a stray Nord cyan border here, a
Catppuccin-ish background there). This phase formalizes one palette and applies it consistently
across every piece already built: Hyprland borders/blur, Waybar, Rofi, dunst, and hyprlock.
Nothing new gets installed — this is config only.

**Palette (Tokyo Night):**

| Role | Hex |
|------|-----|
| Background | `#1a1b26` |
| Surface (panels, input fields) | `#24283b` |
| Highlight (hover/selected) | `#292e42` |
| Foreground | `#c0caf5` |
| Muted foreground | `#565f89` |
| Accent — blue | `#7aa2f7` |
| Accent — purple | `#bb9af7` |
| Warning | `#e0af68` |
| Critical | `#f7768e` |

## Hyprland: borders, blur, rounding

`col.active_border` becomes a blue-to-purple gradient, `col.inactive_border` a translucent
dark surface color, corner `rounding` goes from 6 to 10, and window blur gets enabled (small
`size`/`passes` — this machine's internal panel is driven by the AMD iGPU, not the NVIDIA GPU,
see the NVIDIA appendix below, so a light blur has headroom to spare). `active_opacity` /
`inactive_opacity` add a subtle transparency to unfocused windows.

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

> **Bug found while verifying this phase:** the files had been named `Config.jsonc` /
> `Style.css` (capital first letter) since Phase 3. Waybar's config search path only ever
> checks lowercase `config`/`config.jsonc` and `style.css` — confirmed with
> `waybar --log-level debug`, which showed `Found config file: /etc/xdg/waybar/config.jsonc`.
> The capitalized filenames meant every "Phase 3 baseline"-onward screenshot/description was
> silently describing a config that was never actually loaded; Waybar had been falling back to
> the distro's example config (cpu/memory/temperature/pulseaudio/tray and more) the whole time.
> Renamed to `config.jsonc` / `style.css` — verify with the same debug flag after any future
> Waybar change:
> ```bash
> waybar --log-level debug 2>&1 | grep "Found config file"
> ```

## Rofi, dunst, hyprlock

Same palette swapped into `rofi/config.rasi`, `dunst/dunstrc`, and `hyprlock.conf` — background,
borders/frame, text, and the critical-urgency/red accent all point at the same hex values as
Waybar and Hyprland now.

## Media keys

Since Waybar now shows volume, the media keys to actually change it were missing. Added to
`hyprland.conf`, next to the existing brightness keys:

```
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
  Hyprland community configs, but on this install (0.56.1) `hyprctl keyword layerrule "blur,
  waybar"` reliably returns `invalid field blur: missing a value`, and trying variants
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

```bash
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
every module "having" an icon. Fix: generate the file with a small `python3` script using
`\uXXXX` escapes instead of the literal character, e.g.:

```python
icon = ""  # Arch logo, nf-linux-archlinux
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
  Bluetooth picker script below.
- Full module list now: `custom/arch`, `hyprland/workspaces`, `mpris` (left) — `hyprland/window`
  (center) — one `group/status` pill (idle_inhibitor, backlight, network, wireplumber,
  custom/bluetooth, battery), tray, clock (right).

## kitty — first real theme since Phase 1

`dotfiles/kitty/kitty.conf` (new file, new `~/.config/kitty` symlink — it was an empty directory
before this). Tokyo Night ANSI palette, JetBrainsMono Nerd Font, subtle `background_opacity`.
Window rounding/blur come from Hyprland's compositor-level `decoration` block, not kitty itself.

## fastfetch and btop — not runnable yet

Both configs are written and ready but **need packages this session couldn't install**
(`sudo pacman -S fastfetch btop` requires an interactive sudo password, which an agent session
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

The documented way to make one specific window float/size/center
(`windowrulev2 = float, class:^(name)$`) prints a "deprecated, see wiki" notice but — confirmed
by checking `hyprctl clients -j` afterward — the rule is silently ignored; the window opens
tiled, at default size. The replacement unified `windowrule` keyword hits the exact same
unresolvable `invalid field type X` wall as `layerrule` (see Phase 10's blur note) — no way to
tell a wrong-syntax error from an unrecognized-field error through trial and error.

**What actually works:** inline bracket rules on the `exec` dispatcher itself, verified with
`hyprctl dispatch exec` then checking `hyprctl clients -j` for the resulting window geometry:

```
bind = $mainMod, Escape, exec, [float;size 800 500;center] kitty --class btop-floating -e btop
```

## Rofi: mode-switcher + a lightweight control center

- `configuration.modi` now lists `drun,run,filebrowser,window`, and `mode-switcher` was added
  to `mainbox`'s children — Rofi draws a row of mode buttons at the bottom, all built in, no
  new packages.
- Three new scripts under `scripts/`, each a self-contained Rofi `-dmenu` menu instead of
  installing a dedicated control-panel app:
  - `rofi-wifi.sh` — lists networks via `nmcli`, prompts for a password with `rofi -password`
    if the network is secured, connects.
  - `rofi-bluetooth.sh` — lists paired devices via `bluetoothctl`, connect/disconnect toggle,
    plus a power on/off entry.
  - `rofi-audio.sh` — lists audio sinks by parsing `wpctl status` (piped through a small Python
    regex — more reliable than awk/sed against `wpctl`'s tree-drawing output), switches default
    sink, toggles mute.
- Wired into Waybar: click the network icon → `rofi-wifi.sh`; click the Bluetooth icon →
  `rofi-bluetooth.sh`; left-click volume → mute toggle (unchanged), right-click volume →
  `rofi-audio.sh`.

> **Bug found after this section first shipped:** every app row rendered as a white card with
> a drop shadow — theme-looking-unapplied, even though `rofi -show drun -dump-theme` echoed
> back all the correct dark colors. Root cause: on Rofi 2.0.0, `element-icon` and
> `element-text` paint an opaque white background by default and don't inherit transparency
> from the parent `element {}`. Fixed by giving both an explicit
> `background-color: transparent;` in `config.rasi` (confirmed live, screenshot-compared
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

# Appendix — NVIDIA Hybrid GPU (RTX 3060 + AMD) troubleshooting

This section documents a specific hardware issue encountered on the Dell G15 Ryzen Edition
build (RTX 3060 laptop GPU + AMD iGPU, hybrid graphics). Kept here so a reinstall doesn't have
to rediscover this from scratch.

## Context

- Arch discontinued the closed `nvidia-dkms` driver for Ampere-and-newer GPUs (Dec 2025).
  The RTX 3060 can only use `nvidia-open-dkms` via official repos — there is no way back to
  the closed driver through pacman.
- With this open driver on a hybrid NVIDIA+AMD laptop, there is a known, still-open community
  bug (tracked upstream, same symptom reported against Hyprland on hybrid NVIDIA setups): the
  NVIDIA GPU can fail to re-initialize cleanly on reboot, intermittently. No confirmed official
  fix exists — the mitigation below reduced/eliminated the symptom in testing on this machine,
  but is not a guaranteed permanent fix.

## The fix that worked

In `/etc/modprobe.d/` (create a file, e.g. `nvidia.conf`), set:

```
options nvidia NVreg_PreserveVideoMemoryAllocations=1 NVreg_TemporaryFilePath=/var/tmp NVreg_EnableGpuFirmware=0
```

Rebuild the initramfs after adding this (required for modprobe.d changes to take effect at
boot):

```bash
sudo mkinitcpio -P
```

**Validated with:**
- Multiple consecutive cold reboots — no NVIDIA error in `journalctl -b -1 | grep -i nvidia`.
- Reboot immediately after real GPU load (3x `glmark2` runs via PRIME offload, scores
  ~1500-1560, GPU at 45% util / 65°C) — still no error, confirming the fix holds under load,
  not just at idle.

Since the underlying bug is intermittent per community reports, don't trust a single clean
reboot — repeat 2-3 times, including at least one after real GPU load, before considering it
resolved on a fresh install.

## Separate, unresolved side-note

`nvidia_drm.modeset=0` is present on this system's kernel command line, which forces AMD to
handle display/KMS instead of NVIDIA — useful since the laptop's internal panel (eDP-1) is
wired to the AMD GPU, not the NVIDIA one. An earlier attempt to force this explicitly via
`AQ_DRM_DEVICES=/dev/dri/card1` (through uwsm env / environment.d) did not take effect
(`printenv` came back empty after reboot) — but the kernel parameter above appears to already
achieve the same outcome through a different mechanism, so this may be a non-issue in practice.
Worth confirming explicitly on a fresh install rather than assuming.

## Critical hardware caveat — never load-test the GPU on battery

On this specific model (Dell G15 Ryzen Edition, which shares its motherboard/platform with the
Alienware line — hence `alienware_wmi` kernel modules loading despite the G15 badging), running
sustained CPU+GPU load (e.g. a benchmark, a game) **on battery power** caused an instant, total
power cutoff — screen and all fan/system noise stopped in the same instant, no crash/freeze
behavior, and no trace at all in `journalctl` for that boot (consistent with a hardware/firmware-level
power protection cutting supply before the kernel has any chance to log it, not a software
crash). This is unrelated to the NVIDIA driver bug above.

**Always connect the charger before running any GPU benchmark, game, or sustained load test.**

## Useful diagnostic commands (kept for reference)

```bash
# confirm which GPU a process is actually using
nvidia-smi

# force a command onto the NVIDIA GPU (see scripts/nvidia-run in this repo)
__NVIDIA_PRIME_RENDER_OFFLOAD=1 __GLX_VENDOR_LIBRARY_NAME=nvidia __VK_LAYER_NV_optimus=NVIDIA_only <command>

# check the previous boot's log (the one that just ended) for NVIDIA-related errors
journalctl -b -1 | grep -i nvidia

# list all recorded boots with start/end timestamps
journalctl --list-boots

# check for thermal/power/panic events in the previous boot
journalctl -b -1 | grep -i -E "thermal|temperature|power|panic|shutdown|emergency"

# lightweight, no-download GPU stress test
sudo pacman -S glmark2
```

---

# Next Phases

The remaining phases will be developed incrementally.

- Utility Scripts
- Boot menu (Visor, github.com/IO-ZetZor/Visor-BootManager) — deliberately deferred, not part
  of the Phase 11 desktop pass. Unlike everything above, this replaces the bootloader itself
  (compiles from source, writes to the EFI System Partition) — real risk of an unbootable
  machine if misconfigured. Do this as its own phase, with a rescue USB on hand, only when
  explicitly asked.