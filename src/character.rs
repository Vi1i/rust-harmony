use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{HexPosition, item::{Item, EquipmentSlot, RaceType, ItemType, EquipmentType, WeaponType}};

const INVENTORY_WEIGHT_LIMIT: f32 = 100.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub id: String,
    pub name: String,
    pub race: RaceType,
    pub position: HexPosition,
    pub stats: CharacterStats,
    pub health: Health,
    pub movement: Movement,
    pub inventory: Vec<Item>,
    pub equipment: HashMap<EquipmentSlot, Item>,
    pub level: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterStats {
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub wisdom: i32,
    pub charisma: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub maximum: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Movement {
    pub speed: i32,      // Number of hexes that can be moved per turn
    pub remaining: i32,  // Remaining movement points this turn
}

impl Character {
    pub fn new(name: String, race: RaceType, stats: CharacterStats) -> Self {
        let health = Health {
            current: 10 + stats.constitution,
            maximum: 10 + stats.constitution,
        };

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            race,
            position: HexPosition::new_2d(0, 0),
            stats,
            health,
            movement: Movement {
                speed: 6,
                remaining: 6,
            },
            inventory: Vec::new(),
            equipment: HashMap::new(),
            level: 1,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.health.current > 0
    }

    pub fn reset_movement(&mut self) {
        self.movement.remaining = self.movement.speed;
    }

    pub fn current_weight(&self) -> f32 {
        self.inventory.iter().map(|item| item.weight).sum()
    }

    pub fn can_carry(&self, item: &Item) -> bool {
        self.current_weight() + item.weight <= INVENTORY_WEIGHT_LIMIT
    }

    pub fn add_to_inventory(&mut self, item: Item) -> Result<(), String> {
        if !self.can_carry(&item) {
            return Err("Cannot carry more weight".to_string());
        }
        self.inventory.push(item);
        Ok(())
    }

    pub fn remove_from_inventory(&mut self, item_id: &str) -> Option<Item> {
        if let Some(pos) = self.inventory.iter().position(|item| item.id == item_id) {
            Some(self.inventory.remove(pos))
        } else {
            None
        }
    }

    pub fn equip_item(&mut self, item_id: &str) -> Result<(), String> {
        let item_pos = self.inventory
            .iter()
            .position(|item| item.id == item_id)
            .ok_or("Item not found in inventory".to_string())?;

        let item = &self.inventory[item_pos];

        // Check if it's equipment
        let equipment_type = match &item.item_type {
            ItemType::Equipment(eq_type) => eq_type,
            _ => return Err("Item is not equipment".to_string()),
        };

        // Check level requirement
        if item.level_requirement > self.level {
            return Err("Level requirement not met".to_string());
        }

        // Get the appropriate slot
        let slot = match equipment_type {
            EquipmentType::Helmet => EquipmentSlot::Head,
            EquipmentType::Necklace => EquipmentSlot::Neck,
            EquipmentType::ChestPiece => EquipmentSlot::Chest,
            EquipmentType::Leggings => EquipmentSlot::Legs,
            EquipmentType::Boots => EquipmentSlot::Feet,
            EquipmentType::Gloves => EquipmentSlot::Hands,
            EquipmentType::Ring => {
                // Try left ring first, then right ring if left is occupied
                if !self.equipment.contains_key(&EquipmentSlot::RingLeft) {
                    EquipmentSlot::RingLeft
                } else if !self.equipment.contains_key(&EquipmentSlot::RingRight) {
                    EquipmentSlot::RingRight
                } else {
                    return Err("No free ring slots".to_string());
                }
            }
            EquipmentType::Weapon(weapon_type) => {
                match weapon_type {
                    WeaponType::OneHanded => {
                        if !self.equipment.contains_key(&EquipmentSlot::MainHand) {
                            EquipmentSlot::MainHand
                        } else if !self.equipment.contains_key(&EquipmentSlot::OffHand) {
                            EquipmentSlot::OffHand
                        } else {
                            return Err("No free hand slots".to_string());
                        }
                    }
                    WeaponType::TwoHanded => {
                        if self.equipment.contains_key(&EquipmentSlot::MainHand) ||
                           self.equipment.contains_key(&EquipmentSlot::OffHand) {
                            return Err("Hands not free for two-handed weapon".to_string());
                        }
                        EquipmentSlot::MainHand
                    }
                }
            }
        };

        // Check if the item can be equipped in this slot
        if !item.can_equip(&slot, &self.race) {
            return Err("Cannot equip this item in this slot".to_string());
        }

        // If there's an item in the slot, unequip it first
        if let Some(old_item) = self.equipment.remove(&slot) {
            self.inventory.push(old_item);
        }

        // Remove the item from inventory and equip it
        let item = self.inventory.remove(item_pos);
        self.equipment.insert(slot, item);

        Ok(())
    }

    pub fn unequip_item(&mut self, slot: &EquipmentSlot) -> Result<(), String> {
        if let Some(item) = self.equipment.remove(slot) {
            if self.can_carry(&item) {
                self.inventory.push(item);
                Ok(())
            } else {
                // Put the item back if we can't carry it
                self.equipment.insert(*slot, item);
                Err("Inventory too full to unequip".to_string())
            }
        } else {
            Err("No item equipped in that slot".to_string())
        }
    }

    pub fn get_total_stats(&self) -> CharacterStats {
        let mut total = self.stats.clone();

        // Add bonuses from all equipped items
        for item in self.equipment.values() {
            if let Some(item_stats) = &item.stats {
                total.strength += item_stats.strength_bonus;
                total.dexterity += item_stats.dexterity_bonus;
                total.constitution += item_stats.constitution_bonus;
                total.intelligence += item_stats.intelligence_bonus;
                total.wisdom += item_stats.wisdom_bonus;
                total.charisma += item_stats.charisma_bonus;
            }
        }

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_character_creation() {
        let stats = CharacterStats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };
        let character = Character::new("Test Character".to_string(), RaceType::Human, stats);
        assert_eq!(character.health.maximum, 20);
        assert!(character.is_alive());
    }
}
