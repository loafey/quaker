use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_renet::renet::*;
use macros::error_return;
use serde::{Deserialize, Serialize};

use crate::player::Player;

pub mod client;
pub mod server;

pub fn send_messages(
    mut events: EventReader<ClientMessage>,
    client: Option<ResMut<RenetClient>>,
    server: Option<ResMut<RenetServer>>,
    current_id: Res<CurrentClientId>,
    mut players: Query<(Entity, &Player, &mut Transform)>,
) {
    let mut send: Box<dyn FnMut(ClientMessage)> = if let Some(mut client) = client {
        Box::new(move |message| {
            client.send_message(ClientChannel::Input as u8, error_return!(message.bytes()));
        })
    } else if let Some(mut server) = server {
        Box::new(move |message| {
            server::handle_client_message(
                &mut server,
                ClientId::from_raw(current_id.0),
                message,
                &mut players,
                current_id.0,
            )
        })
    } else {
        error!("no way to handle messages");
        return;
    };

    for event in events.read() {
        send(event.clone());
    }
}

#[derive(Debug, Resource)]
pub struct IsSteam;

#[derive(Debug, States, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum NetState {
    #[default]
    Offline,
    Server,
    Client,
}

#[derive(Debug, Serialize, Deserialize, Event, Clone)]
pub enum ClientMessage {
    UpdatePosition { position: Vec3 },
}
impl ClientMessage {
    pub fn bytes(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    SetMap(PathBuf),
    SpawnPlayer { id: u64, translation: Vec3 },
    PlayerUpdate { id: u64, message: ClientMessage },
    DespawnPlayer { id: u64 },
}
impl ServerMessage {
    pub fn bytes(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
}

pub const PROTOCOL_ID: u64 = 7;

#[derive(Debug, Resource)]
pub struct CurrentClientId(pub u64);

#[repr(u8)]
pub enum ClientChannel {
    Input,
    Command,
}

#[repr(u8)]
pub enum ServerChannel {
    ServerMessages,
    NetworkedEntities,
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::Input as u8,
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
            ChannelConfig {
                channel_id: Self::Command as u8,
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::ZERO,
                },
            },
        ]
    }
}
impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ChannelConfig {
                channel_id: Self::NetworkedEntities as u8,
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::ServerMessages as u8,
                max_memory_usage_bytes: 10 * 1024 * 1024,
                send_type: SendType::ReliableOrdered {
                    resend_time: Duration::from_millis(200),
                },
            },
        ]
    }
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        server_channels_config: ServerChannel::channels_config(),
    }
}
