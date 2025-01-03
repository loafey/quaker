use bevy::{
    asset::{Handle, UntypedHandle},
    ecs::system::{Res, Resource},
    image::Image,
    log::info,
    math::Vec3,
    prelude::States,
};
use data::{PickupData, WeaponData};
use faststr::FastStr;
use macros::error_return;
use qwak::QwakPlugin;
use std::{collections::HashMap, fs, path::PathBuf};

pub mod data;
pub mod entropy;
pub mod inputs;

#[derive(Debug, Resource)]
pub struct Qwaks {
    pub default: QwakPlugin,
    // pub other: HashMap<FastStr, QwakPlugin>,
}
impl Qwaks {
    pub fn new() -> Self {
        info!("Loading qwaks...");
        let default = match QwakPlugin::new("assets/qwaks/default.wasm") {
            Ok(o) => o,
            Err(e) => panic!("failed loading default qwak: {e}"),
        };

        info!("Done loading qwaks...");
        Self { default }
    }
}

/// Represents the current game stage
#[derive(Debug, Resource, PartialEq, Eq, States, Default, Hash, Clone, Copy)]
pub enum CurrentStage {
    #[default]
    Startup,
    MainMenu,
    InGame,
}

/// String to the current map
#[derive(Debug, Resource)]
pub struct CurrentMap(pub PathBuf);

/// Represents the pause state of the game
#[derive(Debug, Resource)]
pub struct Paused(pub bool);
#[allow(unused)]
pub fn if_not_paused(val: Res<Paused>) -> bool {
    !val.0
}

/// True if the map is done and loaded
#[derive(Resource)]
pub struct MapDoneLoading(pub bool);

pub fn if_map_done_loading(val: Res<MapDoneLoading>) -> bool {
    val.0
}

/// Represents where a player will spawn in the current level
#[derive(Resource)]
pub struct PlayerSpawnpoint(pub Vec3);

/// A list of which textures are currently being loaded
#[derive(Debug, Resource, Default)]
pub struct TexturesLoading(pub Vec<UntypedHandle>);

#[derive(Debug, Resource)]
pub enum TextureLoadingState {
    NotLoaded,
    Loading,
    Done,
}

#[allow(dead_code)]
pub fn if_textures_not_loaded(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::NotLoaded)
}
pub fn if_texture_loading(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::Loading)
}
pub fn if_texture_done_loading(text: Res<TextureLoadingState>) -> bool {
    matches!(*text, TextureLoadingState::Done)
}

/// A map which provides Path -> Handle for textures
#[derive(Debug, Resource, Default)]
pub struct TextureMap(pub HashMap<FastStr, Handle<Image>>);

/// A map with pickup data
#[derive(Debug, Resource, Default)]
pub struct PickupMap(pub HashMap<FastStr, PickupData>);
impl PickupMap {
    pub fn new() -> Self {
        info!("Loading pickups...");
        let data = error_return!(fs::read_to_string("assets/pickups.json"));
        let parsed = error_return!(serde_json::from_str::<Vec<PickupData>>(&data));

        let mut map = HashMap::new();
        for item in parsed {
            map.insert(item.classname.clone(), item);
        }

        info!("Done loading pickups...");
        Self(map)
    }
}

/// A map with weapon data
#[derive(Debug, Resource, Default)]
pub struct WeaponMap(pub HashMap<FastStr, WeaponData>);
impl WeaponMap {
    pub fn new() -> Self {
        info!("Loading pickups...");
        let data = error_return!(fs::read_to_string("assets/weapons.json"));
        let parsed = error_return!(serde_json::from_str::<Vec<WeaponData>>(&data));

        let mut map = HashMap::new();
        for item in parsed {
            map.insert(item.id.clone(), item);
        }

        info!("Done loading weapons...");
        Self(map)
    }
}
