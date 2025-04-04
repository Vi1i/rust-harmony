use bevy::{
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};
use bevy_mod_picking::prelude::*;
use bevy_prototype_debug_lines::*;
use harmony::{grid::TerrainType, map::{ChunkPosition, MapChunk, WorldMap}, HexPosition};

const HEX_RADIUS: f32 = 1.0;
const SQRT_3: f32 = 1.732_050_8;
const HEX_SPACING: f32 = 0.0; // No gap between hexes

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Harmony Map Viewer".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(DebugLinesPlugin::default())
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(WorldState::default())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup)
        .add_systems(Update, (
            handle_input,
            update_world_view,
            handle_hex_hover,
            draw_grid,
        ))
        .run();
}

#[derive(Resource)]
struct WorldState {
    world: WorldMap,
    selected_hex: Option<HexPosition>,
    chunks: Vec<MapChunk>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            world: WorldMap::new(20),
            selected_hex: None,
            chunks: Vec::new(),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world_state: ResMut<WorldState>,
) {
    // Create camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 20.0, 20.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Add main directional light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        ..default()
    });

    // Add ambient light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: false,
            illuminance: 3000.0,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            std::f32::consts::FRAC_PI_4,
            -std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        ..default()
    });

    // Create materials for different terrain types
    let materials = create_terrain_materials(&mut materials);

    // Generate initial chunks in a 3x3 grid
    for x in -1..=1 {
        for y in -1..=1 {
            let chunk_pos = ChunkPosition { x, y };
            let chunk = world_state.world.get_or_generate_chunk(chunk_pos).clone();
            world_state.chunks.push(chunk.clone());
            spawn_chunk(&mut commands, &chunk, &mut meshes, &materials);
        }
    }
}

