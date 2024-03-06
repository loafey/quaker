use std::collections::HashMap;

use bevy::{
    asset::{Handle, UntypedHandle},
    ecs::system::{Res, Resource},
    math::Vec3,
    render::texture::Image,
};

/// String to the current map
#[derive(Debug, Resource)]
pub struct CurrentMap(pub String);

/// Represents the pause state of the game
#[derive(Debug, Resource)]
pub struct Paused(pub bool);
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

/// A map which provides Path -> Handle for textures
#[derive(Debug, Resource, Default)]
pub struct TextureMap(pub HashMap<String, Handle<Image>>);
