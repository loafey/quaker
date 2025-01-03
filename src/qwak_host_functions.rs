use qwak::{Function, PTR, UserData};

use crate::net::server::NW_PTR;

pub fn qwak_functions() -> impl IntoIterator<Item = Function> {
    [
        Function::new(
            "debug_log",
            [PTR],
            [],
            UserData::Rust(std::sync::Mutex::new(()).into()),
            debug_log,
        ),
        Function::new(
            "broadcast_message",
            [PTR],
            [],
            UserData::Rust(std::sync::Mutex::new(()).into()),
            broadcast_message,
        ),
    ]
}
fn inner_debug_log(value: String) {
    println!("{value}");
}
qwak::host_fn!(debug_log(value: String) {
    inner_debug_log(value);
    Ok(())
});

fn inner_broadcast_message(value: String) {
    let nw = NW_PTR.read().unwrap();
    let nw = nw.as_ref().unwrap();
    println!("Broadcast message: {:?}", nw.current_id);
}
qwak::host_fn!(broadcast_message(value: String) {
    inner_broadcast_message(value);
    Ok(())
});
