use std::fs;

use bevy::log::warn;
use macros::error_return;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(tag = "pickup_type")]
pub enum PickupData {
    Weapon {
        classname: String,
        gives: String,
        pickup_model: String,
        pickup_material: String,
    },
}

pub fn load_pickups() {
    warn!("Loading pickups...");
    let data = error_return!(fs::read_to_string("assets/pickups.json"));
    warn!("Done loading pickups...");
}
