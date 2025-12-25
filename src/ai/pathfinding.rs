use std::collections::HashMap;

use bevy::{
    app::{App, Plugin},
    ecs::system::ResMut,
    math::Vec2,
    prelude::Resource,
};

use crate::{level::Level, utils::line_intersect, GRAVITY_STRENGTH};

use super::{platformer_ai::PLATFORMER_AI_JUMP_FORCE, pursue_ai::PURSUE_AI_AGENT_RADIUS};

// Pathfinding constants
const PATHFINDING_NODE_SPACING: f32 = 20.0;
const PATHFINDING_NODE_DIRECTION_THRESHOLD: f32 = -0.1;
const JUMPABILITY_CHECK_TIMESTEP_DIVISIONS: i32 = 10;
const SPATIAL_CELL_SIZE: f32 = 50.0; // ~2.5x node spacing

pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PathfindingGraph {
            nodes: Vec::new(),
            spatial_grid: HashMap::new(),
            grid_bounds: (Vec2::ZERO, Vec2::ZERO),
        });
    }
}

pub fn init_pathfinding_graph(level: &Level, mut pathfinding: ResMut<PathfindingGraph>) {
    place_nodes(&mut pathfinding, level);

    make_walkable_connections_2_way(&mut pathfinding);

    remove_duplicate_nodes(&mut pathfinding);

    make_node_ids_indices(&mut pathfinding);

    make_jumpable_connections(&mut pathfinding, level, PURSUE_AI_AGENT_RADIUS);

    make_droppable_connections(&mut pathfinding, level, PURSUE_AI_AGENT_RADIUS);

    calculate_normals(&mut pathfinding, level);

    setup_corners(&mut pathfinding);

    build_spatial_index(&mut pathfinding);
}

#[derive(Debug, Clone)]
pub enum PathfindingGraphConnectionType {
    Walkable,
    Jumpable,
    Droppable,
}

#[derive(Debug, Clone)]
pub struct PathfindingGraphConnection {
    pub node_id: usize,
    pub dist: f32,
    pub connection_type: PathfindingGraphConnectionType,
    pub effort: f32,
}

#[derive(Debug, Clone)]
pub struct PathfindingGraphNode {
    pub id: usize,
    pub position: Vec2,
    pub polygon_index: usize,
    pub line_indicies: Vec<usize>,
    pub walkable_connections: Vec<PathfindingGraphConnection>,
    pub jumpable_connections: Vec<PathfindingGraphConnection>,
    pub droppable_connections: Vec<PathfindingGraphConnection>,
    pub normal: Vec2,
    pub is_corner: bool,
    pub is_external_corner: Option<bool>,
}

#[derive(Resource)]
pub struct PathfindingGraph {
    pub nodes: Vec<PathfindingGraphNode>,
    pub spatial_grid: HashMap<(i32, i32), Vec<usize>>,
    pub grid_bounds: (Vec2, Vec2), // (min, max) for bounds checking
}

impl PathfindingGraph {
    /// Convert a world position to a grid cell coordinate
    pub fn position_to_cell(&self, pos: Vec2) -> (i32, i32) {
        let x = ((pos.x - self.grid_bounds.0.x) / SPATIAL_CELL_SIZE).floor() as i32;
        let y = ((pos.y - self.grid_bounds.0.y) / SPATIAL_CELL_SIZE).floor() as i32;
        (x, y)
    }

    /// Get node indices in cells near the given position (3x3 grid search)
    pub fn get_nearby_node_indices(&self, pos: Vec2) -> Vec<usize> {
        let (cx, cy) = self.position_to_cell(pos);
        let mut indices = Vec::new();

        // Search 3x3 grid of cells
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(cell_nodes) = self.spatial_grid.get(&(cx + dx, cy + dy)) {
                    indices.extend(cell_nodes.iter().copied());
                }
            }
        }
        indices
    }
}

