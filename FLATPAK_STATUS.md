# Flatpak build status

Builds **cleanly and runs locally**: `cargo build --release` succeeds in ~5 min.

## Blocker (sandboxed/offline build only)
Cannot complete an offline flatpak-builder build. `cosmic-panel-config`
(transitive, via cosmic-panel @ 6119bb1) references libcosmic's `cosmic-config`
crate by a BARE git source (`git+https://github.com/pop-os/libcosmic`, no rev).
In `cargo --offline` mode Cargo refuses to resolve that bare reference to the
vendored copy, even though libcosmic IS vendored at the pinned commit:

    error: failed to get `cosmic-config` as a dependency of package
      `cosmic-panel-config` ... unable to update
      https://github.com/pop-os/libcosmic#4657b6a ... can't checkout ...
      you are in the offline mode (--offline)

libcosmic is pinned to 4657b6a (the commit cosmic-panel @ 6119bb1 expects).
`cargo-sources.json` is generated via flatpak-cargo-generator.py from Cargo.lock.

## Suspected root cause
Pinned pop-os crate commits are not a coordinated/tested-together set
(e.g. libcosmic master expects cosmic-protocols 178eb0b; we have c253ec1).

## Next steps to try
- Ask flatpak/flatpak-builder-tools issues, or the cosmic community
  (cosmic-utils), re: offline vendoring of a bare transitive libcosmic ref.
- OR align all cosmic crate revs to a coordinated set from a shipping
  cosmic flatpak (e.g. cosmic-ext-classic-menu) or a cosmic-epoch release.
