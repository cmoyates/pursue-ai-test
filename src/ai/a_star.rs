use std::{cmp::Ordering, collections::{BinaryHeap, HashMap, HashSet}};

use bevy::math::Vec2;

use super::pathfinding::{PathfindingGraph, PathfindingGraphConnection, PathfindingGraphNode};

pub fn find_path(
    pathfinding: &PathfindingGraph,
    start_position: Vec2,
    goal_position: Vec2,
) -> Option<Vec<PathNode>> {
    let goal_node_id = get_goal_node_id(pathfinding, goal_position)?;
    let start_node_id = get_start_node_id(pathfinding, start_position, goal_position)?;

    // Early termination: if start == goal, return empty path
    if start_node_id == goal_node_id {
        return Some(vec![]);
    }

    let mut open_list: BinaryHeap<AStarNode> = BinaryHeap::new();
    let mut closed_set: HashSet<usize> = HashSet::new();
    let mut came_from: HashMap<usize, (usize, Vec2)> = HashMap::new(); // node_id -> (parent_id, position)

    // Get the start node
    let start_graph_node = &pathfinding.nodes[start_node_id];
    let mut start_node = AStarNode::new(start_graph_node);
    start_node.h_cost = (goal_position - start_node.position).length();

    // Add the start node to the open list
    open_list.push(start_node);

    loop {
        // If the open list is empty, there is no path
        if open_list.is_empty() {
            return None;
        }

        // Get the node with the lowest f-cost
        let current_node = open_list.pop().unwrap();

        // If the node is in the closed set, skip it
        if closed_set.contains(&current_node.id) {
            continue;
        }

        // Store parent information for path reconstruction (before checking if goal)
        // Store: node_id -> (parent_id, node_position) so we can reconstruct the path
        if let Some(parent_id) = current_node.parent {
            came_from.insert(current_node.id, (parent_id, current_node.position));
        }

        // If the current node is the goal, reconstruct the path
        if current_node.id == goal_node_id {
            let mut path: Vec<PathNode> = vec![];

            let mut current_id = current_node.id;
            while let Some((parent_id, _)) = came_from.get(&current_id) {
                // Get the parent's position from the graph
                let parent_position = pathfinding.nodes[*parent_id].position;
                path.push(PathNode::new(*parent_id, parent_position));
                current_id = *parent_id;
            }

            path.reverse();

            return Some(path);
        }

        // Add the current node to the closed set
        closed_set.insert(current_node.id);

        // For each connection of the current node
        let current_graph_node = &pathfinding.nodes[current_node.id];
        for connection in current_graph_node.walkable_connections
            .iter()
            .chain(current_graph_node.jumpable_connections.iter())
        {
            let connected_node_id = connection.node_id;

            // Skip if already in closed set
            if closed_set.contains(&connected_node_id) {
                continue;
            }

            let connected_graph_node = &pathfinding.nodes[connected_node_id];
            let mut new_node = AStarNode::new(connected_graph_node);

            // Set the g-cost to the distance to the start node
            new_node.g_cost = connection.dist + current_node.g_cost;

            // Set the h-cost to the distance to the goal
            new_node.h_cost = (goal_position - new_node.position).length();

            // Set the parent of the new node
            new_node.parent = Some(current_node.id);

            open_list.push(new_node);
        }
    }
}

fn get_start_node_id(
    pathfinding: &PathfindingGraph,
    start_position: Vec2,
    goal_position: Vec2,
) -> Option<usize> {
    let mut start_node_id: Option<usize> = None;
    let mut start_graph_node_distance = f32::MAX;

    for (node_index, node) in pathfinding.nodes.iter().enumerate() {
        let distance = (start_position - node.position).length_squared();

        if distance > start_graph_node_distance {
            continue;
        }

        if distance == start_graph_node_distance {
            if let Some(existing_id) = start_node_id {
                let existing_node = &pathfinding.nodes[existing_id];
                let existing_node_to_goal = (goal_position - existing_node.position).length_squared();
                let current_node_to_goal = (goal_position - node.position).length_squared();

                if current_node_to_goal > existing_node_to_goal {
                    continue;
                }
            }
        }

        start_graph_node_distance = distance;
        start_node_id = Some(node_index);
    }

    start_node_id
}

fn get_goal_node_id(
    pathfinding: &PathfindingGraph,
    goal_position: Vec2,
) -> Option<usize> {
    let mut goal_node_id: Option<usize> = None;
    let mut closest_distance = f32::MAX;
    for (node_index, node) in pathfinding.nodes.iter().enumerate() {
        let distance = (goal_position - node.position).length_squared();

        if distance < closest_distance {
            closest_distance = distance;
            goal_node_id = Some(node_index);
        }
    }

    goal_node_id
}

#[derive(Clone, Debug)]
pub struct AStarNode {
    pub position: Vec2,
    pub id: usize,
    #[allow(dead_code)]
    pub connections: Vec<PathfindingGraphConnection>,
    pub g_cost: f32,
    pub h_cost: f32,
    pub parent: Option<usize>,
    #[allow(dead_code)]
    pub is_corner: bool,
    #[allow(dead_code)]
    pub is_external_corner: Option<bool>,
}

impl AStarNode {
    pub fn new(graph_node: &PathfindingGraphNode) -> AStarNode {
        let connections = [
            graph_node.walkable_connections.as_slice(),
            graph_node.jumpable_connections.as_slice(),
        ]
        .concat();

        AStarNode {
            position: graph_node.position,
            id: graph_node.id,
            connections,
            g_cost: 0.0,
            h_cost: 0.0,
            parent: None,
            is_corner: graph_node.is_corner,
            is_external_corner: graph_node.is_external_corner,
        }
    }

    pub fn get_f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_f_cost = self.get_f_cost();
        let other_f_cost = other.get_f_cost();

        match self_f_cost.partial_cmp(&other_f_cost) {
            Some(Ordering::Equal) => self
                .h_cost
                .partial_cmp(&other.h_cost)
                .unwrap_or(Ordering::Equal),
            Some(order) => order.reverse(),
            None => Ordering::Equal,
        }
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub struct PathNode {
    pub id: usize,
    pub position: Vec2,
}

impl PathNode {
    pub fn new(id: usize, position: Vec2) -> PathNode {
        PathNode { id, position }
    }
}
