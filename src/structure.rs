use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[allow(unused_imports)]
use crate::{
    grid::{HexGrid, TerrainType},
    HexPosition,
    template::StructureTemplate,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Structure {
    pub template: StructureTemplate,
    pub base_position: HexPosition,
    pub occupied_positions: HashSet<HexPosition>,
}

impl Structure {
    pub fn new(template: StructureTemplate, base_position: HexPosition) -> Self {
        let mut occupied_positions = HashSet::new();
        
        // Calculate all positions this structure occupies based on its footprint
        for offset in &template.footprint {
            let pos = HexPosition::new(
                base_position.q + offset.q,
                base_position.r + offset.r,
                base_position.z,
            );
            occupied_positions.insert(pos);
        }

        Self {
            template,
            base_position,
            occupied_positions,
        }
    }

    pub fn can_place_at(&self, grid: &HexGrid) -> bool {
        // Check if all required positions are available and meet requirements
        for pos in &self.occupied_positions {
            if let Some(cell) = grid.get_cell(pos) {
                // Check terrain requirements
                if let Some(req_terrain) = &self.template.required_terrain {
                    if cell.terrain != *req_terrain {
                        return false;
                    }
                }

                // Check elevation requirements
                if let Some(elev_req) = &self.template.elevation_requirements {
                    let base_elevation = if elev_req.relative_to_base {
                        grid.get_cell(&self.base_position)
                            .map(|c| c.elevation)
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    let target_elevation = cell.elevation - base_elevation;
                    if target_elevation < elev_req.min || target_elevation > elev_req.max {
                        return false;
                    }
                }
            } else {
                return false;
            }
        }
        true
    }

    pub fn apply_to_grid(&self, grid: &mut HexGrid) {
        for offset in &self.template.footprint {
            let pos = HexPosition::new(
                self.base_position.q + offset.q,
                self.base_position.r + offset.r,
                self.base_position.z,
            );
            
            if let Some(cell) = grid.get_cell(&pos) {
                grid.add_cell(pos, offset.terrain, cell.elevation);
            }
        }
    }
}
