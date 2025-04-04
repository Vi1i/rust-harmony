use clap::{Parser, ValueEnum};
use colors_transform::{Color, Rgb};
use image::{ImageBuffer, Rgb as ImageRgb};

#[allow(unused_imports)]
use harmony::{
    WorldMap, MapGenerator, HexPosition, grid::TerrainType,
    map::{ChunkPosition, MapChunk, StructureType},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Type of map to generate
    #[arg(value_enum)]
    map_type: MapTypes,

    /// Size of the map (width and height in chunks)
    #[arg(short, long, default_value_t = 1)]
    size: i32,

    /// Size of each chunk
    #[arg(short, long, default_value_t = 20)]
    chunk_size: i32,

    /// Spacing between hexagons in pixels
    #[arg(short = 'g', long, default_value_t = 0)]
    spacing: i32,

    /// Random seed for map generation
    #[arg(short = 'd', long)]
    seed: Option<u64>,

    /// Output file name
    #[arg(short, long, default_value = "map.png")]
    output: String,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum MapTypes {
    World,
    Town,
    Forest,
}

const HEX_RADIUS: f32 = 20.0;  // Radius of the hex (distance from center to corner)
const SQRT_3: f32 = 1.732_050_8;

// Padding for the image
const PADDING: f32 = 40.0;

fn main() {
    let cli = Cli::parse();

    match cli.map_type {
        MapTypes::World => generate_world_map(&cli),
        MapTypes::Town => generate_template_map(&cli, "town"),
        MapTypes::Forest => generate_template_map(&cli, "forest"),
    }
}

fn generate_world_map(cli: &Cli) {
    let mut world = if let Some(seed) = cli.seed {
        WorldMap::with_seed(cli.chunk_size, seed)
    } else {
        WorldMap::new(cli.chunk_size)
    };
    let mut chunks = Vec::new();

    // Generate chunks
    for x in 0..cli.size {
        for y in 0..cli.size {
            let pos = ChunkPosition { x, y };
            chunks.push(world.get_or_generate_chunk(pos).clone());
        }
    }

    render_chunks(&chunks, cli);
}

fn generate_template_map(cli: &Cli, template: &str) {
    let mut generator = if let Some(seed) = cli.seed {
        MapGenerator::with_seed(seed)
    } else {
        MapGenerator::new()
    };
    
    if let Some(chunk) = generator.generate_map(template) {
        render_chunks(&[chunk], cli);
    }
}

fn render_chunks(chunks: &[MapChunk], cli: &Cli) {
    // Calculate image dimensions with padding
    let hex_width = HEX_RADIUS * SQRT_3;
    let hex_height = HEX_RADIUS * 1.5;
    let gap = cli.spacing as f32;
    
    let total_width = (cli.size * cli.chunk_size) as f32 * (hex_width + gap) + PADDING * 2.0;
    let total_height = (cli.size * cli.chunk_size) as f32 * (hex_height + gap) + PADDING * 2.0;
    
    let mut img = ImageBuffer::new(total_width as u32, total_height as u32);

    for chunk in chunks {
        for (pos, cell) in chunk.grid.iter_cells() {
            let (center_x, center_y) = hex_to_pixel(pos, cli.spacing);
            
            // Draw the hex
            draw_hex(
                &mut img,
                center_x,
                center_y,
                get_terrain_color(&cell.terrain, cell.elevation),
                cli.spacing,
                cell.elevation,
            );

            // Draw structures
            if let Some(structure) = chunk.structures.get(pos) {
                draw_structure(&mut img, center_x, center_y, structure);
            }
        }
    }

    img.save(&cli.output).expect("Failed to save image");
    println!("Map saved to {}", cli.output);
}

fn hex_to_pixel(hex: &HexPosition, spacing: i32) -> (f32, f32) {
    // For pointy-topped hexagons:
    // Horizontal distance between centers = 2 * radius * cos(30°) = radius * √3
    // Vertical distance between centers = radius * (1 + sin(60°)) = radius * 1.5
    
    let hex_width = HEX_RADIUS * SQRT_3;
    let hex_height = HEX_RADIUS * 1.5;
    let gap = spacing as f32;
    
    // Offset every other row by half a hex width
    let row_offset = if hex.r % 2 == 0 { 0.0 } else { (hex_width + gap) * 0.5 };
    let x = PADDING + (hex_width + gap) * hex.q as f32 + row_offset;
    let y = PADDING + (hex_height + gap) * hex.r as f32;
    (x, y)
}

fn get_terrain_color(terrain: &TerrainType, elevation: i32) -> Rgb {
    let base_color = match terrain {
        TerrainType::Plain => Rgb::from(120.0, 180.0, 80.0),  // Green
        TerrainType::Rough => Rgb::from(140.0, 140.0, 100.0), // Brown-ish
        TerrainType::Water => {
            if elevation < 0 {
                Rgb::from(40.0, 80.0, 150.0)   // Deep water
            } else {
                Rgb::from(80.0, 140.0, 200.0)  // Shallow water
            }
        },
        TerrainType::Wall => Rgb::from(100.0, 100.0, 100.0),  // Gray
        TerrainType::Sand => Rgb::from(240.0, 220.0, 160.0),  // Sand color
        TerrainType::Snow => Rgb::from(250.0, 250.0, 250.0),  // White
        TerrainType::Swamp => Rgb::from(80.0, 100.0, 70.0),   // Dark green
        TerrainType::Lava => Rgb::from(200.0, 50.0, 0.0),     // Red-orange
    };

    // Apply elevation-based shading
    let mut color = base_color;
    
    // Darken valleys and brighten peaks
    let elevation_factor = if elevation >= 0 {
        1.0 + (elevation as f32 * 0.15)  // Peaks get brighter
    } else {
        1.0 / (1.0 - (elevation as f32 * 0.1))  // Valleys get darker
    };

    // Add a slight blue tint to high elevations (atmospheric effect)
    if elevation > 5 {
        let blue_tint = (elevation - 5) as f32 * 0.05;
        color = Rgb::from(
            color.get_red(),
            color.get_green(),
            (color.get_blue() + blue_tint * 255.0).min(255.0)
        );
    }

    // Apply the elevation factor
    Rgb::from(
        (color.get_red() * elevation_factor).min(255.0),
        (color.get_green() * elevation_factor).min(255.0),
        (color.get_blue() * elevation_factor).min(255.0)
    )
}

fn draw_hex(
    img: &mut ImageBuffer<ImageRgb<u8>, Vec<u8>>,
    center_x: f32,
    center_y: f32,
    color: Rgb,
    spacing: i32,
    elevation: i32,
) {
    let points = get_hex_points(center_x, center_y, spacing);
    
    // Calculate shading based on elevation
    let shade_factor = if elevation >= 0 {
        1.0 - (elevation as f32 * 0.05).min(0.3)  // Higher elevation = darker edges
    } else {
        1.0 + (elevation as f32 * 0.05).max(-0.3)  // Lower elevation = lighter edges
    };

    // Draw filled hexagon with gradient
    for y in (center_y - HEX_RADIUS) as i32..(center_y + HEX_RADIUS) as i32 {
        for x in (center_x - HEX_RADIUS) as i32..(center_x + HEX_RADIUS) as i32 {
            if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
                continue;
            }
            if point_in_hexagon((x as f32, y as f32), &points) {
                // Calculate distance from center for gradient effect
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt() / HEX_RADIUS;
                
                // Create gradient effect based on elevation
                let gradient = 1.0 - (dist * shade_factor);
                
                img.put_pixel(
                    x as u32,
                    y as u32,
                    ImageRgb([
                        (color.get_red() * gradient) as u8,
                        (color.get_green() * gradient) as u8,
                        (color.get_blue() * gradient) as u8,
                    ]),
                );
            }
        }
    }

    // Draw elevation contour lines
    if elevation != 0 {
        let contour_points = get_hex_points(center_x, center_y, spacing - elevation.abs() as i32);
        draw_hex_outline(img, &contour_points, color, 0.5);
    }
}

