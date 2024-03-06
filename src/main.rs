extern crate macros;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*,
    render::texture::ImageAddressMode,
};
mod map_gen;
mod player;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use map_gen::{load_map, texture_systems::*};
use player::Player;
use resources::*;

mod resources;

fn main() {
    App::new()
        .insert_resource(CurrentMap("assets/maps/Test.map".to_string()))
        .insert_resource(TexturesLoading::default())
        .insert_resource(TextureMap::default())
        .insert_resource(PlayerSpawnpoint(Vec3::ZERO))
        .insert_resource(MapDoneLoading(false))
        .insert_resource(Paused(true))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default().disabled())
        .add_plugins(DefaultPlugins.set({
            let mut plug = ImagePlugin::default_nearest();
            plug.default_sampler.address_mode_u = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_v = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_w = ImageAddressMode::Repeat;
            plug
        }))
        .add_plugins(TemporalAntiAliasPlugin)
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
            )
                .run_if(if_not_paused),
        )
        .add_systems(Update, (Player::pause_handler, Player::debug))
        .run();
}
