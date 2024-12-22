use super::{Player, PlayerController, PlayerFpsMaterial, PlayerFpsModel, PlayerMpModel};
use crate::{
    net::{
        steam::{CurrentAvatar, SteamClient},
        PlayerInfo,
    },
    queries::NetWorld,
    resources::PlayerSpawnpoint,
};
use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasing,
        prepass::{DepthPrepass, MotionVectorPrepass},
    },
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    render::{camera::TemporalJitter, view::NoFrustumCulling},
};
use bevy_rapier3d::prelude::*;
use bevy_scene_hook::reload::{Hook, SceneBundle as HookedSceneBundle};
use faststr::FastStr;

impl Player {
    pub fn spawn_own_player(
        mut nw: NetWorld,
        player_spawn: Res<PlayerSpawnpoint>,
        avatar: Option<Res<CurrentAvatar>>,
        steam: Option<Res<SteamClient>>,
    ) {
        let id = nw.current_id.0;
        let entity = Self::spawn(
            &mut nw,
            true,
            player_spawn.0,
            id,
            Vec::new(),
            avatar.as_ref(),
        );

        nw.lobby.insert(
            nw.current_id.0,
            PlayerInfo::new(
                entity,
                FastStr::from(steam.map(|s| s.friends().name()).unwrap_or(format!("{id}"))),
            ),
        );
    }

