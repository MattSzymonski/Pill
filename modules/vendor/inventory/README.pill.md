# Vendored `inventory` 0.3.24

`inventory` registers `#[component]`, `#[system]`, `#[resource]`, `#[asset]`
and value-type submissions through `.init_array` constructors. Upstream marks
the link section only for a fixed list of `target_os` values, and `espidf` is
not on it, so every constructor was dropped on the device: the image booted,
`inventory::iter` counted nothing, and the first system spawn failed with
"component ... not registered".

## The delta

One line in `src/lib.rs`, in the `.init_array` `cfg_attr` list, next to the
other ELF operating systems:

```rust
target_os = "espidf",
```

Nothing else differs from the published crate. The source is the normalized
`Cargo.toml` plus `src/` from `inventory-0.3.24.crate`; `Cargo.toml.orig` is
upstream's own manifest, kept as published.

## How it is wired

`modules/Cargo.toml` patches the crates.io copy:

```toml
[patch.crates-io]
inventory = { path = "vendor/inventory" }
```

A patch applies to every dependent in the workspace, host builds included. On
the host the added `target_os` is simply never true, so the expansion is
unchanged.

`pill_engine` names the version exactly (`inventory = "=0.3.24"`). A `[patch]`
entry only takes effect when its version is the one cargo resolves, and a
caret requirement would let the registry's newer 0.3.x win instead, leaving the
patch unused.

## When upstream fixes it

Delete the patch and this directory once `inventory` lists `espidf` itself.
The upstream change is a one-line addition to that same list.

## Updating

Copy a new release over this directory and re-apply the one line. The version
in `Cargo.toml` must match the release, because a `[patch]` entry only
substitutes a crate whose version satisfies the dependents' requirement.
