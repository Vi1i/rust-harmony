//! Tabletop RPG System Library
//! 
//! A modular system for managing tabletop RPG mechanics, including
//! character generation, hex-based combat, and movement systems.

use serde::{Deserialize, Serialize};

pub mod character;
pub mod combat;
pub mod grid;
pub mod dice;
pub mod item;
pub mod map;

// Re-export commonly used types
pub use character::Character;
pub use combat::Combat;
pub use grid::{HexGrid, TerrainType};
pub use map::{WorldMap, MapGenerator, BiomeType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexPosition {
    pub q: i32,  // axial coordinate q
    pub r: i32,  // axial coordinate r
    pub z: i32,  // vertical coordinate (elevation/height)
}

impl HexPosition {
    pub fn new(q: i32, r: i32, z: i32) -> Self {
        Self { q, r, z }
    }

    pub fn new_2d(q: i32, r: i32) -> Self {
        Self { q, r, z: 0 }
    }

    /// Calculate the cube coordinates (used for distance calculations)
    pub fn cube_coords(&self) -> (i32, i32, i32) {
        let x = self.q;
        let z = self.r;
        let y = -x - z;
        (x, y, z)
    }

    /// Calculate the 3D distance between two hex positions
    pub fn distance(&self, other: &HexPosition) -> i32 {
        let (x1, y1, z1) = self.cube_coords();
        let (x2, y2, z2) = other.cube_coords();
        
        // Manhattan distance in 3D cube coordinates plus vertical distance
        let planar_distance = ((x1 - x2).abs() + (y1 - y2).abs() + (z1 - z2).abs()) / 2;
        let height_difference = (self.z - other.z).abs();
        
        planar_distance + height_difference
    }
}

/// Represents a cardinal direction in the hex grid
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_position() {
        let pos = HexPosition::new_2d(0, 0);
        assert_eq!(pos.q, 0);
        assert_eq!(pos.r, 0);
        assert_eq!(pos.z, 0);

        let pos3d = HexPosition::new(1, 2, 3);
        assert_eq!(pos3d.q, 1);
        assert_eq!(pos3d.r, 2);
        assert_eq!(pos3d.z, 3);
    }

    #[test]
    fn test_hex_distance() {
        let pos1 = HexPosition::new(0, 0, 0);
        let pos2 = HexPosition::new(1, 1, 2);
        
        // Distance should include both planar and vertical components
        assert_eq!(pos1.distance(&pos2), 4); // 2 steps in plane + 2 steps up
    }
}
