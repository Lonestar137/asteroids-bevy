use crate::constants::BASE_EXP_PULL;
use crate::game_ui::{GameRuntime, GameState};
use crate::guns::Projectile;
use crate::player::{LevelUpEvent, Player, Warpable, WindowSize};

use bevy::audio::Volume;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Resource)]
struct CollisionSound(Handle<AudioSource>);
#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub collision_damage: f32,
}
#[derive(Component)]
pub struct ExperienceShard(f32);

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EnemyWavePlugin)
            .add_systems(Startup, setup)
            .add_systems(
                PostUpdate,
                exp_pull_system.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                PostUpdate,
                kill_on_contact
                    .run_if(on_event::<CollisionEvent>())
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let bullet_collision_sound = asset_server.load("Sounds/hitmarker_2.ogg");
    // let bullet_collision_sound = asset_server.load("Sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(bullet_collision_sound));
}

fn kill_on_contact(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Velocity, &Projectile, &mut Transform), With<Projectile>>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy), (With<Enemy>, Without<Projectile>)>,
    mut contact_events: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
    asset_server: Res<AssetServer>,
) {
    for contact_event in contact_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = contact_event {
            let bullet_entity = bullets.iter_mut().find(|(bullet_entity, _, _, _)| {
                *bullet_entity == *entity1 || *bullet_entity == *entity2
            });

            let enemy_entity = enemies.iter_mut().find(|(enemy_entity, _, _)| {
                *enemy_entity == *entity1 || *enemy_entity == *entity2
            });

            if let (
                Some((_, mut bullet_velocity, projectile_data, mut bullet_transform)),
                Some((enemy_entity, enemy_transform, mut enemy_data)),
            ) = (bullet_entity, enemy_entity)
            {
                let x_rand = thread_rng().gen_range(-100..100) as f32;
                let y_rand = thread_rng().gen_range(-100..100) as f32;
                let shard_velocity = Vec2::new(x_rand, y_rand);

                debug!("Bullet collision");
                // Play bullet impact sound.
                let audio_settings = PlaybackSettings {
                    volume: Volume::new_relative(0.05),
                    ..default()
                };
                // Volume::Relative(VolumeLevel(1.0))
                commands.spawn(AudioBundle {
                    source: sound.0.clone(),
                    // auto-despawn the entity when playback finishes
                    // settings: PlaybackSettings::DESPAWN,
                    settings: audio_settings,
                });

                // Apply ricochet effect to bullet
                let bul_vel = bullet_velocity.linvel.dot(Vec2::new(0.0, 1.0)) * Vec2::new(0.0, 1.0);
                bullet_velocity.linvel -= 2.0 * bul_vel;
                let angle = bullet_velocity.linvel.y.atan2(bullet_velocity.linvel.x);
                bullet_transform.rotation = Quat::from_rotation_z(angle);

                let enemy_loc = enemy_transform.clone();
                // Despawn enemy
                // TODO replace with spawn pool
                commands
                    .spawn(SpriteBundle {
                        texture: asset_server.load("./xp1.png"),
                        sprite: Sprite {
                            // color: Color::rgb(0.25, 0.25, 0.75),
                            color: Color::rgb(1.2, 1.2, 1.2),
                            custom_size: Some(Vec2::new(50.0, 50.0)),
                            ..default()
                        },
                        // transform: Transform::from_translation(Vec3::new(100., 400., 2.)),
                        transform: enemy_loc,
                        ..default()
                    })
                    .insert(ExperienceShard(10.))
                    .insert(ExternalImpulse {
                        impulse: Vec2::ZERO,
                        torque_impulse: 0.07,
                    })
                    .insert(Warpable)
                    .insert(RigidBody::Dynamic)
                    .insert(Damping {
                        linear_damping: 0.5,
                        angular_damping: 0.0,
                    })
                    .insert(Sensor)
                    .insert(AdditionalMassProperties::Mass(1.0))
                    .insert(GravityScale(0.))
                    .insert(Velocity::linear(shard_velocity));

                enemy_data.health -= projectile_data.damage;
                if enemy_data.health < 0. {
                    info!("Deleting entity. {:?}", enemy_entity);
                    commands.entity(enemy_entity).despawn_recursive();
                }
            }
        }
    }
}