pub fn place_nodes(pathfinding: &mut PathfindingGraph, level: &Level) {
    let mut outer_container_seen = false;

    // Place nodes
    for polygon_index in 0..level.polygons.len() {
        let polygon = &level.polygons[polygon_index];
        if polygon.is_container {
            outer_container_seen = !outer_container_seen;
        }

        if outer_container_seen && polygon.is_container {
            continue;
        }

        for line_index in 1..polygon.points.len() {
            let start = polygon.points[line_index - 1];
            let end = polygon.points[line_index];

            let mut start_to_end = end - start;

            let length = start_to_end.length();

            let nodes_on_line_count = (length.abs() / PATHFINDING_NODE_SPACING).ceil();
            let dist_between_nodes_on_line = length / nodes_on_line_count;

            start_to_end = start_to_end.normalize();

            if start_to_end.dot(Vec2::X) > PATHFINDING_NODE_DIRECTION_THRESHOLD {
                for j in 0..(nodes_on_line_count as i32) {
                    let node_pos = start + start_to_end * (j as f32 * dist_between_nodes_on_line);

                    let mut new_node = PathfindingGraphNode {
                        id: pathfinding.nodes.len(),
                        position: node_pos,
                        polygon_index,
                        line_indicies: vec![(line_index - 1)],
                        walkable_connections: Vec::new(),
                        jumpable_connections: Vec::new(),
                        droppable_connections: Vec::new(),
                        normal: Vec2::ZERO,
                        is_corner: false,
                        is_external_corner: None,
                    };

                    if j > 0 {
                        new_node
                            .walkable_connections
                            .push(PathfindingGraphConnection {
                                node_id: pathfinding.nodes.len() - 1,
                                dist: dist_between_nodes_on_line,
                                connection_type: PathfindingGraphConnectionType::Walkable,
                                effort: 0.0,
                            });
                    }

                    pathfinding.nodes.push(new_node);
                }
                let new_node = PathfindingGraphNode {
                    id: pathfinding.nodes.len(),
                    position: end,
                    polygon_index,
                    line_indicies: vec![(line_index - 1)],
                    walkable_connections: vec![PathfindingGraphConnection {
                        node_id: pathfinding.nodes.len() - 1,
                        dist: dist_between_nodes_on_line,
                        connection_type: PathfindingGraphConnectionType::Walkable,
                        effort: 0.0,
                    }],
                    jumpable_connections: Vec::new(),
                    droppable_connections: Vec::new(),
                    normal: Vec2::ZERO,
                    is_corner: false,
                    is_external_corner: None,
                };

                pathfinding.nodes.push(new_node);
            }
        }
    }
}

/// Makes all of the connections between nodes 2-way
pub fn make_walkable_connections_2_way(pathfinding: &mut PathfindingGraph) {
    for node_index in 0..pathfinding.nodes.len() {
        // Make a clone of the current node to appease the borrow checker
        let node = pathfinding.nodes[node_index].clone();

        // For each connection of the current node
        for connection in node.walkable_connections.iter() {
            // Add the current node to the connections of the other node
            pathfinding.nodes[connection.node_id]
                .walkable_connections
                .push(PathfindingGraphConnection {
                    node_id: node_index,
                    dist: connection.dist,
                    connection_type: PathfindingGraphConnectionType::Walkable,
                    effort: 0.0,
                });
        }
    }
}

