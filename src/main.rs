extern crate macros;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, prelude::*,
    render::texture::ImageAddressMode,
};
mod map_gen;
mod player;
use map_gen::{if_map_done_loading, load_map, texture_systems::*, MapDoneLoading};
use player::{Player, PlayerSpawnpoint};

#[derive(Debug, Resource)]
struct CurrentMap(pub String);

fn main() {
    App::new()
        .insert_resource(CurrentMap("assets/maps/M1.map".to_string()))
        .insert_resource(TexturesLoading::default())
        .insert_resource(TextureMap::default())
        .insert_resource(PlayerSpawnpoint(Vec3::ZERO))
        .insert_resource(MapDoneLoading(false))
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
        .add_systems(Update, Player::update)
        .run();
}
