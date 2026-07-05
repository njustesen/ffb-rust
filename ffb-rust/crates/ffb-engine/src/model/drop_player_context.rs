/// Re-export of the top-level DropPlayerContext, translated from
/// com.fumbbl.ffb.server.model.DropPlayerContext.
pub use crate::drop_player_context::{DropPlayerContext, VictimStateKey};

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::ApothecaryMode;
    use crate::injury::InjuryResult;

    #[test]
    fn new_has_default_fields() {
        let ctx = DropPlayerContext::new();
        assert!(!ctx.end_turn);
        assert!(!ctx.eligible_for_safe_pair_of_hands);
        assert!(!ctx.requires_armour_break);
        assert!(!ctx.already_dropped);
        assert!(!ctx.modified_injury_ends_turn);
        assert!(!ctx.end_turn_without_knockdown);
        assert!(ctx.label.is_none());
        assert!(ctx.player_id.is_none());
        assert!(ctx.apothecary_mode.is_none());
        assert!(ctx.victim_state_key.is_none());
        assert!(ctx.additional_victim_state_keys.is_empty());
        assert!(ctx.injury_result.is_none());
    }

    #[test]
    fn with_injury_sets_fields() {
        let injury_result = InjuryResult::new(ApothecaryMode::Defender);
        let ctx = DropPlayerContext::with_injury(
            injury_result,
            "player-1".to_string(),
            ApothecaryMode::Attacker,
            true,
        );
        assert_eq!(ctx.player_id.as_deref(), Some("player-1"));
        assert_eq!(ctx.apothecary_mode, Some(ApothecaryMode::Attacker));
        assert!(ctx.eligible_for_safe_pair_of_hands);
        assert!(ctx.injury_result.is_some());
    }

    #[test]
    fn victim_state_key_variants_exist() {
        let _ = VictimStateKey::OldDefenderState;
        let _ = VictimStateKey::OldPlayerState;
        let _ = VictimStateKey::ThrownPlayerState;
        let _ = VictimStateKey::KickedPlayerState;
    }
}
