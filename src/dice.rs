use rand::Rng;

/// Represents the result of a dice roll
#[derive(Debug, Clone)]
pub struct RollResult {
    pub value: i32,
    pub dice_count: i32,
    pub dice_type: i32,
    pub modifier: i32,
}

/// Roll dice in standard RPG notation (e.g., "2d6+3")
pub fn roll(dice_count: i32, dice_type: i32, modifier: i32) -> RollResult {
    let mut rng = rand::thread_rng();
    let value: i32 = (0..dice_count)
        .map(|_| rng.gen_range(1..=dice_type))
        .sum::<i32>() + modifier;

    RollResult {
        value,
        dice_count,
        dice_type,
        modifier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll_in_range() {
        let result = roll(2, 6, 0);
        assert!(result.value >= 2 && result.value <= 12);
    }
}
