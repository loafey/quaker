use super::{Player, PlayerFpsAnimations, PlayerFpsMaterial, PlayerFpsModel};
use crate::Paused;
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{
    control::KinematicCharacterController, geometry::Collider, pipeline::QueryFilter,
    plugin::RapierContext,
};
use bevy_scene_hook::reload::{Hook, State as HookState};
use macros::{error_return, option_return};

enum SwitchDirection {
    Back,
    Forward,
}
impl Player {
    pub fn camera_movement(
        mut q_cam: Query<(&Camera3d, &mut Transform)>,
        mut q_model: Query<(&PlayerFpsModel, &mut Transform), Without<Camera3d>>,
        mut q_parent: Query<&mut Player>,
        time: Res<Time>,
    ) {
        for mut player in q_parent.iter_mut() {
            player.camera_movement.cam_rot_current = player.camera_movement.cam_rot_current.lerp(
                player.camera_movement.cam_rot_goal,
                time.delta_seconds() * 10.0,
            );

            player.camera_movement.backdrift = player.camera_movement.backdrift.lerp(
                player.camera_movement.backdrift_goal,
                time.delta_seconds() * 10.0,
            );

            player.camera_movement.bob_current = player.camera_movement.bob_current.lerp(
                player.camera_movement.bob_goal.sin(),
                time.delta_seconds() * 10.0,
            );

            player.camera_movement.switch_offset = player
                .camera_movement
                .switch_offset
                .lerp(0.0, time.delta_seconds() * 10.0);

            let (_, mut cam_trans) =
                error_return!(q_cam.get_mut(option_return!(player.children.camera)));
            cam_trans.rotation.z = player.camera_movement.cam_rot_current;

            let (_, mut trans) =
                error_return!(q_model.get_mut(option_return!(player.children.fps_model)));
            let mut new_trans = player.camera_movement.original_trans;
            new_trans.z += player.camera_movement.backdrift;
            new_trans.x += player.camera_movement.cam_rot_current / 2.0;
            new_trans.y += player.camera_movement.bob_current / 100.0;
            new_trans.y += player.camera_movement.switch_offset;

            trans.translation = new_trans;
        }
    }

