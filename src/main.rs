mod ai;
mod collisions;
mod level;
mod utils;

use ::bevy::prelude::*;
use ai::{
    pathfinding::{init_pathfinding_graph, Pathfinding, PathfindingPlugin},
    platformer_ai::{PlatformerAI, PlatformerAIPlugin, PLATFORMER_AI_AGENT_RADIUS},
};
use bevy::{
    app::AppExit,
    window::{PresentMode, PrimaryWindow},
};
use collisions::{s_collision, CollisionPlugin};
use level::{generate_level_polygons, Level};

pub const GRAVITY_STRENGTH: f32 = 0.5;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(InputDir { dir: Vec2::ZERO })
        .insert_resource(GizmosVisible { visible: false })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Pursue AI Test".to_string(),
                present_mode: PresentMode::AutoVsync,
                focused: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PathfindingPlugin)
        .add_plugins(PlatformerAIPlugin)
        .add_plugins(CollisionPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(Update, s_input)
        .add_systems(Update, s_move_goal_point.after(s_input))
        .add_systems(Update, s_render.after(s_collision))
        .run();
}

#[derive(Resource)]
pub struct InputDir {
    pub dir: Vec2,
}

#[derive(Resource)]
pub struct GizmosVisible {
    pub visible: bool,
}

#[derive(Component)]
pub struct Physics {
    pub prev_position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub radius: f32,
    pub normal: Vec2,
    pub grounded: bool,
    pub walled: i8,
    pub has_wall_jumped: bool,
}

pub fn s_init(mut commands: Commands, pathfinding: ResMut<Pathfinding>) {
    let grid_size = 32.0;

    let (level_polygons, size, half_size) = generate_level_polygons(grid_size);

    let level = Level {
        polygons: level_polygons,
        grid_size,
        size,
        half_size,
    };

    init_pathfinding_graph(&level, pathfinding);

    commands.insert_resource(level);

    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -250.0, 0.0)),
        Physics {
            prev_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: PLATFORMER_AI_AGENT_RADIUS,
            normal: Vec2::ZERO,
            grounded: false,
            walled: 0,
            has_wall_jumped: false,
        },
        PlatformerAI {
            current_target_node: None,
            jump_from_pos: None,
            jump_to_pos: None,
        },
    ));
}

pub fn s_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut input_dir: ResMut<InputDir>,
    mut gizmos_visible: ResMut<GizmosVisible>,
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PlatformerAI)>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mut pathfinding: ResMut<Pathfinding>,
) {
    // Escape to exit (if not WASM)
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit);
    }

    // R to reset
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (mut transform, mut physics, _platformer_ai) in platformer_ai_query.iter_mut() {
            transform.translation = Vec3::new(0.0, -250.0, 0.0);
            physics.prev_position = Vec2::ZERO;
            physics.velocity = Vec2::ZERO;
            physics.acceleration = Vec2::ZERO;
        }
    }

    // Arrow keys to move goal point
    {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        dbg!(direction);

        // Normalize direction
        direction = direction.normalize_or_zero();

        // Set direction resource
        input_dir.dir = direction;
    }

    // G to toggle gizmos
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        gizmos_visible.visible = !gizmos_visible.visible;
    }

    // Space to toggle goal point
    if keyboard_input.just_pressed(KeyCode::Space) {
        pathfinding.active = !pathfinding.active;
        if pathfinding.active {
            // Set the closest node to the node closest to the goal point
            let mut closest_distance = f32::MAX;

            for node_index in 0..pathfinding.nodes.len() {
                let node = &pathfinding.nodes[node_index];

                let distance = (pathfinding.goal_position - node.position).length_squared();

                if distance < closest_distance {
                    closest_distance = distance;
                    pathfinding.goal_graph_node = Some(node.clone());
                }
            }
        } else {
            pathfinding.goal_graph_node = None;
        }
    }

    // Print some debug info if you click on a pathfinding node
    if mouse_buttons.just_pressed(MouseButton::Left) {
        let window_size = q_windows.single().resolution.clone();

        if let Some(position) = q_windows.single().cursor_position() {
            let mut mouse_pos_world =
                position - Vec2::new(window_size.width() / 2.0, window_size.height() / 2.0);
            mouse_pos_world.y *= -1.0;

            for node_index in 0..pathfinding.nodes.len() {
                let node = &pathfinding.nodes[node_index];

                if (mouse_pos_world - node.position).length_squared() < (3.5_f32).powi(2) {
                    println!("Node index: {}", node_index);
                    dbg!(node);
                }
            }
        }
    }
}
pub fn s_move_goal_point(input_dir: Res<InputDir>, mut pathfinding: ResMut<Pathfinding>) {
    pathfinding.goal_position += input_dir.dir * 4.0;

    if pathfinding.active {
        // Set the closest node to the node closest to the goal point
        let mut closest_distance = f32::MAX;
        for node_index in 0..pathfinding.nodes.len() {
            let node = &pathfinding.nodes[node_index];

            let distance = (pathfinding.goal_position - node.position).length_squared();

            if distance < closest_distance {
                closest_distance = distance;
                pathfinding.goal_graph_node = Some(node.clone());
            }
        }
    }
}

pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    platformer_ai_query: Query<(&Transform, &Physics, &PlatformerAI)>,
    pathfinding: Res<Pathfinding>,
    gizmos_visible: Res<GizmosVisible>,
) {
    // Draw the level polygons
    for polygon_index in 0..level.polygons.len() {
        let polygon = &level.polygons[polygon_index];

        gizmos.linestrip_2d(
            polygon.points.iter().cloned().collect::<Vec<Vec2>>(),
            polygon.color,
        );
    }

    // Draw the goal point
    gizmos.circle_2d(
        pathfinding.goal_position,
        7.5,
        if pathfinding.active {
            Color::GREEN
        } else {
            Color::GRAY.with_a(0.2)
        },
    );

    // Draw the AI
    for (transform, physics, platformer_ai) in platformer_ai_query.iter() {
        gizmos.circle_2d(transform.translation.xy(), physics.radius, Color::RED);
    }
}
