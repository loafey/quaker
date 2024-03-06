use bevy::prelude::*;

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct Player {
    self_rot: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self { self_rot: 0.0 }
    }
}
