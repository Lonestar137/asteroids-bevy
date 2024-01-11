use crate::player::{Player, Projectile};
use bevy::prelude::*;
use bevy_rapier2d::{parry::simba::scalar::SupersetOf, prelude::*, rapier::dynamics::RigidBodySet};

#[derive(Component)]
pub struct Enemy;

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(PostUpdate, kill_on_contact);
        // .add_systems(PostUpdate, display_events);
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
        .insert(Enemy)
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
    mut bullets: Query<(Entity, &mut Velocity), With<Projectile>>,
    mut enemies: Query<(Entity, &mut Transform), With<Enemy>>,
    mut contact_events: EventReader<CollisionEvent>,
) {
    for contact_event in contact_events.iter() {
        if let CollisionEvent::Started(entity1, entity2, _) = contact_event {
            let bullet_entity = bullets.iter_mut().find(|(bullet_entity, _)| {
                *bullet_entity == *entity1 || *bullet_entity == *entity2
            });

            let enemy_entity = enemies
                .iter_mut()
                .find(|(enemy_entity, _)| *enemy_entity == *entity1 || *enemy_entity == *entity2);

            if let (Some((bullet_entity, mut bullet_velocity)), Some((enemy_entity, _))) =
                (bullet_entity, enemy_entity)
            {
                // Apply ricochet effect to bullet
                // bullet_velocity.linvel = -bullet_velocity.linvel.reflect(Vec3::new(0.0, 1.0, 0.0)); // You might want to adjust the normal based on your game
                let bul_vel = bullet_velocity.linvel.dot(Vec2::new(0.0, 1.0)) * Vec2::new(0.0, 1.0);
                bullet_velocity.linvel -= 2.0 * bul_vel;

                // Despawn enemy
                commands.entity(enemy_entity).despawn_recursive();
            }
        }
    }
}