/// Removes redundant nodes that occupy the same position
pub fn remove_duplicate_nodes(pathfinding: &mut PathfindingGraph) {
    let mut i = 0;
    while i < pathfinding.nodes.len() {
        let mut j = i + 1;
        while j < pathfinding.nodes.len() {
            if (pathfinding.nodes[i].position - pathfinding.nodes[j].position).length_squared()
                < 1.0
            {
                // Append the connections to the first node
                let mut j_connections = pathfinding.nodes[j].walkable_connections.clone();
                pathfinding.nodes[i]
                    .walkable_connections
                    .append(&mut j_connections);

                // Record the id of the nodes
                let first_node_id = pathfinding.nodes[i].id;
                let second_node_id = pathfinding.nodes[j].id;

                // Append the line indicies to the first node
                let second_node_line_index = pathfinding.nodes[j].line_indicies[0];
                pathfinding.nodes[i]
                    .line_indicies
                    .push(second_node_line_index);

                // Remove the second node
                pathfinding.nodes.remove(j);

                // Update the connections of the nodes that were connected to the second node
                for node in &mut pathfinding.nodes {
                    for connection in &mut node.walkable_connections {
                        if connection.node_id == second_node_id {
                            connection.node_id = first_node_id;
                        }
                    }
                }
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}

/// Updates the ids and connections to reflect the indices of the nodes
pub fn make_node_ids_indices(pathfinding: &mut PathfindingGraph) {
    let pathfinding_nodes_copy = pathfinding.nodes.clone();

    for node_index in 0..pathfinding.nodes.len() {
        pathfinding.nodes[node_index].id = node_index;

        for connection_index in 0..pathfinding.nodes[node_index].walkable_connections.len() {
            let connected_node = pathfinding_nodes_copy
                .iter()
                .find(|n| {
                    n.id == pathfinding.nodes[node_index].walkable_connections[connection_index]
                        .node_id
                })
                .unwrap();

            let connected_node_id = pathfinding_nodes_copy
                .iter()
                .position(|n| n.id == connected_node.id)
                .unwrap();

            pathfinding.nodes[node_index].walkable_connections[connection_index].node_id =
                connected_node_id;
        }
    }
}

pub fn make_jumpable_connections(pathfinding: &mut PathfindingGraph, level: &Level, radius: f32) {
    for i in 0..pathfinding.nodes.len() {
        let main_node = &pathfinding.nodes[i];

        let mut jumpable_connections: Vec<PathfindingGraphConnection> = Vec::new();

        'other_nodes: for j in 0..pathfinding.nodes.len() {
            // Make sure we're not comparing the same node
            if i == j {
                continue;
            }

            let other_node = &pathfinding.nodes[j];

            // Make sure the nodes are not on the same polygon
            if main_node.polygon_index == other_node.polygon_index {
                continue;
            }

            for polygon_index in 0..level.polygons.len() {
                let polygon = &level.polygons[polygon_index];

                'polygon_lines: for line_index in 1..polygon.points.len() {
                    if main_node.polygon_index == polygon_index
                        && main_node.line_indicies.contains(&(line_index - 1))
                        || other_node.polygon_index == polygon_index
                            && other_node.line_indicies.contains(&(line_index - 1))
                    {
                        continue 'polygon_lines;
                    }

                    let start = polygon.points[line_index - 1];
                    let end = polygon.points[line_index];

                    let intersection =
                        line_intersect(start, end, main_node.position, other_node.position);

                    if intersection.is_some() {
                        continue 'other_nodes;
                    }
                }
            }

            let jumpable_velocity = jumpability_check(main_node, other_node, level, radius);

            if jumpable_velocity.is_none() {
                continue 'other_nodes;
            }

            jumpable_connections.push(PathfindingGraphConnection {
                node_id: j,
                dist: (main_node.position - other_node.position).length(),
                connection_type: PathfindingGraphConnectionType::Jumpable,
                effort: jumpable_velocity.unwrap(),
            });
        }

        pathfinding.nodes[i].jumpable_connections = jumpable_connections;
    }
}

pub fn make_droppable_connections(pathfinding: &mut PathfindingGraph, level: &Level, radius: f32) {
    const DROP_EFFORT_MULTIPLIER: f32 = 0.5; // Falling is cheaper than jumping
    const MAX_HORIZONTAL_DROP_OFFSET: f32 = PATHFINDING_NODE_SPACING * 1.5; // Allow small horizontal offset (1.5x node spacing)

    for i in 0..pathfinding.nodes.len() {
        let main_node = &pathfinding.nodes[i];

        let mut droppable_connections: Vec<PathfindingGraphConnection> = Vec::new();

        'other_nodes: for j in 0..pathfinding.nodes.len() {
            // Make sure we're not comparing the same node
            if i == j {
                continue;
            }

            let other_node = &pathfinding.nodes[j];

            // Make sure the nodes are not on the same polygon
            if main_node.polygon_index == other_node.polygon_index {
                continue;
            }

            // Check that target is below source (droppable connections are one-way downward)
            if other_node.position.y >= main_node.position.y {
                continue;
            }

            // Check that target is almost directly below (limit horizontal offset)
            let horizontal_distance = (other_node.position.x - main_node.position.x).abs();
            if horizontal_distance > MAX_HORIZONTAL_DROP_OFFSET {
                continue;
            }

            // Check line-of-sight: ensure no geometry blocks the direct path
            for polygon_index in 0..level.polygons.len() {
                let polygon = &level.polygons[polygon_index];

                'polygon_lines: for line_index in 1..polygon.points.len() {
                    // Skip lines that belong to the source or target nodes
                    if main_node.polygon_index == polygon_index
                        && main_node.line_indicies.contains(&(line_index - 1))
                        || other_node.polygon_index == polygon_index
                            && other_node.line_indicies.contains(&(line_index - 1))
                    {
                        continue 'polygon_lines;
                    }

                    let start = polygon.points[line_index - 1];
                    let end = polygon.points[line_index];

                    let intersection =
                        line_intersect(start, end, main_node.position, other_node.position);

                    if intersection.is_some() {
                        continue 'other_nodes;
                    }
                }
            }

            // Check if the falling trajectory is valid
            let drop_effort = droppability_check(main_node, other_node, level, radius);

            if drop_effort.is_none() {
                continue 'other_nodes;
            }

            let drop_distance = (main_node.position - other_node.position).length();
            let effort = drop_distance * DROP_EFFORT_MULTIPLIER;

            droppable_connections.push(PathfindingGraphConnection {
                node_id: j,
                dist: drop_distance,
                connection_type: PathfindingGraphConnectionType::Droppable,
                effort,
            });
        }

        pathfinding.nodes[i].droppable_connections = droppable_connections;
    }
}

