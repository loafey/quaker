use super::{Player, PlayerSpawnpoint};
use bevy::{
    core_pipeline::{
        experimental::taa::TemporalAntiAliasBundle,
        prepass::{DepthPrepass, MotionVectorPrepass},
    },
    pbr::ScreenSpaceAmbientOcclusionBundle,
    prelude::*,
    render::camera::TemporalJitter,
};
use bevy_rapier3d::prelude::*;

impl Player {
    pub fn spawn(mut commands: Commands, player_spawn: Res<PlayerSpawnpoint>) {
        let player_spawn = player_spawn.0; // Vec3::new(0.0, 10.0, 0.0);

        //commands
        //    .spawn(Collider::cuboid(100.0, 0.1, 100.0))
        //    .insert(TransformBundle::from(Transform::from_xyz(0.0, -2.0, 0.0)));

        commands
            .spawn(RigidBody::Dynamic)
            .add(move |mut c: EntityWorldMut| {
                let mut trans = Transform::from_translation(player_spawn);
                trans.rotate_x(std::f32::consts::PI / -8.0);

                c.insert(Collider::cylinder(0.5, 0.15))
                    .insert(Restitution::coefficient(0.0))
                    .insert(LockedAxes::ROTATION_LOCKED)
                    .insert(GravityScale(0.0))
                    .insert(Player { self_rot: 0.0 })
                    .insert(GlobalTransform::default())
                    .insert(trans);
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
}
