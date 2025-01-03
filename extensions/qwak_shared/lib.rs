#[qwak_macro::plugin]
pub trait QwakPlugin {
    fn plugin_name() -> String {
        "default plugin".to_string()
    }
}
