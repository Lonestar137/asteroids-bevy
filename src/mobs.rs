use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn(
            (SpriteBundle {
                texture: asset_server.load("./asteroid1.png"),
                sprite: Sprite {
                    // color: Color::rgb(0.25, 0.25, 0.75),
                    color: Color::rgb(1.2, 1.2, 1.2),
                    custom_size: Some(Vec2::new(50.0, 50.0)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(100., 400., 2.)),
                ..default()
            }),
        )
        .insert(ExternalImpulse {
            impulse: Vec2::new(10., -10.),
            torque_impulse: 0.07,
        })
        .insert(Collider::ball(30.0))
        .insert(RigidBody::Dynamic)
        .insert(AdditionalMassProperties::Mass(100.0))
        .insert(GravityScale(0.))
        .insert(Velocity::linear(Vec2::new(-100., 0.)))
        .insert(CollisionGroups::new(
            Group::GROUP_3,
            Group::GROUP_1 | Group::GROUP_2,
        ))
        .insert(SolverGroups::new(
            Group::GROUP_3,
            Group::GROUP_1 | Group::GROUP_2,
        ));
}