    pub fn update_cam_vert(
        mut query: Query<(&Camera3d, &mut Transform)>,
        q_parent: Query<&Player>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for player in q_parent.iter() {
            if let Ok((_, mut trans)) = query.get_mut(option_return!(player.children.camera)) {
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

    pub fn update_input(
        keys: Res<ButtonInput<KeyCode>>,
        time: Res<Time>,
        mut query: Query<(
            &mut KinematicCharacterController,
            &mut Player,
            &mut Transform,
        )>,
    ) {
        for (mut controller, mut player, mut gt) in &mut query {
            // movement
            let local_z = gt.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            let hort_speed = player.hort_speed;
            if keys.pressed(KeyCode::KeyW) {
                player.velocity += forward * hort_speed * time.delta_seconds();
                player.camera_movement.backdrift_goal = player.camera_movement.backdrift_max;
            } else if keys.pressed(KeyCode::KeyS) {
                player.velocity -= forward * hort_speed * time.delta_seconds();
                player.camera_movement.backdrift_goal = -player.camera_movement.backdrift_max;
            } else {
                player.camera_movement.backdrift_goal = 0.0;
            }

            if keys.pressed(KeyCode::KeyA) {
                player.velocity -= right * hort_speed * time.delta_seconds();
                player.camera_movement.cam_rot_goal = player.camera_movement.cam_rot_max_goal;
            } else if keys.pressed(KeyCode::KeyD) {
                player.velocity += right * hort_speed * time.delta_seconds();
                player.camera_movement.cam_rot_goal = -player.camera_movement.cam_rot_max_goal;
            } else {
                player.camera_movement.cam_rot_goal = 0.0;
            }

            player.camera_movement.backdrift_goal += (player.velocity.y.abs() / 5.0).min(0.03);

            player.velocity.x = player.velocity.x.clamp(-5.0, 5.0);
            player.velocity.z = player.velocity.z.clamp(-5.0, 5.0);

            if player.velocity != Vec3::ZERO {
                player.camera_movement.bob_goal += time.delta_seconds()
                    * (Vec3::new(player.velocity.x, 0.0, player.velocity.z)
                        .abs()
                        .length()
                        - player.velocity.y.abs())
                    .max(0.0)
                    * 2.0;
                if player.camera_movement.bob_goal > std::f32::consts::PI * 2.0 {
                    player.camera_movement.bob_goal -= std::f32::consts::PI * 2.0;
                }
            } else {
                player.camera_movement.bob_goal = 0.0;
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

    pub fn weapon_animations(
        mut commands: Commands,
        players: Query<&Player>,
        q_player_fps_anims: Query<(Entity, &PlayerFpsAnimations, &Children)>,
        q_scenes: Query<&Children>,
        mut q_anim_players: Query<(Entity, &mut AnimationPlayer, &Parent)>,
    ) {
        for player in &players {
            let (ent, anims, children) = option_return!(q_player_fps_anims
                .get(option_return!(player.children.fps_model))
                .ok());

            if children.len() > 1 {
                let mut to_remove = Vec::new();
                for child in children.iter().rev().skip(1) {
                    if let Some(child) = commands.get_entity(*child) {
                        to_remove.push(child.id());
                        child.despawn_recursive();
                    }
                }
                commands.entity(ent).remove_children(&to_remove);
            }

            for child in children {
                if let Ok(children) = q_scenes.get(*child) {
                    // Got GLTF scene
                    for child in children {
                        if let Ok((_, mut anim_player, _)) = q_anim_players.get_mut(*child) {
                            // now we have the animation player
                            let clip = &anims.0[&player.current_weapon_anim];
                            if !anim_player.is_playing_clip(clip) {
                                anim_player.play(clip.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn weaponry_switch(
        mut commands: Commands,
        mut query: Query<&mut Player>,
        mut q_model: Query<
            (
                Entity,
                &mut Handle<Scene>,
                &mut Transform,
                &mut PlayerFpsAnimations,
                &mut PlayerFpsMaterial,
                &mut Hook,
            ),
            With<PlayerFpsModel>,
        >,
        mut mouse_wheel: EventReader<MouseWheel>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        for mut player in query.iter_mut() {
            for ev in mouse_wheel.read() {
                let inv_len = player.weapons.len() - 1;
                if let Some((mut slot, mut row)) = player.current_weapon {
                    let dir = if ev.y < 0.0 {
                        SwitchDirection::Back
                    } else {
                        SwitchDirection::Forward
                    };
                    loop {
                        match dir {
                            SwitchDirection::Back => {
                                if slot == 0 {
                                    slot = inv_len;
                                } else {
                                    slot -= 1;
                                }
                                if !player.weapons[slot].is_empty() {
                                    row = player.weapons[slot].len() - 1;
                                    break;
                                }
                            }
                            SwitchDirection::Forward => {
                                if slot == inv_len {
                                    slot = 0;
                                } else {
                                    slot += 1;
                                }
                                if !player.weapons[slot].is_empty() {
                                    row = 0;
                                    break;
                                }
                            }
                        }
                    }

                    info!("Currently using: {}", player.weapons[slot][row].id);
                    player.current_weapon = Some((slot, row));

                    // TODO replace these with proper gets.
                    if let Ok((ent, mut mesh, mut trans, mut anims, mut mat, mut hook)) =
                        q_model.get_mut(option_return!(player.children.fps_model))
                        && !player.weapons[slot][row].model_file.is_empty()
                    {
                        commands.entity(ent).despawn_descendants();

                        let data = &player.weapons[slot][row];
                        let new_mesh = asset_server.load(&format!("{}#Scene0", data.model_file));

                        let new_mat = StandardMaterial {
                            base_color_texture: Some(asset_server.load(&data.texture_file)),
                            perceptual_roughness: 1.0,
                            reflectance: 0.0,
                            ..Default::default()
                        };
                        materials.remove(mat.0.id());
                        mat.0 = materials.add(new_mat);

                        anims.0.insert(
                            "idle".to_string(),
                            asset_server
                                .load(&format!("{}#{}", data.model_file, data.animations.idle)),
                        );
                        //anim_player
                        //    .play(asset_server.load(&format!("{}#Animation0", data.model_file)))
                        //    .repeat();
                        trans.scale = Vec3::splat(data.scale);
                        trans.rotation = Quat::from_euler(
                            EulerRot::XYZ,
                            data.rotation[0].to_radians(),
                            data.rotation[1].to_radians(),
                            data.rotation[2].to_radians(),
                        );
                        trans.translation = Vec3::from(data.offset);
                        *mesh = new_mesh;
                        player.current_weapon_anim = "idle".to_string();
                        player.camera_movement.original_trans = trans.translation;
                        player.camera_movement.switch_offset = -1.0;
                        hook.state = HookState::MustReload;
                    }
                }
            }
        }

        // for model in model_query.iter() {
        //     println!("{:?}", model);
        // }
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
