use super::{Player, PlayerFpsMaterial, PlayerFpsModel};
use crate::{inputs::PlayerInput, map_gen::entities::data::SoundEffect, Paused};
use bevy::{
    ecs::schedule::SystemConfigs,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_rapier3d::{
    control::KinematicCharacterController, geometry::Collider, pipeline::QueryFilter,
    plugin::RapierContext,
};
use bevy_scene_hook::reload::{Hook, State as HookState};
use macros::{error_return, option_return};
use rand::seq::SliceRandom;

enum SwitchDirection {
    Back,
    Forward,
}
impl Player {
    pub fn systems() -> SystemConfigs {
        (
            Player::update_input,
            Player::update_cam_vert,
            Player::update_cam_hort,
            Player::ground_detection,
            Player::weaponry_switch,
            Player::weaponry_switch_wheel,
            Player::weaponry_switch_keys,
            Player::weapon_animations,
            Player::camera_movement,
            Player::shoot,
        )
            .into_configs()
    }

    pub fn shoot(
        mut q_players: Query<&mut Player>,
        keys: Res<PlayerInput>,
        time: Res<Time>,
        asset_server: Res<AssetServer>,
        audio: Res<Audio>,
    ) {
        for mut player in &mut q_players {
            let (slot, row) = option_return!(player.current_weapon);
            let weapon = &mut player.weapons[slot][row];
            weapon.timer -= time.delta_seconds();
            weapon.timer = weapon.timer.max(-1.0);
            weapon.anim_time -= time.delta_seconds();
            weapon.anim_time = weapon.anim_time.max(-1.0);
            weapon.reload_timer -= time.delta_seconds();
            weapon.reload_timer = weapon.reload_timer.max(-1.0);

            if weapon.timer > 0.0 {
                if weapon.reload_timer <= 0.0 && weapon.data.animations.reload.is_some() {
                    player.current_weapon_anim = "reload".to_string();
                }
                return;
            }
            weapon.need_to_reload = false;

            let mut shot = false;

            if keys.weapon_shoot2_pressed && !weapon.need_to_reload {
                weapon.timer = weapon.data.animations.fire_time2 + time.delta_seconds();
                weapon.anim_time = weapon.data.animations.anim_time2 + time.delta_seconds();
                if weapon.data.animations.reload.is_some() {
                    weapon.need_to_reload = true;
                    weapon.timer = weapon.data.animations.fire_time2
                        + weapon.data.animations.reload_time_skip
                        + time.delta_seconds();
                    weapon.reload_timer = weapon.data.animations.anim_time2 + time.delta_seconds();
                    weapon.anim_time += weapon.data.animations.reload_time + time.delta_seconds();
                }

                player.current_weapon_anim = "shoot2".to_string();
                shot = true;
            } else if keys.weapon_shoot1_pressed && !weapon.need_to_reload {
                weapon.timer = weapon.data.animations.fire_time1 + time.delta_seconds();
                weapon.anim_time = weapon.data.animations.anim_time1 + time.delta_seconds();
                if weapon.data.animations.reload.is_some() {
                    weapon.need_to_reload = true;
                    weapon.timer = weapon.data.animations.fire_time1
                        + weapon.data.animations.reload_time_skip
                        + time.delta_seconds();
                    weapon.reload_timer = weapon.data.animations.anim_time1 + time.delta_seconds();
                    weapon.anim_time += weapon.data.animations.reload_time + time.delta_seconds();
                }

                player.current_weapon_anim = "shoot1".to_string();
                shot = true;
            } else if weapon.anim_time <= 0.0 && player.current_weapon_anim != "idle" {
                player.current_weapon_anim = "idle".to_string();
            }

            if shot {
                player.restart_anim = true;
                match &player.weapons[slot][row].data.shoot_sfx {
                    SoundEffect::Single(path) => {
                        audio.play(asset_server.load(path));
                    }
                    SoundEffect::Random(list) => {
                        audio
                            .play(asset_server.load(list.choose(&mut rand::thread_rng()).unwrap()));
                    }
                    _ => {}
                }
            }
        }
    }

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
        keys: Res<PlayerInput>,
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
            if keys.walk_forward_pressed {
                player.velocity += forward * hort_speed * time.delta_seconds();
                player.camera_movement.backdrift_goal = player.camera_movement.backdrift_max;
            } else if keys.walk_backward_pressed {
                player.velocity -= forward * hort_speed * time.delta_seconds();
                player.camera_movement.backdrift_goal = -player.camera_movement.backdrift_max;
            } else {
                player.camera_movement.backdrift_goal = 0.0;
            }

            if keys.walk_left_pressed {
                player.velocity -= right * hort_speed * time.delta_seconds();
                player.camera_movement.cam_rot_goal = player.camera_movement.cam_rot_max_goal;
            } else if keys.walk_right_pressed {
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
                if keys.jump_just_pressed {
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

            if keys.debug_fly_up_pressed {
                gt.translation.y += 0.1;
            } else if keys.debug_fly_down_pressed {
                gt.translation.y -= 0.1;
            }
        }
    }

    pub fn weapon_animations(
        mut commands: Commands,
        mut players: Query<&mut Player>,
        q_player_fps_anims: Query<(Entity, &PlayerFpsModel, &Children)>,
        q_scenes: Query<&Children>,
        mut q_anim_players: Query<&mut AnimationPlayer>,
    ) {
        for mut player in &mut players {
            let (ent, _, children) = option_return!(q_player_fps_anims
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
                        if let Ok(mut anim_player) = q_anim_players.get_mut(*child) {
                            // now we have the animation player
                            let clip = &player.fps_anims[&player.current_weapon_anim];
                            if player.restart_anim {
                                anim_player.play(clip.clone()).replay();
                            } else if !anim_player.is_playing_clip(clip) {
                                anim_player.play(clip.clone()).repeat();
                            }
                            player.restart_anim = false;
                        }
                    }
                }
            }
        }
    }

    fn switch_weapon(&mut self, dir: SwitchDirection) {
        let inv_len = self.weapons.len() - 1;

        if let Some((mut slot, mut row)) = self.current_weapon {
            match dir {
                SwitchDirection::Back => {
                    if row == 0 {
                        match slot == 0 {
                            true => slot = inv_len,
                            false => slot -= 1,
                        }
                        while self.weapons[slot].is_empty() {
                            match slot == 0 {
                                true => slot = inv_len,
                                false => slot -= 1,
                            }
                            if !self.weapons[slot].is_empty() {
                                row = self.weapons[slot].len() - 1;
                            }
                        }
                    } else {
                        row -= 1;
                    };
                }
                SwitchDirection::Forward => {
                    if row + 1 == self.weapons[slot].len() {
                        match slot == inv_len {
                            true => slot = 0,
                            false => slot += 1,
                        }
                        while self.weapons[slot].is_empty() {
                            match slot == inv_len {
                                true => slot = 0,
                                false => slot += 1,
                            }
                            if !self.weapons[slot].is_empty() {
                                row = 0;
                            }
                        }
                    } else {
                        row += 1;
                    };
                }
            }

            info!("Currently using: {}", self.weapons[slot][row].data.id);
            self.current_weapon = Some((slot, row));
        }
    }

    pub fn weaponry_switch_wheel(keys: Res<PlayerInput>, mut query: Query<&mut Player>) {
        for mut player in query.iter_mut() {
            let dir = if keys.weapon_next_pressed {
                SwitchDirection::Forward
            } else if keys.weapon_previous_pressed {
                SwitchDirection::Back
            } else {
                continue;
            };
            player.switch_weapon(dir);
        }
    }

    pub fn weaponry_switch_keys(keys: Res<PlayerInput>, mut query: Query<&mut Player>) {
        for mut player in &mut query {
            // 🤫
            let slot = match () {
                _ if keys.weapon_slot1_just_pressed => 0,
                _ if keys.weapon_slot2_just_pressed => 1,
                _ if keys.weapon_slot3_just_pressed => 2,
                _ if keys.weapon_slot4_just_pressed => 3,
                _ if keys.weapon_slot5_just_pressed => 4,
                _ if keys.weapon_slot6_just_pressed => 5,
                _ if keys.weapon_slot7_just_pressed => 6,
                _ if keys.weapon_slot8_just_pressed => 7,
                _ if keys.weapon_slot9_just_pressed => 8,
                _ if keys.weapon_slot10_just_pressed => 9,
                _ => continue,
            };

            if player.weapons[slot].is_empty() {
                continue;
            }

            let (old_slot, row) = option_return!(player.current_weapon);
            if slot == old_slot && player.weapons[slot].len() != row + 1 {
                player.switch_weapon(SwitchDirection::Forward);
            }
            if slot == old_slot && player.weapons[slot].len() == row + 1 {
                player.current_weapon = Some((old_slot, 0))
            } else {
                player.current_weapon = Some((slot, 0));
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
                &mut PlayerFpsMaterial,
                &mut Hook,
            ),
            With<PlayerFpsModel>,
        >,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        //println!("{:#?}", keys);
        for mut player in query.iter_mut() {
            if player.current_weapon == player.current_weapon_old {
                continue;
            }
            player.current_weapon_old = player.current_weapon;
            if let Some((slot, row)) = player.current_weapon {
                // TODO replace these with proper gets.
                if let Ok((ent, mut mesh, mut trans, mut mat, mut hook)) =
                    q_model.get_mut(option_return!(player.children.fps_model))
                    && !player.weapons[slot][row].data.model_file.is_empty()
                {
                    commands.entity(ent).despawn_descendants();

                    let new_mesh = player.weapons[slot][row].mesh.clone();

                    let data = &player.weapons[slot][row].data;

                    let new_mat = StandardMaterial {
                        base_color_texture: Some(asset_server.load(&data.texture_file)),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    };
                    materials.remove(mat.0.id());
                    mat.0 = materials.add(new_mat);

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

                    player.fps_anims = [
                        (
                            "idle",
                            asset_server
                                .load(&format!("{}#{}", data.model_file, data.animations.idle)),
                        ),
                        (
                            "shoot1",
                            asset_server
                                .load(&format!("{}#{}", data.model_file, data.animations.shoot1)),
                        ),
                        (
                            "shoot2",
                            asset_server
                                .load(&format!("{}#{}", data.model_file, data.animations.shoot2)),
                        ),
                    ]
                    .into_iter()
                    .map(|(a, b)| (a.to_string(), b))
                    .collect();
                    if let Some(reload) = player.weapons[slot][row].data.animations.reload.clone() {
                        let asset = asset_server.load(&format!(
                            "{}#{}",
                            player.weapons[slot][row].data.model_file, reload
                        ));
                        player.fps_anims.insert("reload".to_string(), asset);
                    }

                    if player.weapons[slot][row].need_to_reload {
                        player.weapons[slot][row].anim_time =
                            player.weapons[slot][row].data.animations.reload_time;
                        player.weapons[slot][row].timer =
                            player.weapons[slot][row].data.animations.reload_time_skip;
                    } else {
                        player.current_weapon_anim = "idle".to_string();
                    }
                    player.camera_movement.original_trans = trans.translation;
                    player.camera_movement.switch_offset = -1.0;
                    hook.state = HookState::MustReload;
                }
            }
        }

        // for model in model_query.iter() {
        //     println!("{:?}", model);
        // }
    }

    pub fn pause_handler(
        keys: Res<PlayerInput>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        mut paused: ResMut<Paused>,
        mut time: ResMut<Time<Virtual>>,
    ) {
        if keys.pause_game_just_pressed || keys.pause_game_alt_just_pressed {
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
