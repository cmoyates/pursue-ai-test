<objective>
Add spatial partitioning for O(1) node lookups instead of O(n) iteration through all nodes.

Currently, finding the closest node to a position requires iterating all nodes in the graph. For larger levels with many nodes, this becomes a bottleneck. Spatial partitioning (grid-based or quadtree) enables O(1) average-case lookups.
</objective>

<context>
This is a Bevy 0.17 platformer game. The pathfinding graph has nodes placed every 20 units along polygon edges.

Key files:
- `src/ai/a_star.rs` - `get_start_node()` and `get_goal_node()` iterate all nodes
- `src/ai/pathfinding.rs` - `PathfindingGraph` resource and node structure
- `src/level.rs` - Level size and grid information

Current O(n) lookups in `a_star.rs`:
```rust
fn get_start_node(...) -> AStarNode {
    for node in pathfinding.nodes.iter() {  // O(n)
        let distance = (start_position - node.position).length_squared();
        // ...
    }
}

fn get_goal_node(...) -> Option<PathfindingGraphNode> {
    for node_index in 0..pathfinding.nodes.len() {  // O(n)
        let distance = (goal_position - node.position).length_squared();
        // ...
    }
}
```

The level has a `grid_size` (default 32.0) and `size` (dimensions in grid cells).
</context>

<requirements>
Implement grid-based spatial partitioning:

1. **Add spatial index to PathfindingGraph**
   - Add a 2D grid that maps cells to lists of node indices
   - Cell size should be ~2x the node spacing (e.g., 40-50 units)
   - Structure: `spatial_grid: Vec<Vec<Vec<usize>>>` or `HashMap<(i32, i32), Vec<usize>>`

2. **Build spatial index during graph initialization**
   - In `init_pathfinding_graph()`, after nodes are placed, populate the spatial grid
   - For each node, calculate its grid cell and add its index

3. **Create lookup function**
   - `fn get_nodes_near_position(graph: &PathfindingGraph, pos: Vec2) -> &[usize]`
   - Returns node indices in the cell containing `pos` and adjacent cells (3x3 search)
   - Falls back to all nodes if position is outside grid bounds

4. **Update get_start_node and get_goal_node**
   - Use spatial lookup to get candidate nodes
   - Only iterate nearby candidates instead of all nodes
   - Keep existing tie-breaking logic

5. **Handle edge cases**
   - Positions outside level bounds → search adjacent cells or fallback
   - Empty cells → expand search to neighbors
   - Ensure correctness: spatial lookup must not miss the actual closest node
</requirements>

<implementation>
Add to `pathfinding.rs`:

```rust
const SPATIAL_CELL_SIZE: f32 = 50.0; // ~2.5x node spacing

#[derive(Resource)]
pub struct PathfindingGraph {
    pub nodes: Vec<PathfindingGraphNode>,
    // New spatial index:
    pub spatial_grid: HashMap<(i32, i32), Vec<usize>>,
    pub grid_bounds: (Vec2, Vec2), // (min, max) for bounds checking
}

impl PathfindingGraph {
    pub fn position_to_cell(&self, pos: Vec2) -> (i32, i32) {
        let x = ((pos.x - self.grid_bounds.0.x) / SPATIAL_CELL_SIZE).floor() as i32;
        let y = ((pos.y - self.grid_bounds.0.y) / SPATIAL_CELL_SIZE).floor() as i32;
        (x, y)
    }

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
```

Add spatial index building in `init_pathfinding_graph()`:
```rust
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
}
```

Update `a_star.rs` to use spatial lookups:
```rust
fn get_start_node(pathfinding: &PathfindingGraph, start_position: Vec2, goal_position: Vec2) -> Option<usize> {
    let nearby = pathfinding.get_nearby_node_indices(start_position);
    
    // If no nearby nodes, fall back to all nodes
    let candidates: &[usize] = if nearby.is_empty() {
        // Fallback: create indices 0..len
        &(0..pathfinding.nodes.len()).collect::<Vec<_>>()
    } else {
        &nearby
    };
    
    // Find closest among candidates
    // ... existing logic but only over candidates
}
```

Don't over-engineer:
- Use HashMap for sparse grid (not all cells have nodes)
- Simple 3x3 cell search is sufficient
- Don't implement quadtree (overkill for current level sizes)
</implementation>

<output>
Modify these files:
- `src/ai/pathfinding.rs` - Add spatial index to PathfindingGraph
- `src/ai/a_star.rs` - Use spatial lookups for start/goal node finding

After changes:
- Node lookups use spatial partitioning
- Fallback to full search if needed
- Same paths produced as before
</output>

<verification>
Before declaring complete:
1. Run `cargo build` and fix any errors
2. Run `cargo clippy --all-targets --all-features -D warnings`
3. Run `cargo run` and verify:
   - AI still navigates correctly
   - No visible behavior changes
   - Paths found are identical to before
4. Verify spatial index is populated:
   - Add debug print in init to show grid cell count
   - Confirm cells contain expected node counts
5. Test edge positions:
   - Goal near level boundaries
   - Start position far from any node
</verification>

<success_criteria>
- PathfindingGraph has spatial_grid field
- Spatial index built during init_pathfinding_graph()
- get_start_node/get_goal_node use spatial lookup first
- Fallback to full search works for edge cases
- No regression in pathfinding behavior
- Performance improved for node lookups (verify with profiler if desired)
</success_criteria>

