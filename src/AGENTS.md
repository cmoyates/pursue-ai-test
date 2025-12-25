# src/AGENTS.md - Core Game Systems

## Overview

This directory contains the core game systems: app initialization, level generation, collision detection, and utility functions.

**Key Files:**
- `main.rs` - App setup, plugin registration, main game loop
- `level.rs` - Level generation from JSON grid data
- `collisions.rs` - Physics-based collision detection system
- `utils.rs` - Math utilities (line intersection, side detection)

---

## Main App (`main.rs`)

### App Setup Pattern

```rust
App::new()
    .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
    .insert_resource(InputDir { dir: Vec2::ZERO })
    .add_plugins(DefaultPlugins.set(WindowPlugin { ... }))
    .add_plugins(PathfindingPlugin)
    .add_plugins(PursueAIPlugin)
    .add_plugins(CollisionPlugin)
    .add_systems(Startup, s_init)
    .add_systems(Update, (s_input, s_move_goal_point).chain())
    .add_systems(Update, s_render.after(s_collision))
```

### Key Components

- **`Physics`** - Physics state (velocity, acceleration, grounded, walled, normal)
- **`GoalPoint`** - Player-controlled goal marker
- **`PursueAI`** - AI agent component (see `ai/pursue_ai/AGENTS.md`)

### Key Resources

- **`InputDir`** - Player input direction (arrow keys)
- **`GizmosVisible`** - Toggle debug rendering (G key)
- **`Level`** - Level geometry (polygons)
- **`PathfindingGraph`** - Navigation graph (see `ai/pathfinding.rs`)

### System Ordering

```rust
// Input must happen before movement
.add_systems(Update, (s_input, s_move_goal_point).chain())

// Collision must happen before rendering
.add_systems(Update, s_render.after(s_collision))

// AI movement must happen before collision
.add_systems(Update, s_collision.after(s_platformer_ai_movement))
```

### Input Handling

- **Arrow Keys**: Move goal point
- **R Key**: Reset AI position
- **G Key**: Toggle gizmos (debug rendering)
- **Escape**: Exit (desktop only)
- **Mouse Click**: Debug pathfinding node info

---

## Level System (`level.rs`)

### Level Generation Flow

1. **Load JSON**: `include_bytes!("../assets/level.json")` → `Vec<Vec<usize>>`
2. **Generate Lines**: Convert grid tiles to line segments
3. **Remove Redundant Points**: Merge parallel lines
4. **Build Polygons**: Connect lines into closed polygons
5. **Calculate Winding**: Ensure correct collision side
6. **Mark Containers**: Identify outer container polygons

### Grid Tile Types

- `0` - Empty space
- `1` - Solid square
- `2-5` - Right triangles (4 orientations)
- `6-9` - Isosceles triangles (currently commented out)

### Polygon Structure

```rust
pub struct Polygon {
    pub points: Vec<Vec2>,      // Closed polygon vertices
    pub color: Color,            // Random color for rendering
    pub is_container: bool,      // Outer boundary polygon
}
```

### Level Resource

```rust
#[derive(Resource)]
pub struct Level {
    pub polygons: Vec<Polygon>,
    pub grid_size: f32,         // Size of each grid cell
    pub size: Vec2,             // Level dimensions in grid cells
    pub half_size: Vec2,        // Half dimensions (for centering)
}
```

### Helper Methods

- `get_polygon(index)` - Get polygon by index
- `get_line(polygon_index, line_index)` - Get line segment
- `line_of_sight_check(start, end)` - Check if line is unobstructed (TODO: currently broken)

---

## Collision System (`collisions.rs`)

### Collision Detection Algorithm

1. **Line Intersection Test**: Raycast from previous position
2. **Point-in-Polygon**: Check if agent is inside polygon
3. **Distance to Line**: Find closest point on each line segment
4. **Collision Response**: Push agent out of geometry
5. **Normal Calculation**: Average normals from touching lines
6. **Physics Update**: Remove velocity in normal direction

