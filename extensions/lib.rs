use extism::{Manifest, Wasm};
use std::sync::{Arc, Mutex};
qwak_shared::plugin_calls!();

pub fn main() {
    let wasm = Wasm::file("target/wasm32-unknown-unknown/debug/default.wasm");
    let manifest = Manifest::new([wasm]);
    let plug = Arc::new(Mutex::new(
        extism::Plugin::new(&manifest, Vec::new(), true).unwrap(),
    ));
    println!("Huh: {:?}", calls::plugin_name(&plug));
}
