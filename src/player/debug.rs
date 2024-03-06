use super::Player;
use bevy::prelude::*;
use bevy_rapier3d::render::DebugRenderContext;

impl Player {
    pub fn debug(keys: Res<ButtonInput<KeyCode>>, mut debug: ResMut<DebugRenderContext>) {
        if keys.just_pressed(KeyCode::F2) {
            debug.enabled = !debug.enabled;
        }
    }
}
