use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.HeatExhaustion.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeatExhaustion {
    pub player_id: Option<String>,
    pub exhausted: bool,
    pub roll: i32,
}

impl HeatExhaustion {
    pub fn new(player_id: impl Into<String>, exhausted: bool, roll: i32) -> Self {
        HeatExhaustion { player_id: Some(player_id.into()), exhausted, roll }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_exhausted(&self) -> bool { self.exhausted }
    pub fn get_roll(&self) -> i32 { self.roll }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let h = HeatExhaustion::new("p1", true, 3);
        assert_eq!(h.get_player_id(), Some("p1"));
        assert!(h.is_exhausted());
        assert_eq!(h.get_roll(), 3);
    }

    #[test]
    fn default_has_no_player_id() {
        let h = HeatExhaustion::default();
        assert_eq!(h.get_player_id(), None);
        assert!(!h.is_exhausted());
        assert_eq!(h.get_roll(), 0);
    }
}
