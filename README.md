# Soulless Launcher

A launcher built my way — the way System76 wanted your computer to be.

Soulless is a COSMIC desktop launcher that puts everything on your computer
at your fingertips. Find apps, files, CLI tools, and directories from one place.
Organize your tools into custom drawers. Secure sensitive files behind an
encrypted vault. Make it yours with custom backgrounds and keybinds.

This is the toolbox you need to organize the tools you use.

## Features

- **Universal search** — apps, files, directories, CLI tools, Steam games, Flatpaks, AppImages, Wine, JetBrains IDEs
- **Custom drawers** — organize apps, files, and directories your way, up to 5 drawers
- **Encrypted vault** — AES-256 encrypted storage for sensitive files
- **File organizer** — background watcher that detects misplaced files and suggests moves system-wide
- **Stable app IDs** — apps pinned to drawers survive renames and reinstalls via UUID registry
- **Live system monitors** — network, CPU, GPU, RAM, FPS, disk built in
- **Widget pages** — yellow/black dots under the toolbox flip the widget area between pages
- **Scratch terminal** — a page-sized command box for quick one-liners; run it, read it, your working terminal stays clean
- **Custom backgrounds** — drop any image into `~/.config/soulless/backgrounds/`
- **Instant startup** — disk cache means it opens instantly every time
- **Keyboard navigation** — Tab cycles drawers, arrow keys navigate grid, Enter launches, Ctrl+1-9 jumps to drawer
- **Super+Space** — open from anywhere via DBus activation
- **RON config** — `~/.config/soulless/config.ron` for monitors, organizer, icon size, theme
- **wgpu accelerated** — buttery smooth on COSMIC

## Requirements

- Pop!_OS or any Linux distro with the COSMIC desktop (the full experience, with the panel applet)
- Or any other Wayland compositor that supports wlr-layer-shell, as a keyboard-summoned launcher — tested on KDE Plasma and Bazzite. See [Other desktops](#other-desktops). GNOME is not supported.

## Install

A fresh Pop!_OS install ships without git, compilers, or the dev headers
this build needs, so install the prerequisites first:

```bash
sudo apt install -y git build-essential pkg-config libxkbcommon-dev rustup
rustup default stable
```

The panel applet lives in a git submodule, so clone with `--recurse-submodules`:

```bash
git clone --recurse-submodules https://github.com/hmrdsmoke/Soulless-Launcher
cd Soulless-Launcher
sudo make install
```

That one command builds the release binaries and installs everything —
binaries, desktop entries, icons, autostart, and the Super+Space shortcut.
(The build runs under sudo; if a later Rust project of your own hits
permission errors, `sudo chown -R $USER:$USER ~/.cargo` fixes it.)

Then open COSMIC Settings → Desktop → Panel (or Dock) → Applets and add **Soulless**.
**Reboot once after installing** — the autostart entry is read by systemd's xdg-autostart generator, which only re-scans at boot; logging out and back in is not enough. (Or start it right away for the current session: run `soulless-launcher`.) After that, Super+Space and the panel button both open Soulless.

Already cloned without submodules? Run `git submodule update --init` before building.

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

## Flatpak / COSMIC Store install

Installing from the COSMIC Store works fully out of the box via the panel
applet button. One thing the Flatpak **cannot** do is bind Super+Space for
you — sandboxed apps aren't permitted to write compositor keybindings
(that's the sandbox doing its job, not a bug). Adding it takes 30 seconds:

**COSMIC Settings → Keyboard → Keyboard Shortcuts → Custom Shortcuts:**

| Field    | Value                                                    |
|----------|----------------------------------------------------------|
| Name     | Soulless                                                 |
| Command  | `flatpak run com.github.hmrdsmoke.soulless-launcher toggle` |
| Shortcut | Super+Space                                              |

`toggle` shows the launcher if hidden and hides it if visible.

## Other desktops

Soulless does not need COSMIC. It needs a Wayland compositor that supports
wlr-layer-shell — tested on KDE Plasma and on Bazzite. GNOME is not supported
(no layer-shell). Off COSMIC there is no panel applet, so the window opens
centered and you summon it with a key.

**From source:** the Install steps above work unchanged. The `apt` line is
Pop!_OS-specific — on other distros install the equivalents (git, a C
toolchain, pkg-config, libxkbcommon development headers, rustup), then clone
with `--recurse-submodules` and `sudo make install`. The applet binary gets
built and installed but simply never runs, and the COSMIC Settings steps do
not apply.

**Prebuilt binary:** the [latest release](https://github.com/hmrdsmoke/Soulless-Launcher/releases/latest)
carries a launcher-only tarball. Verify the sha256 and follow `INSTALL.txt`
inside it.

Either way, bind a key to `soulless-launcher toggle`. The first press starts
the resident daemon and opens the window; every press after that toggles it.

Known limitation: CLI tool tiles open in cosmic-term, so install it or those
tiles will not launch.

## License

GPL-3.0-or-later — see [LICENSE](LICENSE) for full terms.