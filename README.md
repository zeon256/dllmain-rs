# `dllmain-rs`
> A proc macro to generate dllmain

### Cargo.toml
```toml
dllmain-rs = { git = "https://github.com/BudiNverse/dllmain-rs" }
winapi = { version = "0.3.9", features = ["minwindef"]}
```

### Example usage
```rust
use std::thread;
use winapi::shared::minwindef::{self, HINSTANCE, DWORD, LPVOID, BOOL};

#[dllmain_rs::entry]
fn real_entry() {
    // unsafe { consoleapi::AllocConsole(); }
    let base_addr = get_base_addr();
    println!("Found base address: {:#X?}", base_addr);
    thread::spawn(move || {
        // hack thread
    });
}
```