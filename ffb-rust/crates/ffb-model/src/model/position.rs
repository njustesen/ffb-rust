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
