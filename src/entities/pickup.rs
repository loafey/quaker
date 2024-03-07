use crate::map_gen::entities::data::PickupData;
use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    time::Time,
    transform::components::Transform,
};

#[derive(Debug, Component)]
pub struct PickupEntity {
    pub data: PickupData,
}
impl PickupEntity {
    pub fn new(data: PickupData) -> Self {
        Self { data }
    }
    pub fn update(mut query: Query<(&mut PickupEntity, &mut Transform)>, time: Res<Time>) {
        for (_pe, mut trans) in query.iter_mut() {
            trans.rotate_y(time.delta_seconds());
        }
    }
}
