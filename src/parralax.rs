use bevy::prelude::*;
use bevy_parallax::{
    CreateParallaxEvent, LayerData, LayerRepeat, LayerSpeed, ParallaxCameraComponent,
    ParallaxMoveEvent, ParallaxPlugin, ParallaxSystems, RepeatStrategy,
};
// /home/jonesgc/repos/dev/asteroids-bevy/assets/Large_1024x1024/Starfields/Starfield_01-1024x1024.png
const BACKGROUND: &str = "Large_1024x1024/Starfields/Starfield_01-1024x1024.png";
const BACKGROUND_MIDDLE: &str = "Large_1024x1024/Starfields/Starfield_06-1024x1024_clear.png";
const BACKGROUND_FRONT: &str = "Large_1024x1024/Starfields/Starfield_06-1024x1024_clear.png";

// "Large_1024x1024/Green Nebula/Green_Nebula_05-1024x1024_mod.png"
const NEBULA_ONE: &str = "Large_1024x1024/Green Nebula/Green_Nebula_05-1024x1024_mod.png";
const NEBULA_TWO: &str = "Large_1024x1024/Green Nebula/Green_Nebula_05-1024x1024_mod2.png";
const NEBULA_THREE: &str = "Large_1024x1024/Green Nebula/Green_Nebula_05-1024x1024_mod3.png";

// const BACKGROUND: &str = "star_back.png";
// const BACKGROUND_FRONT: &str = "star_front.png";
// const BACKGROUND_MIDDLE: &str = "star_middle.png";

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
            // LayerData {
            //     speed: LayerSpeed::Bidirectional(0.9, 0.9),
            //     repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
            //     path: BACKGROUND.to_string(),
            //     tile_size: Vec2::new(1024., 1024.),
            //     cols: 1,
            //     rows: 1,
            //     scale: 1.0,
            //     z: 0.2,
            //     ..default()
            // },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.7, 0.85),
                repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
                path: BACKGROUND_MIDDLE.to_string(),
                tile_size: Vec2::new(1024., 1024.),
                scale: 1.0,
                z: 0.6,
                position: Vec2::new(0., 48.),
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.6, 0.8),
                repeat: LayerRepeat::horizontally(RepeatStrategy::Same),
                path: BACKGROUND_MIDDLE.to_string(),
                tile_size: Vec2::new(1024., 1024.),
                scale: 1.0,
                z: 0.8,
                position: Vec2::new(0., -64.),
                ..default()
            },
            LayerData {
                speed: LayerSpeed::Bidirectional(0.3, 0.5),
                repeat: LayerRepeat::both(RepeatStrategy::Same),
                path: BACKGROUND_FRONT.to_string(),
                tile_size: Vec2::new(1024., 1024.),
                cols: 1,
                rows: 1,
                scale: 1.0,
                z: 1.0,
                position: Vec2::new(0., -324.),
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
    let speed = 4.;
    let mut direction = Vec2::ZERO;
    direction += Vec2::new(0.008, -0.008);
    // direction += Vec2::new(0.0, -1.0);
    move_event_writer.send(ParallaxMoveEvent {
        // camera_move_speed: direction.normalize_or_zero() * speed,
        camera_move_speed: direction * speed,
        camera: camera,
    });
}