fn create_hex_mesh_with_elevation(
    elevation: i32,
    terrain: TerrainType,
    neighbors: &[(i32, TerrainType); 6],
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    // Calculate vertex positions at hex corners (rotated 30 degrees)
    let mesh_radius = HEX_RADIUS * 0.999; // Slightly smaller to prevent z-fighting
    let mut corner_points = Vec::new();
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32 + std::f32::consts::PI / 6.0;
        let x = mesh_radius * angle.cos();
        let z = mesh_radius * angle.sin();
        corner_points.push((x, z));
    }

    let base_height = 0.2; // Height per elevation level
    let center_y = if terrain == TerrainType::Water {
        0.0
    } else {
        elevation as f32 * base_height
    };

    // Calculate transition heights at corners
    let mut corner_heights = Vec::new();
    for i in 0..6 {
        let prev_neighbor = neighbors[(i + 5) % 6];
        let next_neighbor = neighbors[i];

        // Get heights of both neighbors that share this corner
        let prev_height = if prev_neighbor.1 == TerrainType::Water {
            0.0
        } else {
            prev_neighbor.0 as f32 * base_height
        };

        let next_height = if next_neighbor.1 == TerrainType::Water {
            0.0
        } else {
            next_neighbor.0 as f32 * base_height
        };

        // Corner height is average of this hex and both neighbors
        let corner_height = if terrain == TerrainType::Water || 
                             prev_neighbor.1 == TerrainType::Water || 
                             next_neighbor.1 == TerrainType::Water {
            0.0
        } else {
            (center_y + prev_height + next_height) / 3.0
        };

        corner_heights.push(corner_height);
    }

    // Calculate average normal for the center vertex
    let mut center_normal = Vec3::ZERO;
    for i in 0..6 {
        let v1 = Vec3::new(
            corner_points[i].0,
            corner_heights[i],
            corner_points[i].1,
        );
        let v2 = Vec3::new(
            corner_points[(i + 1) % 6].0,
            corner_heights[(i + 1) % 6],
            corner_points[(i + 1) % 6].1,
        );
        center_normal += calculate_normal(Vec3::new(0.0, center_y, 0.0), v1, v2);
    }
    center_normal = center_normal.normalize();

    // Add center vertex
    vertices.push([0.0, center_y, 0.0]);
    normals.push([center_normal.x, center_normal.y, center_normal.z]);
    uvs.push([0.5, 0.5]);

    // Calculate corner normals (average of adjacent triangles)
    let mut corner_normals = vec![Vec3::ZERO; 6];
    for i in 0..6 {
        let v0 = Vec3::new(0.0, center_y, 0.0);
        let v1 = Vec3::new(
            corner_points[i].0,
            corner_heights[i],
            corner_points[i].1,
        );
        let v2 = Vec3::new(
            corner_points[(i + 1) % 6].0,
            corner_heights[(i + 1) % 6],
            corner_points[(i + 1) % 6].1,
        );
        let v3 = Vec3::new(
            corner_points[(i + 5) % 6].0,
            corner_heights[(i + 5) % 6],
            corner_points[(i + 5) % 6].1,
        );

        // Add normals from both adjacent triangles
        corner_normals[i] += calculate_normal(v0, v1, v2);
        corner_normals[i] += calculate_normal(v0, v3, v1);
    }

    // Add corner vertices with their averaged normals
    for i in 0..6 {
        let (x, z) = corner_points[i];
        let normal = corner_normals[i].normalize();
        vertices.push([x, corner_heights[i], z]);
        normals.push([normal.x, normal.y, normal.z]);
        uvs.push([0.5 + 0.5 * x / HEX_RADIUS, 0.5 + 0.5 * z / HEX_RADIUS]);
    }

    // Create the six triangular faces
    for i in 0..6 {
        indices.extend_from_slice(&[0, i as u32 + 1, ((i + 1) % 6 + 1) as u32]);

        // Add side face if needed (for water or edges)
        if terrain == TerrainType::Water || corner_heights[i] != 0.0 {
            let side_start = vertices.len() as u32;
            let bottom_y = 0.0;

            // Add vertices for side face
            vertices.push([corner_points[i].0, corner_heights[i], corner_points[i].1]);
            vertices.push([corner_points[i].0, bottom_y, corner_points[i].1]);
            vertices.push([corner_points[(i + 1) % 6].0, corner_heights[(i + 1) % 6], corner_points[(i + 1) % 6].1]);
            vertices.push([corner_points[(i + 1) % 6].0, bottom_y, corner_points[(i + 1) % 6].1]);

            // Calculate side normal
            let side_normal = calculate_normal(
                Vec3::new(corner_points[i].0, corner_heights[i], corner_points[i].1),
                Vec3::new(corner_points[i].0, bottom_y, corner_points[i].1),
                Vec3::new(corner_points[(i + 1) % 6].0, corner_heights[(i + 1) % 6], corner_points[(i + 1) % 6].1),
            );

            // Add normals and UVs for side face
            for _ in 0..4 {
                normals.push([side_normal.x, side_normal.y, side_normal.z]);
                uvs.push([0.0, 0.0]);
            }

            // Add indices for side face
            indices.extend_from_slice(&[
                side_start, side_start + 1, side_start + 2,
                side_start + 1, side_start + 3, side_start + 2,
            ]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(Indices::U32(indices)));

    mesh
}

fn create_terrain_materials(
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Vec<Handle<StandardMaterial>> {
    vec![
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.4, 0.8, 0.3),
            metallic: 0.0,
            perceptual_roughness: 0.6,
            reflectance: 0.2,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Plain
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.6, 0.6, 0.4),
            metallic: 0.0,
            perceptual_roughness: 0.8,
            reflectance: 0.1,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Rough
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.2, 0.4, 0.8),
            metallic: 0.0,
            perceptual_roughness: 0.1,
            reflectance: 0.5,
            alpha_mode: AlphaMode::Blend,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Water
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.5, 0.5, 0.5),
            metallic: 0.8,
            perceptual_roughness: 0.2,
            reflectance: 0.5,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Wall
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.9, 0.85, 0.6),
            metallic: 0.0,
            perceptual_roughness: 0.9,
            reflectance: 0.1,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Sand
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.95, 0.95, 0.95),
            metallic: 0.1,
            perceptual_roughness: 0.3,
            reflectance: 0.4,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Snow
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.3, 0.4, 0.3),
            metallic: 0.0,
            perceptual_roughness: 0.7,
            reflectance: 0.2,
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Swamp
        materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.2, 0.0),
            metallic: 0.0,
            perceptual_roughness: 0.3,
            reflectance: 0.3,
            emissive: Color::rgb(0.5, 0.0, 0.0),
            double_sided: true,
            cull_mode: None,
            ..default()
        }), // Lava
    ]
}

fn calculate_normal(v1: Vec3, v2: Vec3, v3: Vec3) -> Vec3 {
    let u = v2 - v1;
    let v = v3 - v1;
    u.cross(v).normalize()
}

