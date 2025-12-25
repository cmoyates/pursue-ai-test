<objective>
Add path caching to avoid recalculating paths every frame when the goal hasn't changed.

Currently, `find_path()` is called every frame for each AI agent, even when the goal position hasn't moved. This causes unnecessary CPU usage. Adding intelligent caching will dramatically reduce pathfinding overhead in typical gameplay.
</objective>

<context>
This is a Bevy 0.17 game with platformer AI that pathfinds to a goal point. The pathfinding is called in `s_platformer_ai_movement` system.

Key files:
- `src/ai/platformer_ai.rs` - Calls `find_path()` in `get_move_inputs()`
- `src/ai/a_star.rs` - The A* algorithm (optimized in previous phase)
- `src/ai/pathfinding.rs` - Graph structure

The previous phase optimized the A* algorithm internals. This phase optimizes *when* we call it.

Current behavior in `platformer_ai.rs`:
```rust
let path = find_path(pathfinding, agent_position, goal_position);
```
This is called every frame regardless of whether goal_position changed.
</context>

<requirements>
Implement intelligent path caching with the following features:

1. **Store cached path in PlatformerAI component**
   - Add fields to `PlatformerAI` for: cached path, last goal position, path validity
   - Structure: `cached_path: Option<Vec<PathNode>>`, `last_goal_position: Option<Vec2>`

2. **Only recalculate when needed**
   - Recalculate if goal moved beyond a threshold (e.g., > 5.0 units)
   - Recalculate if agent deviated significantly from path
   - Recalculate if current path node becomes invalid (e.g., agent passed it)

3. **Path invalidation detection**
   - Track current node index in path
   - Advance to next node when agent gets close enough
   - Invalidate if agent moves too far from expected path

4. **Fallback to recalculation**
   - If cached path is `None`, always recalculate
   - If path is empty or exhausted, recalculate
   - Add a maximum cache age or frame count (optional safety net)

5. **Handle goal changes gracefully**
   - When goal moves, invalidate cache immediately
   - Use squared distance for threshold checks (avoid sqrt)
</requirements>

<implementation>
Modify `PlatformerAI` component in `src/ai/platformer_ai.rs`:

```rust
#[derive(Component)]
pub struct PlatformerAI {
    pub current_target_node: Option<usize>,
    pub jump_from_pos: Option<Vec2>,
    pub jump_to_pos: Option<Vec2>,
    // New caching fields:
    pub cached_path: Option<Vec<PathNode>>,
    pub last_goal_position: Option<Vec2>,
    pub current_path_index: usize,
}
```

Caching constants to add:
```rust
const GOAL_CHANGE_THRESHOLD_SQ: f32 = 25.0; // 5.0 squared
const PATH_DEVIATION_THRESHOLD_SQ: f32 = 100.0; // 10.0 squared
const NODE_REACHED_THRESHOLD_SQ: f32 = 64.0; // 8.0 squared (agent radius squared)
```

Modify `get_move_inputs` (or create helper) to:
1. Check if cached path is still valid
2. Return cached path if valid
3. Recalculate and cache if invalid

Don't over-engineer:
- Keep it simple - just cache the path and last goal
- Don't add complex prediction or multi-frame smoothing
- Don't add path smoothing or post-processing (save for later)
</implementation>

<output>
Modify `src/ai/platformer_ai.rs` with path caching logic.

Ensure the `PlatformerAI` component is initialized correctly in `src/main.rs` where entities are spawned (add default values for new fields).

After changes:
- `cargo build` succeeds
- `cargo clippy --all-targets --all-features -D warnings` passes
- AI still navigates correctly but with fewer path calculations
</output>

<verification>
Before declaring complete:
1. Run `cargo build` and fix compilation errors
2. Run `cargo clippy --all-targets --all-features -D warnings`
3. Run `cargo run` and verify:
   - AI still pathfinds correctly to goal
   - AI updates path when goal moves significantly
   - AI continues following cached path when goal is stationary
4. Optional: Add debug logging to confirm path recalculation frequency reduced
</verification>

<success_criteria>
- PlatformerAI component has cached_path and related fields
- Path is only recalculated when goal moves beyond threshold
- Path index advances as agent progresses along path
- Game runs correctly with caching enabled
- No regressions in AI navigation behavior
</success_criteria>

