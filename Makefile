# MIT License - see LICENSE file for full terms
# Copyright 2026 Michael Van Auker (HMRDSmoke)
# This is my original work with contributions from Claude (Anthropic).
# Do not remove these comments.
# ===========================================================================
# Soulless — build and install

# Preserve PATH and HOME when running under sudo.
# SUDO_USER is set by sudo to the original user; fall back to USER when not sudo.
SUDO_USER     ?= $(USER)
REAL_HOME     := $(shell getent passwd $(SUDO_USER) | cut -d: -f6)
export PATH   := $(REAL_HOME)/.cargo/bin:$(PATH)
export HOME   := $(REAL_HOME)
#
# make              → debug build
# make release      → optimized release build
# make install      → install both binaries + desktop files (requires sudo)
# make uninstall    → remove installed files
# make clean        → remove build artifacts
# ===========================================================================
PREFIX        ?= /usr/local
BINDIR        := $(PREFIX)/bin
APPDIR        := /usr/share/applications
ICONDIR_128   := /usr/share/icons/hicolor/128x128/apps
ICONDIR_16    := /usr/share/icons/hicolor/16x16/apps
ICONDIR_22    := /usr/share/icons/hicolor/22x22/apps
ICONDIR_24    := /usr/share/icons/hicolor/24x24/apps
LAUNCHER_BIN  := target/release/soulless-launcher
APPLET_BIN    := target/release/soulless-applet
LAUNCHER_DESK := assets/soulless-launcher.desktop
APPLET_DESK   := assets/soulless-applet.desktop
ICON          := assets/com.github.hmrdsmoke.soulless-applet.png
APPLET_ID     := com.github.hmrdsmoke.soulless-applet
SHORTCUT_DIR  := $(HOME)/.config/cosmic/com.system76.CosmicSettings.Shortcuts/v1

.PHONY: all release install uninstall clean

# ── Default: debug build ──────────────────────────────────────────────────────
all:
	cargo build

# ── Release build ─────────────────────────────────────────────────────────────
release:
	cargo build --release

# ── Install ───────────────────────────────────────────────────────────────────
install: release
	@echo "Installing soulless-launcher → $(BINDIR)/soulless-launcher"
	install -Dm755 $(LAUNCHER_BIN) $(BINDIR)/soulless-launcher
	@echo "Installing soulless-applet  → $(BINDIR)/$(APPLET_ID)"
	install -Dm755 $(APPLET_BIN) $(BINDIR)/$(APPLET_ID)
	@echo "Installing desktop files → $(APPDIR)"
	install -Dm644 $(LAUNCHER_DESK) $(APPDIR)/soulless-launcher.desktop
	install -Dm644 $(APPLET_DESK)   $(APPDIR)/$(APPLET_ID).desktop
	@echo "Installing icons..."
	install -Dm644 $(ICON) $(ICONDIR_128)/$(APPLET_ID).png
	install -Dm644 $(ICON) $(ICONDIR_16)/$(APPLET_ID).png
	install -Dm644 $(ICON) $(ICONDIR_22)/$(APPLET_ID).png
	install -Dm644 $(ICON) $(ICONDIR_24)/$(APPLET_ID).png
	@echo "Installing soulless-activate → $(BINDIR)/soulless-activate"
	printf '#!/bin/sh\n/usr/bin/dbus-send --session --print-reply --dest=com.github.hmrdsmoke.SoullessApplet /com/github/hmrdsmoke/SoullessApplet com.github.hmrdsmoke.SoullessApplet.Activate\n' > /tmp/soulless-activate
	install -Dm755 /tmp/soulless-activate $(BINDIR)/soulless-activate
	@echo "Installing COSMIC shortcut..."
	mkdir -p $(SHORTCUT_DIR)
	@if [ ! -f $(SHORTCUT_DIR)/custom ]; then 		printf '{\n    (\n        modifiers: [\n            Super,\n        ],\n        key: "space",\n        description: Some("Soulless Launcher"),\n    ): Spawn("soulless-activate"),\n}\n' > $(SHORTCUT_DIR)/custom; 		echo "  → Super+Space shortcut installed."; 	fi
	@echo "Updating icon cache and desktop database..."
	gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
	update-desktop-database $(APPDIR) 2>/dev/null || true
	@echo ""
	@echo "✓ Soulless installed."
	@echo "  → Open COSMIC Panel settings and add the Soulless applet to your dock."

# ── Uninstall ─────────────────────────────────────────────────────────────────
uninstall:
	@echo "Removing Soulless..."
	rm -f $(BINDIR)/soulless-launcher
	rm -f $(BINDIR)/soulless-activate
	rm -f $(BINDIR)/$(APPLET_ID)
	rm -f $(APPDIR)/soulless-launcher.desktop
	rm -f $(APPDIR)/$(APPLET_ID).desktop
	rm -f $(ICONDIR_128)/$(APPLET_ID).png
	rm -f $(ICONDIR_16)/$(APPLET_ID).png
	rm -f $(ICONDIR_22)/$(APPLET_ID).png
	rm -f $(ICONDIR_24)/$(APPLET_ID).png
	gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
	update-desktop-database $(APPDIR) 2>/dev/null || true
	@echo "✓ Soulless uninstalled."

# ── Clean ─────────────────────────────────────────────────────────────────────
clean:
	cargo clean