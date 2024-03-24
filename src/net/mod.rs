use std::{path::PathBuf, time::Duration};

use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioControl};
use bevy_renet::renet::*;
use macros::{error_return, option_return};
use serde::{Deserialize, Serialize};

use crate::{map_gen::entities::data::PickupData, player::Player, resources::WeaponMap};

pub mod client;
pub mod server;

pub fn update_world(
    client_id: u64,
    message: &ClientMessage,
    players: &mut Query<(Entity, &mut Player, &mut Transform)>,
    cameras: &mut Query<(&Camera3d, &mut Transform), Without<Player>>,
    current_id: u64,
    asset_server: &AssetServer,
    weapon_map: &WeaponMap,
    audio: &Audio,
) {
    match message {
        ClientMessage::UpdatePosition {
            position,
            rotation,
            cam_rot,
        } => {
            if current_id != client_id {
                for (_, pl, mut tr) in players.iter_mut() {
                    if pl.id == client_id {
                        tr.translation = *position;
                        tr.rotation = Quat::from_array(*rotation);

                        error_return!(cameras.get_mut(option_return!(pl.children.camera)))
                            .1
                            .rotation
                            .x = *cam_rot;

                        break;
                    }
                }
            }
        }
        ClientMessage::PickupWeapon { weapon } => {
            for (_, mut player, _) in players.iter_mut() {
                if player.id == client_id {
                    if let Some(weapon_data) = weapon_map.0.get(weapon) {
                        let slot = weapon_data.slot;
                        let handle =
                            asset_server.load(format!("{}#Scene0", weapon_data.model_file));
                        if player.add_weapon(weapon_data.clone(), slot, handle) {
                            audio.play(asset_server.load(
                                weapon_data.pickup_sound.clone().unwrap_or(
                                    "sounds/Player/Guns/SuperShotgun/shotgunCock.ogg".to_string(),
                                ),
                            ));
                        }
                    } else {
                        error!("tried to pickup nonexisting weapon: \"{weapon}\"")
                    }

                    break;
                }
            }
        }
        ClientMessage::Fire { slot, row, attack } => {
            error!("unhandled firing [{slot}, {row}] {attack}")
        }
        ClientMessage::SwitchWeapon { slot, row } => {
            error!("unhandled switch weapon [{slot}, {row}]");
            if current_id != client_id {
                for (_, mut pl, _) in players.iter_mut() {
                    if pl.id == client_id {
                        pl.current_weapon = Some((*slot, *row));
                        break;
                    }
                }
            }
        }
    }
}

pub fn send_messages(
    mut events: EventReader<ClientMessage>,
    client: Option<ResMut<RenetClient>>,
    server: Option<ResMut<RenetServer>>,
    current_id: Res<CurrentClientId>,
    mut players: Query<(Entity, &mut Player, &mut Transform)>,
    mut cameras: Query<(&Camera3d, &mut Transform), Without<Player>>,
    asset_server: Res<AssetServer>,
    weapon_map: Res<WeaponMap>,
    audio: Res<Audio>,
) {
    let mut send: Box<dyn FnMut(ClientMessage)> = if let Some(mut client) = client {
        Box::new(move |message| {
            client.send_message(ClientChannel::Input as u8, error_return!(message.bytes()));
        })
    } else if let Some(mut server) = server {
        Box::new(move |message| {
            server::handle_client_message(
                &mut server,
                current_id.0,
                message,
                &mut players,
                &mut cameras,
                current_id.0,
                &asset_server,
                &weapon_map,
                &audio,
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
    UpdatePosition {
        position: Vec3,
        rotation: [f32; 4],
        cam_rot: f32,
    },

    PickupWeapon {
        weapon: String,
    },

    Fire {
        slot: usize,
        row: usize,
        attack: u32,
    },

    SwitchWeapon {
        slot: usize,
        row: usize,
    },
}
impl ClientMessage {
    pub fn bytes(&self) -> Result<Vec<u8>, std::boxed::Box<bincode::ErrorKind>> {
        bincode::serialize(self)
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Box<bincode::ErrorKind>> {
        bincode::deserialize(bytes)
    }
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum SimulationEvent {
    PlayerPicksUpPickup {
        id: u64,
        player: u64,
        pickup: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum ServerMessage {
    SetMap(PathBuf),
    SpawnPlayer {
        id: u64,
        translation: Vec3,
    },
    PlayerUpdate {
        id: u64,
        message: ClientMessage,
    },
    DespawnPlayer {
        id: u64,
    },
    SpawnPickup {
        id: u64,
        translation: Vec3,
        data: PickupData,
    },
    DespawnPickup {
        id: u64,
    },
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
