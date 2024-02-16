use bevy::{
    prelude::*,
    render::mesh::shape::{Circle, Cube},
};

fn test_map() {
    match map_parser::parse(
        &std::fs::read_to_string("crates/map_parser/tests/combined.map").unwrap(),
    ) {
        Ok(tree) => println!("{tree:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}

#[derive(Component, Debug)]
struct Player;
impl Player {
    fn spawn(mut commands: Commands) {
        commands.spawn(Player).with_children(|c| {
            c.spawn(GlobalTransform::default());
            c.spawn(Camera3dBundle::default());
        });
    }
}

fn spawn_3d_stuff(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cube::new(1.0).into()),
        material: materials.add(Color::rgb_u8(124, 144, 255).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_3d_stuff)
        .add_systems(Startup, Player::spawn)
        .run();
}
