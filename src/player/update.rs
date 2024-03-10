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

enum SwitchDirection {
    Back,
    Forward,
}
impl Player {
    pub fn update_cam_vert(
        mut query: Query<(&Camera3d, &mut Transform)>,
        q_parent: Query<(&Player, &Children)>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (_, children) in q_parent.iter() {
            for &child in children.iter() {
                if let Ok((_, mut trans)) = query.get_mut(child) {
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
            // movement
            let local_z = gt.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            let hort_speed = player.hort_speed;
            if keys.pressed(KeyCode::KeyW) {
                player.velocity += forward * hort_speed * time.delta_seconds();
            } else if keys.pressed(KeyCode::KeyS) {
                player.velocity -= forward * hort_speed * time.delta_seconds();
            }

            if keys.pressed(KeyCode::KeyA) {
                player.velocity -= right * hort_speed * time.delta_seconds();
            } else if keys.pressed(KeyCode::KeyD) {
                player.velocity += right * hort_speed * time.delta_seconds();
            }
            player.velocity.x = player.velocity.x.clamp(-5.0, 5.0);
            player.velocity.z = player.velocity.z.clamp(-5.0, 5.0);

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

    //Player <- FPsModel <- Scene thing <- Mesh <- AnimationPlayer
    pub fn weapon_animations(
        players: Query<(&Player, &Children)>,
        cam_query: Query<(&Camera3d, &Children)>,
        player_fps_anims: Query<(&PlayerFpsAnimations, &Children)>,
        scenes: Query<&Children>,
        mut anim_players: Query<&mut AnimationPlayer>,
    ) {
        for (player, children) in &players {
            for child in children {
                if let Ok((_, children)) = cam_query.get(*child) {
                    // Got camera
                    for child in children {
                        if let Ok((anims, children)) = player_fps_anims.get(*child) {
                            // Got FPS model entity
                            for child in children {
                                if let Ok(children) = scenes.get(*child) {
                                    // Got GLTF scene
                                    for child in children {
                                        if let Ok(mut anim_player) = anim_players.get_mut(*child) {
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
                }
            }
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn weaponry_switch(
        mut query: Query<(&mut Player, &Children)>,
        cam_query: Query<(&Camera3d, &Children)>,
        mut model_query: Query<
            (
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
        for (mut player, children) in query.iter_mut() {
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

                    for child in children.iter() {
                        if let Ok((_, children)) = cam_query.get(*child) {
                            for child in children.iter() {
                                if let Ok((mut mesh, mut trans, mut anims, mut mat, mut hook)) =
                                    model_query.get_mut(*child)
                                    && !player.weapons[slot][row].model_file.is_empty()
                                {
                                    let data = &player.weapons[slot][row];
                                    let new_mesh =
                                        asset_server.load(&format!("{}#Scene0", data.model_file));

                                    let new_mat = StandardMaterial {
                                        base_color_texture: Some(
                                            asset_server.load(&data.texture_file),
                                        ),
                                        perceptual_roughness: 1.0,
                                        reflectance: 0.0,
                                        ..Default::default()
                                    };
                                    mat.0 = materials.add(new_mat);

                                    anims.0.insert(
                                        "idle".to_string(),
                                        asset_server.load(&format!(
                                            "{}#{}",
                                            data.model_file, data.animations.idle
                                        )),
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
                                    hook.state = HookState::MustReload;
                                }
                            }
                        }
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
