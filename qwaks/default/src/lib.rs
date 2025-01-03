use qwak_helper_types::{MapInteraction, NetWorldPtr};
use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);

#[extism_pdk::host_fn]
unsafe extern "ExtismHost" {
    unsafe fn debug_log(val: String);
    unsafe fn broadcast_message(nw: NetWorldPtr, val: String);
}

struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_init() {}

    fn plugin_name() -> String {
        "Ondth".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }

    fn map_interact(MapInteraction(nw, arg): MapInteraction) {
        match &*arg {
            "debug_log" => unsafe {
                broadcast_message(nw, format!("ondth plugin whoo!! {nw:?}")).unwrap()
            },
            _ => panic!("unknown interaction: {arg}"),
        }
    }
}
