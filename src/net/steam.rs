use crate::APP_ID;
use bevy::{log::error, prelude::*, render::render_asset::RenderAssetUsages};
use image::{DynamicImage, ImageBuffer};
use macros::{error_return, option_return};
use std::sync::mpsc;
use steamworks::{Client, SingleClient};

#[derive(Resource)]
pub struct SteamClient {
    client: Client,
}
impl SteamClient {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}
impl std::ops::Deref for SteamClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
impl std::ops::DerefMut for SteamClient {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

pub fn try_steam() -> Option<(Client, SingleClient)> {
    let (s, r) = mpsc::channel();
    std::thread::spawn(move || {
        let a = error_return!(steamworks::Client::init_app(APP_ID));
        error_return!(s.send(a));
    });

    match r.recv() {
        Ok(r) => Some(r),
        Err(e) => {
            error!("{e}");
            error!("running without Steam");
            None
        }
    }
}

/// Grabs the current users avatar and stores the handle in [CurrentAvatar](CurrentAvatar)
pub fn grab_avatar(
    mut commands: Commands,
    client: Option<Res<SteamClient>>,
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

#[derive(Debug, Resource)]
pub struct CurrentAvatar(pub Handle<Image>);
