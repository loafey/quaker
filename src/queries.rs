use crate::{
    entities::pickup::PickupEntity,
    net::{CurrentClientId, Lobby},
    particles::ParticleMap,
    player::Player,
    resources::{
        entropy::{EGame, Entropy},
        projectiles::Projectiles,
        PlayerSpawnpoint, WeaponMap,
    },
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier3d::plugin::RapierContext;

#[allow(clippy::type_complexity)]
#[derive(SystemParam)]
pub struct NetWorld<'w, 's> {
    // Checked
    pub commands: Commands<'w, 's>,
    pub players:
        Query<'w, 's, (Entity, &'static mut Player, &'static mut Transform), Without<Camera3d>>,
    pub cameras: Query<'w, 's, (&'static Camera3d, &'static mut Transform), Without<Player>>,
    pub pickups_query: Query<
        'w,
        's,
        (&'static PickupEntity, &'static Transform),
        (Without<Player>, Without<Camera3d>),
    >,
    pub asset_server: Res<'w, AssetServer>,
    pub weapon_map: Res<'w, WeaponMap>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub rapier_context: Res<'w, RapierContext>,
    pub game_entropy: ResMut<'w, Entropy<EGame>>,
    pub projectile_map: Res<'w, Projectiles>,
    pub time: Res<'w, Time>,
    pub current_id: Res<'w, CurrentClientId>,
    pub player_spawn: Res<'w, PlayerSpawnpoint>,
    pub lobby: ResMut<'w, Lobby>,
    pub particles: Res<'w, ParticleMap>,
}
