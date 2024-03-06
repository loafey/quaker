use bevy::prelude::*;

mod debug;
mod spawn;
mod update;

#[derive(Resource)]
pub struct PlayerSpawnpoint(pub Vec3);

#[derive(Component, Debug)]
pub struct Player {
    self_rot: f32,
}
