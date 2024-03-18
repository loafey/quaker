use super::{Player, PlayerFpsMaterial, PlayerFpsModel};
use crate::resources::PlayerSpawnpoint;
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
use bevy_scene_hook::reload::{Hook, SceneBundle as HookedSceneBundle};

impl Player {
    pub fn spawn(
        mut commands: Commands,
        player_spawn: Res<PlayerSpawnpoint>,
        asset_server: Res<AssetServer>,
    ) {
        let player_spawn = player_spawn.0; // Vec3::new(0.0, 10.0, 0.0);
        let mut camera = None;
        let mut fps_model = None;
        commands
            .spawn(Collider::cylinder(0.5, 0.15))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .add(move |mut c: EntityWorldMut| {
                let trans = Transform::from_translation(player_spawn);

                c.insert(KinematicCharacterController::default())
                    .insert(Restitution::coefficient(0.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(Player::default())
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
                        order: 1,
                        clear_color: ClearColorConfig::None,

                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(IsDefaultUiCamera);

                c.spawn(SpriteBundle {
                    texture: asset_server.load("crosshair.png"),
                    ..default()
                });

                camera = Some(new_camera_id);
            })
            .insert(Player {
                children: super::PlayerChildren { camera, fps_model },
                ..Default::default()
            });
    }
}
