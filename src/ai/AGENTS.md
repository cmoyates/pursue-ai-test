# src/ai/AGENTS.md - AI System

## Overview

The AI system provides pathfinding, platformer movement, and pursue AI state machine functionality.

**Key Modules:**
- `pathfinding.rs` - Graph-based pathfinding system
- `a_star.rs` - A* pathfinding algorithm
- `platformer_ai.rs` - Platformer movement logic
- `pursue_ai/` - State machine AI (see `pursue_ai/AGENTS.md`)

---

## Pathfinding System (`pathfinding.rs`)

### PathfindingGraph Resource

```rust
#[derive(Resource)]
pub struct PathfindingGraph {
    pub nodes: Vec<PathfindingGraphNode>,
}
```

### Node Structure

```rust
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
```

### Connection Types

- **Walkable**: Adjacent nodes on same surface (effort: 0.0)
- **Jumpable**: Nodes reachable via jump (effort: jump velocity magnitude)
- **Droppable**: Nodes reachable by dropping (currently unused)

### Graph Initialization Flow

1. **`place_nodes()`** - Place nodes along polygon edges (every 20 units)
2. **`make_walkable_connections_2_way()`** - Make all walkable connections bidirectional
3. **`remove_duplicate_nodes()`** - Merge nodes at same position
4. **`make_node_ids_indices()`** - Update IDs to match array indices
5. **`make_jumpable_connections()`** - Calculate jumpable paths between nodes
6. **`calculate_normals()`** - Calculate surface normals for each node
7. **`setup_corners()`** - Mark corner nodes and external/internal corners

### Jumpability Check

`jumpability_check()` validates if a jump between two nodes is possible:

1. Calculate launch velocity for parabolic trajectory
2. Simulate trajectory in 10 steps
3. Check for collisions with level geometry
4. Return jump velocity magnitude if valid, `None` otherwise

### Key Functions

- `init_pathfinding_graph(level, pathfinding)` - Initialize graph from level
- `place_nodes(pathfinding, level)` - Generate nodes from polygons
- `make_jumpable_connections(pathfinding, level, radius)` - Calculate jumps
- `jumpability_check(start_node, goal_node, level, radius)` - Validate jump

### Usage

```rust
// Initialize in startup system
init_pathfinding_graph(&level, pathfinding);

// Access in systems
let pathfinding: Res<PathfindingGraph> = ...;
let node = &pathfinding.nodes[node_index];
```

---

## A* Pathfinding (`a_star.rs`)

### Algorithm Overview

Standard A* implementation with:
- **G-cost**: Distance from start node
- **H-cost**: Heuristic distance to goal
- **F-cost**: G + H (total estimated cost)

### Key Functions

- **`find_path(pathfinding, start_position, goal_position)`**
  - Returns `Option<Vec<PathNode>>` - Path from start to goal
  - Uses binary heap for open list
  - Handles both walkable and jumpable connections

### Node Selection

- **Start Node**: Closest node to `start_position` (prefer nodes closer to goal)
- **Goal Node**: Closest node to `goal_position`

### Path Node Structure

```rust
pub struct PathNode {
    pub id: usize,
    pub position: Vec2,
}
```

### Usage

```rust
let path = find_path(&pathfinding, agent_pos, goal_pos);
if let Some(path) = path {
    // Path found: path[0] is start, path[path.len()-1] is goal
    for node in &path {
        // Process path nodes
    }
}
```

### AStarNode Implementation

- Implements `Ord` for binary heap (lower F-cost = higher priority)
- Ties broken by H-cost (prefer nodes closer to goal)
- Tracks parent for path reconstruction

---

## Platformer AI (`platformer_ai.rs`)

### Movement System

`s_platformer_ai_movement()` handles:
1. Pathfinding to goal
2. Movement direction calculation
3. Jump velocity calculation
4. Acceleration application
5. Gravity application
6. Jump handling (ground jump, wall jump)
7. Physics update

### Path Following Strategies

```rust
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
```

### Strategy Selection Logic

1. **Falling**: `AgentToNextNodeOffset`
2. **Jumpable Connection**: 
   - If agent will cross node next frame → `AgentToNextNodeOffset`
   - Otherwise → `AgentToCurrentNodeOffset`
