use bevy::prelude::*;
use bevy_cursor::prelude::*;
use bevy_rapier2d::prelude::*;

const BASE_MOVESPEED: f32 = 50.0;

#[derive(Component)]
pub struct Player {
    move_speed: f32,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(FixedUpdate, )
        app.add_plugins(CursorInfoPlugin)
            .add_systems(Startup, setup_player)
            .add_systems(Update, look_at_cursor)
            // .add_systems(Update, movement_system);
            .add_systems(Update, modify_player_translation);
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
            transform: Transform::from_translation(Vec3::new(0., 0., 0.)),
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
        .insert(GravityScale(0.));
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

// fn movement_system(
//     mut player_query: Query<(Entity, &mut Transform, &mut Player), With<Player>>,
//     keyboard_input: Res<Input<KeyCode>>,
//     time_step: Res<Time>,
// ) {
//     let (_, mut player_transform, player) = player_query.single_mut();
//     let move_speed = player.move_speed;

//     // TODO replace this with rigidbody force velocity

//     if keyboard_input.pressed(KeyCode::A) {
//         player_transform.translation.x -= move_speed;
//     } else if keyboard_input.pressed(KeyCode::D) {
//         player_transform.translation.x += move_speed;
//     }

//     if keyboard_input.pressed(KeyCode::W) {
//         player_transform.translation.y += move_speed;
//     } else if keyboard_input.pressed(KeyCode::S) {
//         player_transform.translation.y -= move_speed;
//     }
// }

fn modify_player_translation(
    mut query: Query<(&mut ExternalImpulse, &Transform, &Player), With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    cursor: Res<CursorInfo>,
) {
    if keyboard_input.pressed(KeyCode::Space) || mouse_input.just_pressed(MouseButton::Left) {
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
