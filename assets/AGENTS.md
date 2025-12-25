# assets/AGENTS.md - Asset Pipeline

## Overview

This directory contains game assets, primarily level data.

**Key Files:**
- `level.json` - Level grid data (tile-based)

---

## Level Data (`level.json`)

### Format

2D array of integers representing tile types:

```json
[
  [1, 1, 1, ...],
  [1, 0, 0, ...],
  ...
]
```

### Tile Types

- **`0`** - Empty space (air)
- **`1`** - Solid square tile
- **`2`** - Right triangle (bottom-left)
- **`3`** - Right triangle (top-left)
- **`4`** - Right triangle (bottom-right)
- **`5`** - Right triangle (top-right)
- **`6-9`** - Isosceles triangles (currently unused in code)

### Loading

Level data is loaded at compile time using `include_bytes!`:

```rust
const LEVEL_DATA: &[u8] = include_bytes!("../assets/level.json");

let level_grid_data: Vec<Vec<usize>> = serde_json::from_str(
    std::str::from_utf8(LEVEL_DATA).unwrap()
).unwrap();
```

### Level Generation

See `src/level.rs` for level generation pipeline:
1. Parse JSON grid data
2. Convert tiles to line segments
3. Merge parallel lines
4. Build polygons from line segments
5. Calculate winding order
6. Mark container polygons

### Grid Coordinate System

- Origin: Top-left of grid
- X-axis: Right (increasing)
- Y-axis: Down (increasing)
- Converted to world coordinates with offset and Y-flip

### Grid Size

Level uses `grid_size: f32 = 32.0` (32 world units per grid cell)

---

## Asset Loading Pattern

### Compile-Time Loading

Current approach uses `include_bytes!` for compile-time asset inclusion:

**Pros:**
- No runtime file I/O
- Assets bundled with binary
- Fast loading

**Cons:**
- No hot-reloading
- Binary size increases
- Must rebuild to change assets

### Usage

```rust
// In level.rs
const LEVEL_DATA: &[u8] = include_bytes!("../assets/level.json");

pub fn generate_level_polygons(grid_size: f32) -> (Vec<Polygon>, Vec2, Vec2) {
    let res = std::str::from_utf8(LEVEL_DATA);
    let level_grid_data: Vec<Vec<usize>> = serde_json::from_str(res.unwrap()).unwrap();
    // ... process level data
}
```

---

## Future Asset Types

### Potential Assets

- **Sprites**: Agent sprites, level tiles (if moving to sprite-based rendering)
- **Audio**: Sound effects, music (if adding audio)
- **Shaders**: Custom shaders (if adding visual effects)
- **Fonts**: UI fonts (if adding text rendering)

### Runtime Asset Loading

If moving to runtime asset loading, use Bevy's `AssetServer`:

```rust
// Example (not currently used)
let level_handle: Handle<LevelAsset> = asset_server.load("levels/level1.json");

// In system
if let Some(level) = level_assets.get(&level_handle) {
    // Use level data
}
```

---

## File Organization

```
assets/
├── level.json          # Level grid data
└── (future assets)
    ├── sprites/
    ├── audio/
    └── shaders/
```

---

## Level Editor Workflow

### Creating Levels

1. Create JSON file with 2D array of tile IDs
2. Use tile IDs: `0` (empty), `1` (square), `2-5` (triangles)
3. Place in `assets/level.json`
4. Rebuild game to see changes

### Level Design Tips

- Use `1` for solid platforms
- Use `2-5` for slopes and ramps
- Ensure outer boundary is container polygon (wrapping `1`s)
- Test collision detection with different tile combinations

---

## Quick Reference

```bash
# View level data
cat assets/level.json

# Validate JSON
python3 -m json.tool assets/level.json

# Count tiles
rg -o "\d+" assets/level.json | sort | uniq -c
```

---

## Notes

- Level data is version-controlled (no secrets)
- JSON format is simple and human-readable
- Consider using a level editor tool in the future
- Current system supports only one level (hardcoded)