3. **Corner (non-jumpable)**: `AgentToNextNode`
4. **Flat Surface**: 
   - If closer to next offset → `AgentToNextNodeOffset`
   - Otherwise → `AgentToCurrentNodeOffset`

### Movement Constants

- `WANDER_MAX_SPEED: f32 = 3.0`
- `PLATFORMER_AI_JUMP_FORCE: f32 = 8.0`
- `ACCELERATION_SCALERS: (f32, f32) = (0.2, 0.4)` - (acceleration, deceleration)

### Acceleration Model

```rust
physics.acceleration = (move_dir * max_speed - physics.velocity) * scaler;
```

- **Acceleration**: When moving (`scaler = 0.2`)
- **Deceleration**: When not moving (`scaler = 0.4`)

### Jump Calculation

For jumpable connections:
```rust
let jump_time = sqrt(sqrt(4 * distance² / gravity²));
let jump_velocity = distance / jump_time - gravity * jump_time / 2.0;
```

### Gravity Application

- **Falling**: Standard gravity (`acceleration.y = -GRAVITY_STRENGTH`)
- **On Surface**: Gravity along surface normal (`normal * GRAVITY_STRENGTH`)

### Component

```rust
#[derive(Component)]
pub struct PlatformerAI {
    pub current_target_node: Option<usize>,
    pub jump_from_pos: Option<Vec2>,
    pub jump_to_pos: Option<Vec2>,
}
```

### System Ordering

```rust
// Platformer AI runs after goal point movement
.add_systems(Update, s_platformer_ai_movement.after(s_move_goal_point))
```

---

## Common Patterns

### Accessing Pathfinding Graph

```rust
// Read-only (most common)
let pathfinding: Res<PathfindingGraph> = ...;

// Mutable (for initialization only)
let mut pathfinding: ResMut<PathfindingGraph> = ...;
```

### Finding Closest Node

```rust
let mut closest_node = None;
let mut closest_dist_sq = f32::MAX;

for node in &pathfinding.nodes {
    let dist_sq = (position - node.position).length_squared();
    if dist_sq < closest_dist_sq {
        closest_dist_sq = dist_sq;
        closest_node = Some(node);
    }
}
```

### Iterating Connections

```rust
for connection in &node.walkable_connections {
    let connected_node = &pathfinding.nodes[connection.node_id];
    // Process connection
}

for connection in &node.jumpable_connections {
    if connection.effort < max_jump_effort {
        // Valid jump
    }
}
```

### Debug Rendering

```rust
if gizmos_visible.visible {
    // Draw pathfinding nodes
    for node in &pathfinding.nodes {
        gizmos.circle_2d(node.position, 5.0, Color::GREEN);
    }
    
    // Draw path
    for i in 1..path.len() {
        gizmos.line_2d(path[i-1].position, path[i].position, Color::GREEN);
    }
}
```

---

## Anti-Patterns

### ❌ Don't

- Mutate `PathfindingGraph` in Update systems (only in Startup)
- Create new nodes during gameplay (graph is static)
- Access nodes by old IDs (use array indices after `make_node_ids_indices()`)
- Hardcode jump parameters (use `PLATFORMER_AI_JUMP_FORCE`)
- Calculate paths every frame (cache when possible)

### ✅ Do

- Initialize graph once in `Startup` system
- Use `Res<PathfindingGraph>` for read access
- Use `find_path()` for pathfinding (don't reimplement A*)
- Use node offsets (position + normal * radius) for path following
- Check `is_jumpable_connection` before calculating jump velocity

---

## Performance Considerations

- **Graph Initialization**: Expensive (runs once at startup)
- **Jumpability Check**: Very expensive (called during graph init)
- **A* Pathfinding**: Moderate cost (runs every frame, consider caching)
- **Node Lookups**: Fast (direct array access by index)

### Optimization Tips

- Cache paths when goal doesn't change
- Limit pathfinding frequency (not every frame)
- Use spatial partitioning for node lookups (if graph grows large)
- Pre-calculate common paths (if applicable)

---

## Quick Reference

```bash
# Find pathfinding usage
rg -n "PathfindingGraph|find_path" src/

# Find jump calculations
rg -n "jump_velocity|jumpability" src/

# Find movement strategies
rg -n "PathFollowingStrategy" src/
```

