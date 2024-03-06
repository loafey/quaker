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
    on_ground: bool,

    half_height: f32,
    radius: f32,
    air_time: Option<std::time::Instant>,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            self_rot: 0.0,
            velocity: Vec3::ZERO,
            hort_friction: 1.0,
            hort_speed: 4.0,
            hort_max_speed: 0.4,
            jump_height: 10.0,
            jump_timer: 0.0,
            gravity: 350.0,
            on_ground: false,
            half_height: 0.5,
            radius: 0.15,
            air_time: None,
        }
    }
}
