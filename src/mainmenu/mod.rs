use crate::{
    net::{self, steam::SteamClient, NetState},
    resources::{CurrentMap, CurrentStage},
    APP_ID,
};
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_simple_text_input::{TextInputBundle, TextInputSettings, TextInputValue};
use macros::{error_continue, error_return};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use steamworks::FriendFlags;

#[derive(Debug, Component)]
pub struct MainMenuEnt;

#[derive(Debug, Component)]
pub enum ButtonEvent {
    Solo,
    StartMp,
    JoinMp,
}

#[derive(Debug, Component)]
pub struct LevelButton(PathBuf);

#[derive(Debug, Component)]
pub struct FriendButton(u64);

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
pub fn buttons(world: &mut World) {
    let mut state: SystemState<(
        Query<(&Interaction, &ButtonEvent), (Changed<Interaction>, With<Button>)>,
        Query<&TextInputValue>,
        ResMut<NextState<CurrentStage>>,
        ResMut<NextState<NetState>>,
        Option<Res<SteamClient>>,
    )> = SystemState::new(world);
    // yea this is cursed, but i am lazy, bypassing the borrow checker like a baus
    let world_copy = unsafe { &mut *(world as *mut World) };

    let (query, text_inputs, mut next_state, mut next_net_state, steam_client) =
        state.get_mut(world);
    let input = &error_return!(text_inputs.get_single()).0;

    for (interaction, event) in &query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        match event {
            ButtonEvent::Solo => {
                error!("solo games are currently disabled");
            }
            ButtonEvent::StartMp => {
                info!("starting multiplayer game");
                if net::server::init_server(world_copy, &mut next_net_state, &steam_client) {
                    next_state.set(CurrentStage::InGame);
                }
            }
            ButtonEvent::JoinMp => {
                net::client::init_client(world_copy, &mut next_net_state, input, &steam_client);
            }
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
            curlevel.0.clone_from(&button.0);
            info!("set level to: {:?}", curlevel.0);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn update_id_buttons(
    query: Query<(&Interaction, &FriendButton), (Changed<Interaction>, With<Button>)>,
    mut text_input: Query<&mut TextInputValue>,
) {
    for (interaction, button) in &query {
        if matches!(interaction, Interaction::Pressed) {
            let mut inp = error_continue!(text_input.get_single_mut());
            inp.0 = format!("{}", button.0);
            info!("set join id to: {:?}", button.0);
        }
    }
}

pub fn setup(mut commands: Commands, steam_client: Option<Res<SteamClient>>) {
    let map_files = error_return!(get_mapfiles("assets/maps"));
    let friends = steam_client
        .as_ref()
        .map(|sc| sc.friends().get_friends(FriendFlags::ALL))
        .map(|friends| {
            friends
                .into_iter()
                .filter(|f| {
                    f.game_played()
                        .map(|f| f.game.app_id() == APP_ID)
                        .unwrap_or_default()
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

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
                    .insert(ButtonEvent::Solo);

                c.spawn(ButtonBundle::default())
                    .insert(TextBundle::from_section(
                        "Start MP",
                        TextStyle {
                            font_size: 32.0,
                            ..Default::default()
                        },
                    ))
                    .insert(ButtonEvent::StartMp);

                c.spawn(NodeBundle::default()).insert(
                    TextInputBundle {
                        settings: TextInputSettings {
                            retain_on_submit: true,
                        },
                        value: TextInputValue("127.0.0.1:8000".to_string()),
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
                    ))
                    .insert(ButtonEvent::JoinMp);
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
                c.spawn(TextBundle::from_section(
                    "Maps:".to_string(),
                    TextStyle {
                        font_size: 32.0,
                        ..Default::default()
                    },
                ));

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

                c.spawn(TextBundle::from_section(
                    "Friends:".to_string(),
                    TextStyle {
                        font_size: 32.0,
                        ..Default::default()
                    },
                ));
                for friend in friends {
                    c.spawn(ButtonBundle {
                        style: Style {
                            border: UiRect::all(Val::Px(5.0)),
                            ..Default::default()
                        },
                        border_color: BorderColor(Color::BLACK),
                        ..Default::default()
                    })
                    .insert(TextBundle::from_section(
                        friend.name(),
                        TextStyle {
                            font_size: 16.0,
                            ..Default::default()
                        },
                    ))
                    .insert(FriendButton(friend.id().raw()));
                }
            });
        })
        .insert(MainMenuEnt);
}