pub fn jumpability_check(
    start_graph_node: &PathfindingGraphNode,
    goal_graph_node: &PathfindingGraphNode,
    level: &Level,
    radius: f32,
) -> Option<f32> {
    let start_node = start_graph_node;
    let start_pos = start_node.position;

    let goal_node = goal_graph_node;
    let goal_pos = goal_node.position;

    let delta_p = goal_pos - start_pos;
    let acceleration = Vec2::new(0.0, -GRAVITY_STRENGTH);
    let v_max = PLATFORMER_AI_JUMP_FORCE;
    let b1 = delta_p.dot(acceleration) + v_max * v_max;
    let discriminant = b1 * b1 - acceleration.dot(acceleration) * delta_p.dot(delta_p);

    let mut jump_possible = discriminant >= 0.0;

    let t_low_energy = (4.0 * delta_p.dot(delta_p) / acceleration.dot(acceleration))
        .sqrt()
        .sqrt();
    let launch_velocity = delta_p / t_low_energy - acceleration * t_low_energy / 2.0;
    let timestep = t_low_energy / JUMPABILITY_CHECK_TIMESTEP_DIVISIONS as f32;

    if jump_possible {
        'polygon: for polygon_index in 0..level.polygons.len() {
            let polygon = &level.polygons[polygon_index];
            'line: for line_index in 1..polygon.points.len() {
                let start_node_on_line = start_node.polygon_index == polygon_index
                    && start_node.line_indicies.contains(&(line_index - 1));
                let goal_node_on_line = goal_node.polygon_index == polygon_index
                    && goal_node.line_indicies.contains(&(line_index - 1));

                if start_node_on_line || goal_node_on_line {
                    continue 'line;
                }

                let line_start = polygon.points[line_index - 1];
                let line_end = polygon.points[line_index];

                let mut prev_pos = start_pos;

                for i in 1..=JUMPABILITY_CHECK_TIMESTEP_DIVISIONS {
                    let t = timestep * i as f32;
                    let pos = start_pos + launch_velocity * t + acceleration * t * t / 2.0;

                    let line_dir = (pos - prev_pos).normalize();

                    let line_normal = Vec2::new(-line_dir.y, line_dir.x);

                    let line_beginning_offset_1 = prev_pos + line_normal * radius;
                    let line_beginning_offset_2 = prev_pos - line_normal * radius;
                    let line_end_offset_1 = pos + line_normal * radius;
                    let line_end_offset_2 = pos - line_normal * radius;

                    let offset_1_intersection = line_intersect(
                        line_beginning_offset_1,
                        line_end_offset_1,
                        line_start,
                        line_end,
                    );

                    if offset_1_intersection.is_some() {
                        jump_possible = false;
                        break 'polygon;
                    }

                    let offset_2_intersection = line_intersect(
                        line_beginning_offset_2,
                        line_end_offset_2,
                        line_start,
                        line_end,
                    );

                    if offset_2_intersection.is_some() {
                        jump_possible = false;
                        break 'polygon;
                    }

                    prev_pos = pos;
                }

                let line_dir = (goal_pos - prev_pos).normalize();

                let line_normal = Vec2::new(-line_dir.y, line_dir.x);

                let line_beginning_offset_1 = prev_pos + line_normal * radius;
                let line_beginning_offset_2 = prev_pos - line_normal * radius;
                let line_end_offset_1 = goal_pos + line_normal * radius;
                let line_end_offset_2 = goal_pos - line_normal * radius;

                let offset_1_intersection = line_intersect(
                    line_beginning_offset_1,
                    line_end_offset_1,
                    line_start,
                    line_end,
                );

                if offset_1_intersection.is_some() {
                    jump_possible = false;
                    break 'polygon;
                }

                let offset_2_intersection = line_intersect(
                    line_beginning_offset_2,
                    line_end_offset_2,
                    line_start,
                    line_end,
                );

                if offset_2_intersection.is_some() {
                    jump_possible = false;
                    break 'polygon;
                }
            }
        }
    }

    if jump_possible {
        Some(launch_velocity.length())
    } else {
        None
    }
}

