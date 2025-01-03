use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);
struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_name() -> String {
        "Ondth default qwak".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }
}
