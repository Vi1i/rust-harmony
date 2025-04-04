use crate::{Character, dice};

pub struct Combat {
    participants: Vec<Character>,
    current_turn: usize,
}

#[derive(Debug)]
pub struct AttackResult {
    pub hit: bool,
    pub damage: i32,
    pub critical: bool,
}

impl Combat {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            current_turn: 0,
        }
    }

    pub fn add_participant(&mut self, character: Character) {
        self.participants.push(character);
    }

    pub fn next_turn(&mut self) -> Option<&Character> {
        if self.participants.is_empty() {
            return None;
        }

        // Reset movement for the next character
        if let Some(character) = self.participants.get_mut(self.current_turn) {
            character.reset_movement();
        }

        self.current_turn = (self.current_turn + 1) % self.participants.len();
        self.participants.get(self.current_turn)
    }

    pub fn attack(&mut self, attacker_idx: usize, defender_idx: usize) -> Option<AttackResult> {
        let (attacker, defender) = match self.get_two_mut(attacker_idx, defender_idx) {
            Some(pair) => pair,
            None => return None,
        };

        // Basic attack roll (d20 + strength modifier)
        let attack_roll = dice::roll(1, 20, (attacker.stats.strength - 10) / 2);
        let defense = 10 + (defender.stats.dexterity - 10) / 2;

        if !defender.is_alive() {
            return None;
        }

        let critical = attack_roll.value == 20;
        let hit = critical || attack_roll.value >= defense;

        if hit {
            // Damage roll (1d6 + strength modifier)
            let mut damage = dice::roll(1, 6, (attacker.stats.strength - 10) / 2).value;
            if critical {
                damage *= 2;
            }

            defender.health.current -= damage;

            Some(AttackResult {
                hit: true,
                damage,
                critical,
            })
        } else {
            Some(AttackResult {
                hit: false,
                damage: 0,
                critical: false,
            })
        }
    }

    fn get_two_mut(&mut self, i: usize, j: usize) -> Option<(&mut Character, &mut Character)> {
        if i == j || i >= self.participants.len() || j >= self.participants.len() {
            return None;
        }

        // Safe because we checked the indices are different and in bounds
        unsafe {
            let ptr = self.participants.as_mut_ptr();
            Some((
                &mut *ptr.add(i),
                &mut *ptr.add(j),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::CharacterStats;
    use crate::item::RaceType;

    #[test]
    fn test_combat_turn_order() {
        let mut combat = Combat::new();
        
        let stats = CharacterStats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        };

        combat.add_participant(Character::new("Fighter 1".to_string(), RaceType::Human, stats.clone()));
        combat.add_participant(Character::new("Fighter 2".to_string(), RaceType::Elf, stats.clone()));

        let next = combat.next_turn().unwrap();
        assert_eq!(next.name, "Fighter 2");
    }
}
