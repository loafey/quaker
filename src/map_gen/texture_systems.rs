use bevy::{asset::LoadState, prelude::*};
use macros::error_return;
use resources::{CurrentMap, TextureLoadingState, TextureMap, TexturesLoading};
use std::collections::HashMap;

pub fn register_textures(
    asset_server: Res<AssetServer>,
    current_map: Res<CurrentMap>,
    mut textures_loading: ResMut<TexturesLoading>,
    mut loading_state: ResMut<TextureLoadingState>,
    mut texture_map: ResMut<TextureMap>,
) {
    info!("Registering textures...");
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
    info!(
        "Done registering textures, took {}s",
        time.elapsed().as_secs_f32()
    );
    *loading_state = TextureLoadingState::Loading;
}

pub fn texture_waiter(
    mut textures_loading: ResMut<TexturesLoading>,
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<TextureLoadingState>,
) {
    let mut to_remove = Vec::new();
    for (i, tex) in textures_loading.0.iter().enumerate() {
        let g = asset_server.get_load_state(tex.id());
        if let Some(LoadState::Failed(e)) = g {
            error!("texture loading error: {e}");
            to_remove.push(i)
        } else if let Some(LoadState::Loaded | LoadState::Failed(_)) =
            asset_server.get_load_state(tex.id())
        {
            to_remove.push(i)
        }
    }
    for (offset, i) in to_remove.into_iter().enumerate() {
        textures_loading.0.remove(i - offset);
    }

    if textures_loading.0.is_empty() {
        info!("Texture loading done...");
        *loading_state = TextureLoadingState::Done;
    }
}
