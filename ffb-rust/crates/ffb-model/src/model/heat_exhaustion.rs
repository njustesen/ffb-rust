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

    #[test]
    fn not_exhausted_stores_correctly() {
        let h = HeatExhaustion::new("p2", false, 6);
        assert!(!h.is_exhausted());
        assert_eq!(h.get_roll(), 6);
    }

    #[test]
    fn serde_round_trip() {
        let h = HeatExhaustion::new("p1", true, 3);
        let s = serde_json::to_string(&h).unwrap();
        let back: HeatExhaustion = serde_json::from_str(&s).unwrap();
        assert_eq!(back.get_player_id(), Some("p1"));
        assert!(back.is_exhausted());
        assert_eq!(back.get_roll(), 3);
    }

    #[test]
    fn roll_boundary_values() {
        let h_min = HeatExhaustion::new("p", false, 1);
        let h_max = HeatExhaustion::new("p", false, 6);
        assert_eq!(h_min.get_roll(), 1);
        assert_eq!(h_max.get_roll(), 6);
    }
}
