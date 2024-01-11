use crate::constants::*;
use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy_cursor::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier2d::prelude::*;

const BALL_SIZE: Vec3 = Vec3::new(20., 20., 0.);
const BASE_MOVESPEED: f32 = 50.0;

#[derive(Resource)]
pub struct ProjectilePool(Vec<Entity>);
#[derive(Component)]
pub struct ExhaustEffect;
#[derive(Component)]
pub struct Projectile;
#[derive(Component)]
pub struct Player {
    move_speed: f32,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(FixedUpdate, )
        app.add_plugins(CursorInfoPlugin)
            .insert_resource(ProjectilePool(Vec::new()))
            .add_systems(Startup, setup_player)
            .add_systems(Startup, setup_projectiles)
            // .add_systems(
            //     Update,
            //     (add_thrust_particles_to_ship, update_thrust_particles),
            // )
            .add_systems(Update, ship_warp)
            .add_systems(Update, look_at_cursor)
            // .add_systems(Update, movement_system);
            .add_systems(Update, shoot_projectile)
            .add_systems(Update, modify_player_translation);
    }
}

fn setup_projectiles(
    mut commands: Commands,
    mut spawnpool: ResMut<ProjectilePool>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for _ in 1..=2 {
        commands
            .spawn((MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::default().into()).into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.5, 1., 0.5))),
                transform: Transform {
                    translation: Vec3::new(500., 500., 3.),
                    // scale: BALL_SIZE.clone(),
                    scale: BALL_SIZE,
                    ..default()
                },
                ..default()
            },))
            .insert(Collider::ball(0.6))
            .insert(Sleeping {
                sleeping: true,
                ..default()
            })
            .insert(Projectile)
            .insert(Visibility::Hidden)
            .insert(ExternalImpulse {
                impulse: Vec2::new(0., 0.),
                torque_impulse: 0.0,
            })
            .insert(Damping {
                linear_damping: 0.5,
                angular_damping: 5.0,
            })
            .insert(RigidBody::Dynamic)
            .insert(AdditionalMassProperties::Mass(2.0))
            .insert(GravityScale(0.))
            .insert(Velocity::zero())
            .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_3))
            .insert(SolverGroups::new(Group::GROUP_2, Group::GROUP_3));

        // spawnpool.0.push(entity);
    }
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("./ship2.png"),
            sprite: Sprite {
                // color: Color::rgb(0.25, 0.25, 0.75),
                color: Color::rgb(1.2, 1.2, 1.2),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0., 0., 2.)),
            ..default()
        })
        .insert(Player {
            move_speed: BASE_MOVESPEED,
        })
        .insert(ExternalImpulse {
            impulse: Vec2::new(0., 0.),
            torque_impulse: 0.0,
        })
        .insert(Damping {
            linear_damping: 0.5,
            angular_damping: 5.0,
        })
        .insert(RigidBody::Dynamic)
        .insert(Collider::triangle(
            Vec2::new(0., 32.),
            Vec2::new(-32., -28.),
            Vec2::new(32., -28.),
        ))
        .insert(AdditionalMassProperties::Mass(10.0))
        .insert(GravityScale(0.))
        .insert(CollisionGroups::new(Group::GROUP_1, Group::GROUP_3))
        .insert(SolverGroups::new(Group::GROUP_1, Group::GROUP_3));
}

fn look_at_cursor(cursor: Res<CursorInfo>, mut player_query: Query<&mut Transform, With<Player>>) {
    let mut player_transform = player_query.single_mut();
    // get the player translation in 2D
    let player_translation = player_transform.translation.xy();

    if let Some(cursor_pos) = cursor.position() {
        let to_player = (player_translation - cursor_pos).normalize();

        // get the quaternion to rotate from the initial enemy facing direction to the direction
        // facing the player
        let rotate_to_player = Quat::from_rotation_arc(-Vec3::Y, to_player.extend(0.));

        // rotate the enemy to face the player
        player_transform.rotation = rotate_to_player;
    }
}

