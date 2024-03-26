use crate::{
    entities::{hitscan_hit_gfx, pickup::PickupEntity},
    map_gen,
    player::Player,
    queries::NetWorld,
    resources::{CurrentMap, CurrentStage},
};

use super::{
    connection_config, update_world, CurrentClientId, IsSteam, NetState, ServerChannel,
    ServerMessage, PROTOCOL_ID,
};
use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{
            common_conditions::resource_exists, IntoSystemConfigs, NextState, SystemConfigs,
        },
        system::{NonSend, Query, ResMut},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
};
use bevy_renet::renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
    RenetClient,
};
use macros::{error_continue, error_return};
use renet_steam::{bevy::SteamTransportError, SteamClientTransport};
use std::{net::UdpSocket, time::SystemTime};
use steamworks::SteamId;

pub fn handle_messages(
    pickups: Query<(Entity, &PickupEntity)>,
    mut client: ResMut<RenetClient>,
    mut current_stage: ResMut<CurrentMap>,
    mut state: ResMut<NextState<CurrentStage>>,
    mut net_world: NetWorld,
) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages as u8) {
        let message = error_continue!(ServerMessage::from_bytes(&message));
        match message {
            ServerMessage::SetMap(map) => {
                info!("setting map to: {map:?}");
                current_stage.0 = map;
                state.set(CurrentStage::InGame);
            }
            ServerMessage::SpawnPlayer {
                id,
                translation,
                weapons,
            } => {
                if id != net_world.current_id.0 {
                    println!("Spawning player: {id}");
                    Player::spawn(&mut net_world, false, translation, id, weapons, None);
                }
            }
            ServerMessage::DespawnPlayer { id } => {
                for (ent, player, _) in &net_world.players {
                    if player.id == id {
                        net_world.commands.entity(ent).despawn_recursive();
                    }
                }
            }
            ServerMessage::SpawnPickup {
                id,
                translation,
                data,
            } => {
                map_gen::entities::spawn_pickup(
                    id,
                    false,
                    translation,
                    &net_world.asset_server,
                    &data,
                    &mut net_world.commands,
                    &mut net_world.materials,
                );
            }
            x => {
                error!("unhandled ServerMessages message: {x:?}")
            }
        }
    }

    while let Some(message) = client.receive_message(ServerChannel::NetworkedEntities as u8) {
        let message = error_continue!(ServerMessage::from_bytes(&message));
        #[allow(clippy::single_match)]
        match message {
            ServerMessage::PlayerUpdate { id, message } => {
                update_world(id, &message, &mut net_world);
            }
            ServerMessage::DespawnPickup { id } => {
                for (ent, pickup) in &pickups {
                    if pickup.id == id {
                        net_world.commands.entity(ent).despawn_recursive();
                    }
                }
            }
            ServerMessage::HitscanHits { hits } => hitscan_hit_gfx(
                &mut net_world.commands,
                &hits,
                &mut net_world.meshes,
                &mut net_world.materials,
            ),
            x => {
                error!("unhandled NetworkedEntities message: {x:?}")
            }
        }
    }
}

pub fn init_client(
    world: &mut World,
    next_state: &mut NextState<NetState>,
    ip: &String,
    steam_client: &Option<NonSend<steamworks::Client>>,
) -> bool {
    info!("joining: {ip}");
    let client = RenetClient::new(connection_config());

    if let Some(sc) = steam_client {
        let server_steam_id = SteamId::from_raw(error_return!(ip.parse()));

        sc.networking_utils().init_relay_network_access();

        let transport = error_return!(SteamClientTransport::new(sc, &server_steam_id));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(sc.user().steam_id().raw()));
    } else {
        let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));

        let server_addr = error_return!(ip.parse());
        let socket = error_return!(UdpSocket::bind("127.0.0.1:0"));

        let client_id = current_time.as_micros() as u64;

        let authentication = ClientAuthentication::Unsecure {
            client_id,
            protocol_id: PROTOCOL_ID,
            server_addr,
            user_data: None,
        };

        let transport = error_return!(NetcodeClientTransport::new(
            current_time,
            authentication,
            socket
        ));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(client_id));
    }
    world.insert_resource(client);
    next_state.set(NetState::Client);
    info!("started client");
    true
}

pub fn systems() -> SystemConfigs {
    (handle_messages,).into_configs()
}

pub fn errors() -> SystemConfigs {
    (panic_on_error_system.run_if(resource_exists::<NetcodeClientTransport>),).into_configs()
}

pub fn errors_steam() -> SystemConfigs {
    (panic_on_error_system_steam.run_if(resource_exists::<IsSteam>),).into_configs()
}

pub fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        panic!("{}", e);
    }
}

pub fn panic_on_error_system_steam(mut renet_error: EventReader<SteamTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        panic!("{}", e);
    }
}
