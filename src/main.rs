#![feature(let_chains)]
extern crate macros;
use crate::net::{
    steam::{try_steam, SteamClient},
    SimulationEvent,
};
use bevy::{
    core_pipeline::experimental::taa::TemporalAntiAliasPlugin, image::ImageAddressMode,
    log::LogPlugin, prelude::*, window::WindowMode,
};
use bevy_hanabi::HanabiPlugin;
use bevy_obj::ObjPlugin;
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::RapierDebugRenderPlugin,
};
use bevy_renet::{
    netcode::{NetcodeClientPlugin, NetcodeServerPlugin},
    steam::{SteamClientPlugin, SteamServerPlugin},
    RenetClientPlugin, RenetServerPlugin,
};
use bevy_scene_hook::reload::Plugin as HookPlugin;
use bevy_simple_text_input::TextInputPlugin;
use net::ClientMessage;
use plugins::{ClientPlugin, GameStage, MainMenuStage, Resources, ServerPlugin, StartupStage};
use steamworks::{AppId, SingleClient};

mod entities;
mod integrity;
mod mainmenu;
mod map_gen;
mod net;
mod particles;
mod player;
mod plugins;
mod queries;
mod resources;
mod startup;

const APP_ID: AppId = AppId(480);

fn steam_callbacks(client: NonSend<SingleClient>) {
    client.run_callbacks();
}

fn main() {
    println!("Running with asset hash: {}", integrity::get_asset_hash());

    let mut app = App::new();
    app.add_plugins(
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
                ..default()
            }), // .set(WindowPlugin {
                // primary_window: Some(Window {
                // mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                // ..default()
                // }),
                // ..default()
                // }),
                //bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
    );

    app.add_event::<ClientMessage>()
        .add_event::<SimulationEvent>();

    app.add_plugins(Resources);
    app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
    app.add_plugins(RapierDebugRenderPlugin::default().disabled());
    app.add_plugins(TemporalAntiAliasPlugin);
    app.add_plugins(ObjPlugin);
    app.add_plugins(HookPlugin);
    app.add_plugins((StartupStage, MainMenuStage, GameStage));
    app.add_plugins(HanabiPlugin);
    app.add_plugins(TextInputPlugin);
    // MP
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(ServerPlugin);
    app.add_plugins(ClientPlugin);

    app.add_systems(Startup, particles::register_particles);
    app.add_systems(Update, particles::ParticleLifetime::update);

    if let Some((steam, single_client)) = try_steam() {
        app.insert_non_send_resource(single_client);
        app.insert_resource(SteamClient::new(steam));
        app.add_plugins((SteamServerPlugin, SteamClientPlugin));
        app.add_systems(PreUpdate, steam_callbacks);
        app.add_systems(Startup, net::steam::grab_avatar);
    } else {
        app.add_plugins((NetcodeServerPlugin, NetcodeClientPlugin));
    }

    app.run();
}
