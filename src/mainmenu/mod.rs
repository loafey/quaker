use crate::resources::{CurrentMap, CurrentStage};
use bevy::prelude::*;
use bevy_simple_text_input::{TextInputBundle, TextInputSettings};
use macros::error_return;
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Component)]
pub struct MainMenuEnt;
#[derive(Debug, Component)]
pub struct StartButton;
#[derive(Debug, Component)]
pub struct LevelButton(PathBuf);

fn get_mapfiles<P: AsRef<Path>>(dir: P) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let dir = fs::read_dir(dir)?;
    for f in dir {
        let f = f?.path();

        if f.is_dir() {
            files.append(&mut get_mapfiles(f)?);
        } else {
            files.push(f);
        }
    }

    Ok(files)
}

#[allow(clippy::type_complexity)]
pub fn start_level(
    query: Query<(&Interaction, &StartButton), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<CurrentStage>>,
) {
    for (interaction, _) in &query {
        if matches!(interaction, Interaction::Pressed) {
            next_state.set(CurrentStage::InGame);
        }
    }
}

pub fn clear(query: Query<(Entity, &MainMenuEnt)>, mut commands: Commands) {
    for (ent, _) in &query {
        commands.entity(ent).despawn_recursive();
    }
}

#[allow(clippy::type_complexity)]
pub fn update_level_buttons(
    query: Query<(&Interaction, &LevelButton), (Changed<Interaction>, With<Button>)>,
    mut curlevel: ResMut<CurrentMap>,
) {
    for (interaction, button) in &query {
        if matches!(interaction, Interaction::Pressed) {
            curlevel.0 = button.0.clone();
            warn!("set level to: {:?}", curlevel.0);
        }
    }
}

pub fn setup(mut commands: Commands) {
    let map_files = error_return!(get_mapfiles("assets/maps"));

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
        .with_children(|c| {
            c.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    height: Val::Vh(100.0),
                    left: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    TextBundle::from_section(
                        "Quaker!",
                        TextStyle {
                            font_size: 100.0,
                            ..default()
                        },
                    )
                    .with_text_justify(JustifyText::Center)
                    .with_style(Style { ..default() }),
                    Label,
                ));
                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Solo",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ))
                    .insert(StartButton);

                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Start MP",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ));

                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Start MP Steam",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ));

                c.spawn(NodeBundle::default()).insert(
                    TextInputBundle {
                        settings: TextInputSettings {
                            retain_on_submit: true,
                        },
                        ..Default::default()
                    }
                    .with_text_style(TextStyle {
                        font_size: 32.0,
                        ..Default::default()
                    }),
                );

                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Join IP",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ));

                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Join Steam",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ));
            });

            c.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    height: Val::Vh(100.0),
                    right: Val::Px(0.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            })
            .with_children(|c| {
                for map in map_files {
                    c.spawn(ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..Default::default()
                    })
                    .insert(TextBundle::from_section(
                        format!("{map:?}"),
                        TextStyle {
                            font_size: 16.0,
                            ..Default::default()
                        },
                    ))
                    .insert(LevelButton(map.clone()));
                }
            });
        })
        .insert(MainMenuEnt);
}
