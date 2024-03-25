use crate::{
    entities::pickup::PickupEntity,
    map_gen::SCALE_FIX,
    resources::{PickupMap, PlayerSpawnpoint},
};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::system::{Commands, Res, ResMut},
    log::error,
    math::{EulerRot, Quat, Vec3},
    pbr::{
        DirectionalLight, DirectionalLightBundle, PbrBundle, PointLight, PointLightBundle,
        StandardMaterial,
    },
    render::color::Color,
    transform::{components::Transform, TransformBundle},
};
use bevy_rapier3d::{
    dynamics::Ccd,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
};
use std::collections::HashMap;

use self::data::PickupData;

pub mod data;

fn parse_vec(str: &str) -> Vec3 {
    let mut splat = str.split_whitespace();
    let x = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();
    let y = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();
    let z = splat
        .next()
        .unwrap_or_default()
        .parse::<f32>()
        .unwrap_or_default();

    Vec3::new(x, z, -y) / SCALE_FIX
}

pub fn spawn_entity(
    id: u64,
    is_client: bool,
    asset_server: &Res<AssetServer>,
    attributes: HashMap<String, String>,
    commands: &mut Commands,
    player_spawn: &mut ResMut<PlayerSpawnpoint>,
    pickup_map: &PickupMap,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    match attributes.get("classname").as_ref().map(|s| &s[..]) {
        Some("light") => {
            let light_level = attributes
                .get("light")
                .and_then(|l| l.parse::<f32>().ok())
                .unwrap_or(150.0);

            let pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            commands.spawn(PointLightBundle {
                transform: Transform::from_translation(pos),
                point_light: PointLight {
                    intensity: light_level * 100.0,
                    range: light_level * 100.0,
                    shadows_enabled: false,
                    ..Default::default()
                },
                ..Default::default()
            });
        }
        Some("directional_light") => {
            let light_level = attributes
                .get("light")
                .and_then(|l| l.parse::<f32>().ok())
                .unwrap_or(1000.0);
            let trans =
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -45.0, -45.0, -45.0));

            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    color: Color::WHITE,
                    illuminance: light_level,
                    shadows_enabled: true,
                    ..Default::default()
                },
                transform: trans,
                ..Default::default()
            });
        }
        Some("info_player_start") => {
            let mut pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            pos.y += 0.5;

            player_spawn.0 = pos;
        }
        Some(x) if pickup_map.0.contains_key(x) && !is_client => {
            let data = pickup_map.0.get(x).unwrap();

            let pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            spawn_pickup(id, true, pos, asset_server, data, commands, materials);
        }
        _ => error!("unhandled entity: {attributes:?}"),
    }
}

pub fn spawn_pickup(
    id: u64,
    host: bool,
    pos: Vec3,
    asset_server: &Res<AssetServer>,
    data: &PickupData,
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let PickupData {
        pickup_model,
        texture_file,
        scale,
        ..
    } = &data;

    let mesh_handle = asset_server.load(pickup_model);
    let mut trans = Transform::from_translation(pos);
    trans.scale = Vec3::splat(*scale);

    let mat_handle = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(texture_file)),
        diffuse_transmission: 0.64,
        specular_transmission: 0.5,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        metallic: 0.0,
        ..Default::default()
    });

    let mut pickup = if host {
        let mut pickup = commands.spawn(Collider::cylinder(1.0, 10.0));
        pickup
            .insert(Sensor)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(TransformBundle::from(Transform::from_translation(pos)))
            .insert(ActiveCollisionTypes::all())
            .insert(Ccd::enabled())
            .insert(PbrBundle {
                mesh: mesh_handle,
                material: mat_handle,
                transform: trans,
                ..Default::default()
            });
        pickup
    } else {
        commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: mat_handle,
            transform: trans,
            ..Default::default()
        })
    };
    pickup
        .insert(PointLightBundle {
            transform: trans,
            point_light: PointLight {
                color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                intensity: 200.0,
                radius: 4.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PickupEntity::new(id, data.clone()));
}
