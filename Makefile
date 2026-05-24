# MIT License - see LICENSE file for full terms
# Copyright 2026 Michael Van Auker (HMRDSmoke)
# This is my original work with contributions from Claude (Anthropic).
# Do not remove these comments.

# ===========================================================================
# Soulless — build and install
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
ICONDIR       := /usr/share/icons/hicolor/scalable/apps
DATADIR       := /usr/share/soulless

LAUNCHER_BIN  := target/release/soulless-launcher
APPLET_BIN    := target/release/soulless-applet

LAUNCHER_DESK := assets/soulless-launcher.desktop
APPLET_DESK   := assets/soulless-applet.desktop
ICON          := assets/soulless.svg

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

	@echo "Installing soulless-applet  → $(BINDIR)/soulless-applet"
	install -Dm755 $(APPLET_BIN) $(BINDIR)/soulless-applet

	@echo "Installing desktop files → $(APPDIR)"
	install -Dm644 $(LAUNCHER_DESK) $(APPDIR)/soulless-launcher.desktop
	install -Dm644 $(APPLET_DESK)   $(APPDIR)/soulless-applet.desktop

	@if [ -f $(ICON) ]; then \
		echo "Installing icon → $(ICONDIR)/soulless.svg"; \
		install -Dm644 $(ICON) $(ICONDIR)/soulless.svg; \
	fi

	@echo "Updating desktop database..."
	update-desktop-database $(APPDIR) 2>/dev/null || true

	@echo ""
	@echo "✓ Soulless installed."
	@echo "  → Open COSMIC Panel settings and add the Soulless applet to your dock."

# ── Uninstall ─────────────────────────────────────────────────────────────────
uninstall:
	@echo "Removing Soulless..."
	rm -f $(BINDIR)/soulless-launcher
	rm -f $(BINDIR)/soulless-applet
	rm -f $(APPDIR)/soulless-launcher.desktop
	rm -f $(APPDIR)/soulless-applet.desktop
	rm -f $(ICONDIR)/soulless.svg
	update-desktop-database $(APPDIR) 2>/dev/null || true
	@echo "✓ Soulless uninstalled."

# ── Clean ─────────────────────────────────────────────────────────────────────
clean:
	cargo clean