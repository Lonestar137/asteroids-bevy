use asteroids_bevy::constants::{BG_COLOR, WH, WW};
use asteroids_bevy::mobs::MobPlugin;
use asteroids_bevy::parralax::ParallaxBackgroundPlugin;
use asteroids_bevy::player::PlayerPlugin;

use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping};
use bevy::{prelude::*, window::close_on_esc};
use bevy_hanabi::prelude::*;
// use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        // .insert_resource(ClearColor(Color::rgba_u8(
        //     BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        // )))
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resizable: false,
                        // mode: WindowMode::Fullscreen,
                        focused: true,
                        resolution: (WW as f32, WH as f32).into(),
                        title: "Bevy Defense".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.))
        .add_plugins(RapierDebugRenderPlugin::default())
        // .add_plugins(HanabiPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(ParallaxBackgroundPlugin)
        .add_systems(Update, close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0., 0., 50.)),
            camera: Camera {
                hdr: true,
                ..default()
            },
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
        BloomSettings::default(),
    ));
}
