use bevy::{
    app::{App, Plugin, Update},
    color::Color,
    ecs::{
        component::Component,
        query::{With, Without},
        schedule::IntoScheduleConfigs,
        system::{Query, Res},
    },
    gizmos::gizmos::Gizmos,
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};

use crate::{s_move_goal_point, GoalPoint, GizmosVisible, Physics, GRAVITY_STRENGTH};

use super::{a_star::{find_path, PathNode}, pathfinding::PathfindingGraph};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum PathFollowingStrategy {
    CurrentNodeToNextNode,
    CurrentNodeOffsetToNextNodeOffset,
    AgentToCurrentNode,
    AgentToCurrentNodeOffset,
    AgentToNextNode,
    AgentToNextNodeOffset,
    AgentToGoal,
    None,
}

const WANDER_MAX_SPEED: f32 = 3.0;
// const PURSUE_MAX_SPEED: f32 = 5.0;
// const ATTACK_MAX_SPEED: f32 = 7.0;

// const STEERING_SCALE: f32 = 0.1;

pub const PLATFORMER_AI_JUMP_FORCE: f32 = 8.0;

pub const ACCELERATION_SCALERS: (f32, f32) = (0.2, 0.4);

// Platformer AI movement constants
const GIZMO_LINE_LENGTH: f32 = 15.0;
const VELOCITY_MAGNITUDE_THRESHOLD: f32 = 0.1;
const JUMP_TIME_MULTIPLIER: f32 = 1.0;
const PATHFINDING_NODE_GIZMO_RADIUS: f32 = 5.0;

// Path caching constants (using squared distances to avoid sqrt)
const GOAL_CHANGE_THRESHOLD_SQ: f32 = 25.0; // 5.0 squared
const PATH_DEVIATION_THRESHOLD_SQ: f32 = 100.0; // 10.0 squared
const NODE_REACHED_THRESHOLD_SQ: f32 = 64.0; // 8.0 squared (agent radius squared)

#[allow(dead_code)]
pub struct PlatformerAIPlugin;

impl Plugin for PlatformerAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_platformer_ai_movement.after(s_move_goal_point));
    }
}

#[derive(Component)]
pub struct PlatformerAI {
    pub current_target_node: Option<usize>,
    pub jump_from_pos: Option<Vec2>,
    pub jump_to_pos: Option<Vec2>,
    // Path caching fields
    pub cached_path: Option<Vec<PathNode>>,
    pub last_goal_position: Option<Vec2>,
    pub current_path_index: usize,
}

pub fn s_platformer_ai_movement(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PlatformerAI)>,
    pathfinding: Res<PathfindingGraph>,
    gismo_visible: Res<GizmosVisible>,
    goal_point_query: Query<&Transform, (With<GoalPoint>, Without<PlatformerAI>)>,
    mut gizmos: Gizmos,
) {
    // Get goal point position, default to (0, 0) if not found
    let goal_position = goal_point_query
        .single()
        .ok()
        .map(|t| t.translation.xy())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut physics, mut platformer_ai) in platformer_ai_query.iter_mut() {
        let (move_dir, jump_velocity, jump_from_node, jump_to_node) = get_move_inputs(
            pathfinding.as_ref(),
            transform.translation.xy(),
            &physics,
            &mut platformer_ai,
            &mut gizmos,
            gismo_visible.visible,
            goal_position,
        );

        if gismo_visible.visible {
            gizmos.line_2d(
                transform.translation.xy(),
                transform.translation.xy() + move_dir * GIZMO_LINE_LENGTH,
                Color::srgb(1.0, 0.0, 0.0),
            );
        }

        let falling = physics.normal.length_squared() == 0.0;
        let no_move_dir = move_dir.length_squared() == 0.0;

        apply_movement_acceleration(&mut physics, &move_dir, falling, no_move_dir);

        apply_gravity_toward_normal(&mut physics, falling /*, player_move_off_wall*/);

        // Jumping
        {
            // If the player is trying to jump
            if jump_velocity.length_squared() > 0.0 && !falling {
                // If on the ground
                if physics.grounded {
                    // Jump
                    physics.velocity = jump_velocity;
                    physics.acceleration.x = 0.0;
                    physics.acceleration.y = -GRAVITY_STRENGTH;
                    physics.grounded = false;
                    physics.has_wall_jumped = false;
                    physics.walled = 0;

                    platformer_ai.jump_from_pos = jump_from_node;
                    platformer_ai.jump_to_pos = jump_to_node;
                }
                // If on a wall
                else if physics.walled != 0 {
                    // Wall jump
                    physics.velocity = jump_velocity;
                    physics.acceleration.x = 0.0;
                    physics.acceleration.y = -GRAVITY_STRENGTH;
                    physics.walled = 0;
                    physics.grounded = false;
                    physics.has_wall_jumped = true;
                    platformer_ai.jump_from_pos = jump_from_node;
                    platformer_ai.jump_to_pos = jump_to_node;
                }
            }
        }

        update_physics_and_transform(&mut physics, &mut transform);

        // dbg!(physics.velocity);
    }
}

