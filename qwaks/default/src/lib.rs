use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);
struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_init() {}

    fn plugin_name() -> String {
        "Ondth".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }

    fn map_interact(arg: String) {
        match &*arg {
            "debug_log" => {
                println!("yo")
            }
            _ => panic!("unknown interaction: {arg}"),
        }
    }
}
