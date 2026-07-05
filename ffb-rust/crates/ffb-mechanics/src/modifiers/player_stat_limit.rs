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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_min_max() {
        let limit = PlayerStatLimit::new(1, 9);
        assert_eq!(limit.get_min(), 1);
        assert_eq!(limit.get_max(), 9);
    }

    #[test]
    fn default_is_zero_zero() {
        let limit = PlayerStatLimit::default();
        assert_eq!(limit.min, 0);
        assert_eq!(limit.max, 0);
    }
}
