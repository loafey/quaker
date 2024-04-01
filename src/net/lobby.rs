use bevy::ecs::{entity::Entity, system::Resource};
use faststr::FastStr;
use std::collections::BTreeMap;

type K = u64;
type V = PlayerInfo;
type InnerLobby = BTreeMap<K, V>;

#[derive(Debug)]
pub struct PlayerInfo {
    pub entity: Entity,
    pub name: FastStr,
    pub kills: u64,
    pub deaths: u64,
}
impl PlayerInfo {
    pub fn new(entity: Entity, name: FastStr) -> Self {
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
    type Item = (&'a K, &'a V);

    type IntoIter = std::collections::btree_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.players.iter()
    }
}