pub fn droppability_check(
    start_graph_node: &PathfindingGraphNode,
    goal_graph_node: &PathfindingGraphNode,
    level: &Level,
    radius: f32,
) -> Option<f32> {
    let start_pos = start_graph_node.position;
    let goal_pos = goal_graph_node.position;

    // Ensure goal is below start (already checked in make_droppable_connections, but double-check)
    if goal_pos.y >= start_pos.y {
        return None;
    }

    // Calculate falling time: t = sqrt(2 * distance / gravity)
    let delta_y = start_pos.y - goal_pos.y;
    let delta_x = goal_pos.x - start_pos.x;
    let fall_time = (2.0 * delta_y / GRAVITY_STRENGTH).sqrt();

    // Calculate horizontal velocity needed (if any)
    let horizontal_velocity = if fall_time > 0.0 {
        delta_x / fall_time
    } else {
        0.0
    };

    // Simulate falling trajectory in discrete steps
    let timestep = fall_time / JUMPABILITY_CHECK_TIMESTEP_DIVISIONS as f32;
    let acceleration = Vec2::new(0.0, -GRAVITY_STRENGTH);
    let initial_velocity = Vec2::new(horizontal_velocity, 0.0);

    // Check for collisions along the falling path
    'polygon: for polygon_index in 0..level.polygons.len() {
        let polygon = &level.polygons[polygon_index];
        'line: for line_index in 1..polygon.points.len() {
            // Skip lines that belong to the source or target nodes
            let start_node_on_line = start_graph_node.polygon_index == polygon_index
                && start_graph_node.line_indicies.contains(&(line_index - 1));
            let goal_node_on_line = goal_graph_node.polygon_index == polygon_index
                && goal_graph_node.line_indicies.contains(&(line_index - 1));

            if start_node_on_line || goal_node_on_line {
                continue 'line;
            }

            let line_start = polygon.points[line_index - 1];
            let line_end = polygon.points[line_index];

            let mut prev_pos = start_pos;

            // Simulate trajectory in steps
            for i in 1..=JUMPABILITY_CHECK_TIMESTEP_DIVISIONS {
                let t = timestep * i as f32;
                let pos = start_pos + initial_velocity * t + acceleration * t * t / 2.0;

                // Check if we've passed the goal (shouldn't happen, but safety check)
                if pos.y < goal_pos.y {
                    break 'polygon;
                }

                let line_dir = (pos - prev_pos).normalize_or_zero();
                let line_normal = Vec2::new(-line_dir.y, line_dir.x);

                // Check collision with agent radius offset on both sides
                let line_beginning_offset_1 = prev_pos + line_normal * radius;
                let line_beginning_offset_2 = prev_pos - line_normal * radius;
                let line_end_offset_1 = pos + line_normal * radius;
                let line_end_offset_2 = pos - line_normal * radius;

                let offset_1_intersection = line_intersect(
                    line_beginning_offset_1,
                    line_end_offset_1,
                    line_start,
                    line_end,
                );

                if offset_1_intersection.is_some() {
                    return None;
                }

                let offset_2_intersection = line_intersect(
                    line_beginning_offset_2,
                    line_end_offset_2,
                    line_start,
                    line_end,
                );

                if offset_2_intersection.is_some() {
                    return None;
                }

                prev_pos = pos;
            }

            // Check final segment to goal
            let line_dir = (goal_pos - prev_pos).normalize_or_zero();
            let line_normal = Vec2::new(-line_dir.y, line_dir.x);

            let line_beginning_offset_1 = prev_pos + line_normal * radius;
            let line_beginning_offset_2 = prev_pos - line_normal * radius;
            let line_end_offset_1 = goal_pos + line_normal * radius;
            let line_end_offset_2 = goal_pos - line_normal * radius;

            let offset_1_intersection = line_intersect(
                line_beginning_offset_1,
                line_end_offset_1,
                line_start,
                line_end,
            );

            if offset_1_intersection.is_some() {
                return None;
            }

            let offset_2_intersection = line_intersect(
                line_beginning_offset_2,
                line_end_offset_2,
                line_start,
                line_end,
            );

            if offset_2_intersection.is_some() {
                return None;
            }
        }
    }

    // If we made it here, the drop is valid
    // Return the drop distance as effort (will be multiplied by DROP_EFFORT_MULTIPLIER in make_droppable_connections)
    Some((start_pos - goal_pos).length())
}

