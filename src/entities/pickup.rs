use crate::{map_gen::entities::data::PickupData, net::SimulationEvent, player::Player};
use bevy::{
    ecs::{
        component::Component,
        event::{EventReader, EventWriter},
        schedule::{IntoSystemConfigs, SystemConfigs},
        system::{Commands, Query, Res},
    },
    time::Time,
    transform::components::Transform,
};
use bevy_rapier3d::{pipeline::CollisionEvent, rapier::geometry::CollisionEventFlags};

#[derive(Debug, Component)]
pub struct PickupEntity {
    pub id: u64,
    pub data: PickupData,
}
impl PickupEntity {
    pub fn systems() -> SystemConfigs {
        (PickupEntity::update, PickupEntity::handle_pickups).into_configs()
    }

    pub fn new(id: u64, data: PickupData) -> Self {
        Self { id, data }
    }

    pub fn handle_pickups(
        mut commands: Commands,
        pickups: Query<&PickupEntity>,
        mut players: Query<&mut Player>,
        mut reader: EventReader<CollisionEvent>,
        mut server_event: EventWriter<SimulationEvent>,
    ) {
        for event in reader.read() {
            if let CollisionEvent::Started(ent_pickup, player, CollisionEventFlags::SENSOR) = event
            {
                if let (Ok(player), Ok(pickup)) =
                    (players.get_mut(*player), pickups.get(*ent_pickup))
                {
                    server_event.send(SimulationEvent::PlayerPicksUpPickup {
                        id: pickup.id,
                        player: player.id,
                        pickup: pickup.data.gives.clone(),
                    });

                    commands.entity(*ent_pickup).despawn();
                }
            }
        }
    }
    pub fn update(mut query: Query<(&mut PickupEntity, &mut Transform)>, time: Res<Time>) {
        for (_pe, mut trans) in query.iter_mut() {
            trans.rotate_y(time.delta_seconds());
        }
    }
}
