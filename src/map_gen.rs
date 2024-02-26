use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use macros::error_return;

#[derive(Debug, Clone, Copy, PartialEq)]
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
        let d = -n.dot(a);

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

    pub fn classify_point(&self, point: Vec3) -> InPlane {
        let v = self.n.dot(point) + self.d;
        if v < -std::f32::EPSILON {
            InPlane::Back
        } else if v > std::f32::EPSILON {
            InPlane::Front
        } else {
            InPlane::In
        }
    }
}

fn get_intersection(p1: Plane, p2: Plane, p3: Plane) -> Option<Vec3> {
    let denom = p1.n.dot(p2.n.cross(p3.n));
    if denom.abs() < f32::EPSILON {
        return None;
    }
    let p =
        -p1.d * (p2.n.cross(p3.n)) - p2.d * (p3.n.cross(p1.n)) - p3.d * (p1.n.cross(p2.n)) / denom;
    Some(p)
}

fn get_vertices(brush: &[Plane]) -> Vec<Vec3> {
    let mut vertices = Vec::new();
    for fi in brush {
        for fj in brush {
            if fi == fj {
                continue;
            }
            for fk in brush {
                if fk == fi || fk == fj {
                    continue;
                }

                if let Some(val) = get_intersection(*fi, *fj, *fk) {
                    let mut legal = true;
                    for f in brush {
                        if f.n.dot(val) + f.d > 0.0 {
                            legal = false;
                            break;
                        }
                    }

                    if legal {
                        vertices.push(val);
                    }
                }
            }
        }
    }
    vertices
}

fn sort_verticies_cw(vertices: &mut [Vec3], normal: Vec3) {
    let mut center = Vec3::default();
    for vec in vertices.iter() {
        center += *vec;
    }
    center /= vertices.len() as f32;

    for i in 0..vertices.len() - 2 {
        let a = (vertices[i] - center).normalize();
        let p = Plane::from_points(vertices[i], center, center + normal);

        let mut smallest_angle = -1.0;
        // assuming a mesh does not consist of 18446744073709551615
        // or 4294967295 (for the 32bit scrubs) planes
        let mut smallest = usize::MAX;
        #[allow(clippy::needless_range_loop)]
        for j in i + 1..vertices.len() - 1 {
            if !matches!(p.classify_point(vertices[j]), InPlane::Back) {
                let b = (vertices[j] - center).normalize();

                let angle = a.dot(b);

                if angle > smallest_angle {
                    smallest_angle = angle;
                    smallest = j;
                }
            }
        }

        if smallest == usize::MAX {
            error!("degenerate polygon");
            continue;
        }

        vertices.swap(smallest, i + 1);
    }
}

pub fn test_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map = error_return!(std::fs::read_to_string("crates/map_parser/tests/paper.map"));
    let map = error_return!(map_parser::parse(&map));

    for entity in map {
        for brush in entity.brushes {
            // Calculate the verticies for the mesh
            let brush = brush.into_iter().map(Plane::from_data).collect::<Vec<_>>();
            // wha??
            let mut vertices = get_vertices(&brush);
            // Calculate texture coordinates
            warn!("Texture coordinates not implemented yet...");

            println!("{vertices:?}");

            // Sort the vectors
            for brush in &brush {
                let normal = brush.n;
                //sort_verticies_cw(&mut vertices, normal);
            }

            let mut verts = Vec::new();
            for vertex in vertices {
                let vertex = vertex / 80.0;

                if cfg!(debug_assertions) {
                    let cube = Cuboid::new(0.1, 0.1, 0.1);
                    commands.spawn(PbrBundle {
                        mesh: meshes.add(cube),
                        material: materials.add(Color::rgb_u8(0, 255, 0)),
                        transform: Transform::from_xyz(vertex.x, vertex.y, vertex.z),
                        ..default()
                    });
                }
                verts.push(vertex);
            }

            let mut new_mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts);
            new_mesh.compute_flat_normals();

            commands.spawn(PbrBundle {
                mesh: meshes.add(new_mesh),
                material: materials.add(Color::rgb_u8(255, 0, 0)),
                transform: Transform::default(),
                ..default()
            });
        }
    }
}