pub fn calculate_normals(pathfinding: &mut PathfindingGraph, level: &Level) {
    for node_index in 0..pathfinding.nodes.len() {
        let node = &pathfinding.nodes[node_index];

        let mut normal = Vec2::ZERO;

        for line_index in node.line_indicies.iter() {
            let line = level.polygons[node.polygon_index].points[*line_index + 1]
                - level.polygons[node.polygon_index].points[*line_index];

            let line_normal = Vec2::new(-line.y, line.x).normalize_or_zero();

            normal += line_normal;
        }

        pathfinding.nodes[node_index].normal = normal.normalize_or_zero();
    }
}

pub fn setup_corners(pathfinding: &mut PathfindingGraph) {
    for node_index in 0..pathfinding.nodes.len() {
        // let node = &mut pathfinding.nodes[node_index];

        pathfinding.nodes[node_index].is_corner =
            pathfinding.nodes[node_index].line_indicies.len() > 1;

        if pathfinding.nodes[node_index].is_corner {
            let mut line_dir = Vec2::ZERO;

            for connection in pathfinding.nodes[node_index].walkable_connections.iter() {
                let line = pathfinding.nodes[connection.node_id].position
                    - pathfinding.nodes[node_index].position;
                line_dir += line;
            }

            pathfinding.nodes[node_index].is_external_corner =
                Some(line_dir.dot(pathfinding.nodes[node_index].normal) < 0.0);
        }
    }
}

/// Build spatial index for O(1) node lookups
fn build_spatial_index(pathfinding: &mut PathfindingGraph) {
    // Calculate bounds from all nodes
    let mut min = Vec2::splat(f32::MAX);
    let mut max = Vec2::splat(f32::MIN);
    for node in &pathfinding.nodes {
        min = min.min(node.position);
        max = max.max(node.position);
    }
    pathfinding.grid_bounds = (min, max);

    // Populate spatial grid
    pathfinding.spatial_grid.clear();
    for (idx, node) in pathfinding.nodes.iter().enumerate() {
        let cell = pathfinding.position_to_cell(node.position);
        pathfinding.spatial_grid.entry(cell).or_default().push(idx);
    }

    // Debug: verify spatial index is populated
    println!(
        "Spatial index built: {} nodes in {} grid cells",
        pathfinding.nodes.len(),
        pathfinding.spatial_grid.len()
    );
}
