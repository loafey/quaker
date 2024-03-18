use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct MainMenuEnt;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands
        .spawn(Camera2dBundle::default())
        .insert(MainMenuEnt);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .insert(MainMenuEnt);
}
