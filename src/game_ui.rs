use std::any::Any;

use crate::player::Player;
use bevy::prelude::*;

pub struct GameInterfacePlugin;
impl Plugin for GameInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hud)
            .add_systems(FixedUpdate, update_health_system);
    }
}

fn setup_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn the health bars.
    commands
        .spawn(ImageBundle {
            style: Style {
                width: Val::Percent(35.0),
                height: Val::Percent(5.0),
                top: Val::Percent(40.),
                ..default()
            },
            image: asset_server.load("healthbar.png").into(),
            ..default()
        })
        .with_children(|bar| {
            // The colored inner health bar

            bar.spawn(NodeBundle {
                style: Style {
                    width: Val::Percent(99.0),
                    height: Val::Percent(20.0),
                    position_type: PositionType::Absolute, // Use absolute positioning
                    left: Val::Percent(0.),
                    top: Val::Percent(20.),
                    ..default()
                },
                background_color: Color::DARK_GRAY.into(),
                z_index: ZIndex::Global(-2),
                ..default()
            })
            .insert(Name::new("BackgroundHealthBar"))
            .with_children(|healthbar| {
                // The colored part of the healthbar
                healthbar
                    .spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(30.0), // The health percentage(i.e. 100% == 100% health)
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute, // Use absolute positioning
                            left: Val::Percent(0.),
                            top: Val::Percent(0.),
                            ..default()
                        },
                        background_color: Color::GREEN.into(),
                        z_index: ZIndex::Global(-1),
                        ..default()
                    })
                    .insert(Name::new("ForegroundHealthBar"));
            });

            bar.spawn((TextBundle::from_section(
                "HP",
                TextStyle {
                    font_size: 30.0,
                    ..default()
                },
            )
            .with_style(Style {
                margin: UiRect::all(Val::Px(1.)),
                position_type: PositionType::Absolute, // Use absolute positioning
                top: Val::Percent(1.),
                left: Val::Percent(1.),
                ..default()
            }),));

            bar.spawn((
                TextBundle::from_section(
                    "1000/1000",
                    TextStyle {
                        font_size: 30.0,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(1.)),
                    position_type: PositionType::Absolute, // Use absolute positioning
                    top: Val::Percent(35.),
                    left: Val::Percent(60.),
                    ..default()
                }),
                Label,
            ))
            .insert(Name::new("HealthText"));
        });
}

fn update_health_system(mut player_query: Query<&Player>, mut query: Query<(&Name, &mut Style)>) {
    for (ui_name, mut style) in query.iter_mut() {
        if ui_name.as_str() == "ForegroundHealthBar" {
            info!("{:?}", ui_name);
            let mut player = player_query.single_mut();
            // Update the foreground health bar width based on the health percentage
            let health_percentage = player.health_current / player.health_max;
            let new_width = Val::Percent(health_percentage * 100.0);
            style.width = new_width;

            // Update the health text
            // text.sections[0].value = format!("{}/{}", health.current as i32, health.max as i32);
        }
    }
}
