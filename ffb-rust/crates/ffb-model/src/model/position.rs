use crate::enums::{PlayerType, PlayerGender};

/// 1:1 translation of com.fumbbl.ffb.model.Position (Java interface).
pub trait Position {
    fn get_type(&self) -> PlayerType;
    fn get_gender(&self) -> PlayerGender;
    fn get_movement(&self) -> i32;
    fn get_strength(&self) -> i32;
    fn get_agility(&self) -> i32;
    fn get_passing(&self) -> i32;
    fn get_armour(&self) -> i32;
    fn get_cost(&self) -> i32;
    fn get_name(&self) -> &str;
    fn get_shorthand(&self) -> &str;
    fn get_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender};

    struct TestPos;
    impl Position for TestPos {
        fn get_type(&self) -> PlayerType { PlayerType::Regular }
        fn get_gender(&self) -> PlayerGender { PlayerGender::Male }
        fn get_movement(&self) -> i32 { 6 }
        fn get_strength(&self) -> i32 { 3 }
        fn get_agility(&self) -> i32 { 3 }
        fn get_passing(&self) -> i32 { 4 }
        fn get_armour(&self) -> i32 { 9 }
        fn get_cost(&self) -> i32 { 50000 }
        fn get_name(&self) -> &str { "Lineman" }
        fn get_shorthand(&self) -> &str { "Lin" }
        fn get_id(&self) -> &str { "pos_1" }
    }

    #[test]
    fn get_name_returns_name() {
        assert_eq!(TestPos.get_name(), "Lineman");
    }

    #[test]
    fn get_movement_is_six() {
        assert_eq!(TestPos.get_movement(), 6);
    }
}
