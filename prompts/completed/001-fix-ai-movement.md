<objective>
Fix the AI entity so it moves (falls with gravity and can pursue the goal point).

The AI entity is currently spawned but doesn't move at all - no gravity, no pathfinding movement. 
This needs to be debugged and fixed so the AI can navigate the level.
</objective>

<context>
This is a Bevy 0.17.3 game with a Pursue AI system. The AI should pathfind toward a goal point and 
move through the level using platformer physics (walking, jumping, falling).

Key files to examine:
- `src/main.rs` - App setup, entity spawning, plugin registration
- `src/ai/platformer_ai.rs` - Movement system (`s_platformer_ai_movement`)
- `src/ai/pursue_ai/mod.rs` - AI state machine plugin
- `src/ai/pursue_ai/wander.rs` - Wander state implementation
- `src/collisions.rs` - Collision detection (queries PursueAI entities)
</context>

<analysis>
Thoroughly analyze the movement system flow to identify why the AI isn't moving:

1. **Entity Component Check**: 
   - What components does the AI entity have when spawned?
   - What components does `s_platformer_ai_movement` query for?
   - Is there a mismatch?

2. **Plugin Registration Check**:
   - Is `PlatformerAIPlugin` registered in the app?
   - What systems does it add and are they running?

3. **System Flow Check**:
   - Trace the execution path from `s_pursue_ai_update` → `wander_update` → `wander_movement`
   - Does `wander_movement` actually apply any physics/movement?

4. **Physics Application Check**:
   - Where is gravity supposed to be applied?
   - Where is velocity supposed to update the transform?
   - Is any of this actually happening for the AI entity?
</analysis>

<requirements>
1. Fix the AI entity spawn to include all required components for movement
2. Ensure all necessary plugins are registered in the correct order
3. Ensure movement physics (gravity, velocity) are being applied to the AI entity
4. The AI should fall due to gravity when not on a surface
5. The AI should be able to pathfind and move toward the goal point

After fixes:
- Run `cargo clippy --all-targets --all-features -D warnings` to check for errors
- Run `cargo run` to verify the AI moves
</requirements>

<implementation>
Follow these principles:
- Don't duplicate movement logic - use the existing `s_platformer_ai_movement` system
- Maintain proper system ordering (AI update → movement → collision → render)
- Use existing components (`PlatformerAI`) rather than creating new ones
- Follow the existing codebase patterns for component composition
</implementation>

<output>
Modify the necessary files to fix AI movement:
- `src/main.rs` - Fix entity spawning and/or plugin registration
- Other files as needed based on analysis

Run the game after changes to verify the AI:
1. Falls due to gravity when in the air
2. Lands on surfaces
3. Attempts to pathfind toward the goal point
</output>

<verification>
Before declaring complete:
1. `cargo clippy --all-targets --all-features -D warnings` passes
2. `cargo run` launches without errors
3. The AI (red circle) visibly moves/falls in the game
4. When gizmos are enabled (G key), pathfinding visualization should appear
</verification>

<success_criteria>
- AI entity responds to gravity (falls when not on surface)
- AI entity can land on and move along surfaces  
- AI entity attempts to navigate toward the goal point
- No clippy warnings or errors
</success_criteria>

