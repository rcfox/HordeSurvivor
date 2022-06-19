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

pub struct CollisionEvent {
    collider: Entity,
    obstacle: Entity,
}

pub struct DeathEvent {
    entity: Entity,
}

#[derive(Component)]
struct EnemySpawnConfig {
    timer: Timer,
    desired_amount: usize,
}

#[derive(Component)]
struct Owner(Option<Entity>);

#[derive(Component)]
struct Name(String);

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
struct Damage {
    damage: u32,
}

#[derive(Component)]
struct ShootBullet {
    cooldown: Timer,
    damage: u32,
    size: f32,
    speed: f32,
}

#[derive(Component)]
struct Bullet;

#[derive(Bundle)]
struct BulletBundle {
    bullet: Bullet,
    speed: Velocity,
    damage: Damage,
    owner: Owner,

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
            .insert(Name(String::from("Enemy")))
            .insert(Health { max: 1, current: 1 })
            .insert(Damage { damage: 1 })
            .insert(Velocity {
                speed: 60.0,
                direction: Vec3::ZERO,
            })
            .insert(PreventOverlap)
            .insert(Owner(None))
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
        .insert(Name(String::from("Player")))
        .insert(Health {
            max: 100,
            current: 100,
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
    for mut velocity in query.iter_mut() {
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
}

fn enemy_ai(
    mut query: Query<(&mut Velocity, &Transform), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
) {
    for player in player_query.iter() {
        for (mut velocity, transform) in query.iter_mut() {
            velocity.direction = (player.translation - transform.translation).normalize_or_zero();
        }
    }
}

fn move_things(time: Res<Time>, mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.speed * velocity.direction * time.delta_seconds();
    }
}

fn precheck_collisions(
    mut collision_events: EventWriter<CollisionEvent>,
    time: Res<Time>,
    mut collider: Query<
        (Entity, &mut Velocity, &Transform),
        (
            With<PreventOverlap>,
            Or<(Changed<Transform>, Changed<Velocity>)>,
        ),
    >,
    obstacles: Query<(Entity, &Transform), With<PreventOverlap>>,
) {
    for (collider, mut collider_velocity, collider_transform) in collider.iter_mut() {
        for (obstacle, obstacle_transform) in obstacles.iter() {
            if obstacle == collider {
                continue;
            }
            let new_pos = collider_transform.translation
                + collider_velocity.speed * collider_velocity.direction * time.delta_seconds();
            let collision = collide(
                new_pos,
                collider_transform.scale.truncate(),
                obstacle_transform.translation,
                obstacle_transform.scale.truncate(),
            );
            if let Some(side) = collision {
                match side {
                    Collision::Left => {
                        collider_velocity.direction.x = if collider_velocity.direction.x > 0.0 {
                            0.0
                        } else {
                            collider_velocity.direction.x
                        }
                    }
                    Collision::Right => {
                        collider_velocity.direction.x = if collider_velocity.direction.x < 0.0 {
                            0.0
                        } else {
                            collider_velocity.direction.x
                        }
                    }
                    Collision::Top => {
                        collider_velocity.direction.y = if collider_velocity.direction.y < 0.0 {
                            0.0
                        } else {
                            collider_velocity.direction.y
                        }
                    }
                    Collision::Bottom => {
                        collider_velocity.direction.y = if collider_velocity.direction.y > 0.0 {
                            0.0
                        } else {
                            collider_velocity.direction.y
                        }
                    }
                    Collision::Inside => {
                        collider_velocity.direction.x = 0.0;
                        collider_velocity.direction.y = 0.0;
                    }
                }
                collision_events.send(CollisionEvent { collider, obstacle });
            }
        }
    }
}

fn check_collisions(
    mut events: EventWriter<CollisionEvent>,
    collider: Query<(Entity, &Transform), Changed<Transform>>,
    obstacles: Query<(Entity, &Transform)>,
) {
    for (collider, collider_transform) in collider.iter() {
        for (obstacle, obstacle_transform) in obstacles.iter() {
            if obstacle == collider {
                continue;
            }
            let collision = collide(
                collider_transform.translation,
                collider_transform.scale.truncate(),
                obstacle_transform.translation,
                obstacle_transform.scale.truncate(),
            );
            if collision.is_some() {
                events.send(CollisionEvent { collider, obstacle });
            }
        }
    }
}

fn collision_damage(
    mut collision_events: EventReader<CollisionEvent>,
    mut death_events: EventWriter<DeathEvent>,
    damagers: Query<(&Damage, &Owner)>,
    mut damagees: Query<(Entity, &mut Health, &Name)>,
) {
    for event in collision_events.iter() {
        if let Ok((damage, owner)) = damagers.get(event.collider) {
            if let Ok((entity, mut health, _name)) = damagees.get_mut(event.obstacle) {
                if owner.0 == Some(entity) {
                    continue;
                }
                if health.current > damage.damage {
                    health.current -= damage.damage;
                } else {
                    health.current = 0;
                }
                if health.current > health.max {
                    health.current = health.max;
                }
                //println!("{} health: {}", name.0, health.current);
                if health.current == 0 {
                    death_events.send(DeathEvent { entity });
                }
            }
        }
    }
}

fn bullet_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut death_events: EventWriter<DeathEvent>,
    bullets: Query<Entity, With<Bullet>>,
    obstacles: Query<Entity, (Without<Bullet>, Without<Player>)>,
) {
    for event in collision_events.iter() {
        if let Ok(entity) = bullets.get(event.collider) {
            if obstacles.get(event.obstacle).is_ok() {
                death_events.send(DeathEvent { entity });
            }
        }
    }
}

fn handle_death(mut death_events: EventReader<DeathEvent>, mut commands: Commands) {
    let mut handled: std::collections::HashSet<Entity> = std::collections::HashSet::new();
    for event in death_events.iter() {
        if !handled.contains(&event.entity) {
            commands.entity(event.entity).despawn_recursive();
            handled.insert(event.entity);
        }
    }
}

fn shoot_bullet(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShootBullet, &Transform)>,
    targets: Query<&Transform, With<Enemy>>,
) {
    let dt = time.delta();
    for (owner, mut shoot, transform) in query.iter_mut() {
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
                    damage: Damage {
                        damage: shoot.damage,
                    },
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
                    bullet: Bullet,
                    owner: Owner(Some(owner)),
                });
            }
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        static CLEANUP: &str = "CLEANUP_STAGE";
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
        .add_event::<CollisionEvent>()
        .add_event::<DeathEvent>()
        .add_stage_after(CoreStage::Update, CLEANUP, SystemStage::single_threaded())
        .add_startup_system(setup)
        .add_system(enemy_ai)
        .add_system(precheck_collisions.after(enemy_ai))
        .add_system(move_things.after(precheck_collisions))
        .add_system(handle_input.before(move_things))
        .add_system(shoot_bullet.before(move_things))
        .add_system(check_collisions.after(move_things))
        .add_system(collision_damage.after(check_collisions))
        .add_system(bullet_collision.after(check_collisions))
        .add_system_to_stage(CLEANUP, handle_death)
        .add_system(spawn_new_enemies)
        .add_system(bevy::input::system::exit_on_esc_system);
    }
}
