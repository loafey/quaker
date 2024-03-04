use crate::CurrentMap;
use bevy::prelude::*;
use macros::error_return;
use std::collections::HashMap;

#[derive(Debug, Resource, Default)]
pub struct TexturesLoading(Vec<UntypedHandle>);

#[derive(Debug, Resource, Default)]
pub struct TextureMap(pub HashMap<String, Handle<Image>>);
pub fn load_textures(
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    mut textures_loading: ResMut<TexturesLoading>,
    mut texture_map: ResMut<TextureMap>,
) {
    warn!("Registering textures...");
    let time = std::time::Instant::now();
    let map = error_return!(std::fs::read_to_string(&current_map.0));
    let map = error_return!(map_parser::parse(&map));

    let mut textures = map
        .into_iter()
        .flat_map(|e| e.brushes)
        .flatten()
        .map(|p| p.texture)
        .collect::<Vec<_>>();

    textures.sort();
    textures.dedup();

    let mut map = HashMap::new();
    for texture in textures {
        //let handle = asset_server.load_with_settings::<Image, ImageLoaderSettings>(
        //    &format!("textures/{texture}.png"),
        //    |s| {s.sampler.},
        //);
        let handle = asset_server.load::<Image>(&format!("textures/{texture}.png"));
        textures_loading.0.push(handle.clone().untyped());
        map.insert(texture, handle);
    }
    texture_map.0 = map;
    warn!(
        "Done registering textures, took {}s",
        time.elapsed().as_secs_f32()
    );
}

pub fn if_texture_loading(text: Res<TexturesLoading>) -> bool {
    !text.0.is_empty()
}
pub fn if_texture_done_loading(text: Res<TexturesLoading>) -> bool {
    text.0.is_empty()
}

pub fn texture_checker(
    mut textures_loading: ResMut<TexturesLoading>,
    asset_server: Res<AssetServer>,
) {
    use bevy::asset::LoadState::*;
    let mut to_remove = Vec::new();
    for (i, tex) in textures_loading.0.iter().enumerate() {
        if let Some(Loaded | Failed) = asset_server.get_load_state(tex.id()) {
            to_remove.push(i)
        }
    }
    for (offset, i) in to_remove.into_iter().enumerate() {
        textures_loading.0.remove(i - offset);
    }

    if textures_loading.0.is_empty() {
        warn!("Texture loading done...");
    }
}
