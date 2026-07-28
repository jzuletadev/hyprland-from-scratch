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

# Next Phases

The remaining phases will be developed incrementally.

- Waybar
- Application Launcher
- Notifications
- Wallpapers
- Lock Screen
- Idle Management
- Theming
- SDDM (optional)
- Utility Scripts
