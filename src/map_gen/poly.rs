use std::ops::Div;

use bevy::{asset::Assets, ecs::system::Res, render::texture::Image};
use macros::error_return;
use map_parser::parser::TextureOffset;

use crate::TextureMap;

use super::{plane::Plane, vertex::Vertex};

#[derive(Debug)]
pub struct Poly {
    pub verts: Vec<Vertex>,
    pub plane: Plane,
    pub texture: Option<String>,
    pub x_offset: TextureOffset,
    pub y_offset: TextureOffset,
    pub rotation: f32,
    pub x_scale: f32,
    pub y_scale: f32,
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
        let tex_ref = error_return!(self.texture.as_ref().ok_or("missing texture for object"));
        let tex = error_return!(texture_map.0.get(tex_ref).ok_or("missing texture in map"));
        let tex = error_return!(images.get(tex).ok_or("missing texture"));

        println!("{tex:?}");
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
