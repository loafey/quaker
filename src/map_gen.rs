use std::{collections::HashMap, ops::Div};

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{encase::rts_array::Length, PrimitiveTopology},
    },
    utils::warn,
};
use macros::error_return;
use map_parser::parser::{Brush, TextureOffset};

use crate::CurrentMap;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Plane {
    n: Vec3,
    d: f32,
}
#[derive(Debug, Clone, Copy)]
pub enum InPlane {
    Front,
    Back,
    In,
}
impl Plane {
    pub fn from_points(a: Vec3, b: Vec3, c: Vec3) -> Self {
        // calculate the normal vector
        let n = (c - b).cross(a - b).normalize();
        // calculate the parameter
        let d = n.dot(a);

        // // calculate the normal vector
        // let n = -(p2 - p1).cross(p3 - p1).normalize();
        // // calculate the parameter
        // let d = (-p1).dot(n);
        Self { n, d }
    }

    pub fn from_data(plane: map_parser::parser::Plane) -> Self {
        let p1 = Vec3::new(plane.p1.0, plane.p1.1, plane.p1.2);
        let p2 = Vec3::new(plane.p2.0, plane.p2.1, plane.p2.2);
        let p3 = Vec3::new(plane.p3.0, plane.p3.1, plane.p3.2);

        Self::from_points(p1, p2, p3)
    }

    pub fn distance_to_plane(&self, v: Vec3) -> f32 {
        self.n.dot(v) + self.d
    }

    pub fn classify_point(&self, point: Vec3) -> InPlane {
        use std::f32::EPSILON as E;
        let distance = self.distance_to_plane(point);
        if distance > E * 100.0 {
            InPlane::Front
        } else if distance < -E {
            InPlane::Back
        } else {
            InPlane::In
        }
    }

    pub fn get_intersection(&self, a: &Plane, b: &Plane) -> Option<Vec3> {
        let denom = self.n.dot(a.n.cross(b.n));
        match denom.abs() < f32::EPSILON {
            true => None,
            false => Some(
                ((a.n.cross(b.n)) * -self.d
                    - (b.n.cross(self.n)) * a.d
                    - (self.n.cross(a.n)) * b.d)
                    / denom,
            ),
        }
    }

    pub fn calculate_plane(&self, verts: &[Vertex]) -> Option<Self> {
        let mut plane = self.clone();
        let mut center_of_mass = Vec3::ZERO;
        if verts.len() < 3 {
            return None;
        }

        plane.n = Vec3::ZERO;

        for i in 0..verts.len() {
            let j = if (i + 1) >= verts.len() { 0 } else { i + 1 };
            plane.n.x += (verts[i].p.y - verts[j].p.y) * (verts[i].p.z + verts[j].p.z);
            plane.n.y += (verts[i].p.z - verts[j].p.z) * (verts[i].p.x + verts[j].p.x);
            plane.n.z += (verts[i].p.x - verts[j].p.x) * (verts[i].p.y + verts[j].p.y);

            center_of_mass += verts[i].p;
        }

        if (plane.n.x.abs() < f32::EPSILON)
            && (plane.n.y.abs() < f32::EPSILON)
            && (plane.n.z.abs() < f32::EPSILON)
        {
            return None;
        }

        let magnitude =
            (plane.n.x * plane.n.x + plane.n.y * plane.n.y + plane.n.z * plane.n.z).sqrt();

        if magnitude < f32::EPSILON {
            return None;
        }

        plane.n /= magnitude;

        center_of_mass /= verts.len() as f32;

        plane.d = center_of_mass.dot(plane.n);

        Some(plane)
    }
}

#[derive(Debug, Resource, Default)]
pub struct TexturesLoading(Vec<UntypedHandle>);

#[derive(Debug, Resource, Default)]
pub struct TextureMap(HashMap<String, Handle<Image>>);
pub fn load_textures(
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    mut textures_loading: ResMut<TexturesLoading>,
    mut texture_map: ResMut<TextureMap>,
) {
    warn!("Registering textures...");
    let time = std::time::Instant::now();
    let map = error_return!(std::fs::read_to_string(&current_map.0));
    let map = error_return!(map_parser::parse(&map));

    let mut textures = map
        .into_iter()
        .flat_map(|e| e.brushes)
        .flatten()
        .map(|p| p.texture)
        .collect::<Vec<_>>();

    textures.dedup();

    let mut map = HashMap::new();
    for texture in textures {
        let handle = asset_server.load::<Image>(&format!("textures/{texture}.png"));
        textures_loading.0.push(handle.clone().untyped());
        map.insert(texture, handle);
    }
    texture_map.0 = map;
    warn!(
        "Done registering textures, took {}s",
        time.elapsed().as_secs_f32()
    );
}

pub fn if_texture_loading(text: Res<TexturesLoading>) -> bool {
    !text.0.is_empty()
}
pub fn if_texture_done_loading(text: Res<TexturesLoading>) -> bool {
    text.0.is_empty()
}

pub fn texture_checker(
    mut textures_loading: ResMut<TexturesLoading>,
    asset_server: Res<AssetServer>,
) {
    use bevy::asset::LoadState::*;
    let mut to_remove = Vec::new();
    for (i, tex) in textures_loading.0.iter().enumerate() {
        if let Some(Loaded | Failed) = asset_server.get_load_state(tex.id()) {
            to_remove.push(i)
        }
    }
    for (offset, i) in to_remove.into_iter().enumerate() {
        textures_loading.0.remove(i - offset);
    }

    if textures_loading.0.is_empty() {
        warn!("Texture loading done...");
    }
}

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

                let old_plane = plane.clone();
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

#[derive(Debug)]
struct Poly {
    verts: Vec<Vertex>,
    plane: Plane,
    texture: Option<String>,
    x_offset: TextureOffset,
    y_offset: TextureOffset,
    rotation: f32,
    x_scale: f32,
    y_scale: f32,
}
impl Poly {
    pub fn calculate_indices(&self) -> Vec<u32> {
        let mut indices = Vec::new();

        let mut verts = (0..self.verts.len() as u32).collect::<Vec<_>>();
        while verts.len() > 2 {
            indices.push(verts[0]);
            indices.push(verts[1]);
            indices.push(verts[2]);
            verts.remove(1);
        }

        indices
    }

    pub fn calculate_textcoords(
        &self,
        images: &Res<Assets<Image>>,
        texture_map: &TextureMap,
    ) -> Vec<[f32; 2]> {
        let tex = images
            .get(texture_map.0[self.texture.as_ref().unwrap()].clone())
            .unwrap();
        let width = tex.texture_descriptor.size.width;
        let height = tex.texture_descriptor.size.height;
        vec![]
    }
}
impl Div<f32> for Poly {
    type Output = Poly;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            verts: self.verts.into_iter().map(|v| v / rhs).collect(),
            plane: self.plane,
            texture: self.texture,
            x_offset: self.x_offset,
            y_offset: self.y_offset,
            rotation: self.rotation,
            x_scale: self.x_scale,
            y_scale: self.y_scale,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Vertex {
    p: Vec3,
    uv: [f32; 2],
}
impl Vertex {
    pub fn from_p(p: Vec3) -> Self {
        Self { p, ..default() }
    }
}
impl Div<f32> for Vertex {
    type Output = Vertex;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            p: self.p / rhs,
            uv: self.uv,
        }
    }
}