fn modify_player_translation(
    mut query: Query<(&mut ExternalImpulse, &Transform, &Player), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        match cursor.position() {
            Some(cursor_direction) => {
                let (mut ext_impulse, transform, player) = query.single_mut();
                let direction = cursor_direction - transform.translation.truncate();

                // Apply force in the direction the sprite is facing
                ext_impulse.impulse = direction.normalize() * player.move_speed;
                // Adjust magnitude as needed
            }
            _ => (),
        }
    }
}

fn shoot_projectile(
    mut projectile_query: Query<
        (
            &mut ExternalImpulse,
            &mut Velocity,
            &mut Transform,
            &mut Visibility,
        ),
        (With<Projectile>, Without<Player>),
    >,
    player_query: Query<&Transform, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    let mut spawn_limit = 1;
    if keyboard_input.pressed(KeyCode::S) || mouse_input.just_pressed(MouseButton::Left) {
        for (i, (mut ext_impulse, mut velocity, mut transform, mut visibility)) in
            projectile_query.iter_mut().enumerate()
        {
            match cursor.position() {
                Some(cursor_direction) => {
                    if i < spawn_limit {
                        *velocity = Velocity::zero();
                        // Retrieve player position
                        let player_transform = player_query.single();
                        // Set projectile transform to player position
                        // *transform = *player_transform;
                        *transform = *player_transform;
                        transform.scale = BALL_SIZE;

                        // Calculate direction vector from projectile position to cursor position
                        let direction = cursor_direction - transform.translation.truncate();

                        // Normalize direction vector
                        let normalized_direction = direction.normalize();

                        // Apply force in the direction of the normalized direction
                        ext_impulse.impulse = normalized_direction * 10000.0;

                        // Update projectile transform to face the cursor direction (optional)
                        let angle = normalized_direction.y.atan2(normalized_direction.x);
                        transform.rotation = Quat::from_rotation_z(angle);

                        *visibility = Visibility::Visible;
                    }
                    spawn_limit = i;
                }
                _ => (),
            }
        }
    }
}

fn ship_warp(
    mut query: ParamSet<(
        Query<(&mut Transform, &Sprite), With<Player>>,
        Query<&Transform, With<Camera>>,
    )>,
) {
    let camera_transform = query.p1().single().clone();
    // let (mut transform, mut sprite) = query.p0().single_mut();

    for (mut transform, sprite) in query.p0().iter_mut() {
        let size = sprite
            .custom_size
            .expect("Player sprite doesn't have a custom size");
        let xy = transform.translation.xy();
        let x_pad = size.x * 0.9;
        let y_pad = size.y * 0.9;

        if xy.y > camera_transform.translation.y + TOP_WALL + y_pad {
            // works
            transform.translation.y = camera_transform.translation.y + BOTTOM_WALL;
        } else if xy.y < camera_transform.translation.y + BOTTOM_WALL - y_pad {
            // works
            transform.translation.y = camera_transform.translation.y + TOP_WALL;
        } else if xy.x > camera_transform.translation.x + RIGHT_WALL + x_pad {
            transform.translation.x = camera_transform.translation.x + LEFT_WALL;
        } else if xy.x < camera_transform.translation.x + LEFT_WALL - x_pad {
            transform.translation.x = camera_transform.translation.x + RIGHT_WALL;
        }
    }
}

