<objective>
Fix pathfinding logic issues and improve path quality through better heuristics and cost calculation.

The current A* implementation has logic issues affecting path quality: the goal node is missing from reconstructed paths, effort costs aren't factored into g_cost, and the heuristic doesn't account for platformer movement characteristics.
</objective>

<context>
This is a Bevy 0.17 platformer with graph-based pathfinding. The AI needs to walk and jump between nodes.

Key files:
- `src/ai/a_star.rs` - Path reconstruction and cost calculation
- `src/ai/pathfinding.rs` - Connection effort values
- `src/ai/platformer_ai.rs` - How paths are consumed

Current issues:

1. **Goal node missing from path**: Path reconstruction builds from goal backwards but doesn't include the goal itself:
```rust
// Current (missing goal):
while let Some(parent_id) = current_node.parent {
    path.push(PathNode::new(parent_id, parent_node.position));
    // ... never pushes the goal node itself
}
```

2. **Effort not used in g_cost**: Connections have `effort` field but it's not used:
```rust
// In pathfinding.rs - walkable effort is 0.0, jumpable is velocity magnitude
// In a_star.rs - only distance is used:
new_node.g_cost = connection.dist + current_node.g_cost;
// Should incorporate: connection.effort
```

3. **Heuristic doesn't account for jumps**: Pure Euclidean distance underestimates cost of vertical movement (jumps are expensive).

4. **Droppable connections unused**: `droppable_connections` exists but is never populated or considered in A*.
</context>

<requirements>
Fix these logic issues:

1. **Fix path reconstruction to include goal node**
   - The returned path should include the goal node as the final element
   - Path should be: [start_adjacent, ..., goal_adjacent, goal]
   - Verify path[path.len()-1] is the goal node

2. **Integrate effort into g_cost**
   - Modify g_cost calculation: `g_cost = parent.g_cost + distance + effort_weight * effort`
   - Use a tunable weight for effort (e.g., 0.5 or 1.0)
   - This makes jumpable connections more expensive, preferring walking when possible

3. **Improve heuristic for platformer movement**
   - Current: `(goal_position - node.position).length()`
   - Better: Weight vertical distance more heavily since jumps are costly
   - Suggested: `h = sqrt(dx² + (k * dy)²)` where k > 1 for upward movement
   - Or: `h = horizontal_dist + vertical_penalty * max(0, dy)` for upward bias

4. **Consider droppable connections (optional)**
   - If droppable connections are meant to be used, include them in A* neighbor iteration
   - If not intended for use yet, add a TODO comment explaining future purpose

5. **Tie-breaking improvements**
   - When f_cost is equal, prefer nodes closer to goal (current behavior)
   - When f_cost AND h_cost are equal, prefer lower g_cost (more direct paths)
</requirements>

<implementation>
Constants to add in `a_star.rs`:
```rust
const EFFORT_WEIGHT: f32 = 1.0; // Weight for jump effort in g_cost
const VERTICAL_HEURISTIC_WEIGHT: f32 = 1.5; // Penalize upward movement
```

Fix path reconstruction:
```rust
// After finding goal, build path including goal
let mut path: Vec<PathNode> = vec![];

// First, add the goal node
path.push(PathNode::new(current_node.id, current_node.position));

// Then trace back through parents
let mut trace_id = current_node.parent;
while let Some(parent_id) = trace_id {
    let parent_data = came_from.get(&parent_id).unwrap();
    path.push(PathNode::new(parent_id, parent_data.position));
    trace_id = parent_data.parent;
}

path.reverse();
```

Improved g_cost:
```rust
new_node.g_cost = current_node.g_cost + connection.dist + EFFORT_WEIGHT * connection.effort;
```

Improved heuristic:
```rust
fn calculate_heuristic(from: Vec2, to: Vec2) -> f32 {
    let dx = (to.x - from.x).abs();
    let dy = to.y - from.y; // Signed: positive = upward
    let vertical_cost = if dy > 0.0 {
        dy * VERTICAL_HEURISTIC_WEIGHT
    } else {
        dy.abs() // Falling is cheaper
    };
    (dx * dx + vertical_cost * vertical_cost).sqrt()
}
```

Follow project conventions:
- Run `cargo fmt`
- Ensure clippy passes
- Test that paths still work correctly
</implementation>

<output>
Modify `src/ai/a_star.rs` with the logic fixes.

After changes:
- Path reconstruction includes goal node
- g_cost incorporates effort
- Heuristic accounts for vertical movement cost
</output>

<verification>
Before declaring complete:
1. Run `cargo build` and fix any errors
2. Run `cargo clippy --all-targets --all-features -D warnings`
3. Run `cargo run` and verify:
   - AI still navigates to goal correctly
   - Paths include the goal node (debug by enabling gizmos with G key)
   - AI prefers walking over jumping when both are viable
4. Test edge cases:
   - Goal on same platform (should walk, not jump elsewhere)
   - Goal above agent (should find jumping path)
   - Goal below agent (should find dropping/walking path)
</verification>

<success_criteria>
- Path reconstruction includes goal node as final element
- g_cost uses effort weighting for jump vs walk preference
- Heuristic penalizes upward movement appropriately
- AI navigation behavior improved (fewer unnecessary jumps)
- All existing functionality preserved
</success_criteria>

