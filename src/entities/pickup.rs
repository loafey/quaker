use crate::{map_gen::entities::data::PickupData, player::Player};
use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        event::EventReader,
        system::{Commands, Query, Res},
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
        mut commands: Commands,
        mut query: Query<(&mut PickupEntity, &mut Transform)>,
        mut entities: Query<&mut Player>,
        mut reader: EventReader<CollisionEvent>,
        time: Res<Time>,
    ) {
        for event in reader.read() {
            if let CollisionEvent::Started(pickup, player, CollisionEventFlags::SENSOR) = event {
                if let Ok(player) = entities.get_mut(*player) {
                    println!("{:?}", player.weapons);

                    commands.entity(*pickup).despawn();
                }
            }
        }

        for (_pe, mut trans) in query.iter_mut() {
            trans.rotate_y(time.delta_seconds());
        }
    }
}
