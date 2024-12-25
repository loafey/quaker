use crate::{
    net::{self, steam::SteamClient, NetState},
    resources::{CurrentMap, CurrentStage},
    APP_ID,
};
use bevy::{ecs::system::SystemState, prelude::*};
use bevy_simple_text_input::{TextInput, TextInputSettings, TextInputTextFont, TextInputValue};
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
    commands.insert_resource(AmbientLight::default());
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

pub fn update_point_light(mut query: Query<&mut PointLight>) {
    for mut light in query.iter_mut() {
        light.intensity += 0.1;
        light.intensity *= 1.02;
        light.intensity = light.intensity.min(65000.0);
    }
}

pub fn setup(
    mut commands: Commands,
    steam_client: Option<Res<SteamClient>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(AmbientLight {
        brightness: 0.0,
        ..default()
    });

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
        .spawn(Camera2d)
        .insert(Camera {
            order: 2,
            clear_color: ClearColorConfig::None,
            is_active: true,
            ..default()
        })
        .insert(MainMenuEnt);
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load("ui/main_menu.png")),
        alpha_mode: AlphaMode::Add,
        ..default()
    });
    // cube
    let scale = 0.5;
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(16.0 * scale, 9.0 * scale, 1.0))),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.6, 0.0),
        MainMenuEnt,
    ));
    // light
    for x in -1..=1 {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.6, 0.6),
                shadows_enabled: false,
                intensity: 1000.0,
                range: 1000.0,
                ..default()
            },
            Transform::from_xyz(x as f32 * 2.0, 0.0, -9.0),
            MainMenuEnt,
        ));
    }

    // camera
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -9.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainMenuEnt,
    ));

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .with_children(|c| {
            c.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Vw(100.0),
                border: UiRect::all(Val::Px(2.0)),
                left: Val::Px(0.0),
                bottom: Val::Px(76.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|c| {
                c.spawn(Button)
                    .insert((
                        Text::new("Solo"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                    ))
                    .insert(ButtonEvent::Solo);

                c.spawn(Button)
                    .insert((
                        Text::new("Start MP"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                    ))
                    .insert(ButtonEvent::StartMp);

                c.spawn(Node::default()).insert((
                    TextInput,
                    TextInputValue("127.0.0.1:8000".to_string()),
                    TextInputSettings {
                        retain_on_submit: true,
                        ..default()
                    },
                    TextInputTextFont(TextFont {
                        font_size: 32.0,
                        ..default()
                    }),
                ));

                c.spawn(Button)
                    .insert((
                        Text::new("Join IP"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                    ))
                    .insert(ButtonEvent::JoinMp);
            });

            c.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Px(400.0),
                border: UiRect::all(Val::Px(2.0)),
                height: Val::Vh(100.0),
                right: Val::Px(0.0),
                flex_direction: FlexDirection::Column,
                // background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    Text::new("Maps:".to_string()),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                ));

                for map in map_files {
                    c.spawn(
                        Button, /*{
                                   border: UiRect::all(Val::Px(5.0)),
                                   border_color: BorderColor(Color::BLACK),
                                   ..default()
                               }*/
                    )
                    .insert(Text::new(format!("{map:?}")))
                    .insert(TextFont {
                        font_size: 16.0,
                        ..default()
                    })
                    .insert(LevelButton(map.clone()));
                }

                c.spawn((
                    Text::new("Friends:".to_string()),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                ));
                for friend in friends {
                    c.spawn(
                        Button, /*{
                                   style: Style {
                                       border: UiRect::all(Val::Px(5.0)),
                                       ..default()
                                   },
                                   border_color: BorderColor(Color::BLACK),
                                   ..default()
                               }*/
                    )
                    .insert(Text::new(friend.name()))
                    .insert(TextFont {
                        font_size: 16.0,
                        ..default()
                    })
                    .insert(FriendButton(friend.id().raw()));
                }
            });
        })
        .insert(MainMenuEnt);
}
