use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    utils::petgraph::matrix_graph::Zero,
};
use macros::error_return;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Plane {
    n: Vec3,
    n_prime: Vec3,
    d: Vec3,
}
impl Plane {
    pub fn from_data(plane: &map_parser::parser::Plane) -> Self {
        let p1 = Vec3::new(plane.p1.0, plane.p1.1, plane.p1.2);
        let p2 = Vec3::new(plane.p2.0, plane.p2.1, plane.p2.2);
        let p3 = Vec3::new(plane.p3.0, plane.p3.1, plane.p3.2);

        // calculate the normal vector
        let n_prime = (p2 - p1).cross(p3 - p1);
        // normalize it
        let n = n_prime
            / ((n_prime.x * n_prime.x) + (n_prime.y * n_prime.y) + (n_prime.z * n_prime.z)).sqrt();
        // calculate the parameter
        let d = (-p1) * n;
        Self { n, n_prime, d }
    }
}

fn get_intersection(i: Plane, j: Plane, k: Plane) -> Option<Vec3> {
    let denom = i.n.dot(j.n.cross(k.n));
    if denom < f32::EPSILON || denom.is_nan() {
        return None;
    }
    let p = -i.d * (j.n.cross(k.n)) - j.d * (k.n.cross(i.n)) - k.d * (i.n.cross(j.n)) / denom;
    Some(p)
}

pub fn test_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map = error_return!(std::fs::read_to_string("crates/map_parser/tests/220.map"));
    let map = error_return!(map_parser::parse(&map));

    for entity in map {
        for brush in entity.brushes {
            let brush = brush.iter().map(Plane::from_data).collect::<Vec<_>>();
            let mut vertices = Vec::new();
            for fi in &brush {
                for fj in &brush {
                    if fi == fj {
                        continue;
                    }
                    for fk in &brush {
                        if fk == fi || fk == fj {
                            continue;
                        }

                        if let Some(val) = get_intersection(*fi, *fj, *fk) {
                            let mut legal = true;
                            for f in &brush {
                                if f.n.dot(val) > f.d.distance(Vec3::ZERO) {
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

            println!("{vertices:?}");

            let mut center = Vec3::default();
            for vec in &vertices {
                center += *vec;
            }
            let len = vertices.len() as f32;
            center /= Vec3::new(len, len, len);

            for n in 0..vertices.len() - 3 {
                let a = (vertices[n] - center).normalize();
            }

            let mut verts = Vec::new();
            for vertex in vertices {
                let vertex = vertex / 16.0;

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
            let new_mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, verts);

            commands.spawn(PbrBundle {
                mesh: meshes.add(new_mesh),
                material: materials.add(Color::rgb_u8(255, 0, 0)),
                transform: Transform::default(),
                ..default()
            });
        }
    }
}
