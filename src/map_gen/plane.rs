use bevy::math::Vec3;

use super::vertex::Vertex;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
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
        let mut plane = *self;
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
