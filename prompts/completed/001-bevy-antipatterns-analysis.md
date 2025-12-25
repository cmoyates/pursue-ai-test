<objective>
Thoroughly analyze this Bevy ECS game codebase for antipatterns and implement practical design pattern improvements.

This is a small 2D platformer AI test project (Rain World-inspired lizard AI). The goal is to improve code quality and maintainability while respecting the small scope - avoid over-engineering. Focus on idiomatic Bevy patterns and Rust best practices.
</objective>

<context>
**Tech Stack**: Rust 2021, Bevy 0.17.3, rand 0.9, serde/serde_json
**Architecture**: Plugin-based ECS with state machine AI

Key files to examine:
- `src/main.rs` - App setup, resources, components, main systems
- `src/collisions.rs` - Physics collision detection
- `src/level.rs` - Level generation from JSON
- `src/ai/mod.rs` - AI module organization
- `src/ai/pathfinding.rs` - Graph-based pathfinding
- `src/ai/platformer_ai.rs` - Movement logic
- `src/ai/pursue_ai/mod.rs` - State machine coordinator
- `src/ai/pursue_ai/movement.rs` - Movement helpers
- `src/ai/pursue_ai/wander.rs` - Wander state

Read `AGENTS.md` at project root and in subdirectories for project conventions.
</context>

<analysis_requirements>
Deeply analyze the codebase for these categories of antipatterns:

**1. Bevy ECS Antipatterns**
- Systems with too many parameters (`#[allow(clippy::too_many_arguments)]`)
- Using `ResMut` when `Res` would suffice (unnecessary mutable borrows)
- Unused system parameters (prefixed with `_`)
- Queries that could use filters (`With<T>`, `Without<T>`) more effectively
- Missing or incorrect system ordering
- Systems doing too much (violating single responsibility)

**2. Rust Idioms**
- Code duplication (e.g., movement.rs vs platformer_ai.rs)
- Magic numbers without named constants
- Excessive use of `#[allow(dead_code)]`
- Commented-out code that should be removed or documented
- Error handling patterns (`unwrap()` vs proper handling)

**3. Component/Resource Design**
- Components with too many fields (could be split)
- Resources that could be components (or vice versa)
- State machine patterns that could be improved

**4. Architecture**
- Plugin boundaries and responsibilities
- Module organization and exports
- System set organization for better ordering

Consider what is appropriate for a small project - not every antipattern requires a fix. Prioritize changes that:
- Improve readability significantly
- Prevent future bugs
- Make the codebase more idiomatic
- Enable easier extension of the AI states
</analysis_requirements>

<implementation>
After analysis, implement the following improvements (prioritized):

**High Priority (must fix):**
1. Replace `ResMut` with `Res` where mutation isn't needed
2. Remove unused system parameters or use them
3. Extract magic numbers into named constants in appropriate locations
4. Remove or document commented-out code blocks

**Medium Priority (should fix if straightforward):**
5. Consolidate duplicate code between `movement.rs` and `platformer_ai.rs`
6. Refactor systems with too many arguments by:
   - Splitting into multiple focused systems, OR
   - Using SystemParam structs if appropriate, OR
   - Using query bundles
7. Add proper system sets for clearer ordering

**Low Priority (only if clearly beneficial):**
8. Consider component splits if a component is doing too much
9. Improve module re-exports for cleaner API

**What NOT to do:**
- Don't add abstraction layers that aren't needed
- Don't create traits/generics without clear benefit
- Don't refactor working state machine pattern unless significantly improved
- Don't add configuration systems for hardcoded values that won't change
- Don't introduce new dependencies
</implementation>

<constraints>
- Maintain all existing functionality - game must still compile and run
- Keep changes focused and minimal - this is a small project
- Follow existing naming conventions (`s_` prefix for systems)
- Preserve plugin-based architecture
- Don't modify the pathfinding algorithm logic or level generation
- Run `cargo fmt` and `cargo clippy --all-targets --all-features -D warnings` after changes
</constraints>

<output>
1. Create a brief analysis summary (as code comments or inline) noting the antipatterns found
2. Modify the source files to implement the improvements
3. Ensure the project compiles cleanly with no warnings

Files that will likely need modification:
- `./src/main.rs`
- `./src/collisions.rs`
- `./src/ai/platformer_ai.rs`
- `./src/ai/pursue_ai/mod.rs`
- `./src/ai/pursue_ai/movement.rs`
</output>

<verification>
Before declaring complete:
1. Run `cargo build` - must compile successfully
2. Run `cargo clippy --all-targets --all-features -D warnings` - no warnings
3. Run `cargo fmt --check` - properly formatted
4. Briefly run `cargo run` to verify basic functionality works
</verification>

<success_criteria>
- No more `#[allow(clippy::too_many_arguments)]` attributes (systems refactored)
- No `ResMut` used where `Res` would work
- No unused system parameters
- Magic numbers extracted to named constants
- Commented-out code either removed or has documentation explaining why kept
- Duplicate movement code consolidated
- All Clippy warnings resolved
- Project compiles and runs correctly
</success_criteria>

