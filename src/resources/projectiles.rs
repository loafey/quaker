use std::fs::read_to_string;

use bevy::{ecs::system::Resource, utils::HashMap};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Projectile {
    id: String,
    model_file: String,
    texture_file: String,
    scale: f32,
    rotation: [f32; 3],
}

#[derive(Debug, Resource)]
pub struct Projectiles(pub HashMap<String, Projectile>);
impl Projectiles {
    pub fn new() -> Self {
        let input = read_to_string("assets/projectiles.json").unwrap();
        let json = serde_json::from_str::<Vec<Projectile>>(&input).unwrap();
        Self(json.into_iter().map(|p| (p.id.clone(), p)).collect())
    }
}
