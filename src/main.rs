use bevy::prelude::*;

fn test_map() {
    match map_parser::parse(
        &std::fs::read_to_string("crates/map_parser/tests/combined.map").unwrap(),
    ) {
        Ok(tree) => println!("{tree:#?}"),
        Err(e) => eprintln!("{e}"),
    }
}

#[derive(Component, Debug)]
struct Player {
    rot: f32,
}
impl Player {
    fn spawn(mut commands: Commands) {
        commands
            .spawn(Player { rot: 0.0 })
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
    fn update(time: Res<Time>, mut query: Query<(&mut Player, &mut Transform)>) {
        for (mut player, mut gt) in &mut query {
            player.rot += time.delta_seconds();
            gt.translation = Vec3::new(player.rot.sin() * 4.0, 2.5, player.rot.cos() * 4.0);
            gt.rotate_y(time.delta_seconds());
        }
    }
}

fn spawn_3d_stuff(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(124, 144, 255)),
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
        .add_systems(Update, Player::update)
        .run();
}
