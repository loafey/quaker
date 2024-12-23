use self::{
    plane::{InPlane, Plane},
    poly::Poly,
    vertex::Vertex,
};
use crate::resources::{CurrentMap, MapDoneLoading, PickupMap, PlayerSpawnpoint, TextureMap};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{encase::rts_array::Length, PrimitiveTopology},
    },
};
use bevy_rapier3d::geometry::Collider;
use bevy_renet::renet::RenetClient;
use entities::spawn_entity;
use macros::error_return;
use map_parser::parser::Brush;

pub mod entities;
mod plane;
mod poly;
pub mod texture_systems;
mod vertex;

const EPSILON: f32 = 0.008;
const ROTATION_FIX: f32 = -90.0;
pub const SCALE_FIX: f32 = 44.0;
fn vec_fix(mut v: Vec3) -> Vec3 {
    std::mem::swap(&mut v.y, &mut v.z);
    v.x *= -1.0;
    v.y *= -1.0;
    v
}

#[allow(clippy::too_many_arguments)]
pub fn load_map(
    client: Option<Res<RenetClient>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    images: Res<Assets<Image>>,
    current_map: Res<CurrentMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    pickup_map: Res<PickupMap>,
    texture_map: Res<TextureMap>,
    mut done_loading: ResMut<MapDoneLoading>,
    mut player_spawn: ResMut<PlayerSpawnpoint>,
) {
    let map = error_return!(std::fs::read_to_string(&current_map.0));
    let map = error_return!(map_parser::parse(&map));

    let t = std::time::Instant::now();
    info!("Loading map...");

    for (id, entity) in map.into_iter().enumerate() {
        spawn_entity(
            id as u64,
            client.is_some(),
            &asset_server,
            entity.attributes,
            &mut commands,
            &mut player_spawn,
            &pickup_map,
            &mut materials,
        );

        for brush in entity.brushes {
            // Calculate the verticies for the mesh
            let polys = sort_verticies_cw(get_polys_brush(brush));

            let mut brush_poly = Vec::new();
            for mut poly in polys {
                let mut plane_center = Vec3::ZERO;
                for vert in &poly.verts {
                    plane_center += vert.p;
                }
                plane_center /= poly.verts.len() as f32;

                let indices = poly.calculate_indices();
                let verts = poly.verts.iter().map(|p| p.p).collect::<Vec<_>>();
                brush_poly.append(&mut verts.clone());
                let mut new_mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts)
                .with_inserted_indices(Indices::U32(indices));

                let mat = if let Some(text) = poly.texture.clone() {
                    let uv = poly.calculate_textcoords(&images, &texture_map);
                    let tangent = poly.calculate_tangent();
                    let texture_handle = texture_map
                        .0
                        .get(&text)
                        .unwrap_or_else(|| panic!("missing texture: {text:?}"));
                    let path = texture_handle.path().unwrap();
                    // very hacky :)
                    if !format!("{path}").ends_with("Invisible.png") {
                        new_mesh = new_mesh
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_UV_0,
                                VertexAttributeValues::Float32x2(uv),
                            )
                            .with_inserted_attribute(
                                Mesh::ATTRIBUTE_TANGENT,
                                VertexAttributeValues::Float32x4(tangent),
                            );
                        StandardMaterial {
                            base_color: Color::srgb(1.0, 1.0, 1.0),
                            base_color_texture: Some(texture_handle.clone()),
                            unlit: false,
                            perceptual_roughness: 1.0,
                            reflectance: 0.0,
                            ..default()
                        }
                    } else {
                        StandardMaterial {
                            base_color: Color::srgba(0.0, 0.0, 0.0, 0.0),
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        }
                    }
                } else {
                    StandardMaterial {
                        base_color: Color::srgb(0.0, 1.0, 0.0),
                        alpha_mode: AlphaMode::Blend,
                        unlit: true,
                        ..default()
                    }
                };
                if new_mesh.count_vertices() != 0 {
                    new_mesh.duplicate_vertices();
                    new_mesh.compute_flat_normals();

                    commands.spawn((
                        Mesh3d(meshes.add(new_mesh)),
                        MeshMaterial3d(materials.add(mat)),
                        Transform::default(),
                    ));
                }
            }

            if !brush_poly.is_empty() {
                // TODO This crashes in rust 1.81, and not being able to dedup leads to complicated collision meshes
                // brush_poly.sort_by(|a, b| {
                // a.x.total_cmp(&b.x)
                // .cmp(&a.y.total_cmp(&b.y))
                // .cmp(&a.z.total_cmp(&b.z))
                // });
                // brush_poly.dedup();

                if let Some(col) = Collider::convex_hull(&brush_poly) {
                    commands.spawn(col);
                } else {
                    error!("failed to create collider!!");
                }
            }
        }
    }

    info!("Done loading map, took {}s", t.elapsed().as_secs_f32());
    done_loading.0 = true;
}

