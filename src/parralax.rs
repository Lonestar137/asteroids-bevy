use bevy::prelude::*;
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    ParallaxMoveEvent, ParallaxPlugin, ParallaxSystems, RepeatStrategy,
};

const BACKGROUND: &str = "star_back.png";
const BACKGROUND_FRONT: &str = "star_front.png";
const BACKGROUND_MIDDLE: &str = "star_middle.png";
// const BACKGROUND: &str = "cyberpunk_back.png";
// const BACKGROUND_FRONT: &str = "cyberpunk_front.png";
// const BACKGROUND_MIDDLE: &str = "cyberpunk_middle.png";

pub struct ParallaxBackgroundPlugin;

impl Plugin for ParallaxBackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ParallaxPlugin)
            .add_systems(Startup, initialize_camera_system)
            .add_systems(Update, move_camera_system.before(ParallaxSystems))
        // .insert_resource(ClearColor(Color::rgb_u8(42, 0, 63)));
        ;
    }
}

// Put a ParallaxCameraComponent on the camera used for parallax
pub fn initialize_camera_system(
    mut commands: Commands,
    mut create_parallax: EventWriter<CreateParallaxEvent>,
) {
    let camera = commands
        .spawn(Camera2dBundle::default())
        .insert(ParallaxCameraComponent::default())
        .id();
    create_parallax.send(CreateParallaxEvent {
        layers_data: vec![
            LayerData {
                speed: LayerSpeed::Bidirectional(0.9, 0.9),
                repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
                path: BACKGROUND.to_string(),
                tile_size: Vec2::new(96.0, 160.0),
                cols: 1,
                rows: 1,
                scale: 4.5,
                z: 0.0,
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.7, 0.85),
                repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
                path: BACKGROUND_MIDDLE.to_string(),
                tile_size: Vec2::new(144.0, 160.0),
                scale: 4.5,
                z: 0.5,
                position: Vec2::new(0., 48.),
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.6, 0.8),
                repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
                path: BACKGROUND_MIDDLE.to_string(),
                tile_size: Vec2::new(144.0, 160.0),
                scale: 4.5,
                z: 1.0,
                position: Vec2::new(0., -64.),
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.1, 0.3),
                repeat: LayerRepeat::both(RepeatStrategy::Mirror),
                path: BACKGROUND_FRONT.to_string(),
                tile_size: Vec2::new(272.0, 160.0),
                cols: 1,
                rows: 1,
                scale: 4.5,
                z: 2.0,
                ..default()
            },
        ],
        camera: camera,
    })
}

// Send a ParallaxMoveEvent with the desired camera movement speed
pub fn move_camera_system(
    mut move_event_writer: EventWriter<ParallaxMoveEvent>,
    mut camera_query: Query<Entity, With<Camera>>,
) {
    let camera = camera_query.get_single_mut().unwrap();
    let speed = 9.;
    let mut direction = Vec2::ZERO;
    direction += Vec2::new(0.01, -0.01);
    // direction += Vec2::new(0.0, -1.0);
    move_event_writer.send(ParallaxMoveEvent {
        // camera_move_speed: direction.normalize_or_zero() * speed,
        camera_move_speed: direction * speed,
        camera: camera,
    });
}
