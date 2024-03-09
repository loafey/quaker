use crate::{map_gen::entities::data::PickupData, player::Player, WeaponMap};
use bevy::{
    asset::AssetServer,
    ecs::{
        component::Component,
        event::EventReader,
        system::{Commands, Query, Res},
    },
    log::error,
    time::Time,
    transform::components::Transform,
};
use bevy_kira_audio::Audio;
use bevy_kira_audio::AudioControl;
use bevy_rapier3d::{pipeline::CollisionEvent, rapier::geometry::CollisionEventFlags};

#[derive(Debug, Component)]
pub struct PickupEntity {
    pub data: PickupData,
}
impl PickupEntity {
    pub fn new(data: PickupData) -> Self {
        Self { data }
    }
    pub fn handle_pickups(
        mut commands: Commands,
        pickups: Query<&PickupEntity>,
        mut players: Query<&mut Player>,
        mut reader: EventReader<CollisionEvent>,
        weapon_map: Res<WeaponMap>,
        asset_server: Res<AssetServer>,
        audio: Res<Audio>,
    ) {
        for event in reader.read() {
            if let CollisionEvent::Started(ent_pickup, player, CollisionEventFlags::SENSOR) = event
            {
                if let (Ok(mut player), Ok(pickup)) =
                    (players.get_mut(*player), pickups.get(*ent_pickup))
                {
                    let classname = pickup.data.classname();
                    if let Some(weapon_data) = weapon_map.0.get(classname) {
                        let slot = weapon_data.slot;
                        player.weapons[slot].push(weapon_data.clone());
                        if player.current_weapon.is_none() {
                            player.current_weapon = Some((slot, 0))
                        }
                        audio.play(
                            // TODO, make this customizable
                            asset_server.load("sounds/Player/Guns/SuperShotty/shotgunCock.ogg"),
                        );
                    } else {
                        error!("tried to pickup nonexisting weapon: \"{classname}\"")
                    }

                    println!(
                        "Player inventory: [\n    {}\n]",
                        player
                            .weapons
                            .iter()
                            .map(|v| format!(
                                "[{}]",
                                v.iter()
                                    .map(|w| w.id.clone())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ))
                            .collect::<Vec<_>>()
                            .join(",\n    ")
                    );

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
