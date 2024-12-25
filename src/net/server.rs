use super::{
    connection_config, update_world, ClientChannel, ClientMessage, NetState, SimulationEvent,
    SteamClient, PROTOCOL_ID,
};
use crate::{
    entities::hitscan_hit_gfx,
    map_gen::entities::data::Attack,
    net::{CurrentClientId, IsSteam, Lobby, PlayerInfo, ServerChannel, ServerMessage},
    player::Player,
    queries::NetWorld,
    resources::CurrentMap,
};
use bevy::{
    ecs::{
        event::EventReader,
        schedule::{common_conditions::resource_exists, IntoSystemConfigs, SystemConfigs},
        system::{Res, ResMut},
        world::World,
    },
    hierarchy::DespawnRecursiveExt,
    log::{error, info},
    prelude::NextState,
};
use bevy_renet::{
    netcode::{NetcodeServerTransport, NetcodeTransportError, ServerAuthentication, ServerConfig},
    renet::{RenetServer, ServerEvent},
    steam::SteamTransportError,
};
use faststr::FastStr;
use macros::{error_continue, error_return, option_return};
use renet_steam::{AccessPermission, SteamServerConfig, SteamServerTransport};
use std::{net::UdpSocket, time::SystemTime};
use steamworks::SteamId;

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
                    player.id,
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::Reset.bytes()),
                );
            }

            frags.push((player.id, player.last_hurter));
            player.last_hurter = 0;
        }
    }

    for (id, hurter) in frags {
        server.broadcast_message(
            ServerChannel::ServerMessages as u8,
            error_continue!(ServerMessage::KillStat {
                death: id,
                hurter: (hurter != 0).then_some(hurter)
            }
            .bytes()),
        );
        let id = if let Some(info) = nw.lobby.get_mut(&id) {
            info.deaths += 1;
            info.name.clone()
        } else {
            format!("{id}").into()
        };
        let hurter = if let Some(info) = nw.lobby.get_mut(&hurter) {
            info.kills += 1;
            info.name.clone()
        } else {
            format!("{hurter}").into()
        };
        transmit_message(
            server,
            nw,
            format!(
                "{} GOT FRAGGED BY {}",
                id.to_lowercase(),
                hurter.to_lowercase()
            ),
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn server_events(
    mut events: EventReader<ServerEvent>,
    mut sim_events: EventReader<SimulationEvent>,
    mut server: ResMut<RenetServer>,

    steam: Option<Res<SteamClient>>,
    map: Res<CurrentMap>,
    mut nw: NetWorld,
) {
    frag_checker(&mut server, &mut nw);

    // Handle connection details
    let mut messages = Vec::new();
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                server.send_message(
                    *client_id,
                    ServerChannel::ServerMessages as u8,
                    error_return!(ServerMessage::SetMap(map.0.clone()).bytes()),
                );

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
                for (other_id, info) in &nw.lobby {
                    let (_, pl, trans) = error_continue!(nw.players.get(info.entity));
                    server.send_message(
                        *client_id,
                        ServerChannel::ServerMessages as u8,
                        error_continue!(ServerMessage::SpawnPlayer {
                            name: info.name.clone(),
                            id: *other_id,
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
                let entity =
                    Player::spawn(&mut nw, false, spawn_point, *client_id, Vec::new(), None);
                let name = FastStr::from(
                    steam
                        .as_ref()
                        .map(|s| s.friends().get_friend(SteamId::from_raw(*client_id)))
                        .map(|f| f.name())
                        .unwrap_or(format!("{client_id}")),
                );
                nw.lobby
                    .insert(*client_id, PlayerInfo::new(entity, name.clone()));

                let message = format!("PLAYER {} JOINED", name.to_lowercase());
                info!("{message}");
                messages.push(message);

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::SpawnPlayer {
                        id: *client_id,
                        translation: spawn_point,
                        weapons: Vec::new(),
                        name
                    }
                    .bytes()),
                )
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                if let Some(player_info) = nw.lobby.remove(client_id) {
                    nw.commands.entity(player_info.entity).despawn_recursive();
                    let message = format!(
                        "PLAYER {} LEFT: {}",
                        player_info.name.to_lowercase(),
                        format!("{reason}").to_uppercase()
                    );
                    info!("{message}");
                    messages.push(message);
                }

                server.broadcast_message(
                    ServerChannel::ServerMessages as u8,
                    error_continue!(ServerMessage::DespawnPlayer { id: *client_id }.bytes()),
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
            handle_client_message(&mut server, client_id, message, &mut nw);
        }

        while let Some(message) = server.receive_message(client_id, ClientChannel::Command as u8) {
            let message = error_continue!(ClientMessage::from_bytes(&message));
            handle_client_message(&mut server, client_id, message, &mut nw);
        }
    }
}

pub fn handle_client_message(
    server: &mut RenetServer,
    client_id: u64,
    message: ClientMessage,
    nw: &mut NetWorld,
) {
    let rapier_context = nw.rapier_context.single();
    match message {
        ClientMessage::Fire { attack } => {
            let mut hit_pos = Vec::new();
            let mut hit_ents = Vec::new();

            let player = option_return!(nw.lobby.get(&client_id)).entity;
            let (player_entity, mut player, trans) = error_return!(nw.players.get_mut(player));

            let cam = option_return!(player.children.camera);
            let (_, cam_trans) = error_return!(nw.cameras.get(cam));

            let (slot, row) = option_return!(player.current_weapon);
            let attack_weapon = Some(player.weapons[slot][row].data.id.clone());
            let hits = player.attack(
                attack,
                &mut nw.materials,
                player_entity,
                &mut nw.commands,
                rapier_context,
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
                            hit_player.id,
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

            hitscan_hit_gfx(&nw.asset_server, &mut nw.commands, &hit_pos, &nw.particles);
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
    steam_client: &Option<Res<SteamClient>>,
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
