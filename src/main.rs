use asteroids_bevy::constants::{BG_COLOR, WH, WW};
use asteroids_bevy::player::PlayerPlugin;
use bevy::{prelude::*, window::close_on_esc};

use bevy::core_pipeline::{
    bloom::{BloomCompositeMode, BloomSettings},
    tonemapping::Tonemapping,
};
// use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgba_u8(
            BG_COLOR.0, BG_COLOR.1, BG_COLOR.2, 0,
        )))
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
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(Update, close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            ..default()
        },
        BloomSettings::default(), // 3. Enable bloom for the camera
    ));
}
