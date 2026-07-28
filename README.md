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

Add it to GitHub.

Test the connection.

```bash
ssh -T git@github.com
```

Clone the repository.

```bash
mkdir -p ~/Projects
cd ~/Projects

git clone git@github.com:<user>/hyprland-from-scratch.git
cd hyprland-from-scratch
```

Expected state:

- SSH access works.
- Git is configured.
- GitHub authentication works.
- Repository cloned locally.

---

# Phase 1 — Minimal Hyprland

Install only the required packages.

```bash
sudo pacman -S \
    hyprland \
    kitty \
    xdg-desktop-portal \
    xdg-desktop-portal-hyprland
```

Create the configuration directory.

```bash
mkdir -p ~/.config/hypr
```

Copy the default configuration.

```bash
cp /usr/share/hypr/hyprland.conf ~/.config/hypr/
```

Start Hyprland.

```bash
Hyprland
```

Verify:

```bash
echo $XDG_SESSION_TYPE
```

Expected output:

```text
wayland
```

Expected state:

- Hyprland starts successfully.
- Kitty opens.
- Basic keybinds work.
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
mkdir -p ~/Projects/hyprland-from-scratch/dotfiles

mv ~/.config/hypr \
   ~/Projects/hyprland-from-scratch/dotfiles/
```

Create a symbolic link.

```bash
ln -s \
~/Projects/hyprland-from-scratch/dotfiles/hypr \
~/.config/hypr
```

Verify.

```bash
ls -l ~/.config
```

Expected output:

```text
hypr -> ~/Projects/hyprland-from-scratch/dotfiles/hypr
```

Commit the current state.

```bash
git add .
git commit -m "Initial Hyprland setup"
```

Expected state:

- Hyprland works.
- Configuration is stored inside the repository.
- ~/.config only contains a symbolic link.
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
