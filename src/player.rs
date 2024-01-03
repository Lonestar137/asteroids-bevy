use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_cursor::prelude::*;
use bevy_rapier2d::prelude::*;

const BALL_SIZE: Vec3 = Vec3::new(20., 20., 0.);
const BASE_MOVESPEED: f32 = 50.0;

#[derive(Resource)]
pub struct ProjectilePool(Vec<Entity>);
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
            .add_systems(Update, look_at_cursor)
            // .add_systems(Update, movement_system);
            .add_systems(Update, spawn_projectile)
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
                    translation: Vec3::new(500., 500., 1.),
                    scale: BALL_SIZE.clone(),
                    ..default()
                },
                ..default()
            },))
            .insert(Collider::ball(10.))
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
            .insert(CollisionGroups::new(Group::GROUP_2, Group::NONE))
            .insert(Velocity::zero())
            .insert(SolverGroups::new(Group::GROUP_2, Group::NONE));

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
        .insert(CollisionGroups::new(Group::GROUP_1, Group::NONE))
        .insert(SolverGroups::new(Group::GROUP_1, Group::NONE));
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

fn projectile_system(
    mut commands: Commands,
    mut spawnpool: ResMut<ProjectilePool>,
    mut player_query: Query<(&mut ExternalImpulse, &Transform, &Player), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    let projectile = spawnpool
        .0
        .pop()
        .expect("Spawnpool projectile should be available.");

    if keyboard_input.pressed(KeyCode::S) || mouse_input.just_pressed(MouseButton::Left) {
        match cursor.position() {
            Some(cursor_direction) => {
                let entity = commands
                    .entity(projectile)
                    .remove::<Visibility>()
                    .insert(Visibility::Visible);
                let (mut ext_impulse, transform, player) = player_query.single_mut();
                let direction = cursor_direction - transform.translation.truncate();

                // Apply force in the direction the sprite is facing
                ext_impulse.impulse = direction.normalize() * player.move_speed;
                // Adjust magnitude as needed
            }
            _ => (),
        }
    }
}

fn spawn_projectile(
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
                        *transform = *player_transform;

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