fn setup_particles(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(4.0, 4.0, 0.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 0.0, 0.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 0.0, 0.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.0, Vec2::splat(0.1));
    size_gradient1.add_key(0.3, Vec2::splat(0.1));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let writer = ExprWriter::new();

    // Give a bit of variation by randomizing the age per particle. This will
    // control the starting color and starting size of particles.
    let age = writer.lit(0.).uniform(writer.lit(0.2)).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.8).uniform(writer.lit(1.2)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add constant downward acceleration to simulate gravity
    let accel = writer.lit(Vec3::Y * -8.).expr();
    let update_accel = AccelModifier::new(accel);

    // Add drag to make particles slow down a bit after the initial explosion
    let drag = writer.lit(5.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    // Give a bit of variation by randomizing the initial speed
    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: (writer.rand(ScalarType::Float) * writer.lit(20.) + writer.lit(60.)).expr(),
    };

    let effect = EffectAsset::new(
        32768,
        Spawner::burst(2500.0.into(), 2.0.into()),
        writer.finish(),
    )
    .with_name("firework")
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(update_drag)
    .update(update_accel)
    .render(ColorOverLifetimeModifier {
        gradient: color_gradient1,
    })
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient1,
        screen_space_size: false,
    });

    let effect1 = effects.add(effect);

    commands.spawn((
        Name::new("firework"),
        ParticleEffectBundle {
            effect: ParticleEffect::new(effect1),
            transform: Transform::IDENTITY,
            ..Default::default()
        },
    ));
}

// Add a Particle Effect to every new Ship created
fn add_thrust_particles_to_ship(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    // added_ships: Query<Entity, Added<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    added_ships: Query<Entity, With<Player>>,
) {
    if keyboard_input.pressed(KeyCode::Space) {
        for ship_entity in added_ships.iter() {
            // For Ship exhaust, we store a particle effects on the player

            let writer = ExprWriter::new();
            let lifetime = writer.lit(0.1).expr();
            // Gradient for particle color evolution
            let mut gradient = Gradient::new();
            gradient.add_key(0.0, Vec4::new(0.5, 0.4, 0.7, 0.8));
            gradient.add_key(0.5, Vec4::new(1.0, 0.8, 0.0, 0.8));
            gradient.add_key(1.0, Vec4::ZERO);
            let init_pos = SetPositionCone3dModifier {
                height: writer.lit(-5.0).expr(),
                base_radius: writer.lit(2.).expr(),
                top_radius: writer.lit(1.).expr(),
                dimension: ShapeDimension::Volume,
            };
            let init_vel = SetVelocitySphereModifier {
                speed: writer.lit(100.0).uniform(writer.lit(400.0)).expr(),
                center: writer.lit(Vec3::new(0.0, 1.0, 0.0)).expr(),
            };
            let effect = effects.add(
                EffectAsset::new(16024, Spawner::once(10.0.into(), false), writer.finish())
                    .with_name("Exhaust")
                    .init(init_pos)
                    .init(init_vel)
                    .init(SetAttributeModifier::new(Attribute::LIFETIME, lifetime))
                    .render(ColorOverLifetimeModifier { gradient })
                    .render(SizeOverLifetimeModifier {
                        gradient: Gradient::constant(Vec2::splat(2.)),
                        screen_space_size: true,
                    }),
            );
            commands.entity(ship_entity).with_children(|parent| {
                parent.spawn((
                    ParticleEffectBundle {
                        effect: ParticleEffect::new(effect).with_z_layer_2d(Some(10.)),
                        transform: Transform::from_translation(Vec3::new(0.0, -4.0, 0.0)),
                        ..default()
                    },
                    ExhaustEffect,
                ));
            });
        }
    }
}

// Trigger a new particle spawning whenever the Ship Impulse is non-0
fn update_thrust_particles(
    // player: Query<(&ActionState<PlayerAction>, &Children), Changed<ActionState<PlayerAction>>>,
    player: Query<(Entity, &Children), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut exhaust_effect: Query<&mut EffectSpawner, With<ExhaustEffect>>,
) {
    for (action_state, children) in player.iter() {
        if keyboard_input.pressed(KeyCode::Space) {
            for &child in children.iter() {
                if let Ok(mut spawner) = exhaust_effect.get_mut(child) {
                    spawner.reset();
                }
            }
        }
    }
}
