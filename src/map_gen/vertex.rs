use bevy::math::Vec3;
use std::ops::Div;

#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    pub p: Vec3,
    pub uv: [f32; 2],
}
impl Vertex {
    pub fn from_p(p: Vec3) -> Self {
        Self { p, uv: [0.0; 2] }
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
