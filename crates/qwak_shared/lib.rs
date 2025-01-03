#[qwak_macro::plugin]
pub trait QwakPlugin {
    fn plugin_name() -> String;
    fn plugin_version() -> [i32; 3];
}
