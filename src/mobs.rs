use crate::constants::BASE_EXP_PULL;
use crate::player::{Player, Projectile, Warpable};

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct Enemy {
    health: f32,
}
#[derive(Component)]
pub struct ExperienceShard(f32);

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(PostUpdate, exp_pull_system)
            .add_systems(PostUpdate, kill_on_contact);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpriteBundle {
            texture: asset_server.load("./asteroid1.png"),
            sprite: Sprite {
                // color: Color::rgb(0.25, 0.25, 0.75),
                color: Color::rgb(1.2, 1.2, 1.2),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(100., 400., 2.)),
            ..default()
        })
        .insert(Enemy { health: 100. })
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
        ))
        .insert(ActiveEvents::COLLISION_EVENTS);
}

fn kill_on_contact(
    mut commands: Commands,
    mut bullets: Query<(Entity, &mut Velocity, &Projectile), With<Projectile>>,
    mut enemies: Query<(Entity, &mut Transform, &mut Enemy), With<Enemy>>,
    mut contact_events: EventReader<CollisionEvent>,
    asset_server: Res<AssetServer>,
) {
    for contact_event in contact_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = contact_event {
            let bullet_entity = bullets.iter_mut().find(|(bullet_entity, _, _)| {
                *bullet_entity == *entity1 || *bullet_entity == *entity2
            });

            let enemy_entity = enemies.iter_mut().find(|(enemy_entity, _, _)| {
                *enemy_entity == *entity1 || *enemy_entity == *entity2
            });

            if let (
                Some((_, mut bullet_velocity, projectile_data)),
                Some((enemy_entity, enemy_transform, mut enemy_data)),
            ) = (bullet_entity, enemy_entity)
            {
                let x_rand = thread_rng().gen_range(-100..100) as f32;
                let y_rand = thread_rng().gen_range(-100..100) as f32;
                let shard_velocity = Vec2::new(x_rand, y_rand);

                // Apply ricochet effect to bullet
                let bul_vel = bullet_velocity.linvel.dot(Vec2::new(0.0, 1.0)) * Vec2::new(0.0, 1.0);
                bullet_velocity.linvel -= 2.0 * bul_vel;

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
                        transform: enemy_transform.clone(),
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
                info!("{:?}", enemy_data.health);
                if enemy_data.health < 0. {
                    info!("Deleting entity.");
                    commands.entity(enemy_entity).despawn_recursive();
                }
            }
        }
    }
}

fn exp_pull_system(
    mut commands: Commands,
    mut shards: Query<(Entity, &Transform, &mut Velocity), With<ExperienceShard>>,
    player: Query<&Transform, (With<Player>, Without<ExperienceShard>)>,
) {
    let exp_pull_range: f32 = BASE_EXP_PULL;
    let exp_absorb_range: f32 = 20.;
    let player_transform = player.single();
    for (shard_entity, shard_transform, mut shard_velocity) in shards.iter_mut() {
        let distance = player_transform
            .translation
            .distance(shard_transform.translation);

        if distance < exp_pull_range {
            let direction = player_transform.translation - shard_transform.translation;
            let velocity = direction * 2.0; // Adjust speed as needed
            shard_velocity.linvel = velocity.xy();
        }
        if distance < exp_absorb_range {
            commands.entity(shard_entity).despawn_recursive();
            info!("Absorbed EXP!");
        }
    }
}
