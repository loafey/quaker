use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};
use macros::{error_return, npdbg};

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

    pub fn get_intersection(&self, a: &Plane, b: &Plane) -> Option<Vec3> {
        let denom = self.n.dot(a.n.cross(b.n));
        match dbg!(denom.abs()) < f32::EPSILON {
            true => None,
            false => Some(
                ((a.n.cross(b.n)) * -self.d
                    - (b.n.cross(self.n)) * a.d
                    - (self.n.cross(a.n)) * b.d)
                    / denom,
            ),
        }
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
            let faces = brush.into_iter().map(Plane::from_data).collect::<Vec<_>>();
            let polys = get_polys(faces);
            println!("{polys:?}");

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

fn get_polys(faces: Vec<Plane>) -> Vec<Poly> {
    let ui_faces = faces.len();
    let mut polys = Vec::new();
    for _ in 0..ui_faces {
        polys.push(Poly::default());
    }

    for i in 0..faces.len() - 2 {
        let fi = faces[i];
        for j in (i + 1)..faces.len() - 1 {
            let fj = faces[j];
            #[allow(clippy::needless_range_loop)]
            'k: for k in (j + 1)..faces.len() - 1 {
                let fk = faces[k];

                warn!("Should be 4 more here...");
                if let Some(p) = npdbg!(fi.get_intersection(&fj, &fk)) {
                    for f in &faces {
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

#[derive(Debug, Default, Clone, Copy)]
struct Vertex {
    p: Vec3,
}
impl Vertex {
    pub fn from_p(p: Vec3) -> Self {
        Self { p }
    }
}
