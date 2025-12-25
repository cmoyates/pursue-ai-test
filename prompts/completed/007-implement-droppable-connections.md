<objective>
Implement droppable connections in the pathfinding algorithm to allow AI agents to navigate by dropping down from platforms.

Droppable connections enable one-way traversal where an agent can fall from a higher platform to a lower one. This expands the AI's navigation options beyond walking and jumping.
</objective>

<context>
This is a Rust/Bevy 0.17.3 2D platformer AI project with an existing pathfinding system.

**Existing Infrastructure:**
- `PathfindingGraphConnectionType::Droppable` enum variant already exists
- `PathfindingGraphNode.droppable_connections` field already exists (but is always empty)
- `make_jumpable_connections()` provides a pattern for creating non-walkable connections
- A* pathfinding in `a_star.rs` has a TODO to integrate droppable connections (line 114)

**Key Files:**
@src/ai/pathfinding.rs - Graph generation and connection logic
@src/ai/a_star.rs - A* pathfinding algorithm
@src/ai/platformer_ai.rs - AI movement system

**Key Constants:**
- `PURSUE_AI_AGENT_RADIUS: f32 = 8.0` - Agent collision radius
- `GRAVITY_STRENGTH: f32 = 0.5` - Gravity acceleration
- `JUMPABILITY_CHECK_TIMESTEP_DIVISIONS: i32 = 10` - Trajectory simulation steps
</context>

<requirements>
1. **Create `make_droppable_connections()` function** in `pathfinding.rs`:
   - Identify potential drop connections between nodes on different polygons
   - A node can drop to another if:
     - The target node is below the source node (target.y < source.y)
     - There is no line-of-sight obstruction between the nodes
     - The falling trajectory doesn't collide with level geometry
   - Droppable connections are ONE-WAY (source → target only)
   - Set `effort` based on drop distance (falling is cheaper than jumping)

2. **Create `droppability_check()` function** in `pathfinding.rs`:
   - Similar to `jumpability_check()` but simpler (no launch velocity needed)
   - Simulate a falling trajectory from source to target
   - Account for agent radius when checking collisions
   - Return `Option<f32>` with effort value if droppable, `None` otherwise

3. **Integrate into graph initialization** in `init_pathfinding_graph()`:
   - Call `make_droppable_connections()` after `make_jumpable_connections()`

4. **Integrate into A* pathfinding** in `a_star.rs`:
   - Include `droppable_connections` in the connection iteration (line 86-90)
   - Handle effort calculation appropriately (falling should be cheaper than jumping)

5. **Update AI movement** in `platformer_ai.rs` (if needed):
   - Handle droppable connection traversal (may just be "walk off edge" behavior)
   - The agent should recognize when it's on a droppable path and allow falling
</requirements>

<implementation>
Follow the pattern established by `make_jumpable_connections()` and `jumpability_check()`:

```rust
// Example structure for droppability check
pub fn droppability_check(
    start_graph_node: &PathfindingGraphNode,
    goal_graph_node: &PathfindingGraphNode,
    level: &Level,
    radius: f32,
) -> Option<f32> {
    // 1. Check goal is below start
    // 2. Simulate falling trajectory
    // 3. Check for collisions with level geometry
    // 4. Return effort (e.g., drop_distance * some_factor) if valid
}
```

**Effort Calculation:**
- Droppable effort should be lower than jumpable effort (falling is easier)
- Consider: `effort = drop_distance * DROP_EFFORT_MULTIPLIER` where multiplier < 1.0

**Collision Checking:**
- Use the same line intersection approach as `jumpability_check()`
- Simulate falling path in discrete steps
- Account for agent radius on both sides of the trajectory

**Edge Cases to Handle:**
- Don't create droppable connections to nodes directly below on the same surface
- Don't create connections that would land the agent inside geometry
- Consider maximum drop height limit if needed for gameplay
</implementation>

<constraints>
- Follow existing code patterns in `pathfinding.rs` and `a_star.rs`
- Maintain system ordering: droppable connections created during graph init (Startup)
- Use `Res<PathfindingGraph>` for read access in Update systems
- Run `cargo fmt` and `cargo clippy --all-targets --all-features -D warnings`
</constraints>

<output>
Modify these files:
- `./src/ai/pathfinding.rs` - Add `make_droppable_connections()` and `droppability_check()`
- `./src/ai/a_star.rs` - Include droppable connections in pathfinding
- `./src/ai/platformer_ai.rs` - Handle droppable traversal (if needed)
</output>

<verification>
Before declaring complete:
1. Run `cargo build` - should compile without errors
2. Run `cargo clippy --all-targets --all-features -D warnings` - no warnings
3. Run `cargo run` and verify:
   - AI can find paths that include dropping down
   - AI successfully drops from platforms to lower areas
   - No pathfinding regressions (existing jump/walk behavior still works)
4. Toggle gizmos (G key) and verify droppable connections are being created
</verification>

<success_criteria>
- `make_droppable_connections()` populates `droppable_connections` on graph nodes
- A* pathfinding considers droppable connections when finding paths
- AI agent successfully navigates paths that require dropping
- Droppable paths are preferred over longer walking/jumping alternatives when appropriate
- Code passes clippy with no warnings
</success_criteria>

