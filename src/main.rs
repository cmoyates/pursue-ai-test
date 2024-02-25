mod level;
mod utils;

use ::bevy::prelude::*;
use bevy::{app::AppExit, window::PresentMode};
use level::{generate_level_polygons, Level};

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
    let grid_size = 32.0;

    let (level_polygons, size, half_size) = generate_level_polygons(grid_size);

    let level = Level {
        polygons: level_polygons,
        grid_size,
        size,
        half_size,
    };

    commands.insert_resource(level);

    commands.spawn(Camera2dBundle::default());
}

pub fn s_input(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<AppExit>) {
    // Escape to exit (if not WASM)
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }
}

pub fn s_render(mut gizmos: Gizmos, level: Res<Level>) {
    // Draw the level polygons
    for polygon_index in 0..level.polygons.len() {
        dbg!(polygon_index);

        let polygon = &level.polygons[polygon_index];

        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }
}
