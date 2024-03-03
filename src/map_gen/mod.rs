use self::{
    plane::{InPlane, Plane},
    poly::Poly,
    texture_systems::TextureMap,
    vertex::Vertex,
};
use crate::CurrentMap;
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{encase::rts_array::Length, PrimitiveTopology},
    },
};
use macros::error_return;
use map_parser::parser::Brush;

mod plane;
mod poly;
pub mod texture_systems;
mod vertex;

#[allow(clippy::too_many_arguments)]
pub fn load_map(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    current_map: Res<CurrentMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    texture_map: Res<TextureMap>,
) {
    let map = error_return!(std::fs::read_to_string(&current_map.0));
    let map = error_return!(map_parser::parse(&map));

    let t = std::time::Instant::now();
    warn!("Loading map...");

    for entity in map {
        for brush in entity.brushes {
            // Calculate the verticies for the mesh
            let polys = sort_verticies_cw(get_polys_brush(brush));

            for poly in polys {
                let mut plane_center = Vec3::ZERO;
                for vert in &poly.verts {
                    plane_center += vert.p;
                }
                plane_center /= poly.verts.len() as f32;

                let indices = poly.calculate_indices();
                let mut new_mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    poly.verts.iter().map(|p| p.p).collect::<Vec<_>>(),
                )
                .with_inserted_indices(Indices::U32(indices));

                let mat = if let Some(text) = &poly.texture {
                    let uv = poly.calculate_textcoords(&images, &texture_map);
                    let texture_handle = &texture_map.0[text];
                    new_mesh = new_mesh.with_inserted_attribute(
                        Mesh::ATTRIBUTE_UV_0,
                        VertexAttributeValues::Float32x2(uv),
                    );
                    StandardMaterial {
                        base_color: Color::rgb(0.0, 0.0, 1.0),
                        base_color_texture: Some(texture_handle.clone()),
                        ..default()
                    }
                } else {
                    // temp, used as it vizualises Z fighting
                    let n = poly.plane.n * 100.0;
                    StandardMaterial::from(Color::rgb_u8(
                        127 + n.x as u8,
                        127 + n.y as u8,
                        127 + n.z as u8,
                    ))
                };
                new_mesh.duplicate_vertices();
                new_mesh.compute_flat_normals();
                //mat.cull_mode = None;

                commands.spawn(PbrBundle {
                    mesh: meshes.add(new_mesh),
                    material: materials.add(mat),
                    transform: Transform::default(),
                    ..default()
                });
            }
        }
    }

    warn!("Done loading map, took {}s", t.elapsed().as_secs_f32())
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
            std::mem::swap(&mut p.n.y, &mut p.n.z);
            p.n.x *= -1.0;
            p.n.y *= -1.0;
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
            rotation: br.rotation,
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
    polys.into_iter().map(|p| p / 64.0).collect()
}
