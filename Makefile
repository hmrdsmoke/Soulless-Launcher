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
ICON_BASE     := /usr/share/icons/hicolor
ICON_SIZES    := 16 22 24 48 64 128 256
LAUNCHER_BIN  := target/release/soulless-launcher
APPLET_BIN    := target/release/soulless-applet
LAUNCHER_DESK := assets/com.github.hmrdsmoke.soulless-launcher.desktop
APPLET_DESK   := assets/soulless-applet.desktop
LAUNCHER_ID   := com.github.hmrdsmoke.soulless-launcher
APPLET_ID     := com.github.hmrdsmoke.soulless-applet
ICONSRC       := assets/icons/hicolor
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
	install -Dm644 $(LAUNCHER_DESK) $(APPDIR)/com.github.hmrdsmoke.soulless-launcher.desktop
	install -Dm644 $(APPLET_DESK)   $(APPDIR)/$(APPLET_ID).desktop
	@echo "Installing metainfo..."
	install -Dm644 assets/com.github.hmrdsmoke.soulless-launcher.metainfo.xml \
		/usr/share/metainfo/com.github.hmrdsmoke.soulless-launcher.metainfo.xml
	@echo "Installing icons (per-size, launcher + applet)..."
	@for sz in $(ICON_SIZES); do \
		src="$(ICONSRC)/$${sz}x$${sz}/apps/$(LAUNCHER_ID).png"; \
		dir="$(ICON_BASE)/$${sz}x$${sz}/apps"; \
		install -Dm644 "$$src" "$$dir/$(LAUNCHER_ID).png"; \
		install -Dm644 "$$src" "$$dir/$(APPLET_ID).png"; \
		echo "  → $${sz}x$${sz}"; \
	done
	@echo "Installing soulless-activate → $(BINDIR)/soulless-activate"
	printf '#!/bin/sh\n/usr/bin/busctl --user call com.github.hmrdsmoke.SoullessLauncher /com/github/hmrdsmoke/SoullessLauncher org.freedesktop.DbusActivation Activate "a{sv}" 0\n' > /tmp/soulless-activate
	install -Dm755 /tmp/soulless-activate $(BINDIR)/soulless-activate
	@echo "Installing COSMIC shortcut..."
	mkdir -p $(SHORTCUT_DIR)
	@if [ ! -f $(SHORTCUT_DIR)/custom ]; then \
		printf '{\n    (\n        modifiers: [\n            Super,\n        ],\n        key: "space",\n        description: Some("Soulless Launcher"),\n    ): Spawn("soulless-activate"),\n}\n' > $(SHORTCUT_DIR)/custom; \
		echo "  → Super+Space shortcut installed."; \
	fi
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
	rm -f $(APPDIR)/com.github.hmrdsmoke.soulless-launcher.desktop
	rm -f $(APPDIR)/$(APPLET_ID).desktop
	rm -f /usr/share/metainfo/com.github.hmrdsmoke.soulless-launcher.metainfo.xml
	@for sz in $(ICON_SIZES); do \
		rm -f "$(ICON_BASE)/$${sz}x$${sz}/apps/$(LAUNCHER_ID).png"; \
		rm -f "$(ICON_BASE)/$${sz}x$${sz}/apps/$(APPLET_ID).png"; \
	done
	gtk-update-icon-cache /usr/share/icons/hicolor 2>/dev/null || true
	update-desktop-database $(APPDIR) 2>/dev/null || true
	@echo "✓ Soulless uninstalled."

# ── Clean ─────────────────────────────────────────────────────────────────────
clean:
	cargo clean