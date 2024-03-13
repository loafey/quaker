use std::collections::HashMap;

use bevy::prelude::*;

use crate::map_gen::entities::data::WeaponData;

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct PlayerFpsModel;

#[derive(Debug)]
pub struct CameraMovement {
    backdrift: f32,
    backdrift_goal: f32,
    backdrift_max: f32,
    original_trans: Vec3,

    bob_goal: f32,
    bob_current: f32,

    cam_rot_max_goal: f32,
    cam_rot_goal: f32,
    cam_rot_current: f32,

    switch_offset: f32,
}

#[derive(Debug)]
pub struct WeaponState {
    mesh: Option<Handle<Scene>>,
    data: WeaponData,
}

#[derive(Debug, Default)]
pub struct PlayerChildren {
    camera: Option<Entity>,
    fps_model: Option<Entity>,
}

#[derive(Component, Debug, Default)]
pub struct PlayerFpsAnimations(HashMap<String, Handle<AnimationClip>>);
#[derive(Component, Debug, Default)]
pub struct PlayerFpsMaterial(Handle<StandardMaterial>);

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

    camera_movement: CameraMovement,

    children: PlayerChildren,

    weapons: [Vec<WeaponState>; 10],
    current_weapon: Option<(usize, usize)>,
    current_weapon_anim: String,

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
            current_weapon_anim: String::new(),
            children: Default::default(),
            camera_movement: CameraMovement {
                backdrift: 0.0,
                backdrift_goal: 0.0,
                backdrift_max: 0.02,
                original_trans: Vec3::ZERO,
                cam_rot_max_goal: 0.03,
                cam_rot_goal: 0.03,
                cam_rot_current: 0.0,

                bob_current: 0.0,
                bob_goal: 0.0,

                switch_offset: 0.0,
            },
        }
    }
}
impl Player {
    pub fn add_weapon(&mut self, data: WeaponData, slot: usize) -> bool {
        if !self.weapons[slot].iter().any(|c| c.data.id == data.id) {
            self.weapons[slot].push(WeaponState { data, mesh: None });

            if self.current_weapon.is_none() {
                self.current_weapon = Some((slot, 0))
            }

            println!(
                "Player inventory: [\n    {}\n]",
                self.weapons
                    .iter()
                    .map(|v| format!(
                        "[{}]",
                        v.iter()
                            .map(|w| w.data.id.clone())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .collect::<Vec<_>>()
                    .join(",\n    ")
            );
            true
        } else {
            warn!("unhandled: picked up weapon when already had one");
            false
        }
    }
}
