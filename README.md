# `dllmain-rs`
> A proc macro to generate dllmain

### Cargo.toml
```toml
dllmain-rs = { git = "https://github.com/zeon256/dllmain-rs" }
```

### Options
`#[dllmain_rs::entry]` accepts optional arguments:

- `events(process_attach, process_detach, thread_attach, thread_detach)`
- `panic = "abort" | "return_false"`

Defaults:

- `events(process_attach)`
- `panic = "abort"`

### Valid entry function signatures

- `fn name()`
- `fn name(reason: u32)`

Rejected signatures include methods, `async`, `const`, `unsafe`, generic functions,
variadics, non-`()` return types, and more than one argument.

### Example usage
```rust
#[dllmain_rs::entry]
fn on_process_attach() {
    // lightweight, loader-lock-safe setup
}

#[dllmain_rs::entry(events(process_attach, process_detach), panic = "return_false")]
fn on_lifecycle(reason: u32) {
    match reason {
        1 => {
            // process attach
        }
        0 => {
            // process detach
        }
        _ => {}
    }
}
```

### Safety notes

`DllMain` runs under the Windows loader lock. Keep logic minimal, avoid heavy I/O,
waiting on synchronization primitives, and complex re-entrancy during load/unload.
If your function panics, `panic = "abort"` terminates the process, while
`panic = "return_false"` returns `FALSE` from `DllMain`.

### Standalone example crates

See `examples/README.md` for end-to-end `cdylib` consumer crates:

- `examples/minimal-dll`
- `examples/lifecycle-dll`

### Release automation (trusted publishing)

This repository is configured for release-plz with trusted publishing.

- `release-plz` release job requests an OIDC token (`id-token: write`) and does
  not use `CARGO_REGISTRY_TOKEN`.
- Configure trusted publishing in crates.io for this repository and workflow
  before relying on automated publishing.
- First-time publish of a new crate still requires manual publish on crates.io.
