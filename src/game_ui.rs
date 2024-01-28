use std::ops::{Deref, DerefMut};

use crate::guns::Blade;
use crate::player::{LevelUpEvent, Player};
use bevy::a11y::accesskit::TextAlign;
use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::reflect::Map;
use bevy::time::Stopwatch;
use bevy::utils::hashbrown::HashMap;
use bevy_rapier2d::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

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
    LevelingUp,
}

#[derive(Component)]
enum PauseButtons {
    Resume,
    Exit,
}
#[derive(Component)]
enum LevelUpButtons {
    OptionOne,
    OptionTwo,
    OptionThree,
}
#[derive(Component)]
pub struct SelectedOption;

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
            .add_systems(OnExit(GameState::Paused), despawn_menu)
            .add_systems(OnEnter(GameState::LevelingUp), setup_levelup_menu)
            .add_systems(
                FixedUpdate,
                (button_system, apply_pause_menu_button_system).run_if(in_state(GameState::Paused)),
            )
            .add_systems(
                FixedUpdate,
                (button_system, apply_pause_menu_button_system).run_if(in_state(GameState::Paused)),
            );

        // .run_if(on_event::<LevelUpEvent>())
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
                GameState::Paused | GameState::LevelingUp => {
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
        .insert(Interaction::default())
        .insert(Button)
        .insert(PauseButtons::Resume)
        .id();

    let settings_button = commands
        .spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Settings",
                button_text_style.clone(),
            ));
        })
        .insert(Interaction::default())
        .insert(Button)
        .id();

    let exit_button = commands
        .spawn(button.clone())
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Exit", button_text_style.clone()));
        })
        .insert(Interaction::default())
        .insert(Button)
        .insert(PauseButtons::Exit)
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

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, selected) in interaction_query.iter_mut() {
        *color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn apply_pause_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &PauseButtons),
        (Changed<Interaction>, With<Button>),
    >,
    mut gamestate: ResMut<NextState<GameState>>,
    mut event_writer: EventWriter<AppExit>,
) {
    for (interaction, mut color, selected) in interaction_query.iter_mut() {
        match (*interaction, selected) {
            (Interaction::Pressed, PauseButtons::Resume) => {
                // stopwatch
                gamestate.set(GameState::Playing)
            }
            (Interaction::Pressed, PauseButtons::Exit) => event_writer.send(AppExit),
            (_, _) => (),
        };
    }
}

fn setup_levelup_menu(mut commands: Commands) {
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
                right: Val::Percent(20.),
                left: Val::Percent(20.),
                top: Val::Percent(10.),
                bottom: Val::Percent(10.),
                padding: UiRect::all(Val::Px(4.0)),
                flex_direction: FlexDirection::Row,
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
                value: "Level Up!".into(),
                style: TextStyle {
                    font_size: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
            }]),
            ..Default::default()
        })
        .insert(Name::new("LevelUpState"))
        .id();

    let button_style = Style {
        // width: Val::Px(250.0),
        // height: Val::Px(450.0),
        width: Val::Auto,
        height: Val::Auto,
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

    // TODO: randomize how the values into common, epic, legendary variants.
    let blade_powerup = Blade {
        slash_dmg: 1.2,
        bleed: 1.2,
        length: 2.,
        swing_speed: 1.2,
        pierce: 1.0,
    };
    // let powerups = vec![blade_powerup];

    // commands.entity(root).push_children(&[box_and_title]);
    for i in 0..3 {
        // TODO: randomly select powerups to show.
        let powerup = blade_powerup.clone();
        let button = commands
            .spawn(button.clone())
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    format!("{:?}", powerup),
                    button_text_style.clone(),
                ));
            })
            .insert(Interaction::default())
            .insert(Button)
            .insert(Name::new(format!("LevelUpOption_{}", i)))
            .id();

        commands.entity(button).push_children(&[]);
        commands.entity(root).push_children(&[button]);
    }
}

fn apply_levelup_menu_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Name),
        (Changed<Interaction>, With<Button>),
    >,
    mut gamestate: ResMut<NextState<GameState>>,
    mut event_writer: EventWriter<AppExit>,
) {
    for (interaction, mut color, name) in interaction_query.iter_mut() {}
}
