# Harmony

A Rust library for procedural map generation with a focus on 3D hex-based maps. Generate rich, detailed worlds with elevation, terrain types, and structures.

## Features

### Map Generation
- Multiple map types (world, town, forest)
- Procedural terrain generation
- Biome-based generation
- Structure placement
- Seeded random generation for reproducible maps

### 3D Hex Grid System
- True 3D coordinates (q, r, z)
- Elevation-aware pathfinding
- Terrain-specific movement costs
- Advanced hex grid algorithms

### Terrain Types
- Plains
- Rough terrain
- Water (with depth)
- Walls
- Sand
- Snow (elevation-dependent)
- Swamp
- Lava (elevation-restricted)

### Visual Features
- Elevation-based shading
- Terrain color gradients
- Contour lines
- Atmospheric effects
- Customizable hex spacing

## Usage

### Library Usage

Add to your `Cargo.toml`:
```toml
[dependencies]
harmony = "0.1.0"
```

Basic example:
```rust
use harmony::{WorldMap, MapGenerator, HexPosition};

// Create a new world map with chunk size 20
let mut world = WorldMap::new(20);

// Or with a specific seed for reproducible generation
let mut world = WorldMap::with_seed(20, 12345);

// Generate a chunk at position (0, 0)
let chunk = world.get_or_generate_chunk(ChunkPosition { x: 0, y: 0 });
```

### Command Line Tool

The library includes a map generator binary:

```bash
# Generate a world map
cargo run --bin map_generator world --size 2 --chunk-size 15 --output map.png

# Generate with specific seed
cargo run --bin map_generator world --size 2 --chunk-size 15 --seed 12345 --output map.png

# Generate with custom hex spacing
cargo run --bin map_generator world --size 2 --chunk-size 15 --spacing 5 --output map.png
```

Options:
- `--size`: Map size in chunks
- `--chunk-size`: Size of each chunk
- `--spacing`: Spacing between hexes in pixels
- `--seed`: Random seed for reproducible generation
- `--output`: Output PNG file path

## Development

### Building
```bash
cargo build
```

### Testing
```bash
cargo test
```

### Project Structure
- `src/grid.rs`: Core hex grid implementation
- `src/map.rs`: Map generation and chunk management
- `src/bin/map_generator.rs`: CLI tool

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License.
