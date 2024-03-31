use std::collections::BTreeMap;

use bevy::ecs::{entity::Entity, system::Resource};

#[derive(Debug)]
pub struct PlayerInfo {
    pub entity: Entity,
    pub name: String,
    pub kills: u64,
    pub deaths: u64,
}
impl PlayerInfo {
    pub fn new(entity: Entity, name: String) -> Self {
        Self {
            entity,
            name,
            kills: 0,
            deaths: 0,
        }
    }
}

#[derive(Debug, Resource, Default)]
pub struct Lobby {
    pub players: BTreeMap<u64, PlayerInfo>,
}
