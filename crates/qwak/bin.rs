fn main() {
    let mut plugin =
        qwak::QwakPlugin::new("target/wasm32-unknown-unknown/debug/default.wasm").unwrap();

    println!("{:?}", plugin.plugin_name());
    println!("Running version: {:?}", plugin.plugin_version());
}
