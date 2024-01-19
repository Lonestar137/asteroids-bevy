use crate::constants::*;
use crate::game_ui::GameInterfacePlugin;
use crate::mobs::Enemy;

use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use bevy::render::view::WindowSurfaces;
use bevy::{
    core_pipeline::{
        bloom::BloomSettings, clear_color::ClearColorConfig, tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
    sprite::MaterialMesh2dBundle,
    window::{PrimaryWindow, WindowResized},
};
use bevy_cursor::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_rapier2d::prelude::*;
use std::time::{Duration, Instant};

const BALL_SIZE: Vec3 = Vec3::new(20., 20., 0.);
const BASE_MOVESPEED: f32 = 150.0;
const PROJECTILE_LIMIT: i32 = 40;
const COOLDOWN_DURATION_MS: u64 = 200;

#[derive(Resource, Debug)]
struct ShootingCooldown {
    last_shot_time: Instant,
    cooldown_duration: Duration,
}
#[derive(Resource, Debug)]
pub struct WindowSize {
    pub left_wall: f32,
    pub right_wall: f32,
    pub top_wall: f32,
    pub bottom_wall: f32,
}
#[derive(Resource)]
pub struct ProjectilePool(Vec<Entity>);
#[derive(Component)]
pub struct ExhaustEffect;
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
}
#[derive(Component)]
pub struct Player {
    pub health_current: f32,
    pub health_max: f32,
    pub exp_current: f32,
    pub exp_max: f32,
    pub level: u16,
    move_speed: f32,
}
#[derive(Component)]
pub struct Warpable;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(FixedUpdate, )
        app.add_plugins(CursorInfoPlugin)
            .add_plugins(GameInterfacePlugin)
            .insert_resource(ShootingCooldown {
                last_shot_time: Instant::now(),
                cooldown_duration: Duration::from_millis(COOLDOWN_DURATION_MS),
            })
            .insert_resource(ProjectilePool(Vec::new()))
            .add_systems(Startup, setup_player)
            .add_systems(Startup, setup_projectiles)
            // .add_systems(
            //     Update,
            //     (add_thrust_particles_to_ship, update_thrust_particles),
            // )
            .add_systems(Update, ship_warp)
            .add_systems(Update, (shoot_projectile, despawn_projectile))
            .add_systems(Update, handle_player_collision)
            .add_systems(FixedUpdate, look_at_cursor)
            .add_systems(FixedUpdate, modify_player_translation)
            .add_systems(FixedUpdate, update_winsize);
    }
}

