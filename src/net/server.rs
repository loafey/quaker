use bevy::{
    ecs::{
        event::EventReader,
        schedule::{IntoSystemConfigs, NextState, SystemConfigs},
        system::Commands,
    },
    log::info,
};
use bevy_renet::renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    RenetServer, ServerEvent,
};
use macros::error_return;
use std::{net::UdpSocket, time::SystemTime};

use super::{connection_config, NetState, PROTOCOL_ID};

pub fn init_server(commands: &mut Commands, next_state: &mut NextState<NetState>) {
    let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));

    let server = RenetServer::new(connection_config());

    let public_addr = "127.0.0.1:8000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();

    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();

    commands.insert_resource(server);
    commands.insert_resource(transport);
    next_state.set(NetState::Server);
    info!("started server...");
}

pub fn systems() -> SystemConfigs {
    (server_events,).into_configs()
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
