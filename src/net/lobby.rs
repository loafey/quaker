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

type InnerLobby = BTreeMap<u64, PlayerInfo>;

#[derive(Debug, Resource, Default)]
pub struct Lobby {
    players: InnerLobby,
}
impl std::ops::Deref for Lobby {
    type Target = InnerLobby;

    fn deref(&self) -> &Self::Target {
        &self.players
    }
}
impl std::ops::DerefMut for Lobby {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.players
    }
}
impl<'a> std::iter::IntoIterator for &'a Lobby {
    type Item = (&'a u64, &'a PlayerInfo);

    type IntoIter = std::collections::btree_map::Iter<'a, u64, PlayerInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.players.iter()
    }
}
