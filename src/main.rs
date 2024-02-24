use ::bevy::prelude::*;
use bevy::{app::AppExit, window::PresentMode};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pursue AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                focused: true,
                ..default()
            }),
            ..default()
        }))
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, s_render)
        .run();
}

pub fn s_init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn s_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    // Escape to exit (if not WASM)
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

pub fn s_render(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::ZERO, 10.0, Color::WHITE);
}
