use std::collections::HashMap;

use bevy::prelude::*;
use faststr::FastStr;

use crate::{entities::message::Message, map_gen::entities::data::WeaponData};

mod debug;
mod spawn;
mod update;

#[derive(Component, Debug)]
pub struct PlayerFpsModel;

#[derive(Debug, Component)]
pub struct PlayerController;

#[derive(Debug, Component)]
pub struct PlayerMpModel;

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
    mesh: Handle<Scene>,
    timer: f32,
    anim_time: f32,
    need_to_reload: bool,
    reload_timer: f32,
    pub data: WeaponData,
}

#[derive(Debug, Default)]
pub struct PlayerChildren {
    pub camera: Option<Entity>,
    pub fps_model: Option<Entity>,
    pub health_hud: Option<Entity>,
    pub armour_hud: Option<Entity>,
    pub ammo_hud: Option<Entity>,
    pub debug_hud: Option<Entity>,
    pub message_holder: Option<Entity>,
    pub shoot_sound_holder: Option<Entity>,
    pub lobby_hud: Option<Entity>,
}

#[derive(Debug, Default)]
pub struct DebugInfo {
    pub current_speed: f32,
    pub current_falling: f32,
    pub last_airtime: f32,
}

#[derive(Component, Debug, Default)]
pub struct PlayerFpsMaterial(Handle<StandardMaterial>);

#[derive(Component, Debug)]
pub struct Player {
    pub id: u64,
    pub last_hurter: u64,

    pub health: f32,
    pub armour: f32,

    velocity: Vec3,
    hort_speed: f32,
    hort_max_speed: f32,
    hort_friction: f32,
    jump_height: f32,
    jump_timer: f32,
    gravity: f32,
    on_ground: bool,

    camera_movement: CameraMovement,

    fps_anims: HashMap<FastStr, Handle<AnimationClip>>,

    pub children: PlayerChildren,

    pub weapons: [Vec<WeaponState>; 10],
    pub current_weapon: Option<(usize, usize)>,
    current_weapon_old: Option<(usize, usize)>,
    pub current_weapon_anim: FastStr,
    current_weapon_anim_old: FastStr,
    pub restart_anim: bool,

    half_height: f32,
    radius: f32,
    air_time: Option<std::time::Instant>,

    pub debug_info: DebugInfo,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            id: 0,
            last_hurter: 0,
            health: 100.0,
            armour: 100.0,
            velocity: Vec3::ZERO,
            hort_friction: 1.0,
            hort_speed: 4.5,
            hort_max_speed: 0.4,
            jump_height: 0.12,
            jump_timer: 0.0,
            gravity: 4.5,
            on_ground: false,
            half_height: 0.5,
            radius: 0.15,
            air_time: None,
            current_weapon: None,
            current_weapon_old: None,
            weapons: Default::default(),
            current_weapon_anim: Default::default(),
            current_weapon_anim_old: Default::default(),
            restart_anim: false,
            children: Default::default(),
            fps_anims: Default::default(),
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
            debug_info: Default::default(),
        }
    }
}
impl Player {
    pub fn add_weapon(&mut self, data: WeaponData, slot: usize, mesh: Handle<Scene>) -> bool {
        if !self.weapons[slot].iter().any(|c| c.data.id == data.id) {
            self.weapons[slot].push(WeaponState {
                need_to_reload: false,
                data,
                mesh,
                reload_timer: 0.0,
                timer: 0.0,
                anim_time: 0.0,
            });

            if self.current_weapon.is_none() {
                self.current_weapon = Some((slot, 0))
            }
            true
        } else {
            error!("unhandled: picked up weapon when already had one");
            false
        }
    }

    pub fn display_message(
        &self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        message: String,
    ) {
        if let Some(holder) = self.children.message_holder {
            let message_id = commands
                .spawn((
                    Text::new(message),
                    TextFont {
                        font: asset_server.load("ui/Color Basic.otf"),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                ))
                .insert(Message::default())
                .id();
            commands.entity(holder).add_child(message_id);
        } else {
            info!("Got message: {message}")
        }
    }
}

const HEALTH_GLYPH: &str = "+";
const ARMOR_GLYPH: &str = "Δ";