fn setup_projectiles(
    mut commands: Commands,
    mut spawnpool: ResMut<ProjectilePool>,
    asset_server: Res<AssetServer>,
) {
    for _ in 1..=PROJECTILE_LIMIT {
        let entity = commands
            .spawn((SpriteBundle {
                texture: asset_server.load("Lasers/08.png"),
                sprite: Sprite {
                    custom_size: Some(Vec2::splat(3.)),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(500., 500., 3.),
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
            .insert(Projectile { damage: 10. })
            .insert(Visibility::Hidden)
            .insert(ExternalImpulse {
                impulse: Vec2::new(0., 0.),
                torque_impulse: 0.0,
            })
            .insert(Damping {
                linear_damping: 3.5,
                angular_damping: 5.0,
            })
            .insert(RigidBody::Dynamic)
            .insert(AdditionalMassProperties::Mass(2.0))
            .insert(GravityScale(0.))
            .insert(Velocity::zero())
            .insert(CollisionGroups::new(Group::GROUP_2, Group::GROUP_3))
            .insert(SolverGroups::new(Group::GROUP_2, Group::GROUP_3))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .id();

        spawnpool.0.push(entity);
    }
}

fn setup_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let primary_window = window_query.single();
    commands.insert_resource(WindowSize {
        left_wall: -primary_window.width() / 2.,
        right_wall: primary_window.width() / 2.,
        bottom_wall: -primary_window.height() / 2.,
        top_wall: primary_window.height() / 2.,
    });

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
            health_current: 500.,
            health_max: 500.,
            exp_current: 0.,
            exp_max: 1000.,
            level: 1,
            move_speed: BASE_MOVESPEED,
        })
        .insert(Warpable)
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
    mut cooldown: ResMut<ShootingCooldown>,
) {
    let mut spawn_limit = PROJECTILE_LIMIT as usize;
    if keyboard_input.pressed(KeyCode::S) || mouse_input.just_pressed(MouseButton::Left) {
        let current_time = Instant::now();
        let time_since_last_shot = current_time - cooldown.last_shot_time;

        if time_since_last_shot >= cooldown.cooldown_duration {
            for (i, (mut ext_impulse, mut velocity, mut transform, mut visibility)) in
                projectile_query.iter_mut().enumerate()
            {
                if *visibility == Visibility::Hidden {
                    match cursor.position() {
                        Some(cursor_direction) => {
                            if i < spawn_limit {
                                *visibility = Visibility::Visible;
                                *velocity = Velocity::zero();
                                // Retrieve player position
                                let player_transform = player_query.single();

                                // Set projectile transform to player position
                                *transform = *player_transform;
                                transform.scale = BALL_SIZE;

                                // Calculate direction vector from projectile position to cursor position
                                let direction = cursor_direction - transform.translation.truncate();

                                // Normalize direction vector
                                let normalized_direction = direction.normalize();

                                // Apply force in the direction of the normalized direction
                                ext_impulse.impulse = normalized_direction * 10000.0;

                                // Update projectile transform to face the cursor direction
                                let angle = normalized_direction.y.atan2(normalized_direction.x);
                                // transform.rotation =
                                // Quat::from_rotation_arc(-Vec3::Y, angle.to_degrees());
                                transform.rotate_y(45.);
                                cooldown.last_shot_time = current_time;
                            }
                            spawn_limit = i;
                        }
                        _ => (),
                    }
                }
            }
        }
    }
}

fn despawn_projectile(
    mut projectile_query: Query<
        (&mut Velocity, &mut Visibility, &mut Transform),
        (With<Projectile>, Without<Player>),
    >,
) {
    // let (bullet_velocity, bullet_visibility) =
    for (mut bullet_velocity, mut bullet_visibility, mut transform) in projectile_query.iter_mut() {
        if *bullet_visibility == Visibility::Visible {
            let linvel = bullet_velocity.linvel;

            if linvel.x.abs() < 25.0 && linvel.y.abs() < 25.0 {
                debug!("Hiding expired projectile");
                *bullet_visibility = Visibility::Hidden;
                *bullet_velocity = Velocity::zero();
                transform.translation = Vec3::new(10000., 100000., -1.);
            }
        }
    }
}

fn update_winsize(
    mut win_size: ResMut<WindowSize>,
    mut resize_event: EventReader<WindowResized>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for _ in resize_event.read() {
        let primary_window = window_query.single();
        win_size.left_wall = -primary_window.width() / 2.;
        win_size.right_wall = primary_window.width() / 2.;
        win_size.bottom_wall = -primary_window.height() / 2.;
        win_size.top_wall = primary_window.height() / 2.;
        debug!("Window Resized:");
        debug!("  Width {:?}", primary_window.width());
        debug!("  Height {:?}", primary_window.height());
    }
}

fn ship_warp(
    // window_query: Query<&Window, With<PrimaryWindow>>,
    window_query: Res<WindowSize>,
    mut query: ParamSet<(
        Query<(&mut Transform, &Sprite), With<Warpable>>,
        Query<&Transform, With<Camera>>,
    )>,
) {
    let left_wall = window_query.left_wall;
    let right_wall = window_query.right_wall;
    let bottom_wall = window_query.bottom_wall;
    let top_wall = window_query.top_wall;

    let camera_transform = query.p1().single().clone();

    for (mut transform, sprite) in query.p0().iter_mut() {
        let size = sprite
            .custom_size
            .expect("Player sprite doesn't have a custom size");
        let xy = transform.translation.xy();
        let x_pad = size.x * 0.9;
        let y_pad = size.y * 0.9;

        if xy.y > camera_transform.translation.y + top_wall + y_pad {
            // works
            transform.translation.y = camera_transform.translation.y + bottom_wall;
        } else if xy.y < camera_transform.translation.y + bottom_wall - y_pad {
            // works
            transform.translation.y = camera_transform.translation.y + top_wall;
        } else if xy.x > camera_transform.translation.x + right_wall + x_pad {
            transform.translation.x = camera_transform.translation.x + left_wall;
        } else if xy.x < camera_transform.translation.x + left_wall - x_pad {
            transform.translation.x = camera_transform.translation.x + right_wall;
        }
    }
}

fn handle_player_collision(
    mut commands: Commands,
    // mut bullets: Query<(Entity, &mut Velocity, &Projectile), With<EnemyProjectile>>,
    mut player: Query<(Entity, &mut Player)>,
    mut enemies: Query<(Entity, &mut Enemy), With<Enemy>>,
    mut contact_events: EventReader<CollisionEvent>,
) {
    for contact_event in contact_events.read() {
        if let CollisionEvent::Started(entity1, entity2, x) = contact_event {
            let player = player.iter_mut().find(|(player_entity, _)| {
                *player_entity == *entity1 || *player_entity == *entity2
            });

            let enemy = enemies
                .iter_mut()
                .find(|(enemy_entity, _)| *enemy_entity == *entity1 || *enemy_entity == *entity2);

            if let (Some((player_entity, mut player_data)), Some((enemy_entity, mut enemy_data))) =
                (player, enemy)
            {
                // enemy_data.health -= player_data.collision_damage;
                player_data.health_current -= enemy_data.collision_damage;
                if player_data.health_current < 0. {
                    info!("Deleting entity. {:?}", enemy_entity);
                    // TODO: end the game.
                    commands.entity(player_entity).remove::<Visibility>();
                    commands.entity(player_entity).insert(Visibility::Hidden);
                    // commands.entity(player_entity).despawn_recursive();
                }
            }
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
