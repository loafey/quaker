use bevy::prelude::*;

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct Player {
    self_rot: f32,
    velocity: Vec3,
    hort_speed: f32,
    hort_max_speed: f32,
    hort_friction: f32,
    jump_height: f32,
    jump_timer: f32,
    gravity: f32,
    fall_lerp: f32,
    on_ground: bool,

    half_height: f32,
    radius: f32,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            self_rot: 0.0,
            velocity: Vec3::ZERO,
            hort_friction: 12.0,
            hort_speed: 0.05,
            hort_max_speed: 0.4,
            jump_height: 1.0,
            jump_timer: 0.0,
            gravity: -9.82 / 15.0,
            fall_lerp: 0.05,
            on_ground: false,
            half_height: 0.5,
            radius: 0.15,
        }
    }
}