fn draw_structure(
    img: &mut ImageBuffer<ImageRgb<u8>, Vec<u8>>,
    center_x: f32,
    center_y: f32,
    structure: &StructureType,
) {
    let color = match structure {
        StructureType::Building(_) => Rgb::from(200.0, 50.0, 50.0),   // Red
        StructureType::Vegetation(_) => Rgb::from(50.0, 150.0, 50.0), // Green
        StructureType::Landmark(_) => Rgb::from(200.0, 200.0, 50.0),  // Yellow
    };

    // Draw a small filled circle for the structure
    let radius = HEX_RADIUS * 0.2;
    for dy in -radius as i32..=radius as i32 {
        for dx in -radius as i32..=radius as i32 {
            let x = center_x as i32 + dx;
            let y = center_y as i32 + dy;
            
            if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
                continue;
            }

            if (dx * dx + dy * dy) as f32 <= radius * radius {
                img.put_pixel(
                    x as u32,
                    y as u32,
                    ImageRgb([
                        color.get_red() as u8,
                        color.get_green() as u8,
                        color.get_blue() as u8,
                    ]),
                );
            }
        }
    }
}

fn get_hex_points(center_x: f32, center_y: f32, _spacing: i32) -> [(f32, f32); 6] {
    let mut points = [(0.0, 0.0); 6];
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32 + std::f32::consts::PI / 6.0; // Rotate 30 degrees
        points[i] = (
            center_x + HEX_RADIUS * angle.cos(),
            center_y + HEX_RADIUS * angle.sin(),
        );
    }
    points
}

