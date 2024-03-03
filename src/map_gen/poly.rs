use std::ops::Div;

use bevy::{asset::Assets, ecs::system::Res, math::Vec3, render::texture::Image};
use macros::{error_return, npdbg};
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

    pub fn calculate_tangent(&self) -> Vec<[f32; 4]> {
        let tex_axis = [
            Plane::from_texoffset(self.x_offset),
            Plane::from_texoffset(self.y_offset),
        ];
        let u_axis = tex_axis[0].n.normalize();
        let v_axis = tex_axis[1].n.normalize();
        let v_sign = -self.plane.n.cross(u_axis).dot(v_axis).signum();

        vec![[u_axis.x, u_axis.y, u_axis.z, v_sign]; self.verts.len()]
    }

    pub fn calculate_textcoords(
        &mut self,
        images: &Res<Assets<Image>>,
        texture_map: &TextureMap,
    ) -> Vec<[f32; 2]> {
        let tex_ref = error_return!(self.texture.as_ref().ok_or("missing texture for object"));
        let tex = error_return!(texture_map.0.get(tex_ref).ok_or("missing texture in map"));
        let tex = error_return!(images.get(tex).ok_or("missing texture"));

        let tex_width = tex.texture_descriptor.size.width as f32;
        let tex_height = tex.texture_descriptor.size.height as f32;

        let tex_axis = [
            Plane::from_texoffset(self.x_offset),
            Plane::from_texoffset(self.y_offset),
        ];

        let tex_scale = [self.x_scale, self.y_scale];

        let mut biggest_u = 0.0f32;
        let mut biggest_v = 0.0f32;
        for vert in &mut self.verts {
            // let mut u = tex_axis[0].n.dot(vert.p);
            // u /= tex_width / tex_scale[0];
            // u += tex_axis[0].d / tex_width;
            //
            //             // let mut v = tex_axis[0].n.dot(vert.p);
            // v /= tex_height / tex_scale[1];
            // v += tex_axis[1].d / tex_height;
            //
            //             biggest_u = biggest_u.max(u.abs());
            // biggest_v = biggest_v.max(v.abs());

            let mut uv_out = [0.0; 2];
            let u_axis = tex_axis[0].n;
            let v_axis = tex_axis[1].n;
            let u_shift = tex_axis[0].d;
            let v_shift = tex_axis[1].d;

            uv_out[0] = u_axis.dot(vert.p);
            uv_out[1] = v_axis.dot(vert.p);

            uv_out[0] /= tex_width;
            uv_out[1] /= tex_height;

            uv_out[0] /= tex_scale[0];
            uv_out[1] /= tex_scale[1];

            uv_out[0] += u_shift / tex_width;
            uv_out[1] += v_shift / tex_height;

            vert.uv = uv_out;
            biggest_u = biggest_u.max(uv_out[0].abs());
            biggest_v = biggest_v.max(uv_out[1].abs());
        }
        let u_mod = 1.0 / biggest_u;
        let v_mod = 1.0 / biggest_v;
        self.verts.iter_mut().for_each(|v| {
            v.uv[0] *= u_mod;
            v.uv[1] *= v_mod;
            //v.uv[0] += 0.5;
            //v.uv[1] += 0.5;
        });
        println!("{:?}", self.verts.iter().map(|p| p.p).collect::<Vec<_>>());

        // let mut bdo_u = true;
        // let mut bdo_v = true;

        // for vert in &self.verts {
        //     if (vert.uv[0] < 1.0) && (vert.uv[0] > -1.0) {
        //         bdo_u = false;
        //     }

        //     if (vert.uv[1] < 1.0) && (vert.uv[1] > -1.0) {
        //         bdo_v = false;
        //     }
        // }

        // if bdo_u || bdo_v {
        //     let mut nearest_u = 0.0;
        //     let mut u = self.verts[0].uv[0];

        //     let mut nearest_v = 0.0;
        //     let mut v = self.verts[0].uv[1];

        //     if bdo_u {
        //         if u > 1.0 {
        //             nearest_u = u.floor();
        //         } else {
        //             nearest_u = u.ceil();
        //         }
        //     }

        //     if bdo_v {
        //         if v > 1.0 {
        //             nearest_v = v.floor();
        //         } else {
        //             nearest_v = v.ceil();
        //         }
        //     }

        //     for vert in &self.verts {
        //         if bdo_u {
        //             u = vert.uv[0];

        //             if u.abs() < nearest_u.abs() {
        //                 if u > 1.0 {
        //                     nearest_u = u.floor();
        //                 } else {
        //                     nearest_u = u.ceil();
        //                 }
        //             }
        //         }

        //         if bdo_v {
        //             v = vert.uv[1];

        //             if v.abs() < nearest_v.abs() {
        //                 if v > 1.0 {
        //                     nearest_v = v.floor();
        //                 } else {
        //                     nearest_v = v.ceil();
        //                 }
        //             }
        //         }
        //     }

        //     for vert in &mut self.verts {
        //         vert.uv[0] -= nearest_u;
        //         vert.uv[1] -= nearest_v;
        //     }
        // }

        self.verts.iter().map(|v| v.uv).collect()
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
