use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::Rng;

const PLAYER_COLOR: Color = Color::rgb(0.0, 0.0, 1.0);
const ENEMY_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}

pub struct GamePlugin;

// Player component
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Speed(f32, f32);

// spawn player system
fn setup(mut commands: Commands) {
    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: PLAYER_COLOR,
                ..default()
            },
            transform: Transform {
                scale: Vec3::new(10.0, 10.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(Speed(0.0, 0.0))
        .insert(Player);

    let mut rng = rand::thread_rng();
    for _ in 1..100 {
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: ENEMY_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(
                        rng.gen_range(-600.0..600.0),
                        rng.gen_range(-400.0..400.0),
                        1.0,
                    ),
                    scale: Vec3::new(10.0, 10.0, 1.0),
                    ..default()
                },
                ..default()
            })
            .insert(Speed(0.0, 0.0))
            .insert(Enemy);
    }
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player = query.single_mut();

    if keyboard_input.pressed(KeyCode::Left) {
        player.translation.x -= 2.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        player.translation.x += 2.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        player.translation.y += 2.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        player.translation.y -= 2.0;
    }
}

fn enemy_ai(
    mut query: Query<(&mut Speed, &Transform), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player = player_query.single();
    for (mut speed, transform) in query.iter_mut() {
        let dx = player.translation.x - transform.translation.x;
        let dy = player.translation.y - transform.translation.y;
        speed.0 = dx.clamp(-1.0, 1.0);
        speed.1 = dy.clamp(-1.0, 1.0);
    }
}

fn move_things(mut query: Query<(&mut Transform, &Speed)>) {
    for (mut transform, speed) in query.iter_mut() {
        transform.translation.x += speed.0;
        transform.translation.y += speed.1;
    }
}

fn check_collisions(
    mut collider: Query<(Entity, &mut Speed, &Transform)>,
    obstacles: Query<(Entity, &Transform)>,
) {
    for (collider_ent, mut speed, collider) in collider.iter_mut() {
        for (obstacle_ent, obstacle) in obstacles.iter() {
            if obstacle_ent == collider_ent {
                continue;
            }
            let new_pos = collider.translation + Vec3::new(speed.0, speed.1, 0.0);
            let dist = new_pos.distance_squared(obstacle.translation);
            if dist > 400.0 {
                continue;
            }
            let collision = collide(
                new_pos,
                collider.scale.truncate(),
                obstacle.translation,
                obstacle.scale.truncate(),
            );
            if let Some(collision) = collision {
                match collision {
                    Collision::Left => speed.0 = if speed.0 > 0.0 { 0.0 } else { speed.0 },
                    Collision::Right => speed.0 = if speed.0 < 0.0 { 0.0 } else { speed.0 },
                    Collision::Top => speed.1 = if speed.1 < 0.0 { 0.0 } else { speed.1 },
                    Collision::Bottom => speed.1 = if speed.1 > 0.0 { 0.0 } else { speed.1 },
                    _ => {}
                }
            }
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WindowDescriptor {
            title: "Platformer!".to_string(),
            width: 640.0,
            height: 400.0,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_startup_system(setup)
        .add_system(enemy_ai)
        .add_system(check_collisions.after(enemy_ai))
        .add_system(move_things.after(check_collisions))
        .add_system(handle_input.before(move_things))
        .add_system(bevy::input::system::exit_on_esc_system);
    }
}
