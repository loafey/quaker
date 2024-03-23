use std::path::PathBuf;

use crate::entities::{pickup::PickupEntity, ProjectileEntity};
use crate::map_gen::{load_map, texture_systems::*};
use crate::net::{self, NetState};
use crate::player::Player;
use crate::resources::{
    entropy::{entropy_game, entropy_misc},
    inputs::PlayerInput,
    projectiles::Projectiles,
    *,
};
use crate::{mainmenu, startup};
use bevy::prelude::*;

pub struct Resources;
impl Resources {
    fn get_map() -> PathBuf {
        if let Some(map) = std::env::args().nth(1) {
            if std::fs::File::open(&map).is_ok() {
                return map.into();
            } else {
                error!("Can't find map: \"{map}\"")
            }
        }

        "assets/maps/Test.map".into()
    }
}
impl Plugin for Resources {
    fn build(&self, app: &mut App) {
        app.init_state::<CurrentStage>()
            .init_state::<NetState>()
            .insert_resource(CurrentMap(Self::get_map()))
            .insert_resource(TextureLoadingState::NotLoaded)
            .insert_resource(TexturesLoading::default())
            .insert_resource(TextureMap::default())
            .insert_resource(PlayerSpawnpoint(Vec3::ZERO))
            .insert_resource(MapDoneLoading(false))
            .insert_resource(Paused(true))
            .insert_resource(PickupMap::new())
            .insert_resource(WeaponMap::new())
            .insert_resource(PlayerInput::new())
            .insert_resource(entropy_game())
            .insert_resource(entropy_misc())
            .insert_resource(Projectiles::new());
    }
}

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                net::server::systems(),
                net::server::errors(),
                net::server::errors_steam(),
            )
                .run_if(in_state(NetState::Server)),
        );
    }
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                net::client::systems(),
                net::client::errors(),
                net::client::errors_steam(),
            )
                .run_if(in_state(NetState::Client)),
        )
        .add_systems(
            PreUpdate,
            net::send_messages
                .run_if(in_state(NetState::Server).or_else(in_state(NetState::Client))),
        );
    }
}

pub struct StartupStage;
impl Plugin for StartupStage {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            startup::startup_setup.run_if(in_state(CurrentStage::Startup)),
        )
        .add_systems(
            Update,
            startup::startup_update.run_if(in_state(CurrentStage::Startup)),
        );
    }
}

pub struct MainMenuStage;
impl Plugin for MainMenuStage {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(CurrentStage::MainMenu),
            mainmenu::setup.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(
            Update,
            mainmenu::update_level_buttons.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(
            Update,
            mainmenu::buttons.run_if(in_state(CurrentStage::MainMenu)),
        )
        .add_systems(OnExit(CurrentStage::MainMenu), mainmenu::clear);
    }
}

pub struct GameStage;
impl Plugin for GameStage {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(OnEnter(CurrentStage::InGame), register_textures)
            .add_systems(
                Update,
                texture_waiter
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_texture_loading),
            )
            .add_systems(
                Update,
                load_map
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_texture_done_loading.and_then(run_once())),
            )
            .add_systems(
                PreUpdate,
                PlayerInput::update.run_if(in_state(CurrentStage::InGame)),
            )
            .add_systems(
                Update,
                Player::spawn_own_player
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_map_done_loading.and_then(run_once())),
            )
            .add_systems(
                Update,
                (
                    Player::systems(),
                    PickupEntity::systems(),
                    ProjectileEntity::systems(),
                )
                    .run_if(in_state(CurrentStage::InGame))
                    .run_if(if_not_paused),
            )
            .add_systems(
                Update,
                (Player::pause_handler, Player::debug).run_if(in_state(CurrentStage::InGame)),
            );
    }
}