fn sort_verticies_cw(polys: Vec<Poly>) -> Vec<Poly> {
    let mut poly_center = Vec3::ZERO;
    let mut total = 0;
    for p in &polys {
        for v in &p.verts {
            poly_center += v.p;
            total += 1;
        }
    }
    poly_center /= total as f32;
    polys
        .into_iter()
        .filter(|p| p.verts.len() >= 3)
        .map(
            |Poly {
                 mut verts,
                 mut plane,
                 texture,
                 x_offset,
                 y_offset,
                 rotation,
                 x_scale,
                 y_scale,
             }| {
                let mut center = Vec3::ZERO;
                for vert in &verts {
                    center += vert.p;
                }
                center /= verts.length() as f32;

                for i in 0..verts.length() - 1 {
                    let a = (verts[i].p - center).normalize();
                    let mut smallest_angle = -1.0;
                    let mut smallest = usize::MAX;

                    #[allow(clippy::needless_range_loop)]
                    for j in i + 1..verts.length() {
                        let b = (verts[j].p - center).normalize();
                        let angle = a.dot(b);
                        if angle >= smallest_angle {
                            smallest_angle = angle;
                            smallest = j;
                        }
                    }
                    if smallest != usize::MAX {
                        verts.swap(smallest, i + 1);
                    }
                }

                let old_plane = plane;
                if let Some(p) = plane.calculate_plane(&verts) {
                    plane = p;
                }

                if plane.n.dot(old_plane.n) < 0.0 {
                    verts.reverse();
                }

                Poly {
                    verts,
                    plane,
                    texture,
                    x_offset,
                    y_offset,
                    rotation,
                    x_scale,
                    y_scale,
                }
            },
        )
        .collect()
}

fn get_polys_brush(brush: Brush) -> Vec<Poly> {
    let faces = brush
        .iter()
        .map(|p| Plane::from_data(p.clone()))
        .map(|mut p| {
            p.n = vec_fix(p.n);
            p
        })
        .collect::<Vec<_>>();
    let mut polys = brush
        .iter()
        .enumerate()
        .map(|(i, br)| Poly {
            verts: Vec::new(),
            plane: faces[i],
            texture: (!br.texture.is_empty()).then(|| br.texture.clone()),
            x_offset: br.x_offset,
            y_offset: br.y_offset,
            rotation: br.rotation + ROTATION_FIX,
            x_scale: br.x_scale,
            y_scale: br.y_scale,
        })
        .collect::<Vec<_>>();

    for i in 0..faces.len() - 2 {
        for j in (i + 1)..faces.len() - 1 {
            'k: for k in (j + 1)..faces.len() {
                if let Some(p) = faces[i].get_intersection(&faces[j], &faces[k]) {
                    for f in faces.iter() {
                        if matches!(f.classify_point(p), InPlane::Front) {
                            continue 'k;
                        }
                    }
                    let v = Vertex::from_p(p);
                    polys[i].verts.push(v);
                    polys[j].verts.push(v);
                    polys[k].verts.push(v);
                }
            }
        }
    }
    polys.into_iter().map(|p| p / SCALE_FIX).collect()
}
