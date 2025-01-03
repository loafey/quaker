use qwak_shared::QwakPlugin;
qwak_shared::plugin_gen!(Plugin);

#[extism_pdk::host_fn]
unsafe extern "ExtismHost" {
    unsafe fn debug_log(val: String);
    unsafe fn broadcast_message(val: String);
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

    fn map_interact(arg: String) {
        match &*arg {
            "debug_log" => unsafe { broadcast_message("ondth plugin whoo!!".to_string()).unwrap() },
            _ => panic!("unknown interaction: {arg}"),
        }
    }
}