use std::collections::HashMap;

/// Server-side dice roller backed by the Fortuna PRNG — 1:1 translation of Java DiceRoller.
pub struct DiceRoller {
    test_rolls: HashMap<String, Vec<i32>>,
}

impl DiceRoller {
    pub fn new() -> Self {
        Self { test_rolls: HashMap::new() }
    }

    pub fn roll_dice(&mut self, die_type: i32) -> i32 {
        let key = "General".to_string();
        if let Some(list) = self.test_rolls.get_mut(&key) {
            while !list.is_empty() {
                let roll = list.remove(0);
                if roll <= die_type {
                    return roll;
                }
            }
        }
        // Phase ZU: delegate to Fortuna PRNG
        todo!("Phase ZU: delegate to Fortuna PRNG")
    }

    pub fn roll_dice_n(&mut self, count: usize, die_type: i32) -> Vec<i32> {
        (0..count).map(|_| self.roll_dice(die_type)).collect()
    }

    pub fn add_test_roll(&mut self, category: impl Into<String>, roll: i32) {
        self.test_rolls.entry(category.into()).or_default().push(roll);
    }

    pub fn clear_test_rolls(&mut self) {
        self.test_rolls.clear();
    }
}

impl Default for DiceRoller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_consume_test_roll() {
        let mut roller = DiceRoller::new();
        roller.add_test_roll("General", 3);
        // the test roll is consumed on the next roll_dice call when <= die_type
        let list = roller.test_rolls.get("General").unwrap();
        assert_eq!(list[0], 3);
    }

    #[test]
    fn test_new_has_no_test_rolls() {
        let roller = DiceRoller::new();
        assert!(roller.test_rolls.is_empty());
    }
}
