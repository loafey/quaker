use super::{Player, PlayerController};
use bevy::prelude::*;
use bevy_rapier3d::render::DebugRenderContext;
use macros::{error_continue, option_continue};

impl Player {
    pub fn debug(
        keys: Res<ButtonInput<KeyCode>>,
        mut debug: ResMut<DebugRenderContext>,
        players: Query<&Player, With<PlayerController>>,
        mut debug_hud: Query<(&mut Text, &mut Visibility)>,
    ) {
        for player in &players {
            let (mut hud, mut visibility) =
                error_continue!(debug_hud.get_mut(option_continue!(player.children.debug_hud)));

            if keys.just_pressed(KeyCode::F2) {
                debug.enabled = !debug.enabled;

                *visibility = match debug.enabled {
                    true => Visibility::Visible,
                    false => Visibility::Hidden,
                };
            }

            if debug.enabled {
                hud.sections[0].value = format!("{:#?}", player.debug_info);
            }
        }
    }
}