fn spawn_chunk(
    commands: &mut Commands,
    chunk: &MapChunk,
    mesh_handle: &mut ResMut<Assets<Mesh>>,
    materials: &Vec<Handle<StandardMaterial>>,
) {
    for (pos, cell) in chunk.grid.iter_cells() {
        let (x, z) = hex_to_world_coords(pos);
        let y = 0.0; // Height is now handled in the mesh

        // Get neighbor elevations and terrains
        let mut neighbor_info = [(0, TerrainType::Plain); 6];
        for (i, dir) in HEX_DIRECTIONS.iter().enumerate() {
            let neighbor_pos = HexPosition::new(pos.q + dir.0, pos.r + dir.1, pos.z);
            if let Some(neighbor) = chunk.grid.get_cell(&neighbor_pos) {
                neighbor_info[i] = (neighbor.elevation, neighbor.terrain);
            }
        }

        // Create hex mesh with proper elevation transitions
        let hex_mesh = create_hex_mesh_with_elevation(
            cell.elevation,
            cell.terrain,
            &neighbor_info,
        );
        let mesh_handle = mesh_handle.add(hex_mesh);

        let material = match cell.terrain {
            TerrainType::Plain => &materials[0],
            TerrainType::Rough => &materials[1],
            TerrainType::Water => &materials[2],
            TerrainType::Wall => &materials[3],
            TerrainType::Sand => &materials[4],
            TerrainType::Snow => &materials[5],
            TerrainType::Swamp => &materials[6],
            TerrainType::Lava => &materials[7],
        };

        commands.spawn((
            PbrBundle {
                mesh: mesh_handle,
                material: material.clone(),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            PickableBundle::default(),
            HexTile { position: *pos },
        ));
    }
}

fn hex_to_world_coords(hex: &HexPosition) -> (f32, f32) {
    let size = HEX_RADIUS * (1.0 + HEX_SPACING);
    let x = size * SQRT_3 * hex.q as f32;
    let z = size * 1.5 * hex.r as f32;
    let x = if hex.r % 2 == 0 {
        x
    } else {
        x + size * SQRT_3 * 0.5
    };
    (x, z)
}

#[derive(Component)]
struct HexTile {
    position: HexPosition,
}

fn handle_input(
    keyboard: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
) {
    let mut camera = camera_query.single_mut();
    let move_speed = 0.1;
    let rotate_speed = 0.02;

    let mut movement = Vec3::ZERO;

    // Calculate movement vector
    if keyboard.pressed(KeyCode::W) {
        movement += camera.forward();
    }
    if keyboard.pressed(KeyCode::S) {
        movement -= camera.forward();
    }
    if keyboard.pressed(KeyCode::A) {
        movement -= camera.right();
    }
    if keyboard.pressed(KeyCode::D) {
        movement += camera.right();
    }
    if keyboard.pressed(KeyCode::Q) {
        movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::E) {
        movement.y -= 1.0;
    }

    // Apply movement
    if movement != Vec3::ZERO {
        movement = movement.normalize() * move_speed;
        camera.translation += movement;
    }

    // Camera rotation
    if keyboard.pressed(KeyCode::Left) {
        camera.rotate_y(rotate_speed);
    }
    if keyboard.pressed(KeyCode::Right) {
        camera.rotate_y(-rotate_speed);
    }
    if keyboard.pressed(KeyCode::Up) {
        camera.rotate_local_x(-rotate_speed);
    }
    if keyboard.pressed(KeyCode::Down) {
        camera.rotate_local_x(rotate_speed);
    }
}

fn update_world_view(
    mut lines: ResMut<DebugLines>,
) {
    // Draw world axes
    lines.line_colored(
        Vec3::ZERO,
        Vec3::X * 10.0,
        0.0,
        Color::rgba(1.0, 0.0, 0.0, 0.5),
    );
    lines.line_colored(
        Vec3::ZERO,
        Vec3::Y * 10.0,
        0.0,
        Color::rgba(0.0, 1.0, 0.0, 0.5),
    );
    lines.line_colored(
        Vec3::ZERO,
        Vec3::Z * 10.0,
        0.0,
        Color::rgba(0.0, 0.0, 1.0, 0.5),
    );
}

fn draw_grid(
    mut lines: ResMut<DebugLines>,
) {
    let grid_size = 20;
    let grid_spacing = 1.0;
    let grid_color = Color::rgba(0.5, 0.5, 0.5, 0.2);

    // Draw grid lines
    for i in -grid_size..=grid_size {
        let offset = i as f32 * grid_spacing;

        // X lines
        lines.line_colored(
            Vec3::new(-grid_size as f32, 0.0, offset),
            Vec3::new(grid_size as f32, 0.0, offset),
            0.0,
            grid_color,
        );

        // Z lines
        lines.line_colored(
            Vec3::new(offset, 0.0, -grid_size as f32),
            Vec3::new(offset, 0.0, grid_size as f32),
            0.0,
            grid_color,
        );
    }
}

fn handle_hex_hover(
    mut world_state: ResMut<WorldState>,
    hex_query: Query<(&HexTile, &Interaction)>,
) {
    for (hex, interaction) in hex_query.iter() {
        if *interaction == Interaction::Hovered {
            world_state.selected_hex = Some(hex.position);
            break;
        }
    }
}

// Hex neighbor directions in (q, r) offsets
const HEX_DIRECTIONS: [(i32, i32); 6] = [
    (1, 0), (0, 1), (-1, 1),
    (-1, 0), (0, -1), (1, -1),
];
