# Flatpak / COSMIC Store status

**SOLVED (2026-07-13): the sandboxed launcher opens.**

## Root cause (the real one)

cosmic-comp filters 21 privileged Wayland protocols from sandboxed clients —
`zwlr_layer_shell_v1` among them (see `docs/sandbox-withheld-protocols.txt`;
native sees 56 globals, sandboxed sees 35). The gate is `client_not_sandboxed()`
in cosmic-comp `src/state.rs`, which exempts exactly one identity: a client
bearing the `com.system76.CosmicPanel` security-context.

So the launcher's `get_layer_surface` never reached the wire — there was no
layer-shell global to call it on. WAYLAND_DEBUG confirmed: 0 requests sent,
0 configures received.

**The old theory was wrong.** This was never the vendored iced's window-action
dispatch (`b007caba`). libcosmic was bumped to `fc5debcc` (whose iced contains
upstream's revert `9f410558`) and the sandbox still failed identically — which
is what proved the dispatch theory dead. The bump was still worth keeping:
zero API churn, sheds reverted-upstream debt.

## The fix

1. `assets/soulless-applet.desktop` declares `X-HostWaylandDisplay=true`.
   cosmic-panel grants a privileged socket ONLY to applets whose desktop entry
   requests it, passing it as `X_PRIVILEGED_WAYLAND_SOCKET` (an inherited fd).
2. The applet claims that fd in `main()` before libcosmic reads it, then on
   Activate — when nobody owns the D-Bus name — spawns the launcher with
   `WAYLAND_SOCKET=<fd>` and `WAYLAND_DISPLAY` removed.
3. The launcher connects through the pre-connected privileged socket, inherits
   the CosmicPanel security-context, sees all 56 globals including layer-shell,
   and maps its window.

Launcher code required ZERO changes.

## Verified

`[applet] spawn: launched /app/bin/soulless-launcher pid=192 on privileged fd 72`
→ `[launcher] SurfaceConfigured 2560x1440 -> blur rect` → window on screen.
Spawn guard holds: one launcher, subsequent clicks route via D-Bus.

## Open (next front)

- Sandbox indexes only 28 apps (native: 1788). Flatpak refuses `/usr` filesystem
  args as reserved — the app index needs a permitted route to host desktop entries.
- Icon cache prewarms 0 entries in-sandbox (same cause).
- cosmic_config watcher errors in-sandbox (theme/tk config not reachable).
