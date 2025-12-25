mod ai;
mod collisions;
mod level;
mod utils;

use ::bevy::prelude::*;
use ai::{
    pathfinding::{init_pathfinding_graph, PathfindingGraph, PathfindingPlugin},
    platformer_ai::{PlatformerAI, PlatformerAIPlugin},
    pursue_ai::{PursueAI, PursueAIPlugin, PursueAIState, PURSUE_AI_AGENT_RADIUS},
};
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::window::{PresentMode, PrimaryWindow};
use collisions::{s_collision, CollisionPlugin};
use level::{generate_level_polygons, Level};

pub const GRAVITY_STRENGTH: f32 = 0.5;

// Goal point movement constants
const GOAL_POINT_MOVE_SPEED: f32 = 4.0;
const GOAL_POINT_RADIUS: f32 = 7.5;
const PATHFINDING_NODE_CLICK_RADIUS: f32 = 3.5;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .insert_resource(InputDir { dir: Vec2::ZERO })
        .insert_resource(GizmosVisible { visible: false })
        .insert_resource(GoalEnabled { enabled: true })
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
        .add_plugins(PursueAIPlugin)
        .add_plugins(CollisionPlugin)
        // Startup systems
        .add_systems(Startup, s_init)
        // Update systems
        .add_systems(
            Update,
            (
                s_handle_exit,
                s_handle_reset,
                s_handle_goal_point_input,
                s_handle_gizmo_toggle,
                s_handle_goal_toggle,
                s_handle_pathfinding_debug,
            ),
        )
        .add_systems(Update, s_move_goal_point.after(s_handle_goal_point_input))
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

#[derive(Resource)]
pub struct GoalEnabled {
    pub enabled: bool,
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

#[derive(Component)]
pub struct GoalPoint {}

pub fn s_init(mut commands: Commands, pathfinding: ResMut<PathfindingGraph>) {
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

    commands.spawn(Camera2d);

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        GoalPoint {},
    ));

    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, -250.0, 0.0)),
        Physics {
            prev_position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            radius: PURSUE_AI_AGENT_RADIUS,
            normal: Vec2::ZERO,
            grounded: false,
            walled: 0,
            has_wall_jumped: false,
        },
        PlatformerAI {
            current_target_node: None,
            jump_from_pos: None,
            jump_to_pos: None,
            cached_path: None,
            last_goal_position: None,
            current_path_index: 0,
        },
        PursueAI {
            state: PursueAIState::Wander,
            current_wander_goal: None,
        },
    ));
}

pub fn s_handle_exit(keyboard_input: Res<ButtonInput<KeyCode>>, mut exit: MessageWriter<AppExit>) {
    // Escape to exit (if not WASM)
    #[cfg(not(target_arch = "wasm32"))]
    if keyboard_input.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

pub fn s_handle_reset(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut platformer_ai_query: Query<
        (&mut Transform, &mut Physics, &mut PlatformerAI),
        With<PursueAI>,
    >,
) {
    // R to reset
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        for (mut transform, mut physics, _platformer_ai) in platformer_ai_query.iter_mut() {
            transform.translation = Vec3::new(0.0, -250.0, 0.0);
            physics.prev_position = Vec2::ZERO;
            physics.velocity = Vec2::ZERO;
            physics.acceleration = Vec2::ZERO;
        }
    }
}

pub fn s_handle_goal_point_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut input_dir: ResMut<InputDir>,
) {
    // Arrow keys to move goal point
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

    // Normalize direction
    direction = direction.normalize_or_zero();

    // Set direction resource
    input_dir.dir = direction;
}

pub fn s_handle_gizmo_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut gizmos_visible: ResMut<GizmosVisible>,
) {
    // G to toggle gizmos
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        gizmos_visible.visible = !gizmos_visible.visible;
    }
}

pub fn s_handle_goal_toggle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut goal_enabled: ResMut<GoalEnabled>,
) {
    // Space to toggle goal
    if keyboard_input.just_pressed(KeyCode::Space) {
        goal_enabled.enabled = !goal_enabled.enabled;
    }
}

pub fn s_handle_pathfinding_debug(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    pathfinding: Res<PathfindingGraph>,
) {
    // Print some debug info if you click on a pathfinding node
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Ok(window) = q_windows.single() {
            let window_size = window.resolution.clone();

            if let Some(position) = window.cursor_position() {
                let mut mouse_pos_world =
                    position - Vec2::new(window_size.width() / 2.0, window_size.height() / 2.0);
                mouse_pos_world.y *= -1.0;

                for node_index in 0..pathfinding.nodes.len() {
                    let node = &pathfinding.nodes[node_index];

                    if (mouse_pos_world - node.position).length_squared()
                        < PATHFINDING_NODE_CLICK_RADIUS.powi(2)
                    {
                        // Node clicked (debug output removed)
                    }
                }
            }
        }
    }
}

pub fn s_move_goal_point(
    mut goal_point_query: Query<&mut Transform, With<GoalPoint>>,
    input_dir: Res<InputDir>,
) {
    if let Ok(mut goal_point_transform) = goal_point_query.single_mut() {
        goal_point_transform.translation += (input_dir.dir * GOAL_POINT_MOVE_SPEED).extend(0.0);
    }
}

pub fn s_render(
    mut gizmos: Gizmos,
    level: Res<Level>,
    pursue_ai_query: Query<(&Transform, &Physics, &PursueAI)>,
    goal_point_query: Query<&Transform, With<GoalPoint>>,
    goal_enabled: Res<GoalEnabled>,
) {
    // Draw the level polygons
    for polygon_index in 0..level.polygons.len() {
        let polygon = &level.polygons[polygon_index];

        gizmos.linestrip_2d(polygon.points.to_vec(), polygon.color);
    }

    // Draw the goal point (greyed out when disabled, green when enabled)
    if let Ok(goal_point_transform) = goal_point_query.single() {
        let color = if goal_enabled.enabled {
            Color::srgb(0.0, 1.0, 0.0) // Green when enabled
        } else {
            Color::srgb(0.5, 0.5, 0.5) // Grey when disabled
        };
        gizmos.circle_2d(
            goal_point_transform.translation.xy(),
            GOAL_POINT_RADIUS,
            color,
        );
    }

    // Draw the AI
    for (transform, physics, _pursue_ai) in pursue_ai_query.iter() {
        gizmos.circle_2d(
            transform.translation.xy(),
            physics.radius,
            Color::srgb(1.0, 0.0, 0.0),
        );
    }
}
