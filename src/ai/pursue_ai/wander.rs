use bevy::{
    math::{Vec2, Vec3Swizzles},
    transform::components::Transform,
};
use rand::prelude::*;

use crate::{
    ai::{
        pathfinding::{PathfindingGraph, PathfindingGraphNode},
        pursue_ai::PursueAIState,
    },
    Physics,
};

use super::PursueAI;

#[allow(dead_code)]
pub const WANDER_MAX_SPEED: f32 = 3.0;

// Wander AI constants
const WANDER_SAMPLE_COUNT: usize = 3;
const WANDER_GOAL_REACHED_THRESHOLD: f32 = 30.0; // Distance threshold for considering wander goal reached
const WANDER_GOAL_REACHED_THRESHOLD_SQ: f32 =
    WANDER_GOAL_REACHED_THRESHOLD * WANDER_GOAL_REACHED_THRESHOLD; // 900.0 squared

pub fn wander_update(
    transform: &mut Transform,
    _physics: &mut Physics,
    pursue_ai: &mut PursueAI,
    pathfinding: &PathfindingGraph,
) -> Option<PursueAIState> {
    wander_movement(transform, pursue_ai, pathfinding);

    None
}

pub fn wander_movement(
    transform: &mut Transform,
    pursue_ai: &mut PursueAI,
    pathfinding: &PathfindingGraph,
) {
    let agent_position = transform.translation.xy();

    // Check if we have a current wander goal
    if let Some(goal_node_id) = pursue_ai.current_wander_goal {
        // Check if we've reached the goal
        if let Some(goal_node) = pathfinding.nodes.get(goal_node_id) {
            let distance_sq = (agent_position - goal_node.position).length_squared();
            if distance_sq <= WANDER_GOAL_REACHED_THRESHOLD_SQ {
                // Goal reached, clear it so a new one is selected next frame
                pursue_ai.current_wander_goal = None;
            }
        } else {
            // Invalid node ID, clear it
            pursue_ai.current_wander_goal = None;
        }
    }

    // If no goal is set, pick a new random distant node
    if pursue_ai.current_wander_goal.is_none() {
        let goal_node = get_random_goal_node(agent_position, pathfinding);
        // Use the node's ID directly
        pursue_ai.current_wander_goal = Some(goal_node.id);
    }
}

pub fn get_random_goal_node(
    agent_position: Vec2,
    pathfinding: &PathfindingGraph,
) -> PathfindingGraphNode {
    let pathfinding_node_count = pathfinding.nodes.len();

    let mut furthest_node: Option<PathfindingGraphNode> = None;
    let mut furthest_node_distance_sq: f32 = 0.0; // Changed to 0.0 to find furthest, not closest

    for _ in 0..WANDER_SAMPLE_COUNT {
        let random_node_index = rand::rng().random_range(0..pathfinding_node_count);
        let random_node = &pathfinding.nodes[random_node_index];

        let distance_sq = (agent_position - random_node.position).length_squared();

        if distance_sq > furthest_node_distance_sq {
            furthest_node_distance_sq = distance_sq;
            furthest_node = Some(random_node.clone());
        }
    }

    furthest_node.expect("Pathfinding graph should have at least one node")
}
