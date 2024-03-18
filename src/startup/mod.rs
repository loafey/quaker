use bevy::prelude::*;
use bevy::transform::commands;
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioControl;

use crate::resources::CurrentStage;

#[derive(Component)]
pub struct StartupEnt;

#[derive(Debug, Default, Component)]
pub struct StartUpState {
    time: f32,
    played_sound: bool,
}
pub fn startup_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle {
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(StartupEnt);
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("ui/splash.png"),
            sprite: Sprite {
                color: Color::rgba(1.0, 1.0, 1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(StartUpState::default())
        .insert(StartupEnt);
}
pub fn startup_update(
    mut commands: Commands,
    mut query: Query<(&mut Sprite, &mut StartUpState)>,
    time: Res<Time>,
    audio: Res<Audio>,
    asset_server: Res<AssetServer>,
    mut game_stage: ResMut<CurrentStage>,
    ents: Query<(Entity, &StartupEnt)>,
) {
    let mut kill_all = false;
    for (mut sprite, mut state) in &mut query {
        state.time += time.delta_seconds();
        if state.time > 1.0 && !state.played_sound {
            state.played_sound = true;
            audio.play(asset_server.load("sounds/splip.ogg"));
        }

        if state.time < 2.4 {
            sprite.color = Color::rgba(1.0, 1.0, 1.0, ((state.time - 1.0) / 2.0).clamp(0.0, 1.0));
        } else {
            sprite.color = Color::rgba(1.0, 1.0, 1.0, (2.0 - (state.time / 2.0)).clamp(0.0, 1.0));
        }

        if state.time > 5.0 {
            *game_stage = CurrentStage::InGame;
            kill_all = true;
        }
    }
    if kill_all {
        for (ent, _) in &ents {
            commands.entity(ent).despawn_recursive()
        }
    }
}
