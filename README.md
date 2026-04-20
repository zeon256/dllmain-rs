# `dllmain-rs`

[![Crates.io](https://img.shields.io/crates/v/dllmain-rs)](https://crates.io/crates/dllmain-rs)
[![Crates.io Downloads](https://img.shields.io/crates/d/dllmain-rs)](https://crates.io/crates/dllmain-rs)
[![Docs.rs](https://img.shields.io/docsrs/dllmain-rs)](https://docs.rs/dllmain-rs)
[![License](https://img.shields.io/badge/license-Apache--2.0%2FMIT-blue)](#license)

> A proc macro to generate dllmain

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dllmain-rs = "0.1.0"
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

### License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
