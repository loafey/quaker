use crate::{entities::pickup::PickupEntity, map_gen::SCALE_FIX};
use bevy::{
    asset::{AssetServer, Assets},
    color::Color,
    ecs::system::{Commands, Res, ResMut},
    log::error,
    math::{EulerRot, Quat, Vec3},
    pbr::{DirectionalLight, MeshMaterial3d, PointLight, StandardMaterial},
    prelude::Mesh3d,
    transform::components::Transform,
};
use bevy_rapier3d::{
    dynamics::Ccd,
    geometry::{ActiveCollisionTypes, ActiveEvents, Collider, Sensor},
};
use faststr::FastStr;
use resources::{PickupMap, PlayerSpawnpoint, data::PickupData};
use std::collections::HashMap;

use super::Interactable;

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

#[allow(clippy::too_many_arguments)]
pub fn spawn_entity(
    id: u64,
    is_client: bool,
    asset_server: &Res<AssetServer>,
    attributes: HashMap<FastStr, FastStr>,
    commands: &mut Commands,
    player_spawn: &mut ResMut<PlayerSpawnpoint>,
    pickup_map: &PickupMap,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Option<Interactable> {
    match attributes
        .get(&FastStr::from("classname"))
        .as_ref()
        .map(|s| &s[..])
    {
        Some("scriptable") => {
            if let Some(script) = attributes.get(&FastStr::from("script")).as_ref() {
                return Some(Interactable {
                    script: (*script).clone(),
                });
            }
        }
        Some("light") => {
            let light_level = attributes
                .get(&FastStr::from("light"))
                .and_then(|l| l.parse::<f32>().ok())
                .unwrap_or(150.0);

            let pos = attributes
                .get(&FastStr::from("origin"))
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            commands.spawn((
                PointLight {
                    intensity: light_level * 100.0,
                    range: light_level * 100.0,
                    shadows_enabled: false,
                    ..Default::default()
                },
                Transform::from_translation(pos),
            ));
        }
        Some("directional_light") => {
            let light_level = attributes
                .get(&FastStr::from("light"))
                .and_then(|l| l.parse::<f32>().ok())
                .unwrap_or(1000.0);
            let trans =
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -45.0, -45.0, -45.0));

            commands.spawn((
                DirectionalLight {
                    color: Color::WHITE,
                    illuminance: light_level,
                    shadows_enabled: true,
                    ..Default::default()
                },
                trans,
            ));
        }
        Some("info_player_start") => {
            let mut pos = attributes
                .get(&FastStr::from("origin"))
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            pos.y += 0.5;

            player_spawn.0 = pos;
        }
        Some(x) if pickup_map.0.contains_key(&FastStr::from(x)) && !is_client => {
            let data = pickup_map.0.get(&FastStr::from(x)).unwrap();

            let pos = attributes
                .get(&FastStr::from("origin"))
                .map(|p| parse_vec(p))
                .unwrap_or_default();

            spawn_pickup(id, true, pos, asset_server, data, commands, materials);
        }
        _ => error!("unhandled entity: {attributes:?}"),
    };
    None
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

    let mesh_handle = asset_server.load(pickup_model.to_string().to_string());
    let mut trans = Transform::from_translation(pos);
    trans.scale = Vec3::splat(*scale);

    let mat_handle = materials.add(StandardMaterial {
        base_color_texture: Some(asset_server.load(texture_file.to_string().to_string())),
        diffuse_transmission: 0.64,
        specular_transmission: 0.5,
        perceptual_roughness: 1.0,
        reflectance: 0.0,
        metallic: 0.0,
        ..Default::default()
    });

    let mut pickup = if host {
        let mut pickup = commands.spawn(Collider::cylinder(5.0, 10.0));
        pickup
            .insert(Sensor)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Transform::from_translation(pos))
            .insert(ActiveCollisionTypes::all())
            .insert(Ccd::enabled())
            .insert((Mesh3d(mesh_handle), MeshMaterial3d(mat_handle), trans));
        pickup
    } else {
        commands.spawn((Mesh3d(mesh_handle), MeshMaterial3d(mat_handle), trans))
    };
    pickup
        .insert(trans)
        .insert(PickupEntity::new(id, data.clone()));
}
