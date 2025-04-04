use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use crate::{HexPosition, grid::{HexGrid, TerrainType}};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    Forest,
    Mountain,
    Plains,
    Desert,
    Ocean,
    Tundra,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StructureType {
    Building(BuildingType),
    Vegetation(VegetationType),
    Landmark(LandmarkType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    House,
    Shop,
    Temple,
    Castle,
    Tower,
    Inn,
    Stable,
    Wall,
    Gate,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VegetationType {
    Tree,
    Bush,
    Flower,
    Grass,
    DeadTree,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LandmarkType {
    Mountain,
    Hill,
    Rock,
    Statue,
    Well,
    Bridge,
}

#[derive(Debug, Clone)]
pub struct MapChunk {
    pub position: ChunkPosition,
    pub grid: HexGrid,
    pub structures: HashMap<HexPosition, StructureType>,
    pub biome: BiomeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

pub struct WorldMap {
    chunks: HashMap<ChunkPosition, MapChunk>,
    chunk_size: i32,
    rng: rand::rngs::StdRng,
}

impl WorldMap {
    pub fn new(chunk_size: i32) -> Self {
        Self::with_seed(chunk_size, rand::random())
    }

    pub fn with_seed(chunk_size: i32, seed: u64) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }

    pub fn get_or_generate_chunk(&mut self, position: ChunkPosition) -> &MapChunk {
        if !self.chunks.contains_key(&position) {
            let chunk = self.generate_chunk(position);
            self.chunks.insert(position, chunk);
        }
        self.chunks.get(&position).unwrap()
    }

    pub fn get_chunk(&self, position: &ChunkPosition) -> Option<&MapChunk> {
        self.chunks.get(position)
    }

    pub fn get_chunk_position_for_hex(&self, hex: &HexPosition) -> ChunkPosition {
        ChunkPosition {
            x: (hex.q as f32 / self.chunk_size as f32).floor() as i32,
            y: (hex.r as f32 / self.chunk_size as f32).floor() as i32,
        }
    }

    fn generate_chunk(&mut self, position: ChunkPosition) -> MapChunk {
        let biome = self.determine_biome(position);
        let mut grid = HexGrid::new();
        let mut structures = HashMap::new();

        // Generate base terrain
        for q in 0..self.chunk_size {
            for r in 0..self.chunk_size {
                let hex_pos = HexPosition::new_2d(
                    position.x * self.chunk_size + q,
                    position.y * self.chunk_size + r,
                );
                let terrain = self.get_terrain_for_biome(&biome);
                let elevation = match &biome {
                    BiomeType::Mountain => self.rng.gen_range(5..15),
                    BiomeType::Plains => self.rng.gen_range(0..3),
                    BiomeType::Forest => self.rng.gen_range(1..5),
                    BiomeType::Desert => self.rng.gen_range(0..2),
                    BiomeType::Ocean => -1,
                    BiomeType::Tundra => self.rng.gen_range(2..7),
                };
                grid.add_cell(hex_pos, terrain, elevation);

                // Add structures based on biome and terrain
                if let Some(structure) = self.generate_structure(&biome, &terrain) {
                    structures.insert(hex_pos, structure);
                }
            }
        }

        MapChunk {
            position,
            grid,
            structures,
            biome,
        }
    }

    fn determine_biome(&mut self, _position: ChunkPosition) -> BiomeType {
        // TODO: Implement proper biome generation with noise
        match self.rng.gen_range(0..6) {
            0 => BiomeType::Forest,
            1 => BiomeType::Mountain,
            2 => BiomeType::Plains,
            3 => BiomeType::Desert,
            4 => BiomeType::Ocean,
            _ => BiomeType::Tundra,
        }
    }

    fn get_terrain_for_biome(&mut self, biome: &BiomeType) -> TerrainType {
        match biome {
            BiomeType::Forest => {
                if self.rng.gen_bool(0.7) {
                    TerrainType::Plain
                } else {
                    TerrainType::Rough
                }
            }
            BiomeType::Mountain => {
                if self.rng.gen_bool(0.8) {
                    TerrainType::Rough
                } else {
                    TerrainType::Wall
                }
            }
            BiomeType::Plains => TerrainType::Plain,
            BiomeType::Desert => {
                if self.rng.gen_bool(0.9) {
                    TerrainType::Plain
                } else {
                    TerrainType::Rough
                }
            }
            BiomeType::Ocean => TerrainType::Water,
            BiomeType::Tundra => {
                if self.rng.gen_bool(0.6) {
                    TerrainType::Plain
                } else {
                    TerrainType::Rough
                }
            }
        }
    }

    fn generate_structure(
        &mut self,
        biome: &BiomeType,
        terrain: &TerrainType,
    ) -> Option<StructureType> {
        match (biome, terrain) {
            (BiomeType::Forest, TerrainType::Plain) => {
                if self.rng.gen_bool(0.4) {
                    Some(StructureType::Vegetation(VegetationType::Tree))
                } else if self.rng.gen_bool(0.2) {
                    Some(StructureType::Vegetation(VegetationType::Bush))
                } else {
                    None
                }
            }
            (BiomeType::Mountain, TerrainType::Rough) => {
                if self.rng.gen_bool(0.3) {
                    Some(StructureType::Landmark(LandmarkType::Rock))
                } else {
                    None
                }
            }
            (BiomeType::Plains, TerrainType::Plain) => {
                if self.rng.gen_bool(0.1) {
                    Some(StructureType::Building(BuildingType::House))
                } else if self.rng.gen_bool(0.05) {
                    Some(StructureType::Landmark(LandmarkType::Well))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

pub struct MapGenerator {
    templates: HashMap<String, MapTemplate>,
    rng: rand::rngs::StdRng,
}

#[derive(Debug, Clone)]
pub struct MapTemplate {
    pub name: String,
    pub size: (i32, i32),
    pub terrain_distribution: HashMap<TerrainType, f32>,
    pub structure_distribution: HashMap<StructureType, f32>,
}

impl MapGenerator {
    pub fn new() -> Self {
        Self::with_seed(rand::random())
    }

    pub fn with_seed(seed: u64) -> Self {
        let mut generator = Self {
            templates: HashMap::new(),
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        };
        generator.initialize_templates();
        generator
    }

    fn initialize_templates(&mut self) {
        // Add a town template
        let mut town = MapTemplate {
            name: "Town".to_string(),
            size: (20, 20),
            terrain_distribution: HashMap::new(),
            structure_distribution: HashMap::new(),
        };
        town.terrain_distribution.insert(TerrainType::Plain, 0.8);
        town.terrain_distribution.insert(TerrainType::Rough, 0.2);
        town.structure_distribution.insert(StructureType::Building(BuildingType::House), 0.3);
        town.structure_distribution.insert(StructureType::Building(BuildingType::Shop), 0.1);
        town.structure_distribution.insert(StructureType::Building(BuildingType::Inn), 0.05);
        self.templates.insert("town".to_string(), town);

        // Add a forest template
        let mut forest = MapTemplate {
            name: "Forest".to_string(),
            size: (30, 30),
            terrain_distribution: HashMap::new(),
            structure_distribution: HashMap::new(),
        };
        forest.terrain_distribution.insert(TerrainType::Plain, 0.6);
        forest.terrain_distribution.insert(TerrainType::Rough, 0.4);
        forest.structure_distribution.insert(StructureType::Vegetation(VegetationType::Tree), 0.5);
        forest.structure_distribution.insert(StructureType::Vegetation(VegetationType::Bush), 0.2);
        self.templates.insert("forest".to_string(), forest);
    }

    pub fn generate_map(&mut self, template_name: &str) -> Option<MapChunk> {
        let template = self.templates.get(template_name)?.clone();
        let mut grid = HexGrid::new();
        let mut structures = HashMap::new();

        // Generate terrain based on distribution
        for q in 0..template.size.0 {
            for r in 0..template.size.1 {
                let pos = HexPosition::new_2d(q, r);
                let terrain = self.select_random_terrain(&template.terrain_distribution);
                let elevation = self.rng.gen_range(0..5); // Random elevation for template-based maps
                grid.add_cell(pos.clone(), terrain, elevation);

                // Add structures based on distribution
                if let Some(structure) = self.select_random_structure(
                    &template.structure_distribution,
                ) {
                    structures.insert(pos, structure);
                }
            }
        }

        Some(MapChunk {
            position: ChunkPosition { x: 0, y: 0 },
            grid,
            structures,
            biome: BiomeType::Plains, // Default biome for template-based maps
        })
    }

    fn select_random_terrain(
        &mut self,
        distribution: &HashMap<TerrainType, f32>,
    ) -> TerrainType {
        let total: f32 = distribution.values().sum();
        let mut value = self.rng.gen::<f32>() * total;
        
        for (terrain, weight) in distribution.iter() {
            value -= weight;
            if value <= 0.0 {
                return terrain.clone();
            }
        }
        
        TerrainType::Plain // Default
    }

    fn select_random_structure(
        &mut self,
        distribution: &HashMap<StructureType, f32>,
    ) -> Option<StructureType> {
        if self.rng.gen::<f32>() > 0.3 { // 30% chance of having a structure
            return None;
        }

        let total: f32 = distribution.values().sum();
        let mut value = self.rng.gen::<f32>() * total;
        
        for (structure, weight) in distribution.iter() {
            value -= weight;
            if value <= 0.0 {
                return Some(structure.clone());
            }
        }
        
        None
    }
}
