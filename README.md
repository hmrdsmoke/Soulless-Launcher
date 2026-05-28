# Soulless Launcher

A launcher built my way — the way System76 wanted your computer to be.

Soulless is a COSMIC desktop launcher that puts everything on your computer 
at your fingertips. Find apps, files, CLI tools, and directories from one place. 
Organize your tools into custom drawers. Secure sensitive files and apps behind 
an encrypted vault. Make it yours with custom backgrounds and keybinds.

This is the toolbox you need to organize the tools you use.

## Features

- **Universal search** — apps, files, directories, CLI tools, Steam games, Flatpaks, Snaps
- **Custom drawers** — organize your apps your way, up to 5 drawers
- **Encrypted vault** — secure storage for sensitive files and apps
- **Live system monitors** — network, CPU, GPU, RAM, FPS built in
- **Custom backgrounds** — drop any image into `~/.config/soulless/backgrounds/`
- **Instant startup** — disk cache means it opens instantly every time
- **Keyboard navigation** — Tab, S, V, arrow keys, Esc
- **Super+Space** — open from anywhere via DBus activation
- **wgpu accelerated** — buttery smooth on COSMIC

## Install

```bash
git clone https://github.com/hmrdsmoke/Soulless-Launcher
cd Soulless-Launcher
sudo make install
```

Then open COSMIC Panel settings and add the Soulless applet to your dock.

## Customization

See `launcher/src/config/README.md` for background and drawer customization.

## Requirements

- Pop!_OS with COSMIC desktop
- Rust toolchain

## License

MIT