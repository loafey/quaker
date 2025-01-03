use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);
struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_name() -> String {
        "default plugin".to_string()
    }
}
