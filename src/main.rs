#![feature(let_chains)]
extern crate macros;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*,
    render::texture::ImageAddressMode,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_kira_audio::AudioPlugin;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_scene_hook::reload::Plugin as HookPlugin;
use entities::pickup::PickupEntity;
use map_gen::{
    entities::data::{load_pickups, load_weapons},
    load_map,
    texture_systems::*,
};
use player::Player;
use resources::*;

mod entities;
mod map_gen;
mod player;
mod resources;

fn get_map() -> String {
    if let Some(map) = std::env::args().nth(1) {
        if std::fs::File::open(&map).is_ok() {
            return map;
        } else {
            error!("Can't find map: \"{map}\"")
        }
    }

    "assets/maps/Test.map".to_string()
}

fn main() {
    App::new()
        .insert_resource(CurrentMap(get_map()))
        .insert_resource(TexturesLoading::default())
        .insert_resource(TextureMap::default())
        .insert_resource(PlayerSpawnpoint(Vec3::ZERO))
        .insert_resource(MapDoneLoading(false))
        .insert_resource(Paused(true))
        .insert_resource(PickupMap::default())
        .insert_resource(WeaponMap::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default().disabled())
        .add_plugins(DefaultPlugins.set({
            let mut plug = ImagePlugin::default_nearest();
            plug.default_sampler.address_mode_u = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_v = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_w = ImageAddressMode::Repeat;
            plug
        }))
        //.add_plugins(WorldInspectorPlugin::new())
        .add_plugins(TemporalAntiAliasPlugin)
        .add_plugins(ObjPlugin)
        .add_plugins(AudioPlugin)
        .add_plugins(HookPlugin)
        .add_systems(PreStartup, load_pickups)
        .add_systems(PreStartup, load_weapons)
        .add_systems(Startup, load_textures)
        .add_systems(
            Update,
            load_map.run_if(if_texture_done_loading.and_then(run_once())),
        )
        .add_systems(Update, texture_checker.run_if(if_texture_loading))
        .add_systems(
            Update,
            Player::spawn.run_if(if_map_done_loading.and_then(run_once())),
        )
        .add_systems(
            Update,
            (
                Player::update,
                Player::update_cam_vert,
                Player::update_cam_hort,
                Player::ground_detection,
                Player::weaponry_switch,
                Player::weapon_animations,
                PickupEntity::update,
                PickupEntity::handle_pickups,
            )
                .run_if(if_not_paused),
        )
        .add_systems(Update, (Player::pause_handler, Player::debug))
        .run();
}
