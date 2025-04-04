use std::collections::{HashMap, BinaryHeap, HashSet};
use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use crate::HexPosition;

#[derive(Debug, Clone)]
pub struct HexGrid {
    cells: HashMap<HexPosition, Cell>,
    size: (i32, i32), // width, height
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub position: HexPosition,
    pub terrain: TerrainType,
    pub movement_cost: i32,
    pub elevation: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TerrainType {
    Plain,      // Basic traversable terrain
    Rough,      // Difficult terrain like thick brush or rocky ground
    Water,
    Wall,
    Sand,
    Snow,
    Swamp,
    Lava,
}

#[derive(Eq, PartialEq)]
struct Node {
    position: HexPosition,
    cost: i32,
    priority: i32,
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl HexGrid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            size: (0, 0),
        }
    }

    pub fn with_size(width: i32, height: i32) -> Self {
        Self {
            cells: HashMap::new(),
            size: (width, height),
        }
    }

    pub fn add_cell(&mut self, mut position: HexPosition, terrain: TerrainType, elevation: i32) {
        position.z = elevation;

        // Calculate base movement cost
        let base_cost = match terrain {
            TerrainType::Plain => 1,
            TerrainType::Rough => 2,
            TerrainType::Water => 3,
            TerrainType::Wall => i32::MAX,
            TerrainType::Sand => 2,
            TerrainType::Snow => 2,
            TerrainType::Swamp => 3,
            TerrainType::Lava => i32::MAX,
        };

        // Add extra cost for significant elevation changes
        let movement_cost = if let Some(neighbor_cells) = self.get_neighbors(position.clone())
            .iter()
            .filter_map(|pos| self.get_cell(pos))
            .next()
        {
            let elevation_diff = (elevation - neighbor_cells.elevation).abs();
            if elevation_diff > 1 {
                base_cost + elevation_diff * 2  // Make steep climbs more costly
            } else {
                base_cost
            }
        } else {
            base_cost
        };

        self.cells.insert(
            position.clone(),
            Cell {
                position,
                terrain,
                movement_cost,
                elevation,
            },
        );

        // Update grid size if necessary
        self.size.0 = self.size.0.max(position.q + 1);
        self.size.1 = self.size.1.max(position.r + 1);
    }

    pub fn get_neighbors(&self, position: HexPosition) -> Vec<HexPosition> {
        let mut neighbors = Vec::new();

        // Add horizontal neighbors
        let directions = [
            (1, 0),   // East
            (1, -1),  // Northeast
            (0, -1),  // Northwest
            (-1, 0),  // West
            (-1, 1),  // Southwest
            (0, 1),   // Southeast
        ];

        // Add horizontal neighbors
        for (dq, dr) in directions.iter() {
            let neighbor = HexPosition::new(
                position.q + dq,
                position.r + dr,
                position.z
            );
            if self.is_in_bounds(&neighbor) {
                neighbors.push(neighbor);
            }
        }

        // Add vertical neighbors
        let up = HexPosition::new(position.q, position.r, position.z + 1);
        let down = HexPosition::new(position.q, position.r, position.z - 1);
        
        if self.is_in_bounds(&up) {
            neighbors.push(up);
        }
        if self.is_in_bounds(&down) {
            neighbors.push(down);
        }

        neighbors
    }

    pub fn is_in_bounds(&self, position: &HexPosition) -> bool {
        let in_grid = position.q >= 0 && position.q < self.size.0 && 
                     position.r >= 0 && position.r < self.size.1;
        
        if !in_grid {
            return false;
        }

        // Check elevation bounds and terrain-specific rules
        if let Some(cell) = self.get_cell(position) {
            match cell.terrain {
                // Water can't be too high above sea level
                TerrainType::Water => cell.elevation <= 0,
                // Snow only appears at high elevations
                TerrainType::Snow => cell.elevation >= 5,
                // Lava only appears at low elevations
                TerrainType::Lava => cell.elevation <= 2,
                // Other terrain types can be at any reasonable elevation
                _ => cell.elevation >= -10 && cell.elevation <= 15
            }
        } else {
            // If no cell exists yet, use default elevation bounds
            position.z >= -10 && position.z <= 15
        }
    }

    pub fn distance(&self, from: HexPosition, to: HexPosition) -> i32 {
        from.distance(&to)
    }

    pub fn find_path(&self, start: HexPosition, goal: HexPosition) -> Option<Vec<HexPosition>> {
        if !self.is_in_bounds(&start) || !self.is_in_bounds(&goal) {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut closed_set = HashSet::new();

        g_score.insert(start.clone(), 0);
        open_set.push(Node {
            position: start.clone(),
            cost: 0,
            priority: 0,
        });

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                return Some(self.reconstruct_path(came_from, current.position));
            }

            if closed_set.contains(&current.position) {
                continue;
            }
            closed_set.insert(current.position.clone());

            for neighbor in self.get_neighbors(current.position.clone()) {
                if closed_set.contains(&neighbor) {
                    continue;
                }

                let neighbor_cell = match self.cells.get(&neighbor) {
                    Some(cell) => cell,
                    None => continue,
                };

                if neighbor_cell.movement_cost == i32::MAX {
                    continue;
                }

                let tentative_g_score = g_score.get(&current.position).unwrap() + 
                    neighbor_cell.movement_cost + 
                    self.elevation_cost(&current.position, &neighbor);

                if !g_score.contains_key(&neighbor) || 
                   tentative_g_score < *g_score.get(&neighbor).unwrap() {
                    came_from.insert(neighbor.clone(), current.position.clone());
                    g_score.insert(neighbor.clone(), tentative_g_score);
                    
                    let h_score = self.distance(neighbor.clone(), goal.clone());
                    let f_score = tentative_g_score + h_score;

                    open_set.push(Node {
                        position: neighbor,
                        cost: tentative_g_score,
                        priority: f_score,
                    });
                }
            }
        }

        None
    }

    fn elevation_cost(&self, from: &HexPosition, to: &HexPosition) -> i32 {
        let from_cell = match self.cells.get(from) {
            Some(cell) => cell,
            None => return i32::MAX,
        };
        let to_cell = match self.cells.get(to) {
            Some(cell) => cell,
            None => return i32::MAX,
        };

        let elevation_diff = (to_cell.elevation - from_cell.elevation).abs();

        // Calculate base cost based on elevation difference
        let base_cost = if elevation_diff <= 1 {
            elevation_diff  // Normal step
        } else {
            elevation_diff * 2  // Steep climb/descent
        };

        // Apply terrain-specific elevation modifiers
        match (from_cell.terrain, to_cell.terrain) {
            // Moving from water to land or vice versa is extra costly
            (TerrainType::Water, _) | (_, TerrainType::Water) => {
                base_cost * 2
            },
            // Snow terrain makes elevation changes more difficult
            (TerrainType::Snow, _) | (_, TerrainType::Snow) => {
                base_cost * 2
            },
            // Rough terrain increases elevation cost
            (TerrainType::Rough, _) | (_, TerrainType::Rough) => {
                (base_cost as f32 * 1.5) as i32
            },
            // Wall terrain is impassable regardless of elevation
            (TerrainType::Wall, _) | (_, TerrainType::Wall) => {
                i32::MAX
            },
            // Lava terrain is impassable
            (TerrainType::Lava, _) | (_, TerrainType::Lava) => {
                i32::MAX
            },
            // Default case
            _ => base_cost
        }
    }

    fn reconstruct_path(&self, came_from: HashMap<HexPosition, HexPosition>, mut current: HexPosition) -> Vec<HexPosition> {
        let mut path = vec![current.clone()];
        while let Some(previous) = came_from.get(&current) {
            path.push(previous.clone());
            current = previous.clone();
        }
        path.reverse();
        path
    }

    pub fn get_cell(&self, position: &HexPosition) -> Option<&Cell> {
        self.cells.get(position)
    }

    pub fn get_size(&self) -> (i32, i32) {
        self.size
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = (&HexPosition, &Cell)> {
        self.cells.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance() {
        let grid = HexGrid::new();
        let pos1 = HexPosition::new(0, 0, 0);
        let pos2 = HexPosition::new(1, 1, 2);
        assert_eq!(grid.distance(pos1, pos2), 4); // 2 steps in plane + 2 steps up
    }
}
