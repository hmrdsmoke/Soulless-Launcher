# Soulless Launcher

A launcher built my way — the way System76 wanted your computer to be.

Soulless is a COSMIC desktop launcher that puts everything on your computer
at your fingertips. Find apps, files, CLI tools, and directories from one place.
Organize your tools into custom drawers. Secure sensitive files behind an
encrypted vault. Make it yours with custom backgrounds and keybinds.

This is the toolbox you need to organize the tools you use.

## Features

- **Universal search** — apps, files, directories, CLI tools, Steam games, Flatpaks, AppImages, Wine, JetBrains IDEs
- **Custom drawers** — organize apps, files, and directories your way, up to 5 drawers with drag and drop
- **Encrypted vault** — AES-256 encrypted storage for sensitive files
- **File organizer** — background watcher that detects misplaced files and suggests moves system-wide
- **Stable app IDs** — apps pinned to drawers survive renames and reinstalls via UUID registry
- **Live system monitors** — network, CPU, GPU, RAM, FPS, disk built in
- **Custom backgrounds** — drop any image into `~/.config/soulless/backgrounds/`
- **Instant startup** — disk cache means it opens instantly every time
- **Keyboard navigation** — Tab cycles drawers, arrow keys navigate grid, Enter launches, Ctrl+1-9 jumps to drawer
- **Super+Space** — open from anywhere via DBus activation
- **RON config** — `~/.config/soulless/config.ron` for monitors, organizer, icon size, theme
- **wgpu accelerated** — buttery smooth on COSMIC

## Requirements

- Pop!_OS or any Linux distro with the COSMIC desktop
- Rust toolchain
- Git

## Install

The panel applet lives in a git submodule, so clone with `--recurse-submodules`:

```bash
git clone --recurse-submodules https://github.com/hmrdsmoke/Soulless-Launcher
cd Soulless-Launcher
sudo make install
```

Then open COSMIC Settings → Desktop → Panel (or Dock) → Applets and add **Soulless**.
Super+Space works out of the box.

Already cloned without submodules? Run `git submodule update --init` before `make install`.

To remove everything:

```bash
sudo make uninstall
```

## Config

`~/.config/soulless/config.ron`:
```ron
(
    show_system_monitor: true,
    search_file_depth: 2,
    organizer_enabled: true,
    organizer_watch_dirs: [],
    drawer_icon_size: 64.0,
    theme_variant: Chrome,
)
```

Custom backgrounds: drop any `.jpg`, `.png`, or `.webp` into `~/.config/soulless/backgrounds/`

## License

GPL-3.0-or-later — see [LICENSE](LICENSE) for full terms.