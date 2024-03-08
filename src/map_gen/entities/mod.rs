use crate::{entities::pickup::PickupEntity, map_gen::SCALE_FIX, PickupMap, PlayerSpawnpoint};
use bevy::{
    asset::{AssetServer, Assets},
    ecs::{
        system::{Commands, Res, ResMut},
        world::EntityWorldMut,
    },
    hierarchy::BuildChildren,
    log::error,
    math::Vec3,
    pbr::{PbrBundle, PointLight, PointLightBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    scene::SceneBundle,
    transform::{components::Transform, TransformBundle},
};
use bevy_rapier3d::{
    geometry::{ActiveEvents, Collider, Sensor},
    pipeline::CollisionEvent,
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
    asset_server: &Res<AssetServer>,
    attributes: HashMap<String, String>,
    commands: &mut Commands,
    player_spawn: &mut ResMut<PlayerSpawnpoint>,
    pickup_map: &PickupMap,
    meshes: &mut ResMut<Assets<Mesh>>,
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
        Some("info_player_start") => {
            let mut pos = attributes
                .get("origin")
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            pos.y += 0.5;

            player_spawn.0 = pos;
        }
        Some(x) if pickup_map.0.contains_key(x) => {
            let data = pickup_map.0.get(x).unwrap();
            spawn_pickup(asset_server, data, attributes, commands, meshes, materials);
        }
        _ => error!("unhandled entity: {attributes:?}"),
    }
}

fn spawn_pickup(
    asset_server: &Res<AssetServer>,
    data: &PickupData,
    attributes: HashMap<String, String>,
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let pos = attributes
        .get("origin")
        .map(|p| parse_vec(p))
        .unwrap_or_default();

    match data {
        PickupData::Weapon {
            classname,
            gives,
            pickup_model,
            pickup_material,
            texture_file,
            scale,
        } => {
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

            commands
                .spawn(Collider::cylinder(1.0, 10.0))
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(TransformBundle::from(Transform::from_translation(pos)))
                .insert(PickupEntity::new(data.clone()))
                .insert(PbrBundle {
                    mesh: mesh_handle,
                    material: mat_handle,
                    transform: trans,
                    ..Default::default()
                })
                .insert(PointLightBundle {
                    transform: trans,
                    point_light: PointLight {
                        color: Color::rgba(1.0, 1.0, 1.0, 0.5),
                        intensity: 200.0,
                        radius: 4.0,
                        ..Default::default()
                    },
                    ..Default::default()
                });

            // let scene_handle = asset_server.load(pickup_model);
            // let mut trans = Transform::from_translation(pos);
            // trans.scale = Vec3::splat(*scale);

            // commands
            //     .spawn(PickupEntity::new(data.clone()))
            //     .insert(trans)
            //     .insert(SceneBundle {
            //         scene: scene_handle,
            //         transform: trans,
            //         ..Default::default()
            //     });
        }
    }
}
