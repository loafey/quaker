use bevy::log::error;
use macros::error_return;
use std::sync::mpsc;
use steamworks::{Client, SingleClient};

pub fn try_steam() -> Option<(Client, SingleClient)> {
    let (s, r) = mpsc::channel();
    std::thread::spawn(move || {
        let a = error_return!(steamworks::Client::init_app(480));
        error_return!(s.send(a));
    });

    match r.recv() {
        Ok(r) => Some(r),
        Err(e) => {
            error!("{e}");
            error!("running without Steam");
            None
        }
    }
}
