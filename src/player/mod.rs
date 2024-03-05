use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasBundle,
        prepass::{DepthPrepass, MotionVectorPrepass},
    },
    input::mouse::MouseMotion,
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::camera::TemporalJitter,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::prelude::*;

#[derive(Resource)]
pub struct PlayerSpawnpoint(pub Vec3);

#[derive(Component, Debug)]
pub struct Player {
    self_rot: f32,
    no_control: bool,
}
impl Player {
    pub fn spawn(mut commands: Commands, player_spawn: Res<PlayerSpawnpoint>) {
        let player_spawn = player_spawn.0; // Vec3::new(0.0, 10.0, 0.0);

        commands
            .spawn(Collider::cuboid(100.0, 0.1, 100.0))
            .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

        commands
            .spawn(RigidBody::Dynamic)
            .add(move |mut c: EntityWorldMut| {
                c.insert(Collider::cylinder(0.5, 0.15));
                c.insert(Restitution::coefficient(0.0));
                c.insert(LockedAxes::ROTATION_LOCKED);

                c.insert(Player {
                    self_rot: 0.0,
                    no_control: true,
                });
                c.insert(GlobalTransform::default());
                let mut trans = Transform::from_translation(player_spawn);
                trans.rotate_x(std::f32::consts::PI / -8.0);
                c.insert(trans);
            })
            .with_children(|c| {
                c.spawn({
                    Camera3dBundle {
                        projection: Projection::Perspective(PerspectiveProjection {
                            fov: 80.0f32.to_radians(),
                            ..default()
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.25, 0.0)),
                        ..Default::default()
                    }
                })
                .insert(ScreenSpaceAmbientOcclusionBundle::default())
                .insert((DepthPrepass, MotionVectorPrepass, TemporalJitter::default()))
                .insert(TemporalAntiAliasBundle::default());
            });
    }
    pub fn update_cam(
        mut query: Query<(&Camera3d, &mut Transform)>,
        q_parent: Query<(&Player, &Children)>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (_, children) in q_parent.iter() {
            for &child in children.iter() {
                let (_, mut trans) = query.get_mut(child).unwrap();

                for ev in motion_evr.read() {
                    let old = trans.rotation;
                    trans.rotate_local_x(ev.delta.y / -1000.0);
                    if trans.rotation.x < -0.7 || trans.rotation.x > 0.7 {
                        trans.rotation = old;
                    }
                }
            }
        }
    }
    pub fn update(
        keys: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Player, &mut Transform)>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (mut player, mut gt) in &mut query {
            if keys.just_pressed(KeyCode::Tab) {
                player.no_control = !player.no_control;
                let mut primary_window = q_windows.single_mut();
                if player.no_control {
                    primary_window.cursor.grab_mode = CursorGrabMode::None;
                    primary_window.cursor.visible = true;
                } else {
                    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                    primary_window.cursor.visible = false;
                }
            }

            if !player.no_control {
                // handle cursor

                for ev in motion_evr.read() {
                    let x_delta = ev.delta.x / -1000.0;
                    player.self_rot += x_delta;
                    if player.self_rot < 0.0 {
                        player.self_rot = std::f32::consts::PI * 2.0 - x_delta;
                    } else if player.self_rot > std::f32::consts::PI * 2.0 {
                        player.self_rot = x_delta;
                    }
                    gt.rotate_y(x_delta);
                    //gt.rotate_local_x(ev.delta.y / -1000.0);
                }

                // handle input
                let local_z = gt.local_z();
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                if keys.pressed(KeyCode::KeyW) {
                    gt.translation += forward / 10.0;
                } else if keys.pressed(KeyCode::KeyS) {
                    gt.translation -= forward / 10.0;
                }

                if keys.pressed(KeyCode::KeyA) {
                    gt.translation -= right / 10.0;
                } else if keys.pressed(KeyCode::KeyD) {
                    gt.translation += right / 10.0;
                }

                if keys.pressed(KeyCode::ShiftLeft) {
                    gt.translation.y += 0.1;
                } else if keys.pressed(KeyCode::ControlLeft) {
                    gt.translation.y -= 0.1;
                }
            }
        }
    }
}
