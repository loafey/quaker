use qwak::{Function, PTR, UserData};

use crate::net::server::{NW_PTR, transmit_message};

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
    let mut nw = NW_PTR.write().unwrap();
    let (nw, server) = nw.take().unwrap();
    transmit_message(server, nw, value);
}
qwak::host_fn!(broadcast_message(value: String) {
    inner_broadcast_message(value);
    Ok(())
});
