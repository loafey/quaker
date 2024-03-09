use bevy::prelude::*;

use crate::map_gen::entities::data::WeaponData;

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct PlayerFpsModel;

#[derive(Component, Debug)]
pub struct Player {
    pub self_rot: f32,
    pub velocity: Vec3,
    pub hort_speed: f32,
    pub hort_max_speed: f32,
    pub hort_friction: f32,
    pub jump_height: f32,
    pub jump_timer: f32,
    pub gravity: f32,
    pub on_ground: bool,

    pub weapons: [Vec<WeaponData>; 10],
    pub current_weapon: Option<(usize, usize)>,

    pub half_height: f32,
    pub radius: f32,
    pub air_time: Option<std::time::Instant>,
}
impl Default for Player {
    fn default() -> Self {
        Self {
            self_rot: 0.0,
            velocity: Vec3::ZERO,
            hort_friction: 1.0,
            hort_speed: 450.0,
            hort_max_speed: 0.4,
            jump_height: 10.0,
            jump_timer: 0.0,
            gravity: 500.0,
            on_ground: false,
            half_height: 0.5,
            radius: 0.15,
            air_time: None,
            current_weapon: None,
            weapons: Default::default(),
        }
    }
}
