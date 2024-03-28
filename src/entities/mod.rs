use bevy::{
    asset::Assets,
    ecs::{
        component::Component,
        schedule::{IntoSystemConfigs, SystemConfigs},
        system::{Commands, Query},
    },
    math::{primitives::Cuboid, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{color::Color, mesh::Mesh},
    transform::components::Transform,
};

use crate::resources::projectiles::Projectile;

pub mod message;
pub mod pickup;
pub mod projectiles;

#[derive(Component)]
pub struct ProjectileEntity {
    pub dir: Vec3,
    pub data: Projectile,
}
impl ProjectileEntity {
    pub fn systems() -> SystemConfigs {
        (Self::update, Self::collision).into_configs()
    }

    pub fn update(mut query: Query<(&ProjectileEntity, &mut Transform)>) {
        for (ent, mut trans) in &mut query {
            trans.translation += ent.dir * ent.data.speed;
        }
    }

    pub fn collision(_query: Query<&ProjectileEntity>) {}
}

pub fn hitscan_hit_gfx(
    commands: &mut Commands,
    poss: &[Vec3],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for pos in poss {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.1)),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 0.0, 0.0),
                ..Default::default()
            }),
            transform: Transform::from_translation(*pos),
            ..Default::default()
        });
    }
}
