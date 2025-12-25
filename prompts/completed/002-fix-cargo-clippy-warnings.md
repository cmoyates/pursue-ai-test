<objective>
Fix all 57 cargo clippy linting warnings in the pursue-ai-test codebase. The code should compile with zero clippy warnings when complete.
</objective>

<context>
This is a Bevy 0.13 game engine project implementing AI pathfinding and platformer mechanics. The codebase has accumulated various clippy warnings that need to be resolved for code quality.

First, read the project conventions:
@CLAUDE.md (if exists)

Files requiring fixes:
- `src/ai/pathfinding.rs` - redundant field names, unnecessary casts, needless returns
- `src/ai/pursue_ai/wander.rs` - unused imports, unused variables, needless return
- `src/ai/pursue_ai/movement.rs` - dead code (unused functions)
- `src/collisions.rs` - unused imports, unused variables, needless return
- `src/level.rs` - redundant static lifetime, unused variables, needless returns, clone on copy, ptr_arg
- `src/main.rs` - unused imports, too many arguments, iter_cloned_collect
- `src/ai/a_star.rs` - dead code (unused fields), question_mark, needless return
- `src/ai/platformer_ai.rs` - dead code (unused variants/struct), needless borrow, collapsible if
- `src/utils.rs` - manual range contains, needless return
</context>

<requirements>
Fix ALL 57 warnings. The warning categories are:

1. **Redundant field names** (2 warnings) - Use shorthand syntax `polygon_index` instead of `polygon_index: polygon_index`

2. **Unused imports** (4 warnings) - Remove unused imports:
   - `GRAVITY_STRENGTH` in wander.rs
   - `PlatformerAI` in collisions.rs
   - `PlatformerAIPlugin` in main.rs

3. **Redundant static lifetimes** (1 warning) - Remove `'static` from const declarations

4. **Unused variables** (6 warnings) - Prefix with underscore or remove if not needed:
   - `goal_node`, `physics`, `pursue_ai` in wander.rs
   - `gizmos` in collisions.rs
   - `start`, `end` in level.rs

5. **Unused mut** (1 warning) - Remove unnecessary `mut` from `gizmos` in collisions.rs

6. **Dead code** (8 warnings) - Add `#[allow(dead_code)]` annotations OR remove if truly unused:
   - `is_corner`, `is_external_corner` fields in AStarNode (a_star.rs)
   - PathFollowingStrategy variants (platformer_ai.rs)
   - `PlatformerAIPlugin` struct (platformer_ai.rs)
   - Functions in movement.rs: `apply_gravity_toward_normal`, `update_physics_and_transform`, `apply_movement_acceleration`, `handle_jumping`
   - `WANDER_MAX_SPEED` constant (wander.rs)
   - `PolygonLine` struct (level.rs)

7. **Question mark operator** (1 warning) - Use `?` operator instead of explicit `is_none()` check in a_star.rs

8. **Needless returns** (8 warnings) - Remove `return` keyword and semicolon from final expressions

9. **Unnecessary casts** (2 warnings) - Remove redundant `as f32` casts in pathfinding.rs

10. **Needless borrows** (2 warnings) - Remove unnecessary `&` references

11. **Needless range loops** (2 warnings) - Use iterators instead of index-based loops

12. **Collapsible if** (1 warning) - Combine nested if statements in platformer_ai.rs

13. **Unnecessary unwrap** (1 warning) - Use `if let Some(...)` pattern in level.rs

14. **Clone on copy** (10+ warnings) - Remove `.clone()` calls on Copy types (Vec2)

15. **Mut range bound** (1 warning) - Fix mutable range bound issue in level.rs

16. **ptr_arg** (2 warnings) - Change `&Vec<T>` to `&[T]` in function signatures

17. **Manual range contains** (2 warnings) - Use `(0.0..=1.0).contains(&value)` in utils.rs

18. **Too many arguments** (1 warning) - Add `#[allow(clippy::too_many_arguments)]` to s_input function in main.rs (Bevy systems often need many parameters)

19. **iter_cloned_collect** (1 warning) - Use `.to_vec()` instead of `.iter().cloned().collect()` in main.rs
</requirements>

<implementation>
Work through files systematically. For each file:
1. Read the file to understand context
2. Apply all relevant fixes
3. Move to the next file

For dead code warnings, prefer `#[allow(dead_code)]` if the code appears intentionally written for future use. Remove code only if it's clearly obsolete.

For the `too_many_arguments` warning on s_input, this is a Bevy system that naturally needs many resource/query parameters - use `#[allow(clippy::too_many_arguments)]` rather than restructuring.
</implementation>

<verification>
After all fixes, run:
```bash
cargo clippy 2>&1
```

The output should show:
- No warnings
- `Finished` status with 0 warnings

If any warnings remain, fix them before completing.
</verification>

<success_criteria>
- All 57 clippy warnings are resolved
- Code compiles successfully with `cargo build`
- `cargo clippy` produces zero warnings
- No functionality is broken by the changes
</success_criteria>

