use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct MainMenuEnt;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(MainMenuEnt);

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|c| {
            c.spawn((
                TextBundle::from_section(
                    "Quaker!",
                    TextStyle {
                        font_size: 100.0,
                        ..default()
                    },
                )
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(5.0),
                    top: Val::Px(5.0),
                    ..default()
                }),
                Label,
            ));

            c.spawn(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Px(400.0),
                    border: UiRect::all(Val::Px(2.0)),
                    height: Val::Vh(100.0),
                    right: Val::Px(0.0),
                    ..default()
                },
                background_color: Color::rgb(0.65, 0.65, 0.65).into(),
                ..default()
            });
        })
        .insert(MainMenuEnt);
}
