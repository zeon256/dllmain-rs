#[dllmain_rs::entry]
fn on_process_attach() {
    // Keep this minimal; DllMain runs under loader lock.
}
