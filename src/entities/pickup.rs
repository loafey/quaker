use crate::{map_gen::entities::data::PickupData, player::Player};
use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        system::{Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_rapier3d::{
    geometry::{ActiveEvents, Sensor},
    pipeline::{CollisionEvent, ContactForceEvent},
    rapier::geometry::CollisionEventFlags,
};

#[derive(Debug, Component)]
pub struct PickupEntity {
    pub data: PickupData,
}
impl PickupEntity {
    pub fn new(data: PickupData) -> Self {
        Self { data }
    }
    pub fn update(
        mut query: Query<(&mut PickupEntity, &mut Transform)>,
        mut entities: Query<&mut Player>,
        mut reader: EventReader<CollisionEvent>,
        time: Res<Time>,
    ) {
        for event in reader.read() {
            if let CollisionEvent::Started(_, e2, CollisionEventFlags::SENSOR) = event {
                if let Ok(player) = entities.get_mut(*e2) {
                    println!("{player:?}");
                }
            }
        }

        for (_pe, mut trans) in query.iter_mut() {
            trans.rotate_y(time.delta_seconds());
        }
    }
}
