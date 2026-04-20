#[dllmain_rs::entry(events(process_attach, process_detach), panic = "return_false")]
fn on_lifecycle(reason: u32) {
    match reason {
        1 => {
            // PROCESS_ATTACH
        }
        0 => {
            // PROCESS_DETACH
        }
        _ => {}
    }
}
