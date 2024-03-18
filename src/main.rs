#![feature(let_chains)]
extern crate macros;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, log::LogPlugin, prelude::*,
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
use plugins::{Game, Resources};

mod entities;
mod mainmenu;
mod map_gen;
mod player;
mod plugins;
mod resources;

fn main() {
    App::new()
        .add_plugins((
            Resources,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default().disabled(),
            DefaultPlugins
                .set({
                    let mut plug = ImagePlugin::default_nearest();
                    plug.default_sampler.address_mode_u = ImageAddressMode::Repeat;
                    plug.default_sampler.address_mode_v = ImageAddressMode::Repeat;
                    plug.default_sampler.address_mode_w = ImageAddressMode::Repeat;
                    plug
                })
                .set(LogPlugin {
                    filter: "bevy_ecs=error,wgpu=error,naga=warn,present_frames=warn".into(),
                    level: bevy::log::Level::INFO,
                    ..Default::default()
                }),
            WorldInspectorPlugin::new(),
            TemporalAntiAliasPlugin,
            ObjPlugin,
            AudioPlugin,
            HookPlugin,
            Game,
        ))
        .run();
}
