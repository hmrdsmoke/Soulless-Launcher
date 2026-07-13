# Flatpak / COSMIC Store status

**WORKING (2026-07-13).** The sandboxed launcher opens, indexes the host, and launches apps.

## Root cause of the original failure

cosmic-comp filters 21 privileged Wayland protocols from sandboxed clients —
`zwlr_layer_shell_v1` among them (see `docs/sandbox-withheld-protocols.txt`:
native sees 56 globals, sandboxed sees 35). The gate is `client_not_sandboxed()`
in cosmic-comp `src/state.rs`, which exempts exactly one identity: a client
bearing the `com.system76.CosmicPanel` security-context.

The launcher's `get_layer_surface` never reached the wire — there was no
layer-shell global to call it on. WAYLAND_DEBUG confirmed: 0 requests sent,
0 configures received.

**The old theory (vendored iced's window-action dispatch, `b007caba`) was wrong.**
libcosmic was bumped to `fc5debcc` — whose iced contains upstream's revert
`9f410558` — and the sandbox still failed identically. The bump was kept anyway:
zero API churn, sheds reverted-upstream debt.

## The fixes

1. **Layer shell.** `assets/soulless-applet.desktop` declares
   `X-HostWaylandDisplay=true`. cosmic-panel grants a privileged socket only to
   applets that request it, passing it as `X_PRIVILEGED_WAYLAND_SOCKET` (an
   inherited fd). The applet claims that fd in `main()` before libcosmic reads
   it, then on Activate — when nobody owns the D-Bus name — spawns the launcher
   with `WAYLAND_SOCKET=<fd>` and `WAYLAND_DISPLAY` removed. The launcher
   inherits the CosmicPanel security-context, sees all 56 globals, and maps its
   window.

2. **Indexing.** Flatpak refuses `--filesystem=/usr` (reserved). The sanctioned
   route is `--filesystem=host-os:ro`, mounting the host's `/usr` at
   `/run/host/usr`. `search/indexer/hostpath.rs` prefixes host paths when
   `FLATPAK_ID` is set; native gets an empty prefix and is unchanged. Two traps
   found along the way:
   - `/run/host/usr/bin` is full of symlinks whose targets don't resolve inside
     the sandbox. `is_file()` follows symlinks and returned false for all 2182
     host binaries — use `symlink_metadata()`.
   - The CLI quality gate ("does this tool have a man page?") read the sandbox's
     own `/usr/share/man`, which is the runtime's and has none. It silently
     dropped all 2764 indexed tools. It now reads the host's man tree.

3. **Launching.** `utils::spawn_exec()` routes every launch through
   `flatpak-spawn --host` when sandboxed (requires
   `--talk-name=org.freedesktop.Flatpak`). Without it the sandbox's `sh` can't
   find host binaries and every click did nothing. Native takes the plain
   `sh -c` branch.

## Verified

Sandboxed: `Index loaded: 1734 apps, 1653 cli, 1562 files` (native: 1788/1705/1558).
GUI apps launch. CLI tools open `--help` in a host terminal. Blur, theming, and
drawers all work.

## Notes for future work

- The vault's `xdg-open` path (`vault/mod.rs`) has not been routed through
  `flatpak-spawn` yet — vault file opening in-sandbox is untested.
- Monitor widgets shell out to `nvidia-smi`, `df`, `ping`, `dmidecode` — these
  binaries don't exist in the runtime and are unverified in-sandbox.
