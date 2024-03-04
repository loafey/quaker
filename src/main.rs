extern crate macros;
use bevy::{prelude::*, render::texture::ImageAddressMode};
mod map_gen;
mod player;
use map_gen::{load_map, texture_systems::*};
use player::Player;

fn spawn_3d_stuff(mut commands: Commands) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

#[derive(Debug, Resource)]
struct CurrentMap(pub String);

fn main() {
    App::new()
        .insert_resource(CurrentMap("assets/maps/M1.map".to_string()))
        .insert_resource(TexturesLoading::default())
        .insert_resource(TextureMap::default())
        .add_plugins(DefaultPlugins.set({
            let mut plug = ImagePlugin::default_nearest();
            plug.default_sampler.address_mode_u = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_v = ImageAddressMode::Repeat;
            plug.default_sampler.address_mode_w = ImageAddressMode::Repeat;
            plug
        }))
        .add_systems(Startup, spawn_3d_stuff)
        .add_systems(Startup, load_textures)
        .add_systems(
            Update,
            load_map.run_if(if_texture_done_loading.and_then(run_once())),
        )
        .add_systems(Update, texture_checker.run_if(if_texture_loading))
        .add_systems(Startup, Player::spawn)
        .add_systems(Update, Player::update)
        .run();
}
