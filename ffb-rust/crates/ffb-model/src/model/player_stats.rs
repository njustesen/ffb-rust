/// 1:1 translation of com.fumbbl.ffb.model.PlayerStats.
pub trait PlayerStats {
    fn move_stat(&self) -> i32;
    fn strength(&self) -> i32;
    fn agility(&self) -> i32;
    fn passing(&self) -> i32;
    fn armour(&self) -> i32;
}