fn exp_pull_system(
    mut commands: Commands,
    mut shards: Query<(Entity, &Transform, &mut Velocity, &ExperienceShard), With<ExperienceShard>>,
    mut player: Query<(&Transform, &mut Player), (With<Player>, Without<ExperienceShard>)>,
    mut event_writer: EventWriter<LevelUpEvent>,
) {
    let exp_pull_range: f32 = BASE_EXP_PULL;
    let exp_absorb_range: f32 = 40.;
    let (player_transform, mut player_data) = player.single_mut();
    for (shard_entity, shard_transform, mut shard_velocity, shard_data) in shards.iter_mut() {
        let distance = player_transform
            .translation
            .distance(shard_transform.translation);

        if distance < exp_pull_range {
            let direction = player_transform.translation - shard_transform.translation;
            let velocity = direction * 5.0; // Adjust speed as needed
            shard_velocity.linvel = velocity.xy();
        }
        if distance < exp_absorb_range {
            player_data.exp_current += shard_data.0;
            if player_data.exp_current > player_data.exp_max {
                info!("LEVEL UP {:?}", player_data.level);
                event_writer.send(LevelUpEvent);
                player_data.level += 1;
                player_data.exp_max = player_data.exp_max * 1.2;
            }
            commands.entity(shard_entity).despawn_recursive();
            debug!("Absorbed EXP!");
        }
    }
}

/*
    Mob plugins
        Asteroid field
        Stalkers (light enemy that follows player)
        Turret (Stationary enemy)
*/

#[derive(Resource)]
struct CurrentWave(i32);

