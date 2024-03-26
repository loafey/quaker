use super::{
    connection_config, update_world, ClientChannel, ClientMessage, NetState, SimulationEvent,
    PROTOCOL_ID,
};
use crate::{
    entities::hitscan_hit_gfx,
    net::{CurrentClientId, IsSteam, ServerChannel, ServerMessage},
    player::Player,
    queries::NetWorld,
    resources::CurrentMap,
};
use bevy::{
    ecs::{
        entity::Entity,
        event::EventReader,
        schedule::{
            common_conditions::resource_exists, IntoSystemConfigs, NextState, SystemConfigs,
        },
        system::{NonSend, Res, ResMut, Resource},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
};
use bevy_renet::renet::{
    transport::{
        NetcodeServerTransport, NetcodeTransportError, ServerAuthentication, ServerConfig,
    },
    ClientId, RenetServer, ServerEvent,
};
use macros::{error_continue, error_return, option_continue};
use renet_steam::{
    bevy::SteamTransportError, AccessPermission, SteamServerConfig, SteamServerTransport,
};
use std::{collections::HashMap, net::UdpSocket, time::SystemTime};

#[derive(Debug, Resource, Default)]
pub struct Lobby {
    pub players: HashMap<ClientId, Entity>,
    cam_count: isize,
}

#[allow(clippy::type_complexity)]
pub fn server_events(
    mut events: EventReader<ServerEvent>,
    mut sim_events: EventReader<SimulationEvent>,
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<Lobby>,

    map: Res<CurrentMap>,
    mut net_world: NetWorld,
) {
    // Handle connection details
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Player: {client_id} joined");
                server.send_message(
                    *client_id,
                    ServerChannel::ServerMessages as u8,
                    error_return!(ServerMessage::SetMap(map.0.clone()).bytes()),
                );
                lobby.cam_count += 2;

                for (pickup, trans) in &net_world.pickups_query {
                    server.send_message(
                        *client_id,
                        ServerChannel::ServerMessages as u8,
                        error_continue!(ServerMessage::SpawnPickup {
                            id: pickup.id,
                            translation: trans.translation,
                            data: pickup.data.clone()
                        }
                        .bytes()),
                    )
                }

                // Spawn players for newly joined client
                for (other_id, ent) in &lobby.players {
                    let (_, pl, trans) = error_continue!(net_world.players.get(*ent));
                    server.send_message(
                        *client_id,
                        ServerChannel::ServerMessages as u8,
                        error_continue!(ServerMessage::SpawnPlayer {
                            id: other_id.raw(),
                            translation: trans.translation,
                            weapons: pl
                                .weapons
                                .iter()
                                .map(|v| v.iter().map(|w| w.data.id.clone()).collect())
                                .collect()
                        }
                        .bytes()),
                    );
                }

                let spawn_point = net_world.player_spawn.0;
                let entity = Player::spawn(
                    &mut net_world,
                    false,
                    spawn_point,
                    client_id.raw(),
                    Vec::new(),
                    None,
                );
                lobby.players.insert(*client_id, entity);
                println!("Current players: {:?}", lobby.players);

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::SpawnPlayer {
                        id: client_id.raw(),
                        translation: spawn_point,
                        weapons: Vec::new()
                    }
                    .bytes()),
                )
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player: {client_id} left due to {reason}");

                lobby.players.remove(client_id);

                for (e, p, _) in &net_world.players {
                    if p.id == client_id.raw() {
                        net_world.commands.entity(e).despawn_recursive();
                    }
                }

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::DespawnPlayer {
                        id: client_id.raw(),
                    }
                    .bytes()),
                )
            }
        }
    }

    for message in sim_events.read() {
        match message {
            SimulationEvent::PlayerPicksUpPickup { id, player, pickup } => {
                let remove_message = ServerMessage::DespawnPickup { id: *id };
                server.broadcast_message(
                    ServerChannel::NetworkedEntities as u8,
                    error_continue!(remove_message.bytes()),
                );
                let pickup_message = ClientMessage::PickupWeapon {
                    weapon: pickup.clone(),
                };

                update_world(*player, &pickup_message, &mut net_world);

                let pickup_message_wrapped = ServerMessage::PlayerUpdate {
                    id: *player,
                    message: pickup_message,
                };

                let bytes = error_continue!(pickup_message_wrapped.bytes());

                server.broadcast_message(ServerChannel::NetworkedEntities as u8, bytes.clone());
            }
        }
    }

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id.raw(), message, &mut net_world);
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Command as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id.raw(), message, &mut net_world);
        }
    }
}

