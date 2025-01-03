use crate::{
    entities::pickup::PickupEntity,
    map_gen::Interactable,
    net::{CurrentClientId, Lobby},
    particles::ParticleMap,
    player::Player,
    plugins::Qwaks,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_rapier3d::plugin::RapierContext;
use resources::{
    PlayerSpawnpoint, WeaponMap,
    data::Projectiles,
    entropy::{EGame, Entropy},
};

#[allow(clippy::type_complexity, unused)]
#[derive(SystemParam)]
pub struct NetWorld<'w, 's> {
    // Checked
    pub commands: Commands<'w, 's>,
    pub players:
        Query<'w, 's, (Entity, &'static mut Player, &'static mut Transform), Without<Camera3d>>,
    pub cameras: Query<'w, 's, (&'static Camera3d, &'static mut Transform), Without<Player>>,
    pub interactables: Query<'w, 's, (Entity, &'static Interactable)>,
    pub pickups_query: Query<
        'w,
        's,
        (&'static PickupEntity, &'static Transform),
        (Without<Player>, Without<Camera3d>),
    >,
    pub rapier_context: Query<'w, 's, &'static RapierContext>,
    pub asset_server: Res<'w, AssetServer>,
    pub weapon_map: Res<'w, WeaponMap>,
    pub materials: ResMut<'w, Assets<StandardMaterial>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub game_entropy: ResMut<'w, Entropy<EGame>>,
    pub projectile_map: Res<'w, Projectiles>,
    pub time: Res<'w, Time>,
    pub current_id: Res<'w, CurrentClientId>,
    pub player_spawn: Res<'w, PlayerSpawnpoint>,
    pub lobby: ResMut<'w, Lobby>,
    pub particles: Res<'w, ParticleMap>,
    pub plugins: Res<'w, Qwaks>,
}
