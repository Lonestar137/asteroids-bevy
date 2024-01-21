use std::ops::{Deref, DerefMut};

use crate::player::Player;
use bevy::a11y::accesskit::TextAlign;
use bevy::prelude::*;
use bevy::reflect::Map;
use bevy::time::Stopwatch;
use bevy::utils::hashbrown::HashMap;
use bevy_rapier2d::prelude::*;

#[derive(Resource, Debug)]
pub struct VelocityStorage(pub HashMap<Entity, Velocity>);
// Tracks elapsed time outside Paused state
#[derive(Resource)]
pub struct GameRuntime(pub Stopwatch);
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    Paused,
    #[default]
    Playing,
    StartMenu,
}

pub struct GameInterfacePlugin;
impl Plugin for GameInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(VelocityStorage(HashMap::new()))
            .add_systems(PostStartup, setup_hud.run_if(in_state(GameState::Playing)))
            .add_systems(
                FixedUpdate,
                update_health_system.run_if(in_state(GameState::Playing)),
            )
            .add_systems(Update, save_velocity_system)
            .add_systems(OnEnter(GameState::Paused), setup_pause_menu)
            .add_systems(OnExit(GameState::Paused), despawn_menu);
    }
}

fn setup_hud(mut commands: Commands, player: Query<&Player>, asset_server: Res<AssetServer>) {
    let player_data = player.single();
    // Spawn the health bars.
    commands
        .spawn(ImageBundle {
            style: Style {
                width: Val::Percent(35.0),
                height: Val::Percent(5.0),
                top: Val::Percent(95.),
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
                    font_size: 15.0,
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
                    format!(
                        "{current}/{max}",
                        current = player_data.health_current,
                        max = player_data.health_max
                    ),
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

            bar.spawn((
                TextBundle::from_section(
                    format!("Lv. {}", player_data.level,),
                    TextStyle {
                        font_size: 30.0,
                        ..default()
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(1.)),
                    position_type: PositionType::Absolute, // Use absolute positioning
                    top: Val::Percent(35.),
                    left: Val::Percent(10.),
                    ..default()
                }),
                Label,
            ))
            .insert(Name::new("LevelText"));
        });
}

fn update_health_system(
    player_query: Query<&Player>,
    mut bar_query: Query<(&Name, &mut Style)>,
    mut text_query: Query<(&Name, &mut Text)>,
) {
    let player_data = player_query.single();
    for (ui_name, mut style) in bar_query.iter_mut() {
        if ui_name.as_str() == "ForegroundHealthBar" {
            // Update the foreground health bar width based on the health percentage
            let health_percentage = player_data.health_current / player_data.health_max;
            let new_width = Val::Percent(health_percentage * 100.0);
            style.width = new_width;
        }
    }
    for (ui_name, mut text) in text_query.iter_mut() {
        if ui_name.as_str() == "HealthText" {
            let healthtext = text
                .sections
                .first_mut()
                .expect("Healthtext was not retrieved.");
            let displayed_curr_health = if player_data.health_current < 0. {
                0.
            } else {
                player_data.health_current
            };
            healthtext.value = format!(
                "{current}/{max}",
                current = displayed_curr_health,
                max = player_data.health_max
            );
        } else if ui_name.as_str() == "LevelText" {
            let leveltext = text
                .sections
                .first_mut()
                .expect("leveltext was not retrieved.");
            leveltext.value = format!("Lv. {}", player_data.level);
        }
    }
}

fn save_velocity_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Velocity, &mut ExternalImpulse)>,
    curr_gamestate: Res<State<GameState>>,
    mut velocity_storage: ResMut<VelocityStorage>,
) {
    if curr_gamestate.is_changed() {
        for (entity, mut velocity, mut impulse) in query.iter_mut() {
            let entity_id = commands.entity(entity).id();
            match curr_gamestate.get() {
                GameState::Playing => {
                    // Reload the old force
                    let default_vel = &Velocity::zero();
                    let unpaused_velocity =
                        velocity_storage.0.get(&entity_id).unwrap_or(default_vel);
                    *velocity = *unpaused_velocity;
                }
                GameState::Paused => {
                    // Save old force
                    velocity_storage.0.insert(entity_id, *velocity);

                    // Set forces to zero
                    ExternalImpulse::reset(impulse.deref_mut());
                    *velocity = Velocity::zero();
                }
                _ => (),
            }
        }
    }
}

fn setup_pause_menu(mut commands: Commands) {
    let root = commands
        .spawn((NodeBundle {
            // give it a dark background for readability
            // background_color: BackgroundColor(Color::BLACK.with_a(0.8)),
            background_color: BackgroundColor(Color::MIDNIGHT_BLUE.with_a(0.9)),
            // make it "always on top" by setting the Z index to maximum
            // we want it to be displayed over all other UI
            z_index: ZIndex::Global(i32::MAX),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Percent(40.),
                top: Val::Percent(20.),
                bottom: Val::Auto,
                left: Val::Percent(40.),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        },))
        .insert(Name::new("PauseMenuRoot"))
        .id();

    let box_and_title = commands
        .spawn(TextBundle {
            text: Text::from_sections([TextSection {
                value: "Paused".into(),
                style: TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
            }]),
            ..Default::default()
        })
        .insert(Name::new("PauseState"))
        .id();

    let button_style = Style {
        width: Val::Px(250.0),
        height: Val::Px(65.0),
        margin: UiRect::all(Val::Px(20.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let button_text_style = TextStyle {
        font_size: 24.0,
        ..default()
    };

    let normal_button: Color = Color::rgb(0.15, 0.15, 0.15);
    let button = ButtonBundle {
        style: button_style.clone(),
        background_color: normal_button.into(),
        ..default()
    };

    let resume_button = commands
        .spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Resume",
                button_text_style.clone(),
            ));
        })
        .id();

    let settings_button = commands
        .spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Settings",
                button_text_style.clone(),
            ));
        })
        .id();

    let exit_button = commands
        .spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Exit", button_text_style.clone()));
        })
        .id();

    commands.entity(box_and_title).push_children(&[]);
    commands.entity(root).push_children(&[
        box_and_title,
        resume_button,
        settings_button,
        exit_button,
    ]);
}

fn despawn_menu(mut commands: Commands, query: Query<(Entity, &Name)>) {
    for (entity, name) in query.iter() {
        if name.as_str() == "PauseMenuRoot" {
            commands.entity(entity).despawn_recursive()
        }
    }
}
