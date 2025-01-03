#![allow(clippy::missing_transmute_annotations)]

use super::{
    ARMOR_GLYPH, HEALTH_GLYPH, Player, PlayerController, PlayerFpsMaterial, PlayerFpsModel,
    PlayerMpModel, WeaponState,
};
use crate::{
    entities::ProjectileEntity,
    net::{ClientMessage, Lobby},
};
use bevy::{
    audio::Volume,
    ecs::schedule::SystemConfigs,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_rapier3d::{
    control::KinematicCharacterController, geometry::Collider, pipeline::QueryFilter,
    plugin::RapierContext, prelude::ShapeCastOptions,
};
use bevy_scene_hook::reload::{Hook, State as HookState};
use faststr::FastStr;
use macros::{error_continue, option_continue, option_return};
use resources::{
    Paused,
    data::{Attack, Projectiles, SoundEffect},
    entropy::{EGame, EMisc, Entropy},
    inputs::PlayerInput,
};
use std::{fmt::Write, mem::transmute};

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
            Player::update_hud,
            Player::update_interact,
        )
            .into_configs()
    }

    fn set_anim(weapon: &mut WeaponState, fire_time: f32, anim_time: f32, time: &Time) {
        weapon.timer = fire_time + time.delta_secs();
        weapon.anim_time = anim_time + time.delta_secs();
        if weapon.data.animations.reload.is_some() {
            weapon.need_to_reload = true;
            weapon.timer = fire_time + weapon.data.animations.reload_time_skip + time.delta_secs();
            weapon.reload_timer = anim_time + time.delta_secs();
            weapon.anim_time += weapon.data.animations.reload_time + time.delta_secs();
        }
    }

    pub fn update_hud(
        q_players: Query<&Player, With<PlayerController>>,
        mut text: Query<(&mut Text, &mut Visibility)>,
        input: Res<PlayerInput>,
        lobby: Res<Lobby>,
    ) {
        for player in &q_players {
            let ammo_hud = option_continue!(player.children.ammo_hud);
            let _ammo_hud = error_continue!(text.get_mut(ammo_hud));

            let health_hud = option_continue!(player.children.health_hud);
            let (mut health_hud, _) = error_continue!(text.get_mut(health_hud));
            health_hud.0 = format!("{HEALTH_GLYPH}{}", player.health.round());

            let armour_hud = option_continue!(player.children.armour_hud);
            let (mut armour_hud, _) = error_continue!(text.get_mut(armour_hud));
            armour_hud.0 = format!("{ARMOR_GLYPH}{}", player.armour.round());

            let lobby_hud = option_continue!(player.children.lobby_hud);
            let (mut text, mut vis) = error_continue!(text.get_mut(lobby_hud));
            if input.show_lobby_just_pressed {
                *vis = Visibility::Visible;
                text.0 = format!(
                    "Players:\n{}",
                    lobby.values().fold(String::new(), |mut output, i| {
                        let _ = writeln!(output, "{}: K={}, D={}", i.name, i.kills, i.deaths);
                        output
                    })
                )
            }
            if input.show_lobby_just_released {
                *vis = Visibility::Hidden;
            }
        }
    }

    pub fn shoot(
        mut commands: Commands,
        mut q_players: Query<(Entity, &mut Player, &Transform), With<PlayerController>>,
        mut misc_entropy: ResMut<Entropy<EMisc>>,
        keys: Res<PlayerInput>,
        time: Res<Time>,
        asset_server: Res<AssetServer>,
        mut client_events: EventWriter<ClientMessage>,
    ) {
        for (player_ent, mut player, _) in &mut q_players {
            let (slot, row) = option_continue!(player.current_weapon);
            // let cur = player.current_weapon_anim.clone();
            let weapon = &mut player.weapons[slot][row];
            weapon.timer -= time.delta_secs();
            weapon.timer = weapon.timer.max(-1.0);
            weapon.anim_time -= time.delta_secs();
            weapon.anim_time = weapon.anim_time.max(-1.0);
            weapon.reload_timer -= time.delta_secs();
            weapon.reload_timer = weapon.reload_timer.max(-1.0);
            // println!(
            //     "timer {}:\n\t{:?}\n\t{}\n\t{}",
            //     cur, weapon.timer, weapon.anim_time, weapon.reload_timer
            // );
            if weapon.timer > 0.0 {
                if weapon.reload_timer <= 0.0 && weapon.data.animations.reload.is_some() {
                    if player.current_weapon_anim != "reload" {
                        player.restart_anim = true;
                    }
                    player.current_weapon_anim = FastStr::from("reload");
                }
                return;
            }
            weapon.need_to_reload = false;

            let mut shot = false;

            if keys.weapon_shoot2_pressed && !weapon.need_to_reload {
                player.attack2(&time, &mut client_events);
                shot = true;
            } else if keys.weapon_shoot1_pressed && !weapon.need_to_reload {
                player.attack1(&time, &mut client_events);
                shot = true;
            } else if weapon.anim_time <= 0.0 && player.current_weapon_anim != "idle" {
                player.current_weapon_anim = FastStr::from("idle");
                player.restart_anim = true;
            }

            if shot {
                player.restart_anim = true;
                let sound = match &player.weapons[slot][row].data.shoot_sfx {
                    SoundEffect::Single(path) => Some(path.clone()),
                    SoundEffect::Random(list) if !list.is_empty() => {
                        Some(misc_entropy.choose(list).clone())
                    }
                    _ => None,
                };
                if let Some(sound) = sound {
                    let sound_holder = player.children.shoot_sound_holder.unwrap_or(player_ent);

                    commands.entity(sound_holder).with_children(|c| {
                        c.spawn((
                            Transform::IDENTITY,
                            AudioPlayer::<AudioSource>(asset_server.load(sound.to_string())),
                            PlaybackSettings::DESPAWN
                                .with_spatial(true)
                                .with_volume(Volume::new(0.8)),
                        ));
                    });
                }
            }
        }
    }

    pub fn camera_movement(
        mut q_cam: Query<(&Camera3d, &mut Transform)>,
        mut q_model: Query<(&PlayerFpsModel, &mut Transform), Without<Camera3d>>,
        mut q_parent: Query<&mut Player, With<PlayerController>>,
        time: Res<Time>,
    ) {
        for mut player in q_parent.iter_mut() {
            player.camera_movement.cam_rot_current = player.camera_movement.cam_rot_current.lerp(
                player.camera_movement.cam_rot_goal,
                time.delta_secs() * 10.0,
            );

            player.camera_movement.backdrift = player.camera_movement.backdrift.lerp(
                player.camera_movement.backdrift_goal,
                time.delta_secs() * 10.0,
            );

            player.camera_movement.bob_current = player.camera_movement.bob_current.lerp(
                player.camera_movement.bob_goal.sin(),
                time.delta_secs() * 10.0,
            );

            player.camera_movement.switch_offset = player
                .camera_movement
                .switch_offset
                .lerp(0.0, time.delta_secs() * 10.0);

            let (_, mut cam_trans) =
                error_continue!(q_cam.get_mut(option_continue!(player.children.camera)));
            cam_trans.rotation.z = player.camera_movement.cam_rot_current;

            let (_, mut trans) =
                error_continue!(q_model.get_mut(option_continue!(player.children.fps_model)));
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
        q_parent: Query<&Player, With<PlayerController>>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for player in q_parent.iter() {
            if let Ok((_, mut trans)) = query.get_mut(option_continue!(player.children.camera)) {
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

    pub fn update_cam_hort(
        mut query: Query<(&Player, &mut Transform), With<PlayerController>>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (_, mut gt) in &mut query {
            // handle cursor
            for ev in motion_evr.read() {
                let x_delta = ev.delta.x / -1000.0;
                gt.rotate_y(x_delta);
                //gt.rotate_local_x(ev.delta.y / -1000.0);
            }
        }
    }

    pub fn update_interact(
        keys: Res<PlayerInput>,
        mut query: Query<
            (
                &mut KinematicCharacterController,
                &mut Player,
                &mut Transform,
            ),
            With<PlayerController>,
        >,
        mut client_events: EventWriter<ClientMessage>,
    ) {
        for _ in &mut query {
            if keys.interact_just_pressed {
                client_events.send(ClientMessage::Interact);
            }
        }
    }

    pub fn update_input(
        keys: Res<PlayerInput>,
        time: Res<Time>,
        mut query: Query<
            (
                &mut KinematicCharacterController,
                &mut Player,
                &mut Transform,
            ),
            With<PlayerController>,
        >,
        cameras: Query<(&Camera3d, &Transform), Without<PlayerController>>,
        mut events: EventWriter<ClientMessage>,
    ) {
        for (mut controller, mut player, gt) in &mut query {
            // movement
            let local_z = gt.local_z();
            let forward = -Vec3::new(local_z.x, 0., local_z.z);
            let right = Vec3::new(local_z.z, 0., -local_z.x);

            let hort_speed = player.hort_speed;
            if keys.walk_forward_pressed {
                player.velocity += forward * hort_speed * time.delta_secs();
                player.camera_movement.backdrift_goal = player.camera_movement.backdrift_max;
            } else if keys.walk_backward_pressed {
                player.velocity -= forward * hort_speed * time.delta_secs();
                player.camera_movement.backdrift_goal = -player.camera_movement.backdrift_max;
            } else {
                player.camera_movement.backdrift_goal = 0.0;
            }

            if keys.walk_left_pressed {
                player.velocity -= right * hort_speed * time.delta_secs();
                player.camera_movement.cam_rot_goal = player.camera_movement.cam_rot_max_goal;
            } else if keys.walk_right_pressed {
                player.velocity += right * hort_speed * time.delta_secs();
                player.camera_movement.cam_rot_goal = -player.camera_movement.cam_rot_max_goal;
            } else {
                player.camera_movement.cam_rot_goal = 0.0;
            }

            player.camera_movement.backdrift_goal += (player.velocity.y.abs() / 5.0).min(0.03);

            //player.velocity.x = player.velocity.x.clamp(-5.0, 5.0);
            //player.velocity.z = player.velocity.z.clamp(-5.0, 5.0);

            if player.velocity != Vec3::ZERO {
                player.camera_movement.bob_goal += time.delta_secs()
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
                player.velocity.y += time.delta_secs() * player.jump_timer * player.gravity;
                player.jump_timer -= time.delta_secs() * 50.0;
                player.jump_timer = player.jump_timer.clamp(-0.1, 1.0);
            }

            controller.translation = Some(player.velocity);

            let x = player.velocity.x;
            let z = player.velocity.z;
            player.velocity = Vec3::new(
                x.lerp(0.0, player.hort_friction)
                    .clamp(-player.hort_max_speed, player.hort_max_speed),
                player.velocity.y,
                z.lerp(0.0, player.hort_friction)
                    .clamp(-player.hort_max_speed, player.hort_max_speed),
            );

            events.send(ClientMessage::UpdatePosition {
                position: gt.translation,
                rotation: gt.rotation.into(),
                cam_rot: player
                    .children
                    .camera
                    .and_then(|cam| cameras.get(cam).ok())
                    .map(|(_, t)| t.rotation.x)
                    .unwrap_or_default(),
            });

            player.debug_info.current_speed = Vec2::new(x, z).distance(Vec2::ZERO);
            player.debug_info.current_falling = player.velocity.y;
        }
    }

    pub fn weapon_animations(
        mut commands: Commands,
        mut players: Query<&mut Player>,
        q_player_fps_anims: Query<(Entity, &PlayerFpsModel, &Children)>,
        q_scenes: Query<&Children>,
        mut q_anim_players: Query<(
            Entity,
            &mut AnimationPlayer,
            Option<&mut AnimationGraphHandle>,
        )>,
        mut client_events: EventWriter<ClientMessage>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
    ) {
        for mut player in &mut players {
            let (ent, _, children) = option_continue!(
                q_player_fps_anims
                    .get(option_continue!(player.children.fps_model))
                    .ok()
            );

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
                        if let Ok((par, mut anim_player, anim_graph)) =
                            q_anim_players.get_mut(*child)
                        {
                            if let Some(p) = &player.fps_anim_graph {
                                commands
                                    .entity(par)
                                    .try_insert(AnimationGraphHandle(graphs.add(p.clone())));
                                player.fps_anim_graph_insert_count =
                                    player.fps_anim_graph_insert_count.wrapping_add(1);
                            }
                            if anim_graph.is_some() && player.fps_anim_graph_insert_count > 1 {
                                player.fps_anim_graph = None;
                            }

                            // println!("{}", player.current_weapon_anim);
                            if player.current_weapon_anim != player.current_weapon_anim_old
                                || player.restart_anim
                            {
                                player.current_weapon_anim_old = player.current_weapon_anim.clone();
                                client_events.send(ClientMessage::WeaponAnim {
                                    anim: player.current_weapon_anim.clone(),
                                });
                            }

                            // now we have the animation player
                            if let Some(clip) = player.fps_anims.get(&player.current_weapon_anim) {
                                // println!("play animation");
                                let clip = unsafe { transmute(*clip) };
                                // error!("broken animation!");
                                // println!(
                                //     "{}: {:?}",
                                //     player.current_weapon_anim,
                                //     anim_player.is_playing_animation(clip)
                                // );
                                if player.restart_anim {
                                    anim_player.play(clip).replay();
                                } else if !anim_player.is_playing_animation(clip) {
                                    anim_player.play(clip);
                                }
                                player.restart_anim = false;
                            }
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
            self.current_weapon = Some((slot, row));
        }
    }

    pub fn weaponry_switch_wheel(
        keys: Res<PlayerInput>,
        mut query: Query<&mut Player, With<PlayerController>>,
        mut client_events: EventWriter<ClientMessage>,
    ) {
        for mut player in query.iter_mut() {
            let dir = if keys.weapon_next_pressed {
                SwitchDirection::Forward
            } else if keys.weapon_previous_pressed {
                SwitchDirection::Back
            } else {
                continue;
            };
            player.switch_weapon(dir);

            if let Some((slot, row)) = player.current_weapon {
                client_events.send(ClientMessage::SwitchWeapon { slot, row });
            }
        }
    }

    pub fn weaponry_switch_keys(
        keys: Res<PlayerInput>,
        mut query: Query<&mut Player, With<PlayerController>>,
        mut client_events: EventWriter<ClientMessage>,
    ) {
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

            let (old_slot, row) = option_continue!(player.current_weapon);
            if slot == old_slot && player.weapons[slot].len() != row + 1 {
                player.switch_weapon(SwitchDirection::Forward);
            } else if slot == old_slot && player.weapons[slot].len() == row + 1 {
                player.current_weapon = Some((old_slot, 0))
            } else {
                player.current_weapon = Some((slot, 0));
            }

            client_events.send(ClientMessage::SwitchWeapon { slot, row });
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn weaponry_switch(
        mut commands: Commands,
        mut query: Query<&mut Player>,
        mut q_model: Query<
            (
                Entity,
                &mut SceneRoot,
                &mut Transform,
                &mut PlayerFpsMaterial,
                &mut Hook,
            ),
            (With<PlayerFpsModel>, Without<PlayerMpModel>),
        >,
        mut materials: ResMut<Assets<StandardMaterial>>,
        asset_server: Res<AssetServer>,
    ) {
        for mut player in query.iter_mut() {
            if player.current_weapon == player.current_weapon_old {
                continue;
            }
            player.current_weapon_old = player.current_weapon;
            if let Some((slot, row)) = player.current_weapon {
                // TODO replace these with proper gets.
                if let Ok((ent, mut mesh, mut trans, mut mat, mut hook)) =
                    q_model.get_mut(option_continue!(player.children.fps_model))
                    && !player.weapons[slot][row].data.model_file.is_empty()
                {
                    commands.entity(ent).despawn_descendants();

                    let new_mesh = player.weapons[slot][row].mesh.clone();

                    let data = &player.weapons[slot][row].data;

                    let new_mat = StandardMaterial {
                        base_color_texture: Some(asset_server.load(data.texture_file.to_string())),
                        perceptual_roughness: 1.0,
                        reflectance: 0.0,
                        ..default()
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

                    let path = new_mesh.path().unwrap();
                    let mut clips = vec![
                        asset_server
                            .load(GltfAssetLabel::Animation(data.animations.idle).from_asset(path)),
                        asset_server.load(
                            GltfAssetLabel::Animation(data.animations.shoot1).from_asset(path),
                        ),
                        asset_server.load(
                            GltfAssetLabel::Animation(data.animations.shoot2).from_asset(path),
                        ),
                    ];
                    if let Some(r) = player.weapons[slot][row].data.animations.reload {
                        clips
                            .push(asset_server.load(GltfAssetLabel::Animation(r).from_asset(path)));
                    }
                    let (graph, node_indices) = AnimationGraph::from_clips(clips);

                    player.fps_anims = ["idle", "shoot1", "shoot2", "reload"]
                        .into_iter()
                        .zip(node_indices)
                        .map(|(a, b)| (FastStr::from(a), unsafe { transmute(b) }))
                        .collect();

                    player.fps_anim_graph = Some(graph);
                    player.fps_anim_graph_insert_count = 0;

                    if player.weapons[slot][row].need_to_reload {
                        player.weapons[slot][row].anim_time =
                            player.weapons[slot][row].data.animations.reload_time;
                        player.weapons[slot][row].timer =
                            player.weapons[slot][row].data.animations.reload_time_skip;
                    } else {
                        player.current_weapon_anim = FastStr::from("idle");
                    }
                    player.camera_movement.original_trans = trans.translation;
                    player.camera_movement.switch_offset = -1.0;
                    *mesh = SceneRoot(new_mesh);
                    // commands.entity(ent).insert(SceneRoot(new_mesh));
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
        //mut time: ResMut<Time<Virtual>>,
    ) {
        if keys.pause_game_just_pressed || keys.pause_game_alt_just_pressed {
            paused.0 = !paused.0;
            match paused.0 {
                true => info!("Pausing game"),
                false => info!("Resuming game"),
            }

            let mut primary_window = q_windows.single_mut();
            if paused.0 {
                //rapier_context.
                primary_window.cursor_options.grab_mode = CursorGrabMode::None;
                primary_window.cursor_options.visible = true;
                //time.pause();
            } else {
                primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
                primary_window.cursor_options.visible = false;
                //time.unpause();
            }
        }
    }

    pub fn ground_detection(
        mut query: Query<(&mut Player, &Transform), With<PlayerController>>,
        rapier_context: Query<&RapierContext>,
    ) {
        let rapier_context = rapier_context.single();
        for (mut player, trans) in query.iter_mut() {
            let collider_height = 0.01;
            let shape = Collider::cylinder(collider_height, player.radius);
            let mut shape_pos = trans.translation;
            shape_pos.y -= player.half_height + collider_height * 4.0;
            let shape_rot = Quat::default();
            let shape_vel = Vec3::new(0.0, -0.2, 0.0);
            let max_time_of_impact = 0.0;
            let filter = QueryFilter::default();
            let stop_at_penetration = true;

            player.on_ground = rapier_context
                .cast_shape(
                    shape_pos,
                    shape_rot,
                    shape_vel,
                    &shape,
                    ShapeCastOptions {
                        max_time_of_impact,
                        stop_at_penetration,
                        ..default()
                    },
                    filter,
                )
                .is_some();

            if player.on_ground {
                if let Some(air_time) = player.air_time {
                    if air_time.elapsed().as_secs_f32() > 0.01 {
                        player.debug_info.last_airtime = air_time.elapsed().as_secs_f32();
                        player.air_time = None;
                    }
                }
            }
        }
    }
}

impl Player {
    #[allow(clippy::too_many_arguments)]
    pub fn attack(
        &mut self,
        attack: usize,
        materials: &mut Assets<StandardMaterial>,
        player_entity: Entity,
        commands: &mut Commands,
        rapier_context: &RapierContext,
        cam_trans: &Transform,
        player_trans: &Transform,
        game_entropy: &mut Entropy<EGame>,
        projectile_map: &Projectiles,
        asset_server: &AssetServer,
    ) -> Vec<(Entity, Vec3)> {
        let (slot, row) = option_return!(self.current_weapon);
        let attack = match attack {
            1 => &self.weapons[slot][row].data.attack1,
            2 => &self.weapons[slot][row].data.attack2,
            x => {
                error!("unsupported attack: {x}");
                return Vec::new();
            }
        };

        let origin = player_trans.translation + cam_trans.translation;
        let (_, rot, _) = player_trans.rotation.to_euler(EulerRot::XYZ);
        let Vec3 { x, y, z } = cam_trans.local_z().xyz();

        let sign = player_trans.rotation.xyz();
        let temp_origin = Vec3::new(origin.x, sign.y, origin.z);
        let sign = sign.dot(temp_origin).abs();
        let sign = match sign < 0.5 {
            true => 1.0,
            false => -1.0,
        };

        let dir = -Vec3::new(
            sign * x * rot.cos() + z * rot.sin(),
            y,
            -x * rot.sin() + sign * z * rot.cos(),
        );

        match attack {
            Attack::RayCast {
                amount,
                angle_mod,
                damage: _,
                damage_mod: _,
                range,
            } => {
                let angle_mod = angle_mod.to_radians();

                let mut hits = Vec::new();

                for _ in 0..*amount {
                    let dir = {
                        // https://www.desmos.com/3d/7fc69315ec
                        let angle_offsets = Vec3::new(
                            game_entropy.get_f32() * angle_mod * 2.0 - angle_mod,
                            game_entropy.get_f32() * angle_mod * 2.0 - angle_mod,
                            game_entropy.get_f32() * angle_mod * 2.0 - angle_mod,
                        );
                        dir + angle_offsets
                    };

                    let filter = QueryFilter {
                        exclude_collider: Some(player_entity),
                        ..default()
                    };
                    let res = rapier_context.cast_ray(origin, dir, *range, false, filter);
                    if let Some((ent, distance)) = res {
                        let pos = origin + dir * distance;
                        hits.push((ent, pos));
                    }
                }

                hits
            }
            Attack::Projectile { projectile } => {
                if let Some(proj) = projectile_map.0.get(projectile) {
                    // Fix mesh rotation
                    let mut trans = Transform::from_translation(origin);
                    trans.scale = Vec3::splat(proj.scale);
                    trans.rotation = player_trans.rotation;

                    let look_dir = origin + dir;

                    trans.rotation = player_trans.rotation;
                    trans.look_at(look_dir, Vec3::Y);
                    trans.rotate_x(proj.rotation[0].to_radians());
                    trans.rotate_y(proj.rotation[1].to_radians());
                    trans.rotate_z(proj.rotation[2].to_radians());

                    commands
                        .spawn((
                            Mesh3d(asset_server.load(&proj.model_file)),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color_texture: Some(asset_server.load(&proj.texture_file)),
                                perceptual_roughness: 1.0,
                                reflectance: 0.0,
                                ..default()
                            })),
                            trans,
                        ))
                        .insert(ProjectileEntity {
                            data: proj.clone(),
                            dir,
                        });
                } else {
                    error!("Unknown projectile: {projectile}")
                }
                Vec::new()
            }
            Attack::None => {
                error!("just attacked using None");
                Vec::new()
            }
        }
    }

    fn attack1(&mut self, time: &Time, client_events: &mut EventWriter<ClientMessage>) {
        let (slot, row) = option_return!(self.current_weapon);
        let weapon = &mut self.weapons[slot][row];
        self.current_weapon_anim = FastStr::from("shoot1");
        Self::set_anim(
            weapon,
            weapon.data.animations.fire_time1,
            weapon.data.animations.anim_time1,
            time,
        );

        client_events.send(ClientMessage::Fire { attack: 1 });
        //self.attack(1, attack_args);
    }

    fn attack2(&mut self, time: &Time, client_events: &mut EventWriter<ClientMessage>) {
        let (slot, row) = option_return!(self.current_weapon);
        let weapon = &mut self.weapons[slot][row];
        self.current_weapon_anim = FastStr::from("shoot2");
        Self::set_anim(
            weapon,
            weapon.data.animations.fire_time2,
            weapon.data.animations.anim_time2,
            time,
        );

        client_events.send(ClientMessage::Fire { attack: 2 });
        //self.attack(2, attack_args);
    }
}
