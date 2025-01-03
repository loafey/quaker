use qwak_helper_types::MapInteraction;
use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);

#[extism_pdk::host_fn]
unsafe extern "ExtismHost" {
    unsafe fn debug_log(val: String);
    unsafe fn broadcast_message(val: String);
    unsafe fn get_player_name(id: u64) -> String;
}

// Simple QWAK plugin that contains the required functions.
// This is compiled to WASM.
struct Plugin;
impl QwakPlugin for Plugin {
    fn plugin_init() {}

    fn plugin_name() -> String {
        "Ondth".to_string()
    }

    fn plugin_version() -> [i32; 3] {
        [0, 0, 1]
    }

    fn map_interact(MapInteraction(arg, id): MapInteraction) {
        unsafe {
            match &*arg {
                "debug_log" => {
                    let name = get_player_name(id).unwrap();
                    let prefix = match &*name {
                        "Felony" => "cooler",
                        _ => "cool",
                    };
                    broadcast_message(format!("{name} is a {prefix} duck!")).unwrap()
                }
                _ => panic!("unknown interaction: {arg}"),
            }
        }
    }
}
