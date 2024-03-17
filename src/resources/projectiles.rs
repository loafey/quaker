use std::fs::read_to_string;

use bevy::{ecs::system::Resource, utils::HashMap};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Projectile {
    pub id: String,
    pub model_file: String,
    pub texture_file: String,
    pub scale: f32,
    pub rotation: [f32; 3],
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
