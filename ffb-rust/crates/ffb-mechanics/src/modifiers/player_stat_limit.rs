/// 1:1 translation of com.fumbbl.ffb.modifiers.PlayerStatLimit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerStatLimit {
    pub min: i32,
    pub max: i32,
}

impl PlayerStatLimit {
    pub fn new(min: i32, max: i32) -> Self {
        PlayerStatLimit { min, max }
    }

    pub fn get_min(&self) -> i32 { self.min }
    pub fn get_max(&self) -> i32 { self.max }
}

impl Default for PlayerStatLimit {
    fn default() -> Self { PlayerStatLimit { min: 0, max: 0 } }
}
