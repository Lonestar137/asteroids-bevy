use asteroids_bevy::constants::{WH, WW};
use asteroids_bevy::mobs::MobPlugin;
use asteroids_bevy::parralax::ParallaxBackgroundPlugin;
use asteroids_bevy::player::PlayerPlugin;

use bevy::{prelude::*, window::close_on_esc};
use bevy_hanabi::HanabiPlugin;
// use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_rapier2d::prelude::*;

fn main() {
    App::new()
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
        .add_plugins(HanabiPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(MobPlugin)
        .add_plugins(ParallaxBackgroundPlugin)
        .add_systems(Update, close_on_esc)
        .run();
}
