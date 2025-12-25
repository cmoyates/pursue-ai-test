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
    Physics, GRAVITY_STRENGTH,
};

use super::PursueAI;

pub const WANDER_MAX_SPEED: f32 = 3.0;

pub fn wander_update(
    transform: &mut Transform,
    physics: &mut Physics,
    pursue_ai: &mut PursueAI,
    pathfinding: &mut PathfindingGraph,
) -> Option<PursueAIState> {
    wander_movement(transform, physics, pursue_ai, pathfinding);

    return None;
}

pub fn wander_movement(
    transform: &mut Transform,
    physics: &mut Physics,
    pursue_ai: &mut PursueAI,
    pathfinding: &mut PathfindingGraph,
) {
    // Pick a random goal point
    let goal_node = get_random_goal_node(transform.translation.xy(), pathfinding);

    // If the goal point is reached
    // pick a new goal point
    // else
    // Pathfind to the goal point
}

pub fn get_random_goal_node(
    agent_position: Vec2,
    pathfinding: &mut PathfindingGraph,
) -> PathfindingGraphNode {
    let sample_count = 3;
    let pathfinding_node_count = pathfinding.nodes.len();

    let mut furthest_node: Option<PathfindingGraphNode> = None;
    let mut furthest_node_distance_sq: f32 = f32::MAX;

    for _ in 0..sample_count {
        let random_node_index = rand::rng().random_range(0..pathfinding_node_count);
        let random_node = &pathfinding.nodes[random_node_index];

        let distance_sq = (agent_position - random_node.position).length_squared();

        if distance_sq < furthest_node_distance_sq {
            furthest_node_distance_sq = distance_sq;
            furthest_node = Some(random_node.clone());
        }
    }

    furthest_node.unwrap()
}
