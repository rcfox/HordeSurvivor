use bevy::{
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use rand::Rng;

const PLAYER_COLOR: Color = Color::rgb(0.0, 0.0, 1.0);
const ENEMY_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const BULLET_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(GamePlugin)
        .run();
}

pub struct GamePlugin;

#[derive(Component)]
struct EnemySpawnConfig {
    timer: Timer,
    desired_amount: usize,
}

// Player component
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct PreventOverlap;

#[derive(Component)]
struct Velocity {
    speed: f32,
    direction: Vec3,
}

#[derive(Component)]
struct Health {
    max: u32,
    current: u32,
}

#[derive(Component)]
struct Damage(u32);

#[derive(Component)]
struct ShootBullet {
    cooldown: Timer,
    damage: u32,
    size: f32,
    speed: f32,
}

#[derive(Bundle)]
struct BulletBundle {
    speed: Velocity,
    damage: Damage,

    #[bundle]
    sprite: SpriteBundle,
}

fn spawn_enemies(mut commands: Commands, num: usize) {
    let mut rng = rand::thread_rng();
    for _ in 1..num {
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
            .insert(Health { max: 1, current: 1 })
            .insert(Velocity {
                speed: 60.0,
                direction: Vec3::ZERO,
            })
            .insert(PreventOverlap)
            .insert(Enemy);
    }
}

fn spawn_new_enemies(
    commands: Commands,
    time: Res<Time>,
    mut spawn: ResMut<EnemySpawnConfig>,
    enemies: Query<&Enemy>,
) {
    spawn.timer.tick(time.delta());

    if spawn.timer.finished() {
        let num_enemies = enemies.iter().len();
        if spawn.desired_amount > num_enemies {
            let amount = spawn.desired_amount - num_enemies;

            spawn_enemies(commands, amount.clamp(0, 30));
        }
    }
}

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
        .insert(Velocity {
            speed: 80.0,
            direction: Vec3::ZERO,
        })
        .insert(ShootBullet {
            cooldown: Timer::new(std::time::Duration::from_millis(300), true),
            damage: 1,
            size: 3.0,
            speed: 200.0,
        })
        .insert(PreventOverlap)
        .insert(Player);

    //spawn_enemies(commands, 100);
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    let mut velocity = query.single_mut();

    velocity.direction.x = 0.0;
    velocity.direction.y = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        velocity.direction.x = -1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        velocity.direction.x = 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        velocity.direction.y = 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        velocity.direction.y = -1.0;
    }

    velocity.direction = velocity.direction.normalize_or_zero();
}

fn enemy_ai(
    mut query: Query<(&mut Velocity, &Transform), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player = player_query.single();
    for (mut velocity, transform) in query.iter_mut() {
        velocity.direction = (player.translation - transform.translation).normalize_or_zero();
    }
}

fn move_things(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.speed * velocity.direction * time.delta_seconds();
    }
}

fn check_collisions(
    time: Res<Time>,
    mut collider: Query<(Entity, &mut Velocity, &Transform), With<PreventOverlap>>,
    obstacles: Query<(Entity, &Transform), With<PreventOverlap>>,
) {
    for (collider_ent, mut velocity, collider) in collider.iter_mut() {
        for (obstacle_ent, obstacle) in obstacles.iter() {
            if obstacle_ent == collider_ent {
                continue;
            }
            let new_pos =
                collider.translation + velocity.speed * velocity.direction * time.delta_seconds();
            let collision = collide(
                new_pos,
                collider.scale.truncate(),
                obstacle.translation,
                obstacle.scale.truncate(),
            );
            if let Some(collision) = collision {
                match collision {
                    Collision::Left => {
                        velocity.direction.x = if velocity.direction.x > 0.0 {
                            0.0
                        } else {
                            velocity.direction.x
                        }
                    }
                    Collision::Right => {
                        velocity.direction.x = if velocity.direction.x < 0.0 {
                            0.0
                        } else {
                            velocity.direction.x
                        }
                    }
                    Collision::Top => {
                        velocity.direction.y = if velocity.direction.y < 0.0 {
                            0.0
                        } else {
                            velocity.direction.y
                        }
                    }
                    Collision::Bottom => {
                        velocity.direction.y = if velocity.direction.y > 0.0 {
                            0.0
                        } else {
                            velocity.direction.y
                        }
                    }
                    Collision::Inside => {
                        velocity.direction.x = 0.0;
                        velocity.direction.y = 0.0;
                    }
                }
            }
        }
    }
}

fn check_bullet_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Damage, &Transform)>,
    mut targets: Query<(Entity, &mut Health, &Transform)>,
) {
    for (bullet, bullet_damage, bullet_transform) in bullets.iter() {
        for (target, mut health, target_transform) in targets.iter_mut() {
            if bullet == target {
                continue;
            }
            let dist = bullet_transform
                .translation
                .distance_squared(target_transform.translation);
            if dist > 400.0 {
                continue;
            }
            let collision = collide(
                bullet_transform.translation,
                bullet_transform.scale.truncate(),
                target_transform.translation,
                target_transform.scale.truncate(),
            );
            if let Some(_collision) = collision {
                health.current -= bullet_damage.0;
                if health.current == 0 {
                    commands.entity(target).despawn();
                }
                commands.entity(bullet).despawn();
            }
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut ShootBullet, &Transform)>,
    targets: Query<&Transform, With<Enemy>>,
) {
    let dt = time.delta();
    for (mut shoot, transform) in query.iter_mut() {
        shoot.cooldown.tick(dt);

        if shoot.cooldown.finished() {
            let m = targets.iter().map(|target_transform| {
                (
                    target_transform
                        .translation
                        .distance_squared(transform.translation),
                    target_transform,
                )
            });
            let target = m.min_by(|a, b| a.0.partial_cmp(&b.0).expect("Tried to compare a NaN"));
            if let Some(target) = target {
                let direction = (target.1.translation - transform.translation).normalize_or_zero();
                commands.spawn_bundle(BulletBundle {
                    damage: Damage(shoot.damage),
                    speed: Velocity {
                        speed: shoot.speed,
                        direction,
                    },
                    sprite: SpriteBundle {
                        sprite: Sprite {
                            color: BULLET_COLOR,
                            ..default()
                        },
                        transform: Transform {
                            scale: Vec3::new(shoot.size, shoot.size, 1.0),
                            translation: transform.translation,
                            ..default()
                        },
                        ..default()
                    },
                });
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
        .insert_resource(EnemySpawnConfig {
            timer: Timer::new(std::time::Duration::from_secs(5), true),
            desired_amount: 150,
        })
        .add_startup_system(setup)
        .add_system(enemy_ai)
        .add_system(check_collisions.after(enemy_ai))
        .add_system(move_things.after(check_collisions))
        .add_system(handle_input.before(move_things))
        .add_system(shoot_bullet.before(move_things))
        .add_system(check_bullet_collisions.before(move_things))
        .add_system(spawn_new_enemies)
        .add_system(bevy::input::system::exit_on_esc_system);
    }
}
