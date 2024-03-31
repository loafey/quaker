use bevy::{
    asset::{Assets, Handle},
    ecs::system::{Commands, ResMut, Resource},
};
use bevy_hanabi::EffectAsset;

mod demo;

#[derive(Resource)]
pub struct ParticleMap {
    pub demo: Handle<EffectAsset>,
}

pub fn register_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let map = ParticleMap {
        demo: demo::setup(&mut effects),
    };

    commands.insert_resource(map);
}
