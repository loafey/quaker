use super::Player;
use crate::Paused;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

impl Player {
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

    pub fn pause_handler(
        keys: Res<ButtonInput<KeyCode>>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        mut paused: ResMut<Paused>,
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
            } else {
                primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor.visible = false;
            }
        }
    }
}