### Key Functions

- `s_collision()` - Main collision system
- `find_projection(start, end, point, radius)` - Project point onto line segment

### Collision States

- **`grounded`**: Agent is on a surface (normal.y > 0.01)
- **`walled`**: Agent is on a wall (normal.x.abs() >= 0.8, signed: -1/0/1)
- **`normal`**: Surface normal vector (for gravity direction)

### Collision Response

```rust
// Push agent out of geometry
transform.translation += adjustment.extend(0.0);

// Remove velocity in normal direction
let velocity_adjustment = physics.velocity.dot(new_normal) * new_normal;
physics.velocity -= velocity_adjustment;
```

### System Ordering

```rust
// Collision runs after AI movement
.add_systems(Update, s_collision.after(s_platformer_ai_movement))
```

---

## Utilities (`utils.rs`)

### Math Functions

- **`line_intersect(line_1_start, line_1_end, line_2_start, line_2_end)`**
  - Returns `Option<Vec2>` intersection point
  - Uses cross product for line-line intersection

- **`cross_product(a, b)`**
  - 2D cross product: `a.x * b.y - a.y * b.x`

- **`side_of_line_detection(line_start, line_end, point)`**
  - Returns `-1.0`, `0.0`, or `1.0` indicating which side of line
  - Used for collision detection (prev_position check)

### Usage Patterns

```rust
// Check if two lines intersect
let intersection = line_intersect(start, end, point_a, point_b);
if intersection.is_some() {
    // Lines intersect
}

// Check which side of line a point is on
let side = side_of_line_detection(line_start, line_end, point);
if side > 0.0 {
    // Point is on one side
}
```

---

## Common Patterns

### System Function Signature

```rust
pub fn s_my_system(
    mut query: Query<(&mut Transform, &mut Physics), With<SomeComponent>>,
    resource: Res<SomeResource>,
    mut gizmos: Gizmos,
) {
    // System logic
}
```

### Query Patterns

```rust
// Single entity expected
if let Ok(mut transform) = goal_point_query.single_mut() { }

// Iterate over all matching entities
for (transform, physics) in query.iter() { }

// Filter with components
Query<&mut Transform, With<GoalPoint>>
Query<(&Transform, &Physics, &PursueAI)>
```

### Resource Access

```rust
// Read-only access
let level: Res<Level> = ...;

// Mutable access
let mut pathfinding: ResMut<PathfindingGraph> = ...;
```

### Transform Access

```rust
// Get 2D position
let pos_2d = transform.translation.xy();

// Set 2D position
transform.translation = Vec3::new(x, y, 0.0);

// Modify position
transform.translation += Vec3::new(dx, dy, 0.0);
```

---

## Anti-Patterns

### ❌ Don't

- Access `ResMut` when `Res` is sufficient
- Use `unwrap()` on queries (use `if let Ok(...)` or `single()`)
- Hardcode magic numbers (use constants like `GRAVITY_STRENGTH`)
- Mix collision logic with movement logic
- Create systems that do too much (split into focused systems)

### ✅ Do

- Use system ordering (`.after()`, `.chain()`) for dependencies
- Use `Query` filters (`With<T>`, `Without<T>`) for efficiency
- Keep systems focused on single responsibility
- Use resources for global state, components for entity data
- Prefer `single()` or `single_mut()` when expecting exactly one entity

---

## Constants

- `GRAVITY_STRENGTH: f32 = 0.5` - Gravity acceleration
- `PURSUE_AI_AGENT_RADIUS: f32 = 8.0` - AI agent collision radius

---

## Quick Reference

```bash
# Find all systems
rg -n "pub fn s_" src/

# Find all components
rg -n "derive\(Component\)" src/

# Find all resources
rg -n "derive\(Resource\)" src/

# Find system ordering
rg -n "\.after\(|\.chain\(" src/
```

