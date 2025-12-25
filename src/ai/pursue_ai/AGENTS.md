# src/ai/pursue_ai/AGENTS.md - Pursue AI State Machine

## Overview

The Pursue AI system implements a state machine for AI agents that hunt the player, inspired by Rain World lizards.

**State Machine States:**
- `Wander` - Random exploration (implemented)
- `Pursue` - Chase player (TODO)
- `Search` - Look for lost player (TODO)
- `Attack` - Attack player (TODO)

**Key Files:**
- `mod.rs` - State machine coordinator and plugin
- `movement.rs` - Movement utility functions
- `wander.rs` - Wander state implementation

---

## State Machine Architecture (`mod.rs`)

### State Enum

```rust
pub enum PursueAIState {
    Wander,
    Pursue,
    Search,
    Attack,
}
```

### Component

```rust
#[derive(Component)]
pub struct PursueAI {
    pub state: PursueAIState,
}
```

### State Update Pattern

```rust
pub fn s_pursue_ai_update(
    mut platformer_ai_query: Query<(&mut Transform, &mut Physics, &mut PursueAI)>,
    mut pathfinding: ResMut<PathfindingGraph>,
) {
    for (mut transform, mut physics, mut pursue_ai) in platformer_ai_query.iter_mut() {
        let next_state: Option<PursueAIState> = match pursue_ai.state {
            PursueAIState::Wander => wander::wander_update(...),
            PursueAIState::Pursue => { /* TODO */ None },
            PursueAIState::Search => { /* TODO */ None },
            PursueAIState::Attack => { /* TODO */ None },
        };

        if let Some(new_state) = next_state {
            pursue_ai.state = new_state;
        }
    }
}
```

### State Transition Pattern

State update functions return `Option<PursueAIState>`:
- `Some(new_state)` - Transition to new state
- `None` - Remain in current state

### Plugin Setup

```rust
pub struct PursueAIPlugin;

impl Plugin for PursueAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_pursue_ai_update.after(s_move_goal_point));
    }
}
```

### System Ordering

```rust
// Pursue AI runs after goal point movement (for pathfinding to goal)
.add_systems(Update, s_pursue_ai_update.after(s_move_goal_point))
```

### Constants

- `PURSUE_AI_AGENT_RADIUS: f32 = 8.0` - Agent collision radius

---

## Wander State (`wander.rs`)

### Implementation Status

**Current**: Basic structure, random goal selection  
**TODO**: Pathfinding to goal, goal reached detection, new goal selection

### Functions

- **`wander_update(transform, physics, pursue_ai, pathfinding)`**
  - Main wander state update
  - Currently calls `wander_movement()`
  - Returns `None` (no state transitions yet)

- **`wander_movement(transform, pathfinding)`**
  - Handles wander movement logic
  - Currently selects random goal but doesn't pathfind

- **`get_random_goal_node(agent_position, pathfinding)`**
  - Samples 3 random nodes
  - Returns furthest node from agent (exploration bias)
  - Uses `rand::rng().random_range(0..node_count)`

### Wander Constants

```rust
pub const WANDER_MAX_SPEED: f32 = 3.0;
```

### Random Goal Selection Algorithm

```rust
let sample_count = 3;
let mut furthest_node = None;
let mut furthest_dist_sq = f32::MAX;

for _ in 0..sample_count {
    let random_node = &pathfinding.nodes[random_index];
    let dist_sq = (agent_position - random_node.position).length_squared();
    
    if dist_sq < furthest_dist_sq {
        furthest_dist_sq = dist_sq;
        furthest_node = Some(random_node.clone());
    }
}
```

**Note**: Current implementation selects closest of 3 samples (bug: should be furthest)

### TODO: Complete Wander Implementation

1. Store current goal node in `PlatformerAI.current_target_node`
2. Use `find_path()` to pathfind to goal
3. Use `get_move_inputs()` from `platformer_ai.rs` for movement
4. Detect when goal is reached (distance threshold)
5. Select new random goal when reached

---

## Movement Utilities (`movement.rs`)

### Helper Functions

These functions are currently marked `#[allow(dead_code)]` and are duplicates of functions in `platformer_ai.rs`. They may be used for state-specific movement logic.

- **`apply_gravity_toward_normal(physics, falling)`**
  - Apply gravity along surface normal or standard gravity

- **`update_physics_and_transform(physics, transform)`**
  - Update velocity, position, and previous position

- **`apply_movement_acceleration(physics, move_dir, falling, no_move_dir, max_speed, acceleration_scalers)`**
  - Apply acceleration based on movement direction

