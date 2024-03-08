use crate::map_gen::entities::data::PickupData;
use bevy::{
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_rapier3d::geometry::{ActiveEvents, Sensor};

#[derive(Debug, Component)]
pub struct PickupEntity {
    pub data: PickupData,
}
impl PickupEntity {
    pub fn new(data: PickupData) -> Self {
        Self { data }
    }
    pub fn update(
        mut query: Query<(&mut PickupEntity, &mut Transform, &mut ActiveEvents)>,
        time: Res<Time>,
    ) {
        for (_pe, mut trans, sens) in query.iter_mut() {
            trans.rotate_y(time.delta_seconds());
            println!("{:?}", sens);
        }
    }
}
