#![feature(let_chains)]
extern crate macros;
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, log::LogPlugin, prelude::*,
    render::texture::ImageAddressMode,
};
use bevy_kira_audio::AudioPlugin;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_renet::{
    transport::{NetcodeClientPlugin, NetcodeServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use bevy_scene_hook::reload::Plugin as HookPlugin;
use bevy_simple_text_input::TextInputPlugin;
use net::ClientMessage;
use plugins::{ClientPlugin, GameStage, MainMenuStage, Resources, ServerPlugin, StartupStage};
use renet_steam::bevy::{SteamClientPlugin, SteamServerPlugin};
use steamworks::SingleClient;

use crate::try_steam::try_steam;

mod entities;
mod mainmenu;
mod map_gen;
mod net;
mod player;
mod plugins;
mod resources;
mod startup;
mod try_steam;

fn steam_callbacks(client: NonSend<SingleClient>) {
    client.run_callbacks();
}

fn main() {
    let mut app = App::new();

    app.add_event::<ClientMessage>();

    app.add_plugins((
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
        //bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
        TemporalAntiAliasPlugin,
        ObjPlugin,
        AudioPlugin,
        HookPlugin,
        (StartupStage, MainMenuStage, GameStage),
        TextInputPlugin,
        (
            RenetClientPlugin,
            RenetServerPlugin,
            ServerPlugin,
            ClientPlugin,
        ),
    ));

    if let Some((steam, single_client)) = try_steam() {
        app.insert_non_send_resource(single_client);
        app.insert_non_send_resource(steam);
        app.add_plugins((SteamServerPlugin, SteamClientPlugin));
        app.add_systems(PreUpdate, steam_callbacks);
    } else {
        app.add_plugins((NetcodeServerPlugin, NetcodeClientPlugin));
    }

    app.run();
}
