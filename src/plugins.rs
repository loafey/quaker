use crate::entities::{pickup::PickupEntity, ProjectileEntity};
use crate::map_gen::{load_map, texture_systems::*};
use crate::player::Player;
use crate::resources::{
    entropy::{entropy_game, entropy_misc},
    inputs::PlayerInput,
    projectiles::Projectiles,
    *,
};
use crate::startup;
use bevy::prelude::*;

pub struct Resources;
impl Resources {
    fn get_map() -> String {
        if let Some(map) = std::env::args().nth(1) {
            if std::fs::File::open(&map).is_ok() {
                return map;
            } else {
                error!("Can't find map: \"{map}\"")
            }
        }

        "assets/maps/Test.map".to_string()
    }
}
impl Plugin for Resources {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentMap(Self::get_map()))
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
            .insert_resource(Projectiles::new())
            .insert_resource(CurrentStage::Startup);
    }
}

pub struct StartupStage;
impl Plugin for StartupStage {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            startup::startup_setup.run_if(CurrentStage::on_startup),
        )
        .add_systems(
            Update,
            startup::startup_update.run_if(CurrentStage::on_startup),
        );
    }
}

pub struct GameStage;
impl Plugin for GameStage {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, load_textures)
            .add_systems(PreUpdate, PlayerInput::update.run_if(CurrentStage::in_game))
            .add_systems(
                Update,
                load_map
                    .run_if(CurrentStage::in_game)
                    .run_if(if_texture_done_loading.and_then(run_once())),
            )
            .add_systems(
                Update,
                texture_checker
                    .run_if(CurrentStage::in_game)
                    .run_if(if_texture_loading),
            )
            .add_systems(
                Update,
                Player::spawn
                    .run_if(CurrentStage::in_game)
                    .run_if(if_map_done_loading.and_then(run_once())),
            )
            .add_systems(
                Update,
                (
                    Player::systems(),
                    PickupEntity::systems(),
                    ProjectileEntity::systems(),
                )
                    .run_if(CurrentStage::in_game)
                    .run_if(if_not_paused),
            )
            .add_systems(
                Update,
                (Player::pause_handler, Player::debug).run_if(CurrentStage::in_game),
            );
    }
}
