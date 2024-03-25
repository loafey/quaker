use super::{Player, PlayerController, PlayerFpsMaterial, PlayerFpsModel, PlayerMpModel};
use crate::{
    net::{server::Lobby, CurrentClientId},
    resources::{PlayerSpawnpoint, WeaponMap},
};
use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasBundle,
        prepass::{DepthPrepass, MotionVectorPrepass},
    },
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::{camera::TemporalJitter, view::NoFrustumCulling},
};
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::ClientId;
use bevy_scene_hook::reload::{Hook, SceneBundle as HookedSceneBundle};

impl Player {
    pub fn spawn_own_player(
        mut commands: Commands,
        player_spawn: Res<PlayerSpawnpoint>,
        asset_server: Res<AssetServer>,
        lobby: Option<ResMut<Lobby>>,
        client_id: Res<CurrentClientId>,
        weapon_map: Res<WeaponMap>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        let id = Self::spawn(
            &mut commands,
            &mut materials,
            true,
            player_spawn.0,
            &asset_server,
            client_id.0,
            &weapon_map,
            Vec::new(),
        );
        if let Some(mut lobby) = lobby {
            lobby.players.insert(ClientId::from_raw(client_id.0), id);
        }
    }

    pub fn spawn(
        commands: &mut Commands,
        materials: &mut Assets<StandardMaterial>,
        is_own: bool,
        player_spawn: Vec3,
        asset_server: &AssetServer,
        current_id: u64,
        weapon_map: &WeaponMap,
        weapons: Vec<Vec<String>>,
    ) -> Entity {
        let mut camera = None;
        let mut fps_model = None;
        let mut entity = commands.spawn(Collider::cylinder(0.5, 0.15));

        let commands = entity
            .insert(ActiveEvents::COLLISION_EVENTS)
            .add(move |mut c: EntityWorldMut| {
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
                    .spawn({
                        Camera3dBundle {
                            projection: Projection::Perspective(PerspectiveProjection {
                                fov: 80.0f32.to_radians(),
                                ..default()
                            }),
                            //transform: Transform::from_translation(Vec3::new(0.0, 0.25, 1.0)),
                            transform: Transform::from_translation(Vec3::new(0.0, 0.25, 0.0)),
                            camera: Camera {
                                is_active: is_own,
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .insert(ScreenSpaceAmbientOcclusionBundle::default())
                    .insert((DepthPrepass, MotionVectorPrepass, TemporalJitter::default()))
                    .insert(TemporalAntiAliasBundle::default())
                    .insert(Name::new("player camera"))
                    .with_children(|c| {
                        let new_fps_model = c
                            .spawn(PlayerFpsModel)
                            .insert(HookedSceneBundle {
                                scene: SceneBundle::default(),
                                reload: Hook::new(|entity, commands, world, root| {
                                    if entity.get::<Handle<Mesh>>().is_some() {
                                        commands.insert(NoFrustumCulling);
                                    }
                                    if entity.get::<Handle<StandardMaterial>>().is_some() {
                                        if let Some(material) =
                                            world.entity(root).get::<PlayerFpsMaterial>()
                                        {
                                            commands.insert(material.0.clone());
                                        }
                                    }
                                }),
                            })
                            .insert(PlayerFpsMaterial::default())
                            .insert(Name::new("fps model holder"))
                            .id();
                        fps_model = Some(new_fps_model);
                    })
                    .id();

                c.spawn(Camera2dBundle {
                    camera: Camera {
                        order: 2,
                        clear_color: ClearColorConfig::None,
                        is_active: is_own,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(IsDefaultUiCamera);

                if is_own {
                    c.spawn(SpriteBundle {
                        texture: asset_server.load("crosshair.png"),
                        ..default()
                    });
                }

                camera = Some(new_camera_id);
            });

        let mut player_data = Player {
            id: current_id,
            children: super::PlayerChildren { camera, fps_model },
            ..Default::default()
        };

        for (slot, list) in weapons.into_iter().enumerate() {
            for weapon in list {
                if let Some(weapon_data) = weapon_map.0.get(&weapon) {
                    let handle = asset_server.load(format!("{}#Scene0", weapon_data.model_file));
                    player_data.add_weapon(weapon_data.clone(), slot, handle);
                }
            }
        }
        commands.insert(player_data);

        if is_own {
            commands.insert(PlayerController);
        } else {
            commands.with_children(|c| {
                let mut trans = Transform::from_translation(Vec3::new(0.0, -0.5, 0.0));
                trans.scale = Vec3::splat(0.5);
                trans.rotate_y(180f32.to_radians());
                c.spawn(PbrBundle {
                    mesh: asset_server.load("models/Player/MP/Temp.obj"),
                    material: materials.add(StandardMaterial {
                        base_color_texture: Some(
                            asset_server.load("models/Enemies/DeadMan/deadman.png"),
                        ),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    }),
                    transform: trans,
                    ..Default::default()
                })
                .insert(PlayerMpModel);
            });
        }

        commands.id()
    }
}