fn get_move_inputs(
    pathfinding: &PathfindingGraph,
    agent_position: Vec2,
    agent_physics: &Physics,
    platformer_ai: &mut PlatformerAI,
    gizmos: &mut Gizmos,
    gizmos_visible: bool,
    goal_position: Vec2,
) -> (Vec2, Vec2, Option<Vec2>, Option<Vec2>) {
    let mut move_dir = Vec2::ZERO;
    let mut jump_velocity = Vec2::ZERO;
    let mut jump_from_node = None;
    let mut jump_to_node = None;

    // Check if cached path is still valid
    let path_needs_recalculation = should_recalculate_path(
        platformer_ai,
        agent_position,
        goal_position,
        pathfinding,
    );

    let path = if path_needs_recalculation {
        // Recalculate path
        let new_path = find_path(pathfinding, agent_position, goal_position);
        if let Some(ref path_vec) = new_path {
            platformer_ai.cached_path = Some(path_vec.clone());
        } else {
            platformer_ai.cached_path = None;
        }
        platformer_ai.last_goal_position = Some(goal_position);
        platformer_ai.current_path_index = 0;
        new_path
    } else {
        // Use cached path
        platformer_ai.cached_path.clone()
    };

    if let Some(path) = &path {
        if gizmos_visible {
            let mut prev_pos = agent_position;
            for node in path {
                gizmos.circle_2d(
                    node.position,
                    PATHFINDING_NODE_GIZMO_RADIUS,
                    Color::srgb(0.0, 1.0, 0.0),
                );
                gizmos.line_2d(prev_pos, node.position, Color::srgb(0.0, 1.0, 0.0));

                prev_pos = node.position;
            }
        }

        // Use current_path_index to get the current and next nodes
        let current_idx = platformer_ai.current_path_index;
        if current_idx < path.len() && path.len() > current_idx + 1 {
            let offset_current_node =
                path[current_idx].position + pathfinding.nodes[path[current_idx].id].normal * agent_physics.radius;
            let offset_next_node: Vec2 =
                path[current_idx + 1].position + pathfinding.nodes[path[current_idx + 1].id].normal * agent_physics.radius;

            let agent_on_wall = agent_physics.normal.y > -0.01;

            let corner_is_external = pathfinding.nodes[path[current_idx].id].is_external_corner;

            let current_node_is_corner = corner_is_external.is_some();

            let is_jumpable_connection = pathfinding.nodes[path[current_idx].id]
                .jumpable_connections
                .iter()
                .any(|jumpable_connection| jumpable_connection.node_id == path[current_idx + 1].id);

            let falling = agent_physics.normal.length_squared() <= 0.0;

            let path_following_strategy: PathFollowingStrategy;

            // Agent not falling
            if !falling {
                // Agent jumping
                if is_jumpable_connection {
                    let agent_on_other_side_next_frame = agent_on_other_side_next_frame(
                        agent_position,
                        agent_physics.velocity,
                        path[current_idx].position,
                        agent_on_wall,
                    );

                    let agent_not_moving =
                        agent_physics.velocity.length_squared() < VELOCITY_MAGNITUDE_THRESHOLD;

                    path_following_strategy = if agent_on_other_side_next_frame || agent_not_moving
                    {
                        PathFollowingStrategy::AgentToNextNodeOffset
                    } else {
                        PathFollowingStrategy::AgentToCurrentNodeOffset
                    };
                } else {
                    // Non-jumping corner
                    if current_node_is_corner {
                        path_following_strategy = PathFollowingStrategy::AgentToNextNode;
                    }
                    // Non-jumping flat surface
                    else {
                        let current_pos_to_next_offset = offset_next_node - agent_position;
                        let current_offset_to_next_offset = offset_next_node - offset_current_node;

                        if current_pos_to_next_offset.length_squared()
                            <= current_offset_to_next_offset.length_squared()
                        {
                            path_following_strategy = PathFollowingStrategy::AgentToNextNodeOffset;
                        } else {
                            path_following_strategy =
                                PathFollowingStrategy::AgentToCurrentNodeOffset;
                        }
                    }
                }
            }
            // Agent falling
            else {
                path_following_strategy = PathFollowingStrategy::AgentToNextNodeOffset;
            }

            move_dir = match path_following_strategy {
                PathFollowingStrategy::CurrentNodeToNextNode => path[current_idx + 1].position - path[current_idx].position,
                PathFollowingStrategy::CurrentNodeOffsetToNextNodeOffset => {
                    offset_next_node - offset_current_node
                }
                PathFollowingStrategy::AgentToCurrentNode => path[current_idx].position - agent_position,
                PathFollowingStrategy::AgentToCurrentNodeOffset => {
                    offset_current_node - agent_position
                }
                PathFollowingStrategy::AgentToNextNode => path[current_idx + 1].position - agent_position,
                PathFollowingStrategy::AgentToNextNodeOffset => offset_next_node - agent_position,
                // PathFollowingStrategy::AgentToGoal => pathfinding.goal_position - agent_position,
                PathFollowingStrategy::None => Vec2::ZERO,
                _ => Vec2::ZERO,
            }
            .normalize_or_zero();

            // Jumping
            if (path_following_strategy == PathFollowingStrategy::AgentToNextNodeOffset
                || path_following_strategy == PathFollowingStrategy::AgentToNextNode)
                && is_jumpable_connection
            {
                let node_position_delta = path[current_idx + 1].position - path[current_idx].position;
                let gravity_acceleration = Vec2::new(0.0, -GRAVITY_STRENGTH);
                let jump_time = JUMP_TIME_MULTIPLIER
                    * (4.0 * node_position_delta.dot(node_position_delta)
                        / gravity_acceleration.dot(gravity_acceleration))
                    .sqrt()
                    .sqrt();
                jump_velocity =
                    node_position_delta / jump_time - gravity_acceleration * jump_time / 2.0;

                jump_from_node = Some(offset_current_node);
                jump_to_node = Some(offset_next_node);
            }
        }
    }

    // Advance path index if agent reached current node
    if let Some(ref path) = path {
        advance_path_index(platformer_ai, agent_position, path);
    }

    (move_dir, jump_velocity, jump_from_node, jump_to_node)
}

