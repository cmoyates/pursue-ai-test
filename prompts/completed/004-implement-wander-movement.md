<objective>
Implement the wander movement system for the PursueAI state machine. When the goal is disabled (`GoalEnabled.enabled == false`), the AI should autonomously wander around the level by selecting random pathfinding nodes and navigating to them using the existing platformer AI movement system.
</objective>

<context>
This is a 2D platformer AI test project using Bevy 0.17.3. The AI uses a state machine with states: Wander, Pursue, Search, Attack.

Key existing systems:
- `PlatformerAI` component handles pathfinding and movement via `s_platformer_ai_movement()`
- `PursueAI` component manages state machine via `s_pursue_ai_update()`
- `GoalEnabled` resource toggles whether AI follows the goal point or wanders
- `PathfindingGraph` contains nodes the AI can navigate between
- A* pathfinding already works via `find_path()`

Current state:
- `wander.rs` has skeleton code that picks random nodes but doesn't integrate with movement
- `s_platformer_ai_movement()` already checks `goal_enabled.enabled` but defaults to `Vec2::ZERO` when disabled
- The wander state should reuse the existing movement infrastructure, NOT duplicate it

@src/ai/pursue_ai/wander.rs - Current wander implementation (skeleton)
@src/ai/pursue_ai/mod.rs - PursueAI state machine coordinator
@src/ai/platformer_ai.rs - Movement system that handles pathfinding/jumping
@src/main.rs - GoalEnabled resource, entity spawning
</context>

<requirements>
1. **Modify wander.rs to track wander state**:
   - Add a `current_wander_goal: Option<usize>` field to track the target node ID
   - Store this on `PursueAI` component (not a separate struct)
   - When no goal is set, pick a random distant node using existing `get_random_goal_node()`
   - When goal is reached (distance threshold), pick a new random goal

2. **Integrate wander with platformer AI movement**:
   - `s_platformer_ai_movement()` should get its goal from wander state when `goal_enabled.enabled == false`
   - The wander goal should be the position of the selected random node
   - Reuse all existing pathfinding/movement/jumping logic

3. **Add goal reached detection**:
   - Use `NODE_REACHED_THRESHOLD_SQ` (64.0) or similar threshold from platformer_ai.rs
   - When agent reaches wander goal, clear it so a new one is selected next frame

4. **Update PursueAI component**:
   - Add `current_wander_goal: Option<usize>` to store target node ID
   - Initialize to `None` when spawning entity

5. **Coordinate system ordering**:
   - Wander update must run BEFORE platformer AI movement
   - Wander sets the goal, platformer AI pathfinds and moves toward it
</requirements>

<implementation>
Files to modify:
- `./src/ai/pursue_ai/mod.rs` - Add `current_wander_goal` field to PursueAI
- `./src/ai/pursue_ai/wander.rs` - Implement goal selection and reached detection
- `./src/ai/platformer_ai.rs` - Read wander goal when goal is disabled
- `./src/main.rs` - Initialize new field on entity spawn

Key patterns to follow:
- Use `Res<GoalEnabled>` to check if wandering should be active
- Use `pathfinding.nodes[node_id].position` to get wander goal position
- The existing `get_random_goal_node()` returns furthest of 3 samples (good for exploration)
- Don't duplicate movement logic - let `s_platformer_ai_movement()` handle all physics

System ordering should be:
```rust
s_pursue_ai_update (sets wander goal)
  → s_platformer_ai_movement (pathfinds to goal, applies physics)
    → s_collision (resolves collisions)
```

Avoid:
- Creating new movement functions in wander.rs (movement is already in platformer_ai.rs)
- Adding new systems when existing ones can be extended
- Tight coupling between wander and platformer_ai (use shared data through components)
</implementation>

<verification>
After implementation:
1. Run `cargo clippy --all-targets --all-features -D warnings` - should pass with no errors
2. Run `cargo run` and test:
   - Press Space to toggle goal disabled
   - When goal is disabled (grey), AI should wander to random nodes
   - When goal is enabled (green), AI should pursue the goal point
   - AI should pick new random destinations when reaching each wander goal
3. Press G to toggle gizmos and verify pathfinding lines show path to wander goals
</verification>

<success_criteria>
- AI autonomously wanders when goal is disabled (Space key toggles)
- Wander movement uses existing platformer AI pathfinding and physics
- AI picks new random distant nodes upon reaching each wander destination
- Goal toggle works in both directions (wander ↔ pursue)
- No clippy warnings or errors
- Movement feels smooth with proper jumping and navigation
</success_criteria>

