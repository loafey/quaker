extern crate macros;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
mod map_gen;
use map_gen::{
    if_texture_done_loading, if_texture_loading, load_map, load_textures, texture_checker,
    TextureMap, TexturesLoading,
};

#[derive(Component, Debug)]
struct Player {
    auto_rot: f32,
    self_rot: f32,
    toggle_cam: bool,
}
impl Player {
    fn spawn(mut commands: Commands) {
        commands
            .spawn(Player {
                auto_rot: 0.0,
                self_rot: 0.0,
                toggle_cam: true,
            })
            .add(|mut c: EntityWorldMut| {
                c.insert(GlobalTransform::default());
                let mut trans = Transform::default();
                trans.rotate_x(std::f32::consts::PI / -8.0);
                c.insert(trans);
            })
            .with_children(|c| {
                c.spawn(Camera3dBundle::default());
            });
    }
    fn update(
        time: Res<Time>,
        keys: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Player, &mut Transform)>,
        mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
        mut motion_evr: EventReader<MouseMotion>,
    ) {
        for (mut player, mut gt) in &mut query {
            if keys.just_pressed(KeyCode::Tab) {
                player.toggle_cam = !player.toggle_cam;
                let mut primary_window = q_windows.single_mut();
                if player.toggle_cam {
                    primary_window.cursor.grab_mode = CursorGrabMode::None;
                    primary_window.cursor.visible = true;
                } else {
                    primary_window.cursor.grab_mode = CursorGrabMode::Locked;
                    primary_window.cursor.visible = false;
                }
            }

            if player.toggle_cam {
                player.auto_rot += time.delta_seconds();
                let dist = 7.0;
                gt.translation = Vec3::new(
                    player.auto_rot.sin() * dist,
                    2.5,
                    player.auto_rot.cos() * dist,
                );
                gt.rotate_y(time.delta_seconds());
            } else {
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
                    gt.rotate_local_x(ev.delta.y / -1000.0);
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

fn spawn_3d_stuff(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    let mut circle_trans =
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));
    circle_trans.translation = Vec3::new(0.0, -1.4, 0.0);
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: circle_trans,
        ..default()
    });
    // cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
    //     material: materials.add(Color::rgb_u8(124, 144, 255)),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

#[derive(Debug, Resource)]
struct CurrentMap(pub String);

fn main() {
    App::new()
        .insert_resource(CurrentMap("assets/maps/M1.map".to_string()))
        .insert_resource(TexturesLoading::default())
        .insert_resource(TextureMap::default())
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_3d_stuff)
        .add_systems(Startup, load_textures)
        .add_systems(
            Update,
            load_map.run_if(if_texture_done_loading.and_then(run_once())),
        )
        .add_systems(Update, texture_checker.run_if(if_texture_loading))
        .add_systems(Startup, Player::spawn)
        .add_systems(Update, Player::update)
        .run();
}