fn should_recalculate_path(
    platformer_ai: &PlatformerAI,
    agent_position: Vec2,
    goal_position: Vec2,
    _pathfinding: &PathfindingGraph,
) -> bool {
    // If no cached path, recalculate
    let Some(ref cached_path) = platformer_ai.cached_path else {
        return true;
    };

    // If path is empty or exhausted, recalculate
    if cached_path.is_empty() || platformer_ai.current_path_index >= cached_path.len() {
        return true;
    }

    // If goal moved beyond threshold, recalculate
    if let Some(last_goal) = platformer_ai.last_goal_position {
        let goal_delta_sq = (goal_position - last_goal).length_squared();
        if goal_delta_sq > GOAL_CHANGE_THRESHOLD_SQ {
            return true;
        }
    } else {
        return true;
    }

    // If agent deviated significantly from path, recalculate
    if let Some(current_node) = cached_path.get(platformer_ai.current_path_index) {
        let deviation_sq = (agent_position - current_node.position).length_squared();
        if deviation_sq > PATH_DEVIATION_THRESHOLD_SQ {
            return true;
        }
    }

    false
}

fn advance_path_index(
    platformer_ai: &mut PlatformerAI,
    agent_position: Vec2,
    path: &[PathNode],
) {
    // Advance index if agent reached current node
    while platformer_ai.current_path_index < path.len() {
        let current_node = &path[platformer_ai.current_path_index];
        let distance_sq = (agent_position - current_node.position).length_squared();
        
        if distance_sq <= NODE_REACHED_THRESHOLD_SQ {
            platformer_ai.current_path_index += 1;
        } else {
            break;
        }
    }
}

fn apply_movement_acceleration(
    physics: &mut Physics,
    move_dir: &Vec2,
    falling: bool,
    no_move_dir: bool,
) {
    // If the player is falling
    if falling {
        physics.acceleration = Vec2::ZERO;
        return;
    }

    // Apply acceleration
    physics.acceleration = (*move_dir * WANDER_MAX_SPEED - physics.velocity)
        * if no_move_dir {
            // Deacceleration
            ACCELERATION_SCALERS.1
        } else {
            // Acceleration
            ACCELERATION_SCALERS.0
        };
}

fn apply_gravity_toward_normal(physics: &mut Physics, falling: bool) {
    if falling {
        physics.acceleration.y = -GRAVITY_STRENGTH;
    } else {
        let gravity_normal_dir = physics.normal * GRAVITY_STRENGTH;
        physics.acceleration += gravity_normal_dir;
    }
}

fn update_physics_and_transform(physics: &mut Physics, transform: &mut Transform) {
    // Update velocity
    let new_velocity = physics.velocity + physics.acceleration;
    physics.velocity = new_velocity;

    // Update previous position
    physics.prev_position = transform.translation.xy();
    // Update position
    transform.translation.x += physics.velocity.x;
    transform.translation.y += physics.velocity.y;
}

pub fn agent_on_other_side_next_frame(
    agent_position: Vec2,
    agent_velocity: Vec2,
    node_position: Vec2,
    vertical: bool,
) -> bool {
    let dimension_index = if vertical { 1 } else { 0 };

    let agent_position_next_frame = agent_position + agent_velocity;

    let agent_side_of_corner_current =
        (agent_position[dimension_index] - node_position[dimension_index]).signum();

    let agent_side_of_corner_next_frame =
        (agent_position_next_frame[dimension_index] - node_position[dimension_index]).signum();

    agent_side_of_corner_current != agent_side_of_corner_next_frame
}
