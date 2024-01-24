use crate::game_ui::{GameInterfacePlugin, GameRuntime, GameState};
use crate::player::Player;

use bevy::prelude::*;
use bevy_cursor::prelude::*;
use bevy_rapier2d::prelude::*;
use std::iter::Empty;

const COOLDOWN_DURATION_MS: u64 = 200;
pub const PROJECTILE_LIMIT: i32 = 40;
pub const BALL_SIZE: Vec3 = Vec3::new(20., 20., 0.);

#[derive(Resource)]
pub struct ProjectilePool(Vec<Entity>);
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub damage_modifier: f32,
    pub cooldown: Timer,
    pub range: f32,
    pub size: Vec3,
}
#[derive(Component)]
pub struct Multishot {
    count: f32,
    spread: f32,
}
pub struct Burst {
    rounds: i32,
    damage_modifier: f32,
}
#[derive(Component)]
pub struct Burn {
    dot: f32,
    trail_length: f32,
}
#[derive(Component)]
pub struct Blade {
    slash_dmg: f32,
    bleed: f32,
    length: f32,
}
#[derive(Component)]
pub struct Emp {
    stun_length: f32,
    shield_damage: f32,
}

pub struct WeaponPlugin;
impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ProjectilePool(Vec::new()))
            .insert_resource(ShootingCooldown {
                last_shot_time: Instant::now(),
                cooldown_duration: Duration::from_millis(COOLDOWN_DURATION_MS),
            })
            .add_systems(
                Startup,
                (setup_projectiles).run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                shoot_projectile.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                // TODO: sometimes this runs after the gamestate is updated, removing paused projectiles.
                despawn_projectile.run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_projectiles(
    mut commands: Commands,
    mut spawnpool: ResMut<ProjectilePool>,
    asset_server: Res<AssetServer>,
) {
    let sprite = SpriteBundle {
        texture: asset_server.load("Lasers/08.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::splat(3.)),
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(10000., 10000., 3.),
            scale: BALL_SIZE,
            ..default()
        },
        ..default()
    };
    // Spawn a bunch of projectiles.
    for _ in 1..=PROJECTILE_LIMIT {
        let entity = commands
            .spawn(Collider::ball(0.6))
            .insert(sprite.clone())
            .insert(Sleeping {
                sleeping: true,
                ..default()
            })
            .insert(Projectile {
                damage: 10.,
                damage_modifier: 1.2,
                cooldown: Timer::from_seconds(0.5, TimerMode::Repeating),
                range: 20.,
                size: BALL_SIZE.clone(),
            })
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

use std::time::{Duration, Instant};
#[derive(Resource, Debug)]
struct ShootingCooldown {
    last_shot_time: Instant,
    cooldown_duration: Duration,
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
    if keyboard_input.pressed(KeyCode::S) || mouse_input.pressed(MouseButton::Left) {
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
                                transform.translation = player_transform.translation;
                                transform.scale = BALL_SIZE;

                                // Calculate direction vector from projectile position to cursor position
                                let direction = cursor_direction - transform.translation.truncate();

                                // Normalize direction vector
                                let normalized_direction = direction.normalize();

                                // Apply force in the direction of the normalized direction
                                ext_impulse.impulse = normalized_direction * 10000.0;

                                // Update projectile transform to face the cursor direction
                                // info!("BEFORE {:?}", transform.rotation);
                                let d = (player_transform.translation.xy() - cursor_direction)
                                    .normalize();
                                let proj_rotate = Quat::from_rotation_arc(-Vec3::Z, d.extend(0.));
                                transform.rotation = proj_rotate;
                                // info!("AFTER {:?}", transform.rotation);

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