fn point_in_hexagon(point: (f32, f32), vertices: &[(f32, f32); 6]) -> bool {
    let (x, y) = point;
    let mut inside = false;
    let mut j = vertices.len() - 1;

    for i in 0..vertices.len() {
        let (xi, yi) = vertices[i];
        let (xj, yj) = vertices[j];

        if ((yi > y) != (yj > y)) &&
           (x < (xj - xi) * (y - yi) / (yj - yi) + xi)
        {
            inside = !inside;
        }
        j = i;
    }

    inside
}

fn draw_hex_outline(img: &mut ImageBuffer<ImageRgb<u8>, Vec<u8>>, points: &[(f32, f32); 6], color: Rgb, alpha: f32) {
    for i in 0..6 {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % 6];
        draw_line(img, x1, y1, x2, y2, color, alpha);
    }
}

fn draw_line(img: &mut ImageBuffer<ImageRgb<u8>, Vec<u8>>, x1: f32, y1: f32, x2: f32, y2: f32, color: Rgb, alpha: f32) {
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1.0 } else { -1.0 };
    let sy = if y1 < y2 { 1.0 } else { -1.0 };
    let mut err = dx - dy;
    
    let mut x = x1;
    let mut y = y1;
    
    while (x - x2).abs() > 0.5 || (y - y2).abs() > 0.5 {
        if x >= 0.0 && x < img.width() as f32 && y >= 0.0 && y < img.height() as f32 {
            let pixel = img.get_pixel(x as u32, y as u32);
            img.put_pixel(
                x as u32,
                y as u32,
                ImageRgb([
                    (pixel[0] as f32 * (1.0 - alpha) + color.get_red() * alpha) as u8,
                    (pixel[1] as f32 * (1.0 - alpha) + color.get_green() * alpha) as u8,
                    (pixel[2] as f32 * (1.0 - alpha) + color.get_blue() * alpha) as u8,
                ]),
            );
        }
        
        let e2 = 2.0 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
}