pub fn handle_client_message(
    server: &mut RenetServer,
    client_id: u64,
    message: ClientMessage,
    net_world: &mut NetWorld,
) {
    match message {
        ClientMessage::Fire { attack } => {
            for (player_entity, mut player, trans) in &mut net_world.players {
                if player.id == client_id {
                    let cam = option_continue!(player.children.camera);
                    let (_, cam_trans) = error_continue!(net_world.cameras.get(cam));
                    let hits = player.attack(
                        attack,
                        &mut net_world.materials,
                        player_entity,
                        &mut net_world.commands,
                        &net_world.rapier_context,
                        cam_trans,
                        &trans,
                        &mut net_world.game_entropy,
                        &net_world.projectile_map,
                        &net_world.asset_server,
                    );
                    let hits = hits.into_iter().map(|(_, p)| p).collect::<Vec<_>>();
                    hitscan_hit_gfx(
                        &mut net_world.commands,
                        &hits,
                        &mut net_world.meshes,
                        &mut net_world.materials,
                    );
                    server.broadcast_message(
                        ServerChannel::NetworkedEntities as u8,
                        error_continue!(ServerMessage::HitscanHits { hits }.bytes()),
                    )
                }
            }
        }
        message => {
            update_world(client_id, &message, net_world);
            server.broadcast_message(
                ServerChannel::NetworkedEntities as u8,
                error_return!(ServerMessage::PlayerUpdate {
                    id: client_id,
                    message,
                }
                .bytes()),
            )
        }
    }
}

pub fn init_server(
    world: &mut World,
    next_state: &mut NextState<NetState>,
    steam_client: &Option<NonSend<steamworks::Client>>,
) -> bool {
    let server = RenetServer::new(connection_config());

    if let Some(sc) = steam_client {
        let steam_transport_config = SteamServerConfig {
            max_clients: 64,
            access_permission: AccessPermission::Public,
        };

        let transport = error_return!(SteamServerTransport::new(sc, steam_transport_config));

        world.insert_resource(IsSteam);
        world.insert_non_send_resource(transport);
        world.insert_resource(CurrentClientId(sc.user().steam_id().raw()))
    } else {
        let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));
        let public_addr = error_return!("127.0.0.1:8000".parse());
        let socket = error_return!(UdpSocket::bind(public_addr));

        let server_config = ServerConfig {
            current_time,
            max_clients: 64,
            protocol_id: PROTOCOL_ID,
            public_addresses: vec![public_addr],
            authentication: ServerAuthentication::Unsecure,
        };

        let transport = error_return!(NetcodeServerTransport::new(server_config, socket));

        world.insert_resource(transport);
        world.insert_resource(CurrentClientId(current_time.as_millis() as u64));
    }
    world.insert_resource(server);
    world.insert_resource(Lobby::default());
    next_state.set(NetState::Server);
    info!("started server...");
    true
}

pub fn systems() -> SystemConfigs {
    (server_events,).into_configs()
}

pub fn errors() -> SystemConfigs {
    (error_on_error_system,)
        .into_configs()
        .run_if(resource_exists::<NetcodeServerTransport>)
}

pub fn errors_steam() -> SystemConfigs {
    (error_on_error_system_steam,)
        .into_configs()
        .run_if(resource_exists::<IsSteam>)
}

pub fn error_on_error_system_steam(mut renet_error: EventReader<SteamTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        error!("{}", e);
    }
}

pub fn error_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        error!("{}", e);
    }
}
