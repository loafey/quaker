use qwak_helper_types::MapInteraction;
use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);

#[extism_pdk::host_fn]
unsafe extern "ExtismHost" {
    unsafe fn debug_log(val: String);
    unsafe fn broadcast_message(val: String);
    unsafe fn get_player_name(id: u64) -> String;
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

    fn map_interact(MapInteraction(arg, id): MapInteraction) {
        match &*arg {
            "debug_log" => unsafe {
                let name = get_player_name(id).unwrap();
                broadcast_message(format!("{name} is a crazy duck!")).unwrap()
            },
            _ => panic!("unknown interaction: {arg}"),
        }
    }
}