- **`handle_jumping(physics, falling, jump_velocity)`**
  - Handle ground jumps and wall jumps

### Usage Pattern

```rust
// In state update function
apply_movement_acceleration(
    &mut physics,
    &move_dir,
    falling,
    no_move_dir,
    WANDER_MAX_SPEED,
    ACCELERATION_SCALERS,
);

apply_gravity_toward_normal(&mut physics, falling);

if jump_velocity.length_squared() > 0.0 {
    handle_jumping(&mut physics, falling, jump_velocity);
}

update_physics_and_transform(&mut physics, &mut transform);
```

---

## State Transition Design

### Planned Transitions

Based on README.md state machine:

1. **Wander → Pursue**
   - Condition: AI sees player (line of sight check)
   - TODO: Implement vision system

2. **Pursue → Search**
   - Condition: AI loses track of player
   - TODO: Implement tracking timeout

3. **Pursue → Attack**
   - Condition: AI gets within attack range
   - TODO: Implement attack range check

4. **Search → Wander**
   - Condition: AI gives up searching
   - TODO: Implement search timeout

5. **Attack → Pursue**
   - Condition: Player survives attack
   - TODO: Implement attack system

6. **Attack → Wander**
   - Condition: Player doesn't survive attack
   - TODO: Implement player death system

### State Transition Pattern

```rust
pub fn state_update(...) -> Option<PursueAIState> {
    // Check transition conditions
    if transition_condition_met {
        return Some(PursueAIState::NewState);
    }
    
    // Update state behavior
    // ...
    
    // No transition
    None
}
```

---

## Integration with Platformer AI

The Pursue AI uses the `PlatformerAI` component for movement:

```rust
Query<(&mut Transform, &mut Physics, &mut PursueAI)>
```

Movement is handled by `s_platformer_ai_movement()` which reads:
- `PlatformerAI.current_target_node` - Current pathfinding target
- Goal position (from `GoalPoint` transform)

### Movement Flow

1. **Pursue AI State** → Selects goal (e.g., player position, random node)
2. **Platformer AI System** → Pathfinds to goal, calculates movement
3. **Collision System** → Handles physics collisions
4. **Render System** → Draws agent

---

## Common Patterns

### State Update Function Signature

```rust
pub fn state_update(
    transform: &mut Transform,
    physics: &mut Physics,
    pursue_ai: &mut PursueAI,
    pathfinding: &mut PathfindingGraph,
) -> Option<PursueAIState> {
    // State logic
    None // or Some(new_state)
}
```

### Accessing Goal Point

```rust
// In state update, query for GoalPoint
let goal_query: Query<&Transform, With<GoalPoint>> = ...;
if let Ok(goal_transform) = goal_query.single() {
    let goal_pos = goal_transform.translation.xy();
    // Use goal_pos for pathfinding
}
```

### Pathfinding to Goal

```rust
use crate::ai::a_star::find_path;

let path = find_path(pathfinding, agent_pos, goal_pos);
if let Some(path) = path {
    // Use path for movement (via PlatformerAI system)
}
```

### Setting Movement Target

```rust
// Set target node for PlatformerAI system
platformer_ai.current_target_node = Some(node_id);
```

---

## Anti-Patterns

### ❌ Don't

- Mutate `PathfindingGraph` in state updates (use `Res` not `ResMut`)
- Implement movement logic in state updates (use `PlatformerAI` system)
- Create new states without updating match statement
- Hardcode state transition thresholds (use constants)
- Mix state logic with rendering (keep systems separate)

### ✅ Do

- Return `Option<PursueAIState>` for state transitions
- Use `find_path()` for pathfinding (don't reimplement)
- Use `PlatformerAI` component for movement
- Keep state update functions focused on state logic
- Use constants for state transition thresholds

---

## Constants Reference

- `PURSUE_AI_AGENT_RADIUS: f32 = 8.0` - Agent collision radius
- `WANDER_MAX_SPEED: f32 = 3.0` - Wander movement speed
- `PLATFORMER_AI_JUMP_FORCE: f32 = 8.0` - Jump force (from `platformer_ai.rs`)
- `ACCELERATION_SCALERS: (f32, f32) = (0.2, 0.4)` - Acceleration/deceleration

---

## Quick Reference

```bash
# Find state machine usage
rg -n "PursueAIState|PursueAI" src/

# Find state transitions
rg -n "Some\(PursueAIState" src/

# Find state update functions
rg -n "pub fn.*_update" src/ai/pursue_ai/
```

