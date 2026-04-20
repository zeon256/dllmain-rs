# Examples

This directory contains standalone consumer crates that show realistic `cdylib`
usage with `dllmain-rs`.

## Layout

- `minimal-dll/`: default `#[dllmain_rs::entry]` with process-attach handling.
- `lifecycle-dll/`: explicit `events(...)` and `panic = "return_false"`.

## Build locally

From the repository root:

```bash
cargo check --manifest-path examples/minimal-dll/Cargo.toml
cargo check --manifest-path examples/lifecycle-dll/Cargo.toml
```

To build as a Windows DLL, use a Windows target toolchain, for example:

```bash
cargo build --manifest-path examples/minimal-dll/Cargo.toml --target x86_64-pc-windows-msvc
cargo build --manifest-path examples/lifecycle-dll/Cargo.toml --target x86_64-pc-windows-msvc
```
