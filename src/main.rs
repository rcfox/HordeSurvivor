use bevy::prelude::*;

const PLAYER_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

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
        .insert(Player);
}

fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let mut player_transform = query.single_mut();
    let mut dx = 0.0;
    let mut dy = 0.0;

    if keyboard_input.pressed(KeyCode::Left) {
        dx -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        dx += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        dy += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Down) {
        dy -= 1.0;
    }

    player_transform.translation.x += dx;
    player_transform.translation.y += dy;
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
        .add_system(handle_input)
        .add_system(bevy::input::system::exit_on_esc_system);
    }
}
