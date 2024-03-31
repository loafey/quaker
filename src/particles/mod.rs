use std::collections::HashMap;

use bevy::{
    asset::{Assets, Handle},
    ecs::system::{Commands, ResMut, Resource},
};
use bevy_hanabi::EffectAsset;

mod demo;

#[repr(u8)]
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub enum Particle {
    Demo,
}

#[derive(Resource)]
pub struct EffectMap(HashMap<Particle, Handle<EffectAsset>>);
impl std::ops::Deref for EffectMap {
    type Target = HashMap<Particle, Handle<EffectAsset>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub fn register_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let map = EffectMap(HashMap::from([(Particle::Demo, demo::setup(&mut effects))]));

    commands.insert_resource(map);
}
