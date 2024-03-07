use std::fs;

use bevy::{ecs::system::ResMut, log::warn};
use macros::error_return;
use serde::{Deserialize, Serialize};

use crate::PickupMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "pickup_type")]
pub enum PickupData {
    Weapon {
        classname: String,
        gives: String,
        pickup_model: String,
        pickup_material: String,
        texture_file: String,
        scale: f32,
    },
}
impl PickupData {
    pub fn classname(&self) -> &str {
        match self {
            PickupData::Weapon { classname, .. } => classname,
        }
    }
}

pub fn load_pickups(mut map: ResMut<PickupMap>) {
    warn!("Loading pickups...");
    let data = error_return!(fs::read_to_string("assets/pickups.json"));
    let parsed = error_return!(serde_json::from_str::<Vec<PickupData>>(&data));

    for item in parsed {
        map.0.insert(item.classname().to_string(), item);
    }

    warn!("Done loading pickups...");
}
