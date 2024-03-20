use super::{connection_config, CurrentClientId, NetState, ServerChannel, PROTOCOL_ID};
use bevy::{
    ecs::{
        event::EventReader,
        schedule::{IntoSystemConfigs, NextState, SystemConfigs},
        system::{Commands, ResMut},
    },
    log::info,
};
use bevy_renet::renet::{
    transport::{ClientAuthentication, NetcodeClientTransport, NetcodeTransportError},
    RenetClient,
};
use macros::error_return;
use std::{net::UdpSocket, time::SystemTime};

pub fn init_client(commands: &mut Commands, next_state: &mut NextState<NetState>, ip: &String) {
    info!("joining ip: {ip}");
    let current_time = error_return!(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH));

    let client = RenetClient::new(connection_config());

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

    commands.insert_resource(client);
    commands.insert_resource(transport);
    commands.insert_resource(CurrentClientId(client_id));
    next_state.set(NetState::Client);
    info!("started client");
}

pub fn systems() -> SystemConfigs {
    (panic_on_error_system, print_messages).into_configs()
}

pub fn print_messages(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages as u8) {
        println!("Message: {message:?}");
    }
}

pub fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
    #[allow(clippy::never_loop)]
    for e in renet_error.read() {
        panic!("{}", e);
    }
}
