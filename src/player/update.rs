use super::Player;
use crate::Paused;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{
    control::KinematicCharacterController, geometry::Collider, pipeline::QueryFilter,
    plugin::RapierContext,
};

impl Player {
    pub fn update_cam_vert(
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
                    if trans.rotation.x < -0.5 || trans.rotation.x > 0.7 {
                        trans.rotation = old;
                    }
                }
            }
        }
    }

    pub fn update_cam_hort(
        mut query: Query<(&mut Player, &mut Transform)>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (mut player, mut gt) in &mut query {
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
        }
    }

    pub fn update(
        keys: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
        mut query: Query<(
            &mut KinematicCharacterController,
            &mut Player,
            &mut Transform,
        )>,
    ) {
        for (mut controller, mut player, mut gt) in &mut query {
            // handle input
            let local_z = gt.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            let hort_speed = player.hort_speed;
            if keys.pressed(KeyCode::KeyW) {
                player.velocity += forward * hort_speed;
            } else if keys.pressed(KeyCode::KeyS) {
                player.velocity -= forward * hort_speed;
            }

            if keys.pressed(KeyCode::KeyA) {
                player.velocity -= right * hort_speed;
            } else if keys.pressed(KeyCode::KeyD) {
                player.velocity += right * hort_speed;
            }

            if player.on_ground && player.jump_timer <= 0.0 {
                player.velocity.y = 0.0;
                player.jump_timer = 0.0;
                if keys.just_pressed(KeyCode::Space) {
                    player.jump_timer = 0.1;
                    player.velocity.y = player.jump_height;
                    player.air_time = Some(std::time::Instant::now())
                }
            } else {
                player.velocity.y += time.delta_seconds() * player.jump_timer * player.gravity;
                player.jump_timer -= time.delta_seconds() * 50.0;
                player.jump_timer = player.jump_timer.clamp(-0.1, 1.0);
            }

            // println!(
            //     "onground: {}\t|\tvelocity: {}\t|\tjump timer: {}",
            //     player.on_ground, player.velocity.y, player.jump_timer
            // );

            controller.translation = Some(player.velocity * time.delta_seconds());

            let x = player.velocity.x;
            let z = player.velocity.z;
            player.velocity = Vec3::new(
                x.lerp(0.0, player.hort_friction)
                    .clamp(-player.hort_max_speed, player.hort_max_speed),
                player.velocity.y,
                z.lerp(0.0, player.hort_friction)
                    .clamp(-player.hort_max_speed, player.hort_max_speed),
            );

            if keys.pressed(KeyCode::ShiftLeft) {
                gt.translation.y += 0.1;
            } else if keys.pressed(KeyCode::ControlLeft) {
                gt.translation.y -= 0.1;
            }
        }
    }

    pub fn pause_handler(
        keys: Res<ButtonInput<KeyCode>>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        mut paused: ResMut<Paused>,
        mut time: ResMut<Time<Virtual>>,
    ) {
        if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::CapsLock) {
            paused.0 = !paused.0;
            match paused.0 {
                true => warn!("Pausing game"),
                false => warn!("Resuming game"),
            }

            let mut primary_window = q_windows.single_mut();
            if paused.0 {
                //rapier_context.
                primary_window.cursor.grab_mode = CursorGrabMode::None;
                primary_window.cursor.visible = true;
                time.pause();
            } else {
                primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor.visible = false;
                time.unpause();
            }
        }
    }

    pub fn ground_detection(
        rapier_context: Res<RapierContext>,
        mut query: Query<(&mut Player, &Transform)>,
    ) {
        for (mut player, trans) in query.iter_mut() {
            let collider_height = 0.01;
            let shape = Collider::cylinder(collider_height, player.radius);
            let mut shape_pos = trans.translation;
            shape_pos.y -= player.half_height + collider_height * 4.0;
            let shape_rot = Quat::default();
            let shape_vel = Vec3::new(0.0, -0.2, 0.0);
            let max_toi = 0.0;
            let filter = QueryFilter::default();
            let stop_at_penetration = true;

            player.on_ground = rapier_context
                .cast_shape(
                    shape_pos,
                    shape_rot,
                    shape_vel,
                    &shape,
                    max_toi,
                    stop_at_penetration,
                    filter,
                )
                .is_some();

            if player.on_ground {
                if let Some(air_time) = player.air_time {
                    if air_time.elapsed().as_secs_f32() > 0.01 {
                        println!("Airtime of {}s", air_time.elapsed().as_secs_f32());
                        player.air_time = None;
                    }
                }
            }
        }
    }
}
