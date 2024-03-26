use crate::{
    net::CurrentClientId,
    player::Player,
    resources::{
        entropy::{EGame, Entropy},
        projectiles::Projectiles,
        WeaponMap,
    },
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::Audio;
use bevy_rapier3d::plugin::RapierContext;

#[derive(SystemParam)]
pub struct NetWorld<'w, 's> {
    // Checked
    pub commands: Commands<'w, 's>,
    pub players:
        Query<'w, 's, (Entity, &'static mut Player, &'static mut Transform), Without<Camera3d>>,
    pub cameras: Query<'w, 's, (&'static Camera3d, &'static mut Transform), Without<Player>>,

    // checked
    pub asset_server: Res<'w, AssetServer>,

    // checked
    pub weapon_map: Res<'w, WeaponMap>,

    // checked
    pub audio: Res<'w, Audio>,

    // checked
    pub materials: ResMut<'w, Assets<StandardMaterial>>,

    // checked
    pub meshes: ResMut<'w, Assets<Mesh>>,

    // checked
    pub rapier_context: Res<'w, RapierContext>,

    // checked
    pub game_entropy: ResMut<'w, Entropy<EGame>>,

    // checked
    pub projectile_map: Res<'w, Projectiles>,

    // checked
    pub time: Res<'w, Time>,
    pub current_id: Res<'w, CurrentClientId>,
}
