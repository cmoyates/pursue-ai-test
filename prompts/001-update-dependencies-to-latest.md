<objective>
Update all dependencies in the Rust project's `Cargo.toml` to their latest stable versions, with special attention to Bevy which has breaking changes between minor versions. The project must compile and run correctly after the update.
</objective>

<context>
This is a Bevy-based game project implementing AI pathfinding and pursue behaviors. The current dependency versions are outdated:
- bevy = "0.13.0" → **UPDATE TO 0.15.3** (latest stable)
- rand = "0.8.5" → latest
- serde = "1.0.196" → latest
- serde_json = "1.0.112" → latest

Bevy has significant API changes across versions (0.13 → 0.14 → 0.15), so this update will require code modifications throughout the codebase. This is a multi-version migration.

@Cargo.toml
@src/main.rs
@src/ai/mod.rs
@src/ai/pursue_ai/mod.rs
@src/ai/pursue_ai/movement.rs
@src/ai/pursue_ai/wander.rs
@src/ai/pathfinding.rs
@src/ai/platformer_ai.rs
@src/ai/a_star.rs
@src/level.rs
@src/collisions.rs
@src/utils.rs
</context>

<research>
Before making changes, thoroughly research the migration path:

1. **Bevy 0.13 → 0.14 migration**: Use `btca ask -t bevy -q "What are the breaking changes and migration steps from Bevy 0.13 to 0.14?"` 

2. **Bevy 0.14 → 0.15 migration**: Use `btca ask -t bevy -q "What are the breaking changes and migration steps from Bevy 0.14 to 0.15?"` 

3. **Key areas to research**:
   - Window and camera setup changes
   - System scheduling and ordering changes
   - Input handling API changes
   - Transform and spatial query changes
   - Plugin registration changes
</research>

<requirements>
1. Update all dependencies to their latest stable versions
2. Fix ALL breaking changes in the codebase caused by the updates
3. Ensure the project compiles without errors (`cargo build`)
4. Maintain existing functionality - the AI pathfinding and pursue behavior must still work
5. Update any deprecated API usage to the new recommended patterns
</requirements>

<implementation>
Follow these steps in order:

1. **Research migration guides** using btca for Bevy 0.13→0.14 and 0.14→0.15 changes

2. **Update Cargo.toml** with these specific versions:
   - bevy = "0.15.3" (target version - DO NOT use a lower version)
   - rand = latest stable
   - serde = latest stable
   - serde_json = latest stable

3. **Run cargo build** to identify breaking changes

4. **Fix breaking changes systematically**, starting with:
   - Import path changes
   - Renamed types/functions
   - Changed function signatures
   - New required trait bounds
   - Plugin API changes
   - System parameter changes

5. **Common Bevy breaking changes to watch for**:
   - `Input<T>` → `ButtonInput<T>` (already done in 0.13, but check for others)
   - Window/camera API changes
   - Asset loading API changes
   - Schedule/system set changes
   - Query parameter syntax changes
   - Resource initialization patterns

6. **Test the build** after each major fix to ensure progress

7. **Run the application** to verify functionality
</implementation>

<constraints>
- Bevy MUST be updated to 0.15.3 - do not stop at an intermediate version
- Do NOT downgrade any dependency - only upgrade to latest stable
- Do NOT remove features or functionality - maintain parity with current behavior
- Do NOT skip fixing compiler errors - all code must compile
- Preserve the existing project structure and module organization
- Keep the same game mechanics and AI behavior patterns
</constraints>

<output>
Modified files:
- `./Cargo.toml` - Updated dependency versions
- `./src/*.rs` - Any source files requiring API updates
- `./src/ai/**/*.rs` - AI module files with updated Bevy APIs
</output>

<verification>
Before declaring complete, verify:

1. Run `cargo build` - must compile with zero errors
2. Run `cargo clippy` - address any new warnings if feasible
3. Run the application with `cargo run` - verify it starts and runs
4. Confirm the AI agents still move and behave correctly
5. Check that no deprecated warnings remain for the updated APIs
</verification>

<success_criteria>
- Bevy updated to exactly version 0.15.3 (not lower)
- All other dependencies updated to their latest stable versions
- Zero compilation errors
- Application runs successfully
- AI pathfinding and pursue behavior functions correctly
- No runtime panics or crashes
</success_criteria>

