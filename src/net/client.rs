use crate::{
    entities::{hitscan_hit_gfx, pickup::PickupEntity},
    map_gen,
    net::{Lobby, PlayerInfo},
    player::Player,
    queries::NetWorld,
    resources::{CurrentMap, CurrentStage},
};

use super::{
    connection_config, update_world, CurrentClientId, IsSteam, NetState, ServerChannel,
    ServerMessage, SteamClient, PROTOCOL_ID,
};
use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{
            common_conditions::resource_exists, IntoSystemConfigs, NextState, SystemConfigs,
        },
        system::{Query, Res, ResMut},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
};
use bevy_renet::renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
    RenetClient,
};
use macros::{error_continue, error_return, option_continue};
use renet_steam::{bevy::SteamTransportError, SteamClientTransport};
use std::{net::UdpSocket, time::SystemTime};
use steamworks::SteamId;

pub fn handle_messages(
    pickups: Query<(Entity, &PickupEntity)>,
    mut client: ResMut<RenetClient>,
    mut current_stage: ResMut<CurrentMap>,
    mut state: ResMut<NextState<CurrentStage>>,
    mut nw: NetWorld,
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
                name,
            } => {
                if id != nw.current_id.0 {
                    let entity = Player::spawn(&mut nw, false, translation, id, weapons, None);
                    nw.lobby.players.insert(id, PlayerInfo::new(entity, name));
                }
            }
            ServerMessage::DespawnPlayer { id } => {
                let player = option_continue!(nw.lobby.players.get(&id)).entity;
                nw.commands.entity(player).despawn_recursive();
                nw.lobby.players.remove(&id);
            }
            ServerMessage::Reset => {
                let player = option_continue!(nw.lobby.players.get(&nw.current_id.0)).entity;
                let (_, mut player, mut trans) = error_continue!(nw.players.get_mut(player));
                player.health = 100.0;
                player.armour = 0.0;
                player.last_hurter = 0;
                trans.translation = nw.player_spawn.0;
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
                    &nw.asset_server,
                    &data,
                    &mut nw.commands,
                    &mut nw.materials,
                );
            }
            ServerMessage::Message { text } => {
                let player = option_continue!(nw.lobby.players.get(&nw.current_id.0)).entity;
                let (_, player, _) = error_continue!(nw.players.get(player));
                player.display_message(&mut nw.commands, &nw.asset_server, text);
            }
            ServerMessage::KillStat { death, hurter } => {
                if let Some(info) = nw.lobby.players.get_mut(&death) {
                    info.deaths += 1;
                }
                if let Some(hurter) = hurter
                    && let Some(info) = nw.lobby.players.get_mut(&hurter)
                {
                    info.kills += 1;
                }
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
                update_world(id, &message, &mut nw);
            }
            ServerMessage::DespawnPickup { id } => {
                // TODO: Improve this
                for (ent, pickup) in &pickups {
                    if pickup.id == id {
                        nw.commands.entity(ent).despawn_recursive();
                    }
                }
            }
            ServerMessage::HitscanHits { hits } => {
                hitscan_hit_gfx(&mut nw.commands, &hits, &mut nw.meshes, &mut nw.materials)
            }
            ServerMessage::Hit { amount } => {
                let player = option_continue!(nw.lobby.players.get(&nw.current_id.0)).entity;
                let (_, mut player, _) = error_continue!(nw.players.get_mut(player));
                player.health -= amount;
            }
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
    steam_client: &Option<Res<SteamClient>>,
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
    world.insert_resource(Lobby::default());
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
