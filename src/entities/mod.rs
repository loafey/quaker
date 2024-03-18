use bevy::{
    ecs::{
        component::Component,
        schedule::{IntoSystemConfigs, SystemConfigs},
        system::Query,
    },
    math::Vec3,
    transform::components::Transform,
};

use crate::resources::projectiles::Projectile;

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
