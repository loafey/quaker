use crate::resources::CurrentStage;
use bevy::prelude::*;

#[derive(Component)]
pub struct StartupEnt;

#[derive(Debug, Default, Component)]
pub struct StartUpState {
    time: f32,
    played_sound: bool,
}
pub fn startup_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Camera2d::default(),
            Camera {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..Default::default()
            },
        ))
        .insert(StartupEnt);
    commands
        .spawn(Sprite {
            image: asset_server.load("ui/splash.png"),
            ..Default::default()
        })
        .insert(StartUpState::default())
        .insert(StartupEnt);
}

#[allow(clippy::too_many_arguments)]
pub fn startup_update(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Sprite, &mut StartUpState)>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut game_stage: ResMut<NextState<CurrentStage>>,
    ents: Query<(Entity, &StartupEnt)>,
) {
    let mut kill_all = false;
    if keys.any_pressed([
        KeyCode::Space,
        KeyCode::Enter,
        KeyCode::CapsLock,
        KeyCode::Escape,
    ]) {
        kill_all = true;
    }
    for (mut sprite, mut state) in &mut query {
        state.time += time.delta_secs();
        if state.time > 1.0 && !state.played_sound {
            state.played_sound = true;
            commands
                .spawn(AudioPlayer::<AudioSource>(
                    asset_server.load("sounds/splip.ogg"),
                ))
                .insert(StartupEnt);
        }

        if state.time < 2.4 {
            sprite.color = Color::rgba(1.0, 1.0, 1.0, ((state.time - 1.0) / 2.0).clamp(0.0, 1.0));
        } else {
            sprite.color = Color::rgba(1.0, 1.0, 1.0, (2.0 - (state.time / 2.0)).clamp(0.0, 1.0));
        }

        if state.time > 5.0 {
            kill_all = true;
        }
    }
    if kill_all {
        game_stage.set(CurrentStage::MainMenu);
        for (ent, _) in &ents {
            commands.entity(ent).despawn_recursive()
        }
    }
}
