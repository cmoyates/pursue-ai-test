# AGENTS.md - Pursue AI Test

## Project Snapshot

**Repository Type**: Single Rust crate (not a workspace)  
**Primary Tech Stack**: Rust 2021 Edition + Bevy 0.17.3 + rand 0.9 + serde/serde_json  
**Game Type**: 2D platformer AI test project (Rain World-inspired lizard AI)  
**Architecture**: Plugin-based Bevy ECS with state machine AI system

Sub-modules have their own detailed AGENTS.md files for specific patterns and examples.

---

## Root Setup Commands

### Build & Run
```bash
# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run game
cargo run

# Run with optimizations
cargo run --release
```

### Code Quality
```bash
# Format code
cargo fmt

# Lint (treat warnings as errors)
cargo clippy --all-targets --all-features -D warnings

# Check without building
cargo check
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

### Optional Checks
```bash
# Security audit (if cargo-audit installed)
cargo audit

# Unused dependencies (if cargo-machete installed)
cargo machete
```

---

## Universal Conventions

### Rust Style
- **Formatting**: `rustfmt` required (run `cargo fmt` before commits)
- **Linting**: Clippy warnings treated as errors (`-D warnings`)
- **Edition**: Rust 2021 Edition

### Bevy Architecture
- **Plugin-First**: Encapsulate functionality in plugins (`PathfindingPlugin`, `PursueAIPlugin`, `CollisionPlugin`)
- **System Ordering**: Use `.after()` and `.chain()` for explicit dependencies
- **ECS Patterns**: 
  - Components: Data on entities (`Physics`, `PursueAI`, `GoalPoint`)
  - Resources: Global state (`Level`, `PathfindingGraph`, `InputDir`)
  - Systems: Logic functions (`s_*` prefix convention)

### Performance
- Avoid tight-loop allocations; profile before optimizing
- Use `Query` filters (`With<T>`, `Without<T>`) for efficient entity access
- Prefer `single()` or `single_mut()` when expecting exactly one entity

### Code Organization
- **Module Structure**: Follow Rust conventions (`mod.rs` for module roots)
- **Naming**: Systems prefixed with `s_` (e.g., `s_collision`, `s_input`)
- **State Machines**: Use enums for AI states (`PursueAIState`)

---

## Security & Secrets

- **No API Keys**: This project has no external API dependencies
- **No Secrets**: No sensitive configuration files (level data is JSON)
- **Asset Files**: `assets/level.json` is version-controlled (no secrets)

---

## JIT Index (what to open, not what to paste)

### Directory Map

- **Core game logic**: `src/` → [see src/AGENTS.md](src/AGENTS.md)
- **AI system**: `src/ai/` → [see src/ai/AGENTS.md](src/ai/AGENTS.md)
- **Pursue AI**: `src/ai/pursue_ai/` → [see src/ai/pursue_ai/AGENTS.md](src/ai/pursue_ai/AGENTS.md)
- **Assets**: `assets/` → [see assets/AGENTS.md](assets/AGENTS.md)

### Quick Find Commands

```bash
# Find components
rg -n "derive\(Component\)" -S src

# Find systems (functions with Query/Res/Commands/EventReader/EventWriter)
rg -n "fn s_.*\(.*(Query|Res|Commands|EventReader|EventWriter)" -S src

# Find plugins
rg -n "impl Plugin for" -S src

# Find resources
rg -n "derive\(Resource\)" -S src

# Find state machine enums
rg -n "enum.*State" -S src

# Find system ordering
rg -n "\.after\(|\.chain\(|\.before\(" -S src

# Find pathfinding usage
rg -n "PathfindingGraph|pathfinding" -S src

# Find collision detection
rg -n "s_collision|collisions" -S src
```

### Key Files Reference

- **Entry Point**: `src/main.rs` - App setup, plugin registration, main systems
- **Level Generation**: `src/level.rs` - Polygon generation from JSON
- **Collision System**: `src/collisions.rs` - Physics-based collision detection
- **Pathfinding**: `src/ai/pathfinding.rs` - Graph-based pathfinding system
- **A* Algorithm**: `src/ai/a_star.rs` - Pathfinding algorithm implementation
- **Pursue AI**: `src/ai/pursue_ai/mod.rs` - State machine root
- **AI Movement**: `src/ai/pursue_ai/movement.rs` - Platformer movement logic
- **Wander State**: `src/ai/pursue_ai/wander.rs` - Wander behavior implementation

---

## Common Patterns & Anti-Patterns

### ✅ Good Patterns

```rust
// Plugin-based architecture
pub struct MyPlugin;
impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, my_system.after(other_system));
    }
}

// Explicit system ordering
.add_systems(Update, (s_input, s_move_goal_point).chain())
.add_systems(Update, s_render.after(s_collision))

// Query with filters
Query<&mut Transform, With<GoalPoint>>
```

### ❌ Anti-Patterns

- **Monolithic systems**: Avoid large `s_*` functions; split into focused systems
- **Excessive `ResMut`**: Prefer `Res` when mutation isn't needed
- **Large queries in tight loops**: Use filters and early returns
- **Direct state mutation**: Use state machine pattern for AI states

---

## Development Workflow

1. **Make Changes**: Edit Rust files in `src/`
2. **Format**: Run `cargo fmt`
3. **Check**: Run `cargo clippy --all-targets --all-features -D warnings`
4. **Test**: Run `cargo test` (when tests exist)
5. **Run**: Run `cargo run` to see changes

---

## Next Steps

For detailed patterns and examples:
- **Game Systems**: See `src/AGENTS.md`
- **AI Implementation**: See `src/ai/AGENTS.md`
- **Pursue AI States**: See `src/ai/pursue_ai/AGENTS.md`
- **Asset Pipeline**: See `assets/AGENTS.md` (if created)

