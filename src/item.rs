use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentSlot {
    Head,
    Neck,
    Chest,
    Legs,
    Feet,
    Hands,
    RingLeft,
    RingRight,
    MainHand,
    OffHand,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    Equipment(EquipmentType),
    Consumable(ConsumableType),
    Quest,
    Ingredient,
    Miscellaneous,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentType {
    Helmet,
    Necklace,
    ChestPiece,
    Leggings,
    Boots,
    Gloves,
    Ring,
    Weapon(WeaponType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    OneHanded,
    TwoHanded,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsumableType {
    HealthPotion,
    ManaPotion,
    Scroll,
    Food,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RaceType {
    Human,
    Elf,
    Dwarf,
    Orc,
    // Add more races as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub item_type: ItemType,
    pub level_requirement: i32,
    pub value: i32,
    pub weight: f32,
    pub stats: Option<ItemStats>,
    pub allowed_races: HashSet<RaceType>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemStats {
    pub strength_bonus: i32,
    pub dexterity_bonus: i32,
    pub constitution_bonus: i32,
    pub intelligence_bonus: i32,
    pub wisdom_bonus: i32,
    pub charisma_bonus: i32,
    pub armor: i32,
    pub damage: Option<WeaponDamage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponDamage {
    pub min_damage: i32,
    pub max_damage: i32,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Slashing,
    Piercing,
    Blunt,
    Magic,
}

impl Item {
    pub fn new_equipment(
        name: String,
        equipment_type: EquipmentType,
        stats: ItemStats,
        allowed_races: HashSet<RaceType>,
        level_req: i32,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            item_type: ItemType::Equipment(equipment_type),
            level_requirement: level_req,
            value: 0, // Set appropriate value
            weight: 1.0, // Set appropriate weight
            stats: Some(stats),
            allowed_races,
            description: String::new(), // Set appropriate description
        }
    }

    pub fn new_consumable(
        name: String,
        consumable_type: ConsumableType,
        description: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            item_type: ItemType::Consumable(consumable_type),
            level_requirement: 0,
            value: 0,
            weight: 0.1,
            stats: None,
            allowed_races: RaceType::iter_all().collect(), // Available to all races
            description,
        }
    }

    pub fn can_equip(&self, slot: &EquipmentSlot, race: &RaceType) -> bool {
        if !self.allowed_races.contains(race) {
            return false;
        }

        match &self.item_type {
            ItemType::Equipment(eq_type) => matches!(
                (eq_type, slot),
                (EquipmentType::Helmet, EquipmentSlot::Head)
                    | (EquipmentType::Necklace, EquipmentSlot::Neck)
                    | (EquipmentType::ChestPiece, EquipmentSlot::Chest)
                    | (EquipmentType::Leggings, EquipmentSlot::Legs)
                    | (EquipmentType::Boots, EquipmentSlot::Feet)
                    | (EquipmentType::Gloves, EquipmentSlot::Hands)
                    | (EquipmentType::Ring, EquipmentSlot::RingLeft)
                    | (EquipmentType::Ring, EquipmentSlot::RingRight)
                    | (EquipmentType::Weapon(WeaponType::OneHanded), EquipmentSlot::MainHand)
                    | (EquipmentType::Weapon(WeaponType::OneHanded), EquipmentSlot::OffHand)
                    | (EquipmentType::Weapon(WeaponType::TwoHanded), EquipmentSlot::MainHand)
            ),
            _ => false,
        }
    }
}

impl RaceType {
    pub fn iter_all() -> impl Iterator<Item = RaceType> {
        vec![
            RaceType::Human,
            RaceType::Elf,
            RaceType::Dwarf,
            RaceType::Orc,
        ]
        .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equipment_creation() {
        let mut allowed_races = HashSet::new();
        allowed_races.insert(RaceType::Human);
        allowed_races.insert(RaceType::Elf);

        let sword = Item::new_equipment(
            "Steel Sword".to_string(),
            EquipmentType::Weapon(WeaponType::OneHanded),
            ItemStats {
                strength_bonus: 2,
                dexterity_bonus: 0,
                constitution_bonus: 0,
                intelligence_bonus: 0,
                wisdom_bonus: 0,
                charisma_bonus: 0,
                armor: 0,
                damage: Some(WeaponDamage {
                    min_damage: 2,
                    max_damage: 6,
                    damage_type: DamageType::Slashing,
                }),
            },
            allowed_races,
            1,
        );

        assert!(sword.can_equip(&EquipmentSlot::MainHand, &RaceType::Human));
        assert!(!sword.can_equip(&EquipmentSlot::MainHand, &RaceType::Orc));
    }

    #[test]
    fn test_consumable_creation() {
        let potion = Item::new_consumable(
            "Health Potion".to_string(),
            ConsumableType::HealthPotion,
            "Restores 50 HP".to_string(),
        );

        assert!(matches!(
            potion.item_type,
            ItemType::Consumable(ConsumableType::HealthPotion)
        ));
    }
}
