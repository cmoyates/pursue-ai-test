<objective>
Optimize the A* pathfinding algorithm in `src/ai/a_star.rs` for significant performance improvements.

The current implementation has several inefficiencies that cause unnecessary CPU usage during pathfinding, which runs every frame for each AI agent. This phase focuses on algorithmic fixes that will provide the foundation for subsequent optimizations.
</objective>

<context>
This is a Bevy 0.17 game with Rain World-inspired AI. The pathfinding runs in `s_platformer_ai_movement` every frame via `find_path()`.

Key files to examine:
- `src/ai/a_star.rs` - A* implementation (main focus)
- `src/ai/pathfinding.rs` - Graph structure and node definitions
- `src/ai/platformer_ai.rs` - How paths are consumed

Current issues identified in `a_star.rs`:
1. **Closed list is a Vec** - Uses `iter().any()` for O(n) lookups instead of O(1)
2. **Excessive cloning** - `AStarNode` clones entire connection vectors
3. **Path reconstruction** - Linear search through closed list, clones nodes unnecessarily
4. **Start/goal node selection** - Iterates all nodes twice with full node clones
</context>

<requirements>
Thoroughly analyze the A* implementation and fix these performance issues:

1. **Replace closed list Vec with HashSet**
   - Change `closed_list: Vec<AStarNode>` to use `HashSet<usize>` storing only node IDs
   - Store `AStarNode` data separately for path reconstruction (use a `HashMap<usize, AStarNode>` or similar)
   - This changes O(n) contains-check to O(1)

2. **Reduce AStarNode cloning**
   - The `AStarNode::new()` function concatenates and clones connection vectors
   - Consider storing a reference to the graph node index instead of copying connections
   - Avoid cloning `PathfindingGraphNode` in `get_start_node` and `get_goal_node`

3. **Optimize path reconstruction**
   - Current code uses `closed_list.iter().find()` which is O(n) per parent lookup
   - With HashMap, parent lookup becomes O(1)
   - Build path without cloning entire AStarNode structures

4. **Improve start/goal node selection**
   - `get_start_node` and `get_goal_node` clone `PathfindingGraphNode` unnecessarily
   - Return just the node index and use that to create AStarNode
   - Use `length_squared()` consistently (avoid sqrt in hot path)

5. **Consider early termination**
   - If start node == goal node, return immediately
   - Check if goal is unreachable early (disconnected graph regions)
</requirements>

<implementation>
Follow these Rust and Bevy conventions:
- Use `rustfmt` formatting
- Ensure `cargo clippy --all-targets --all-features -D warnings` passes
- Maintain the existing public API: `find_path(pathfinding, start_position, goal_position) -> Option<Vec<PathNode>>`
- Keep `PathNode` struct unchanged (it's the output contract)

Avoid over-engineering:
- Don't add generics or traits unless truly needed
- Don't restructure the entire file - focus on targeted optimizations
- Keep changes minimal and focused on the identified issues

Reference implementation pattern for closed list:
```rust
use std::collections::{BinaryHeap, HashMap, HashSet};

let mut open_list: BinaryHeap<AStarNode> = BinaryHeap::new();
let mut closed_set: HashSet<usize> = HashSet::new();
let mut came_from: HashMap<usize, (usize, Vec2)> = HashMap::new(); // node_id -> (parent_id, position)
```
</implementation>

<output>
Modify `src/ai/a_star.rs` in place with the optimizations.

After changes, verify:
- `cargo build` succeeds
- `cargo clippy --all-targets --all-features -D warnings` passes
- `cargo run` - pathfinding still works correctly (AI navigates to goal)
</output>

<verification>
Before declaring complete:
1. Run `cargo build` and fix any compilation errors
2. Run `cargo clippy --all-targets --all-features -D warnings` and fix any warnings
3. Run `cargo run` and verify the AI agent still pathfinds correctly to the goal point
4. Confirm the closed list now uses HashSet instead of Vec
5. Confirm path reconstruction uses O(1) lookups instead of linear search
</verification>

<success_criteria>
- A* closed list uses HashSet for O(1) membership checks
- Path reconstruction uses HashMap for O(1) parent lookups
- No unnecessary cloning of PathfindingGraphNode or connection vectors
- All tests pass, clippy clean, game runs correctly
- Pathfinding produces identical results to before (same paths found)
</success_criteria>

