use bevy::{
    ecs::{
        event::EventReader,
        schedule::{
            common_conditions::resource_exists, IntoSystemConfigs, NextState, SystemConfigs,
        },
        system::NonSend,
        world::World,
    },
    log::{error, info},
};
use bevy_renet::renet::{
    transport::{
        NetcodeServerTransport, NetcodeTransportError, ServerAuthentication, ServerConfig,
    },
    RenetServer, ServerEvent,
};
use macros::error_return;
use renet_steam::{
    bevy::SteamTransportError, AccessPermission, SteamServerConfig, SteamServerTransport,
};
use std::{net::UdpSocket, time::SystemTime};

use crate::net::IsSteam;

use super::{connection_config, NetState, PROTOCOL_ID};

pub fn init_server(
    world: &mut World,
    next_state: &mut NextState<NetState>,
    steam_client: &Option<NonSend<steamworks::Client>>,
) {
    let server = RenetServer::new(connection_config());

    if let Some(sc) = steam_client {
        let steam_transport_config = SteamServerConfig {
            max_clients: 64,
            access_permission: AccessPermission::Public,
        };

        let transport = error_return!(SteamServerTransport::new(sc, steam_transport_config));

        world.insert_resource(IsSteam);
        world.insert_non_send_resource(transport);
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
    }
    world.insert_resource(server);
    next_state.set(NetState::Server);
    info!("started server...");
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

pub fn server_events(mut events: EventReader<ServerEvent>) {
    for event in events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => info!("Player: {client_id} joined"),
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Player: {client_id} left due to {reason}")
            }
        }
    }
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
