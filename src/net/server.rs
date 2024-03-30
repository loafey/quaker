use super::{
    connection_config, update_world, ClientChannel, ClientMessage, NetState, SimulationEvent,
    PROTOCOL_ID,
};
use crate::{
    entities::hitscan_hit_gfx,
    map_gen::entities::data::Attack,
    net::{CurrentClientId, IsSteam, Lobby, ServerChannel, ServerMessage},
    player::Player,
    queries::NetWorld,
    resources::CurrentMap,
};
use bevy::{
    ecs::{
        event::EventReader,
        schedule::{
            common_conditions::resource_exists, IntoSystemConfigs, NextState, SystemConfigs,
        },
        system::{NonSend, Res, ResMut},
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
use macros::{error_continue, error_return, option_continue, option_return};
use renet_steam::{
    bevy::SteamTransportError, AccessPermission, SteamServerConfig, SteamServerTransport,
};
use std::{net::UdpSocket, time::SystemTime};

fn transmit_message(server: &mut RenetServer, nw: &mut NetWorld, text: String) {
    for (_, player, _) in &nw.players {
        if player.id == nw.current_id.0 {
            player.display_message(&mut nw.commands, &nw.asset_server, text.clone());
            break;
        }
    }
    server.broadcast_message(
        ServerChannel::ServerMessages as u8,
        error_return!(ServerMessage::Message { text }.bytes()),
    );
}

fn frag_checker(server: &mut RenetServer, nw: &mut NetWorld) {
    let mut frags = Vec::new();
    for (_, mut player, mut trans) in &mut nw.players {
        if player.health <= 0.0 {
            player.health = 100.0;
            player.armour = 0.0;

            if player.id == nw.current_id.0 {
                trans.translation = nw.player_spawn.0;
            } else {
                server.send_message(
                    ClientId::from_raw(player.id),
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::Reset.bytes()),
                );
            }

            frags.push((player.id, player.last_hurter));
        }
    }

    for (id, hurter) in frags {
        transmit_message(server, nw, format!("{} GOT FRAGGED BY {}", id, hurter));
    }
}

#[allow(clippy::type_complexity)]
pub fn server_events(
    mut events: EventReader<ServerEvent>,
    mut sim_events: EventReader<SimulationEvent>,
    mut server: ResMut<RenetServer>,

    map: Res<CurrentMap>,
    mut nw: NetWorld,
) {
    frag_checker(&mut server, &mut nw);

    // Handle connection details
    let mut messages = Vec::new();
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                let message = format!("PLAYER {client_id} JOINED");
                info!("{message}");
                messages.push(message);

                server.send_message(
                    *client_id,
                    ServerChannel::ServerMessages as u8,
                    error_return!(ServerMessage::SetMap(map.0.clone()).bytes()),
                );
                nw.lobby.cam_count += 2;

                for (pickup, trans) in &nw.pickups_query {
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
                for (other_id, ent) in &nw.lobby.players {
                    let (_, pl, trans) = error_continue!(nw.players.get(*ent));
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

                let spawn_point = nw.player_spawn.0;
                let entity = Player::spawn(
                    &mut nw,
                    false,
                    spawn_point,
                    client_id.raw(),
                    Vec::new(),
                    None,
                );
                nw.lobby.players.insert(*client_id, entity);

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
                let message = format!(
                    "PLAYER {client_id} LEFT: {}",
                    format!("{reason}").to_uppercase()
                );
                info!("{message}");
                messages.push(message);

                nw.lobby.players.remove(client_id);

                for (e, p, _) in &nw.players {
                    if p.id == client_id.raw() {
                        nw.commands.entity(e).despawn_recursive();
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

    for message in messages {
        transmit_message(&mut server, &mut nw, message);
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

                update_world(*player, &pickup_message, &mut nw);

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
            handle_client_message(&mut server, client_id.raw(), message, &mut nw);
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Command as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id.raw(), message, &mut nw);
        }
    }
}

pub fn handle_client_message(
    server: &mut RenetServer,
    client_id: u64,
    message: ClientMessage,
    nw: &mut NetWorld,
) {
    match message {
        ClientMessage::Fire { attack } => {
            let mut hit_pos = Vec::new();
            let mut hit_ents = Vec::new();
            let mut attack_weapon = None;
            for (player_entity, mut player, trans) in &mut nw.players {
                if player.id == client_id {
                    let cam = option_continue!(player.children.camera);
                    let (_, cam_trans) = error_continue!(nw.cameras.get(cam));
                    let (slot, row) = option_return!(player.current_weapon);
                    attack_weapon = Some(player.weapons[slot][row].data.id.clone());
                    let hits = player.attack(
                        attack,
                        &mut nw.materials,
                        player_entity,
                        &mut nw.commands,
                        &nw.rapier_context,
                        cam_trans,
                        &trans,
                        &mut nw.game_entropy,
                        &nw.projectile_map,
                        &nw.asset_server,
                    );
                    for (hit, pos) in hits {
                        hit_pos.push(pos);
                        hit_ents.push(hit);
                    }
                    break;
                }
            }

            let attack_weapon = error_return!(attack_weapon
                .ok_or_else(|| format!("player {} attacked without holding weapon", client_id)));
            let attack_weapon = error_return!(nw
                .weapon_map
                .0
                .get(&attack_weapon)
                .ok_or_else(|| format!("failed to find weapon {attack_weapon}")));
            for ent in hit_ents {
                if let Ok((_, mut hit_player, _)) = nw.players.get_mut(ent) {
                    hit_player.last_hurter = client_id;
                    let damage = if attack == 1 {
                        if let Attack::RayCast {
                            damage, damage_mod, ..
                        } = &attack_weapon.attack1
                        {
                            damage + (damage_mod * (nw.game_entropy.get_f32() * 2.0 - 1.0))
                        } else {
                            error!("weird attack 1");
                            0.0
                        }
                    } else if let Attack::RayCast {
                        damage, damage_mod, ..
                    } = &attack_weapon.attack2
                    {
                        damage + (damage_mod * (nw.game_entropy.get_f32() * 2.0 - 1.0))
                    } else {
                        error!("weird attack 2");
                        0.0
                    };
                    hit_player.health -= damage;
                    if hit_player.id != nw.current_id.0 {
                        server.send_message(
                            ClientId::from_raw(hit_player.id),
                            ServerChannel::NetworkedEntities as u8,
                            error_continue!(ServerMessage::Hit { amount: damage }.bytes()),
                        )
                    } else {
                        for (_, mut player, _) in &mut nw.players {
                            if player.id == nw.current_id.0 {
                                player.health -= damage;
                                break;
                            }
                        }
                    }
                }
            }

            hitscan_hit_gfx(
                &mut nw.commands,
                &hit_pos,
                &mut nw.meshes,
                &mut nw.materials,
            );
            server.broadcast_message(
                ServerChannel::NetworkedEntities as u8,
                error_return!(ServerMessage::HitscanHits { hits: hit_pos }.bytes()),
            );
        }
        message => {
            update_world(client_id, &message, nw);
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