struct EnemyWavePlugin;
impl Plugin for EnemyWavePlugin {
    fn build(&self, app: &mut App) {
        // TODO: Spawns a random set of enemies every # minutes.
        app.insert_resource(CurrentWave(1))
            .add_systems(Update, spawn_wave.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_wave(
    mut commands: Commands,
    player: Query<&Transform, (With<Player>, Without<ExperienceShard>)>,
    mut wave: ResMut<CurrentWave>,
    asset_server: Res<AssetServer>,
    time: Res<GameRuntime>,
    win_size: Res<WindowSize>,
) {
    let player_transform = player.single();
    let mut rng = rand::thread_rng();
    let elapsed_seconds = time.0.elapsed_secs();
    let elapsed_minutes = elapsed_seconds / 60.;

    if elapsed_seconds > 10. && wave.0 == 1 {
        debug!("Wave 1 spawned.");
        for _ in 0..40 {
            let random_x = rng.gen_range(-1000. ..1000.) as f32;
            let random_y = rng.gen_range(-1000. ..1000.) as f32;
            let direction = player_transform.translation.xy() - Vec2::new(random_x, random_y);
            let left_or_right = if rng.gen_bool(0.5) {
                let left_pad = win_size.left_wall;
                rng.gen_range(left_pad * 1.2..left_pad) as f32
            } else {
                let right_pad = win_size.right_wall;
                rng.gen_range(right_pad..right_pad * 1.2) as f32
            };
            commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("Asteroids/A3__00004.png"),
                    sprite: Sprite {
                        // color: Color::rgb(0.25, 0.25, 0.75),
                        color: Color::rgb(1.2, 1.2, 1.2),
                        custom_size: Some(Vec2::new(150.0, 150.0)),
                        ..default()
                    },
                    // transform: Transform::from_translation(Vec3::new(-200., -400., 2.)),
                    transform: Transform::from_translation(Vec3::new(left_or_right, random_y, 2.)),
                    ..default()
                })
                .insert(Enemy {
                    health: 100.,
                    collision_damage: 10.,
                })
                .insert(Warpable)
                .insert(ExternalImpulse {
                    impulse: direction * 0.02,
                    torque_impulse: 0.02,
                })
                .insert(Collider::ball(30.0))
                .insert(RigidBody::Dynamic)
                .insert(AdditionalMassProperties::Mass(100.0))
                .insert(GravityScale(0.))
                .insert(Velocity::linear(Vec2::new(0., 0.)))
                .insert(CollisionGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(SolverGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(Sleeping {
                    sleeping: true,
                    ..default()
                })
                .insert(ActiveEvents::COLLISION_EVENTS);
        }

        wave.0 += 1;
    } else if elapsed_minutes > 1. && wave.0 == 2 {
        for _ in 0..60 {
            let random_x = rng.gen_range(-1000. ..1000.) as f32;
            let random_y = rng.gen_range(-1000. ..1000.) as f32;
            let direction = player_transform.translation.xy() - Vec2::new(random_x, random_y);
            let left_or_right = if rng.gen_bool(0.5) {
                let left_pad = win_size.left_wall;
                rng.gen_range(left_pad * 1.2..left_pad) as f32
            } else {
                let right_pad = win_size.right_wall;
                rng.gen_range(right_pad..right_pad * 1.2) as f32
            };
            commands
                .spawn(SpriteBundle {
                    // texture: asset_server.load("./asteroid1.png"),
                    texture: asset_server.load("Asteroids/A1__00000.png"),
                    sprite: Sprite {
                        // color: Color::rgb(0.25, 0.25, 0.75),
                        color: Color::rgb(1.2, 1.2, 1.2),
                        custom_size: Some(Vec2::new(250.0, 250.0)),
                        ..default()
                    },
                    // transform: Transform::from_translation(Vec3::new(-200., -400., 2.)),
                    transform: Transform::from_translation(Vec3::new(left_or_right, random_y, 2.)),
                    ..default()
                })
                .insert(Enemy {
                    health: 100.,
                    collision_damage: 10.,
                })
                .insert(Warpable)
                .insert(ExternalImpulse {
                    impulse: direction * 0.02,
                    torque_impulse: 0.07,
                })
                .insert(Collider::ball(50.0))
                .insert(RigidBody::Dynamic)
                .insert(AdditionalMassProperties::Mass(100.0))
                .insert(GravityScale(0.))
                .insert(Velocity::linear(Vec2::new(0., 0.)))
                .insert(CollisionGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(SolverGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(ActiveEvents::COLLISION_EVENTS);
        }

        wave.0 += 1;
    } else if elapsed_minutes > 1.5 && wave.0 == 3 {
        for _ in 0..80 {
            let random_x = rng.gen_range(-1000. ..1000.) as f32;
            let random_y = rng.gen_range(-1000. ..1000.) as f32;
            let direction = player_transform.translation.xy() - Vec2::new(random_x, random_y);
            let left_or_right = if rng.gen_bool(0.5) {
                let left_pad = win_size.left_wall;
                rng.gen_range(left_pad * 1.2..left_pad) as f32
            } else {
                let right_pad = win_size.right_wall;
                rng.gen_range(right_pad..right_pad * 1.2) as f32
            };
            commands
                .spawn(SpriteBundle {
                    texture: asset_server.load("Asteroids/A4__00001.png"),
                    sprite: Sprite {
                        // color: Color::rgb(0.25, 0.25, 0.75),
                        color: Color::rgb(1.2, 1.2, 1.2),
                        custom_size: Some(Vec2::new(200.0, 200.0)),
                        ..default()
                    },
                    // transform: Transform::from_translation(Vec3::new(-200., -400., 2.)),
                    transform: Transform::from_translation(Vec3::new(left_or_right, random_y, 2.)),
                    ..default()
                })
                .insert(Enemy {
                    health: 100.,
                    collision_damage: 10.,
                })
                .insert(Warpable)
                .insert(ExternalImpulse {
                    impulse: direction * 0.02,
                    torque_impulse: 0.07,
                })
                .insert(Collider::ball(40.0))
                .insert(RigidBody::Dynamic)
                .insert(AdditionalMassProperties::Mass(100.0))
                .insert(GravityScale(0.))
                .insert(Velocity::linear(Vec2::new(0., 0.)))
                .insert(CollisionGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(SolverGroups::new(
                    Group::GROUP_3,
                    Group::GROUP_1 | Group::GROUP_2,
                ))
                .insert(ActiveEvents::COLLISION_EVENTS);
        }

        wave.0 += 1;
    }
}

struct StalkerEnemyPlugin;
impl Plugin for StalkerEnemyPlugin {
    fn build(&self, app: &mut App) {
        debug!("");
        app.add_systems(FixedUpdate, spawn_stalker);
    }
}

fn spawn_stalker(mut commands: Commands, keyboard_input: Res<Input<KeyCode>>) {
    // TODO: replace with a clock.
    if keyboard_input.just_pressed(KeyCode::S) {}
}
