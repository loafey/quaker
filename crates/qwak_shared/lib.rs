#![allow(clippy::unused_unit)]
#[qwak_macro::plugin]
pub trait QwakPlugin {
    fn plugin_init() -> ();
    fn plugin_name() -> String;
    fn plugin_version() -> [i32; 3];
    fn map_interact(arg: String) -> ();
}