    pub fn spawn(
        nw: &mut NetWorld,
        is_own: bool,
        player_spawn: Vec3,
        current_id: u64,
        weapons: Vec<Vec<FastStr>>,
        avatar: Option<&Res<CurrentAvatar>>,
    ) -> Entity {
        let mut camera = None;
        let mut fps_model = None;
        let mut ammo_hud = None;
        let mut armour_hud = None;
        let mut health_hud = None;
        let mut debug_hud = None;
        let mut message_holder = None;
        let mut shoot_sound_holder = None;
        let mut lobby_hud = None;
        let mut entity = nw.commands.spawn(Collider::cylinder(0.5, 0.15));

        let player_commands = entity
            .insert(ActiveEvents::COLLISION_EVENTS)
            .queue(move |mut c: EntityWorldMut| {
                let trans = Transform::from_translation(player_spawn);

                c.insert(KinematicCharacterController::default())
                    .insert(Restitution::coefficient(0.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(GlobalTransform::default())
                    .insert(trans)
                    .insert(Ccd::enabled());
            })
            .with_children(|c| {
                let new_camera_id = c
                    .spawn((
                        Camera3d::default(),
                        Projection::Perspective(PerspectiveProjection {
                            fov: 80.0f32.to_radians(),
                            ..default()
                        }),
                        Transform::from_translation(Vec3::new(0.0, 0.25, 0.0)),
                        Camera {
                            is_active: is_own,
                            ..Default::default()
                        },
                    ))
                    .insert(ScreenSpaceAmbientOcclusion::default())
                    .insert((DepthPrepass, MotionVectorPrepass, TemporalJitter::default()))
                    .insert(TemporalAntiAliasing::default())
                    .insert(Name::new("player camera"))
                    .with_children(|c| {
                        let new_fps_model = c
                            .spawn(PlayerFpsModel)
                            .insert(HookedSceneBundle {
                                scene: SceneRoot::default(),
                                reload: Hook::new(|entity, commands, world, root| {
                                    if entity.get::<Mesh3d>().is_some() {
                                        commands.insert(NoFrustumCulling);
                                    }
                                    if entity.get::<MeshMaterial3d<StandardMaterial>>().is_some() {
                                        if let Some(material) =
                                            world.entity(root).get::<PlayerFpsMaterial>()
                                        {
                                            commands.insert(MeshMaterial3d(material.0.clone()));
                                        }
                                    }
                                }),
                            })
                            .insert(PlayerFpsMaterial::default())
                            .insert(Name::new("fps model holder"))
                            .id();
                        fps_model = Some(new_fps_model);

                        if is_own {
                            c.spawn((Transform::IDENTITY, SpatialListener::new(2.0)));
                        }

                        shoot_sound_holder = Some(c.spawn(Transform::IDENTITY).id());
                    })
                    .id();

                camera = Some(new_camera_id);
                if is_own {
                    c.spawn((
                        Camera2d,
                        Camera {
                            order: 2,
                            clear_color: ClearColorConfig::None,
                            is_active: is_own,
                            ..Default::default()
                        },
                    ))
                    .insert(IsDefaultUiCamera);

                    c.spawn(Sprite {
                        image: nw.asset_server.load("crosshair.png"),
                        ..default()
                    });
                }
            });

        if is_own {
            player_commands.insert(PlayerController);
        } else {
            player_commands.with_children(|c| {
                let mut trans = Transform::from_translation(Vec3::new(0.0, -0.5, 0.0));
                trans.scale = Vec3::splat(0.5);
                trans.rotate_y(180f32.to_radians());
                c.spawn((
                    Mesh3d(nw.asset_server.load("models/Player/MP/Temp.obj")),
                    MeshMaterial3d(nw.materials.add(StandardMaterial {
                        base_color_texture: Some(
                            nw.asset_server.load("models/Enemies/DeadMan/deadman.png"),
                        ),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    })),
                    trans,
                ))
                .insert(PlayerMpModel);
            });
        }
        let id = player_commands.id();
        if is_own {
            nw.commands
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|c| {
                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(128.0 * 3.0),
                            height: Val::Px(32.0 * 3.0),
                            left: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                            // background_color: Color::WHITE.into(),
                            ..default()
                        },
                        ImageNode {
                            image: nw.asset_server.load("ui/PlayerHud.png"),
                            ..default()
                        },
                    ));

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_content: AlignContent::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|c| {
                        lobby_hud = Some(
                            c.spawn((
                                Text::new("HEALTH: 100"),
                                TextFont {
                                    font_size: 32.0,
                                    font: nw.asset_server.load("ui/Pixeled.ttf"),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ))
                            .insert(Visibility::Hidden)
                            .id(),
                        );
                    });

                    message_holder = Some(
                        c.spawn(Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(10.0),
                            top: Val::Px(10.0),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .id(),
                    );

                    let text_color = Color::rgb(0.921, 0.682, 0.203);

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0 * 3.0),
                        bottom: Val::Px(18.0 * 2.0),
                        ..default()
                    })
                    .with_children(|c| {
                        health_hud = Some(
                            c.spawn((
                                Text::new("HEALTH: 100"),
                                TextFont {
                                    font_size: 32.0,
                                    font: nw.asset_server.load("ui/Pixeled.ttf"),
                                    ..default()
                                },
                                TextColor(text_color),
                            ))
                            .id(),
                        );
                    });

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(34.0 * 3.0),
                        bottom: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|c| {
                        armour_hud = Some(
                            c.spawn((
                                Text::new("ARMOUR: 100"),
                                TextFont {
                                    font_size: 32.0,
                                    font: nw.asset_server.load("ui/Pixeled.ttf"),
                                    ..default()
                                },
                                TextColor(text_color),
                            ))
                            .id(),
                        );
                    });

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(269.0),
                        bottom: Val::Px(4.0),
                        ..default()
                    })
                    .with_children(|c| {
                        ammo_hud = Some(
                            c.spawn((
                                Text::new("100\nCRUTONS"),
                                TextFont {
                                    font_size: 32.0,
                                    font: nw.asset_server.load("ui/Pixeled.ttf"),
                                    ..default()
                                },
                                TextColor(text_color),
                                TextLayout::new_with_justify(JustifyText::Center),
                            ))
                            .id(),
                        );
                    });

                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            width: Val::Px(26.0 * 3.0),
                            height: Val::Px(28.0 * 3.0),
                            left: Val::Px(2.0 * 3.0),
                            bottom: Val::Px(2.0 * 3.0),
                            // a `NodeBundle` is transparent by default, so to see the image we have to its color to `WHITE`
                            //background_color: Color::WHITE.into(),
                            ..default()
                        },
                        ImageNode {
                            image: avatar
                                .map(|c| c.0.clone())
                                .unwrap_or_else(|| nw.asset_server.load("ui/PlayerIcon.png")),
                            ..default()
                        },
                    ));

                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        ..default()
                    })
                    .with_children(|c| {
                        let mut bundle = Text::new("yo");

                        debug_hud = Some(
                            c.spawn((
                                Visibility::Hidden,
                                bundle,
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                            ))
                            .id(),
                        );
                    });
                });
        }

        let mut player_commands = nw.commands.get_entity(id).unwrap();

        let mut player_data = Player {
            id: current_id,
            children: super::PlayerChildren {
                camera,
                fps_model,
                ammo_hud,
                armour_hud,
                health_hud,
                debug_hud,
                message_holder,
                shoot_sound_holder,
                lobby_hud,
            },
            ..Default::default()
        };

        for (slot, list) in weapons.into_iter().enumerate() {
            for weapon in list {
                if let Some(weapon_data) = nw.weapon_map.0.get(&weapon) {
                    let handle = nw
                        .asset_server
                        .load(format!("{}#Scene0", weapon_data.model_file));
                    player_data.add_weapon(weapon_data.clone(), slot, handle);
                }
            }
        }
        player_commands.insert(player_data);
        id
    }
}
