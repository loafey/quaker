use crate::queries::NetWorld;
use bevy::prelude::*;
use bevy_renet::renet::*;
use faststr::FastStr;
use macros::{error_return, option_return};
use resources::data::PickupData;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

use self::steam::SteamClient;

pub mod client;
mod lobby;
pub mod server;
pub mod steam;
pub use lobby::*;

pub fn update_world(client_id: u64, message: &ClientMessage, nw: &mut NetWorld) {
    match message {
        ClientMessage::UpdatePosition {
            position,
            rotation,
            cam_rot,
        } => {
            if nw.current_id.0 != client_id {
                let player = option_return!(nw.lobby.get(&client_id)).entity;
                let (_, pl, mut tr) = error_return!(nw.players.get_mut(player));

                tr.translation = tr.translation.lerp(*position, nw.time.delta_secs() * 10.0);
                tr.rotation = Quat::from_array(*rotation);

                error_return!(nw.cameras.get_mut(option_return!(pl.children.camera)))
                    .1
                    .rotation
                    .x = *cam_rot;
            }
        }
        ClientMessage::PickupWeapon { weapon } => {
            let player = option_return!(nw.lobby.get(&client_id)).entity;

            let (player_ent, mut player, _) = error_return!(nw.players.get_mut(player));

            if let Some(weapon_data) = nw.weapon_map.0.get(weapon) {
                let slot = weapon_data.slot;
                let handle = nw
                    .asset_server
                    .load(format!("{}#Scene0", weapon_data.model_file));
                if player.add_weapon(weapon_data.clone(), slot, handle) {
                    nw.commands.entity(player_ent).with_children(|c| {
                        c.spawn(Mesh3d::default()).insert((
                            AudioPlayer::<AudioSource>(
                                nw.asset_server.load(
                                    weapon_data
                                        .pickup_sound
                                        .as_ref()
                                        .map(|f| f.to_string())
                                        .unwrap_or(
                                            "sounds/Player/Guns/SuperShotgun/shotgunCock.ogg"
                                                .to_string(),
                                        ),
                                ),
                            ),
                            PlaybackSettings::DESPAWN.with_spatial(true),
                        ));
                    });

                    if player.id == nw.current_id.0 {
                        player.display_message(
                            &mut nw.commands,
                            &nw.asset_server,
                            format!(
                                "{}{}{}",
                                weapon_data.pickup_message1,
                                weapon_data.fancy_name,
                                weapon_data.pickup_message2
                            ),
                        );
                    }
                }
            } else {
                error!("tried to pickup nonexisting weapon: \"{weapon}\"")
            }
        }
        ClientMessage::WeaponAnim { anim } => {
            if nw.current_id.0 != client_id {
                let player = option_return!(nw.lobby.get(&client_id)).entity;

                let (_, mut pl, _) = error_return!(nw.players.get_mut(player));

                pl.current_weapon_anim.clone_from(anim);
                pl.restart_anim = true;
            }
        }
        ClientMessage::SwitchWeapon { slot, row } => {
            if nw.current_id.0 != client_id {
                let player = option_return!(nw.lobby.get(&client_id)).entity;
                let (_, mut pl, _) = error_return!(nw.players.get_mut(player));
                pl.current_weapon = Some((*slot, *row));
            }
        }
        x => error!("got event {x:?} from server, this was most likely a bug"),
    }
}

pub fn send_messages(
    mut events: EventReader<ClientMessage>,
    client: Option<ResMut<RenetClient>>,
    server: Option<ResMut<RenetServer>>,
    mut nw: NetWorld,
) {
    let mut send: Box<dyn FnMut(ClientMessage)> = if let Some(mut client) = client {
        Box::new(move |message| {
            client.send_message(ClientChannel::Input as u8, error_return!(message.bytes()));
        })
    } else if let Some(mut server) = server {
        Box::new(move |message| {
            server::handle_client_message(&mut server, nw.current_id.0, message, &mut nw)
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
        weapon: FastStr,
    },

    Fire {
        attack: usize,
    },

    SwitchWeapon {
        slot: usize,
        row: usize,
    },

    WeaponAnim {
        anim: FastStr,
    },

    Interact,
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
        pickup: FastStr,
    },
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum ServerMessage {
    SetMap(PathBuf),
    SpawnPlayer {
        id: u64,
        name: FastStr,
        translation: Vec3,
        weapons: Vec<Vec<FastStr>>,
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
    HitscanHits {
        hits: Vec<Vec3>,
    },
    Hit {
        amount: f32,
    },
    Reset,
    Message {
        text: String,
    },
    KillStat {
        death: u64,
        hurter: Option<u64>,
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
