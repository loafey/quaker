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
    pub commands: Commands<'w, 's>,
    pub players:
        Query<'w, 's, (Entity, &'static mut Player, &'static mut Transform), Without<Camera3d>>,
    pub cameras: Query<'w, 's, (&'static Camera3d, &'static mut Transform), Without<Player>>,
    pub asset_server: Res<'w, AssetServer>,
    pub weapon_map: Res<'w, WeaponMap>,
    pub audio: Res<'w, Audio>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub rapier_context: Res<'w, RapierContext>,
    pub game_entropy: ResMut<'w, Entropy<EGame>>,
    pub projectile_map: Res<'w, Projectiles>,
    pub time: Res<'w, Time>,
    pub current_id: Res<'w, CurrentClientId>,
}
