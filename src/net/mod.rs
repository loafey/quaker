use crate::{
    map_gen::entities::data::PickupData,
    queries::NetWorld,
    resources::{projectiles::Projectiles, WeaponMap},
};
use bevy::{prelude::*, render::render_asset::RenderAssetUsages};
use bevy_kira_audio::{Audio, AudioControl};
use bevy_rapier3d::plugin::RapierContext;
use bevy_renet::renet::*;
use image::{DynamicImage, ImageBuffer};
use macros::{error_return, option_return};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use steamworks::Client;

pub mod client;
pub mod server;

#[derive(Debug, Resource)]
pub struct CurrentAvatar(pub Handle<Image>);

pub fn grab_avatar(
    mut commands: Commands,
    client: Option<NonSend<Client>>,
    mut images: ResMut<Assets<Image>>,
) {
    let client = option_return!(client);
    let avatar = option_return!(client
        .friends()
        .get_friend(client.user().steam_id())
        .small_avatar());

    let dyn_img = DynamicImage::ImageRgba8(error_return!(
        ImageBuffer::from_raw(32, 32, avatar).ok_or("failed to parse avatar data")
    ));

    let image = images.add(Image::from_dynamic(
        dyn_img,
        false,
        RenderAssetUsages::RENDER_WORLD,
    ));

    commands.insert_resource(CurrentAvatar(image));
}

pub fn update_world(client_id: u64, message: &ClientMessage, net_world: &mut NetWorld) {
    match message {
        ClientMessage::UpdatePosition {
            position,
            rotation,
            cam_rot,
        } => {
            if net_world.current_id.0 != client_id {
                for (_, pl, mut tr) in net_world.players.iter_mut() {
                    if pl.id == client_id {
                        tr.translation = tr
                            .translation
                            .lerp(*position, net_world.time.delta_seconds() * 10.0);
                        tr.rotation = Quat::from_array(*rotation);

                        error_return!(net_world
                            .cameras
                            .get_mut(option_return!(pl.children.camera)))
                        .1
                        .rotation
                        .x = *cam_rot;

                        break;
                    }
                }
            }
        }
        ClientMessage::PickupWeapon { weapon } => {
            for (_, mut player, _) in net_world.players.iter_mut() {
                if player.id == client_id {
                    if let Some(weapon_data) = net_world.weapon_map.0.get(weapon) {
                        let slot = weapon_data.slot;
                        let handle = net_world
                            .asset_server
                            .load(format!("{}#Scene0", weapon_data.model_file));
                        if player.add_weapon(weapon_data.clone(), slot, handle) {
                            net_world.audio.play(net_world.asset_server.load(
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
        ClientMessage::Fire { .. } => {
            error!("got a fire event! This is wrong!");
        }
        ClientMessage::WeaponAnim { anim } => {
            if net_world.current_id.0 != client_id {
                for (_, mut pl, _) in net_world.players.iter_mut() {
                    if pl.id == client_id {
                        pl.current_weapon_anim = anim.clone();
                        pl.restart_anim = true;
                        break;
                    }
                }
            }
        }
        ClientMessage::SwitchWeapon { slot, row } => {
            if net_world.current_id.0 != client_id {
                for (_, mut pl, _) in net_world.players.iter_mut() {
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
    mut commands: Commands,
    mut events: EventReader<ClientMessage>,
    client: Option<ResMut<RenetClient>>,
    server: Option<ResMut<RenetServer>>,
    current_id: Res<CurrentClientId>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    rapier_context: Res<RapierContext>,
    (projectile_map, time): (Res<Projectiles>, Res<Time>),
    mut net_world: NetWorld,
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
                &asset_server,
                &audio,
                &rapier_context,
                &projectile_map,
                &mut commands,
                &time,
                &mut net_world,
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
        attack: usize,
    },

    SwitchWeapon {
        slot: usize,
        row: usize,
    },

    WeaponAnim {
        anim: String,
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
        weapons: Vec<Vec<String>>,
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
