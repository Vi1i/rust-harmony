use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    grid::{HexGrid, TerrainType},
    HexPosition,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StructureModification {
    AddFloor {
        level: i32,
        terrain: TerrainType,
    },
    AddWall {
        position: HexOffset,
        height: i32,
    },
    AddRoof {
        style: RoofStyle,
        height: i32,
    },
    AddDecoration {
        decoration_type: String,
        position: HexOffset,
    },
    ModifyTerrain {
        position: HexOffset,
        terrain: TerrainType,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RoofStyle {
    Flat,
    Peaked { slope: f32 },
    Domed { radius: i32 },
    Tiered { levels: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub description: String,
    pub rules: Vec<Rule>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Condition {
    TerrainType { terrain: TerrainType },
    ElevationRange { min: i32, max: i32 },
    AdjacentTo { structure_type: String },
    MinDistanceFrom { structure_type: String, distance: i32 },
    MaxDistanceFrom { structure_type: String, distance: i32 },
    BiomeType { biome: String },
    NearWater { distance: i32 },
    HasTag { tag: String },
    PopulationDensity { min: f32, max: f32 },
    ResourceAvailable { resource: String, amount: i32 },
    RoadAccess { distance: i32 },
    SlopeRange { min_degrees: f32, max_degrees: f32 },
    ViewDistance { min: i32 },
    WindExposure { min: f32, max: f32 },
    SunExposure { min: f32, max: f32 },
    TemplateExists { template_name: String },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum Action {
    PlaceStructure { structure: StructureTemplate },
    SetTerrain { terrain: TerrainType },
    SetElevation { elevation: i32 },
    AddTag { tag: String },
    GenerateWall { height: i32, material: TerrainType },
    ApplyTemplate { template_name: String },
    GenerateRoad { 
        width: i32,
        material: TerrainType,
        to: HexPosition,
        style: RoadStyle,
    },
    PlaceStructureCluster {
        structure: StructureTemplate,
        count: i32,
        spacing: i32,
        variation: bool,
    },
    ModifyTerrain {
        radius: i32,
        operation: TerrainOperation,
    },
    SpawnResource {
        resource_type: String,
        amount: i32,
        spread: i32,
    },
    SetBiome { biome: String },
    CreateWaterFeature {
        feature_type: WaterFeatureType,
        size: i32,
    },
    ApplyNoise {
        noise_type: NoiseType,
        amplitude: f32,
        frequency: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureTemplate {
    pub name: String,
    pub structure_type: String,
    pub footprint: Vec<HexOffset>,
    pub required_terrain: Option<TerrainType>,
    pub elevation_requirements: Option<ElevationRequirement>,
    pub tags: Vec<String>,
    pub parent_template: Option<String>,
    pub variants: Vec<StructureVariant>,
    pub generation_rules: GenerationRules,
    pub connections: Vec<ConnectionPoint>,
    pub interior_layout: Option<InteriorLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureVariant {
    pub name: String,
    pub probability: f32,
    pub modifications: Vec<StructureModification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRules {
    pub min_spacing: i32,
    pub max_count: i32,
    pub alignment: AlignmentRule,
    pub growth_pattern: GrowthPattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPoint {
    pub position: HexOffset,
    pub connection_type: ConnectionType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteriorLayout {
    pub rooms: Vec<Room>,
    pub corridors: Vec<Corridor>,
    pub entrances: Vec<HexOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ConnectionType {
    Road,
    Wall,
    Bridge,
    Door,
    Path,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AlignmentRule {
    Grid { spacing: i32 },
    Radial { center: HexOffset, rings: i32 },
    Organic { min_spacing: i32 },
    Linear { direction: i32, spacing: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GrowthPattern {
    Outward,
    Inward,
    Linear { direction: i32 },
    Clustered { cluster_size: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub size: (i32, i32),
    pub purpose: String,
    pub required_connections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Corridor {
    pub start: HexOffset,
    pub end: HexOffset,
    pub width: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RoadStyle {
    Straight,
    Winding { variation: f32 },
    Organic { roughness: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TerrainOperation {
    Smooth,
    Roughen { intensity: f32 },
    Raise { amount: i32 },
    Lower { amount: i32 },
    Flatten { target: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WaterFeatureType {
    Lake,
    River { width: i32 },
    Ocean,
    Pond,
    Canal { width: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NoiseType {
    Perlin,
    Simplex,
    Worley,
    Ridged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexOffset {
    pub q: i32,
    pub r: i32,
    pub terrain: TerrainType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationRequirement {
    pub min: i32,
    pub max: i32,
    pub relative_to_base: bool,
}

#[derive(Debug)]
pub struct TemplateEngine {
    templates: HashMap<String, Template>,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    pub fn load_template(&mut self, yaml: &str) -> Result<(), serde_yaml::Error> {
        let template: Template = serde_yaml::from_str(yaml)?;
        self.templates.insert(template.name.clone(), template);
        Ok(())
    }

    pub fn apply_template(&self, name: &str, grid: &mut HexGrid, position: &HexPosition) -> bool {
        if let Some(template) = self.templates.get(name) {
            let mut rules = template.rules.clone();
            rules.sort_by_key(|r| -r.priority); // Higher priority rules first

            for rule in rules {
                if self.evaluate_conditions(&rule.conditions, grid, position) {
                    self.apply_actions(&rule.actions, grid, position);
                    return true;
                }
            }
        }
        false
    }

    fn evaluate_conditions(&self, conditions: &[Condition], grid: &HexGrid, position: &HexPosition) -> bool {
        conditions.iter().all(|condition| {
            match condition {
                Condition::TerrainType { terrain } => {
                    if let Some(cell) = grid.get_cell(position) {
                        cell.terrain == *terrain
                    } else {
                        false
                    }
                },
                Condition::ElevationRange { min, max } => {
                    if let Some(cell) = grid.get_cell(position) {
                        cell.elevation >= *min && cell.elevation <= *max
                    } else {
                        false
                    }
                },
                // Add more condition evaluations here
                _ => false, // Placeholder for other conditions
            }
        })
    }

    fn apply_actions(&self, actions: &[Action], grid: &mut HexGrid, position: &HexPosition) {
        for action in actions {
            match action {
                Action::SetTerrain { terrain } => {
                    if let Some(cell) = grid.get_cell(position) {
                        grid.add_cell(position.clone(), *terrain, cell.elevation);
                    }
                },
                Action::SetElevation { elevation } => {
                    if let Some(cell) = grid.get_cell(position) {
                        grid.add_cell(position.clone(), cell.terrain, *elevation);
                    }
                },
                // Add more action implementations here
                _ => (), // Placeholder for other actions
            }
        }
    }
}
