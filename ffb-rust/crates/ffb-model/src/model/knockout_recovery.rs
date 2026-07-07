use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.KnockoutRecovery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnockoutRecovery {
    pub player_id: Option<String>,
    pub recovering: bool,
    pub roll: i32,
    pub bloodweiser_babes: i32,
    pub re_roll_reason: Option<String>,
}

impl KnockoutRecovery {
    pub fn new(
        player_id: impl Into<String>,
        recovering: bool,
        roll: i32,
        bloodweiser_babes: i32,
        re_roll_reason: Option<String>,
    ) -> Self {
        KnockoutRecovery {
            player_id: Some(player_id.into()),
            recovering,
            roll,
            bloodweiser_babes,
            re_roll_reason,
        }
    }

    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn is_recovering(&self) -> bool { self.recovering }
    pub fn get_roll(&self) -> i32 { self.roll }
    pub fn get_bloodweiser_babes(&self) -> i32 { self.bloodweiser_babes }
    pub fn get_re_roll_reason(&self) -> Option<&str> { self.re_roll_reason.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_all_fields() {
        let k = KnockoutRecovery::new("p1", true, 4, 2, Some("loner".into()));
        assert_eq!(k.get_player_id(), Some("p1"));
        assert!(k.is_recovering());
        assert_eq!(k.get_roll(), 4);
        assert_eq!(k.get_bloodweiser_babes(), 2);
        assert_eq!(k.get_re_roll_reason(), Some("loner"));
    }

    #[test]
    fn default_has_no_player_id() {
        let k = KnockoutRecovery::default();
        assert_eq!(k.get_player_id(), None);
        assert!(!k.is_recovering());
        assert_eq!(k.get_roll(), 0);
    }

    #[test]
    fn no_re_roll_reason_is_none() {
        let k = KnockoutRecovery::new("p2", false, 3, 0, None);
        assert_eq!(k.get_re_roll_reason(), None);
        assert_eq!(k.get_bloodweiser_babes(), 0);
    }

    #[test]
    fn serde_round_trip() {
        let k = KnockoutRecovery::new("p1", true, 4, 2, Some("loner".into()));
        let s = serde_json::to_string(&k).unwrap();
        let back: KnockoutRecovery = serde_json::from_str(&s).unwrap();
        assert_eq!(back.get_player_id(), Some("p1"));
        assert_eq!(back.get_re_roll_reason(), Some("loner"));
    }

    #[test]
    fn default_bloodweiser_babes_is_zero() {
        let k = KnockoutRecovery::default();
        assert_eq!(k.get_bloodweiser_babes(), 0);
    }
}
