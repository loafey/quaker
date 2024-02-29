use std::ops::Div;

use bevy::{
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_resource::{encase::rts_array::Length, PrimitiveTopology},
    },
};
use macros::error_return;

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

    pub fn calculate_plane(self, verts: &[Vertex]) -> Option<Self> {
        let mut plane = self;
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

pub fn test_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map = error_return!(std::fs::read_to_string(
        "crates/map_parser/tests/rotated.map"
    ));
    let map = error_return!(map_parser::parse(&map));

    for entity in map {
        for brush in entity.brushes {
            // Calculate the verticies for the mesh
            let faces = brush
                .into_iter()
                .map(Plane::from_data)
                .map(|mut p| {
                    std::mem::swap(&mut p.n.y, &mut p.n.z);
                    p.n.y *= -1.0;
                    p
                })
                .collect::<Vec<_>>();
            let polys = get_polys(&faces)
                .into_iter()
                .map(|p| p / 64.0)
                .collect::<Vec<_>>();

            let polys = polys
                .into_iter()
                .zip(faces)
                .map(|(mut p, f)| {
                    p.plane = f;
                    // let mut c = p.verts.clone();
                    // c.reverse();
                    // p.verts.append(&mut c);
                    p
                })
                .collect::<Vec<_>>();

            let polys = sort_verticies_cw(polys);

            for poly in polys {
                // println!("Poly verts amount: {:?}", poly.verts.length());
                let mut plane_center = Vec3::ZERO;
                for vert in &poly.verts {
                    plane_center += vert.p;
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
                        material: materials.add(Color::rgba_u8(0, 255, 0, 20)),
                        transform: Transform::from_translation(vert.p),
                        ..default()
                    });
                }
                plane_center /= poly.verts.len() as f32;

                let indices = poly.calculate_indices();
                let mut new_mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                )
                .with_inserted_attribute(
                    Mesh::ATTRIBUTE_POSITION,
                    poly.verts.into_iter().map(|p| p.p).collect::<Vec<_>>(),
                )
                .with_inserted_indices(Indices::U32(indices));
                new_mesh.duplicate_vertices();
                new_mesh.compute_flat_normals();

                // temp, used as it vizualises Z fighting
                let n = poly.plane.n * 100.0;
                let mat = StandardMaterial::from(Color::rgb_u8(
                    127 + n.x as u8,
                    127 + n.y as u8,
                    127 + n.z as u8,
                ));
                //mat.cull_mode = None;

                commands.spawn(PbrBundle {
                    mesh: meshes.add(new_mesh),
                    material: materials.add(mat),
                    transform: Transform::default(),
                    ..default()
                });
            }

            //
            // let mut new_mesh = Mesh::new(
            //     PrimitiveTopology::TriangleList,
            //     RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            // )
            // .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts);
            // new_mesh.compute_flat_normals();
            //
            // commands.spawn(PbrBundle {
            //     mesh: meshes.add(new_mesh),
            //     material: materials.add(Color::rgb_u8(255, 0, 0)),
            //     transform: Transform::default(),
            //     ..default()
            // });
        }
    }
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
        .map(
            |Poly {
                 mut verts,
                 mut plane,
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

                Poly { verts, plane }
            },
        )
        .collect()
}

fn get_polys(faces: &[Plane]) -> Vec<Poly> {
    let ui_faces = faces.len();
    let mut polys = Vec::new();
    for _ in 0..ui_faces {
        polys.push(Poly::default());
    }

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
    polys
}

#[derive(Debug, Default)]
struct Poly {
    verts: Vec<Vertex>,
    plane: Plane,
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
}
impl Div<f32> for Poly {
    type Output = Poly;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            verts: self.verts.into_iter().map(|v| v / rhs).collect(),
            plane: self.plane,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Vertex {
    p: Vec3,
}
impl Vertex {
    pub fn from_p(p: Vec3) -> Self {
        Self { p }
    }
}
impl Div<f32> for Vertex {
    type Output = Vertex;

    fn div(self, rhs: f32) -> Self::Output {
        Self { p: self.p / rhs }
    }
}
