# `dllmain-rs`
> A proc macro to generate dllmain

### Cargo.toml
```toml
dllmain-rs = { git = "https://github.com/BudiNverse/dllmain-rs" }
```

### Example usage
```rust
use dllmain_rs::dllmain;
use std::thread;

#[dllmain]
fn real_entry() {
    unsafe { consoleapi::AllocConsole(); }
    let base_addr = get_base_addr();
    println!("Found base address: {:#X?}", base_addr);
    thread::spawn(move || {
        // hack thread
    });
}
```
