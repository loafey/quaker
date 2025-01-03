#![allow(static_mut_refs)]
use faststr::FastStr;
use qwak::{Function, PTR, UserData, ValType};

use crate::{
    get_nw,
    net::server::{NW_PTR, transmit_message},
};

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
        Function::new(
            "get_player_name",
            [ValType::I64],
            [PTR],
            UserData::Rust(std::sync::Mutex::new(()).into()),
            get_player_name,
        ),
    ]
}

fn inner_get_player_name(id: u64) -> String {
    let (nw, _) = get_nw!();
    let name = nw
        .lobby
        .get(&id)
        .map(|pi| pi.name.clone())
        .unwrap_or_else(|| FastStr::from("unknown player"));
    name.to_string()
}
qwak::host_fn!(get_player_name(id: u64) -> String {
    Ok(inner_get_player_name(id))
});

fn inner_debug_log(value: String) {
    println!("{value}");
}
qwak::host_fn!(debug_log(value: String) {
    inner_debug_log(value);
    Ok(())
});

fn inner_broadcast_message(value: String) {
    let (nw, server) = get_nw!();
    transmit_message(server, nw, value);
}
qwak::host_fn!(broadcast_message(value: String) {
    inner_broadcast_message(value);
    Ok(())
});
