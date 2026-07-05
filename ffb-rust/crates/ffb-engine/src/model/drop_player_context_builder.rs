/// 1:1 translation of com.fumbbl.ffb.server.model.DropPlayerContextBuilder.
use ffb_model::enums::ApothecaryMode;
use crate::drop_player_context::{DropPlayerContext, VictimStateKey};
use crate::injury::InjuryResult;

pub struct DropPlayerContextBuilder {
    injury_result: Option<InjuryResult>,
    end_turn: bool,
    eligible_for_safe_pair_of_hands: bool,
    requires_armour_break: bool,
    already_dropped: bool,
    modified_injury_ends_turn: bool,
    end_turn_without_knockdown: bool,
    label: Option<String>,
    player_id: Option<String>,
    apothecary_mode: Option<ApothecaryMode>,
    victim_state_key: Option<VictimStateKey>,
    additional_victim_state_keys: Vec<VictimStateKey>,
}

impl DropPlayerContextBuilder {
    /// Java: DropPlayerContextBuilder.builder()
    pub fn builder() -> Self {
        Self {
            injury_result: None,
            end_turn: false,
            eligible_for_safe_pair_of_hands: false,
            requires_armour_break: false,
            already_dropped: false,
            modified_injury_ends_turn: false,
            end_turn_without_knockdown: false,
            label: None,
            player_id: None,
            apothecary_mode: None,
            victim_state_key: None,
            additional_victim_state_keys: Vec::new(),
        }
    }

    /// Java: DropPlayerContextBuilder.from(original)
    pub fn from(original: &DropPlayerContext) -> Self {
        Self {
            injury_result: original.injury_result.as_ref().map(|ir| *ir.clone()),
            end_turn: original.end_turn,
            eligible_for_safe_pair_of_hands: original.eligible_for_safe_pair_of_hands,
            requires_armour_break: original.requires_armour_break,
            already_dropped: original.already_dropped,
            modified_injury_ends_turn: original.modified_injury_ends_turn,
            end_turn_without_knockdown: original.end_turn_without_knockdown,
            label: original.label.clone(),
            player_id: original.player_id.clone(),
            apothecary_mode: original.apothecary_mode,
            victim_state_key: original.victim_state_key,
            additional_victim_state_keys: original.additional_victim_state_keys.clone(),
        }
    }

    pub fn injury_result(mut self, injury_result: InjuryResult) -> Self {
        self.injury_result = Some(injury_result);
        self
    }

    pub fn end_turn(mut self, end_turn: bool) -> Self {
        self.end_turn = end_turn;
        self
    }

    pub fn eligible_for_safe_pair_of_hands(mut self, eligible: bool) -> Self {
        self.eligible_for_safe_pair_of_hands = eligible;
        self
    }

    pub fn requires_armour_break(mut self, requires_armour_break: bool) -> Self {
        self.requires_armour_break = requires_armour_break;
        self
    }

    pub fn already_dropped(mut self, already_dropped: bool) -> Self {
        self.already_dropped = already_dropped;
        self
    }

    pub fn modified_injury_ends_turn(mut self, modified: bool) -> Self {
        self.modified_injury_ends_turn = modified;
        self
    }

    pub fn end_turn_without_knockdown(mut self, end_turn_without_knockdown: bool) -> Self {
        self.end_turn_without_knockdown = end_turn_without_knockdown;
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn player_id(mut self, player_id: impl Into<String>) -> Self {
        self.player_id = Some(player_id.into());
        self
    }

    pub fn apothecary_mode(mut self, mode: ApothecaryMode) -> Self {
        self.apothecary_mode = Some(mode);
        self
    }

    pub fn victim_state_key(mut self, key: VictimStateKey) -> Self {
        self.victim_state_key = Some(key);
        self
    }

    pub fn additional_victim_state_keys(mut self, keys: Vec<VictimStateKey>) -> Self {
        self.additional_victim_state_keys = keys;
        self
    }

    pub fn build(self) -> DropPlayerContext {
        DropPlayerContext {
            injury_result: self.injury_result.map(|ir| Box::new(ir)),
            end_turn: self.end_turn,
            eligible_for_safe_pair_of_hands: self.eligible_for_safe_pair_of_hands,
            requires_armour_break: self.requires_armour_break,
            already_dropped: self.already_dropped,
            modified_injury_ends_turn: self.modified_injury_ends_turn,
            end_turn_without_knockdown: self.end_turn_without_knockdown,
            label: self.label,
            player_id: self.player_id,
            apothecary_mode: self.apothecary_mode,
            victim_state_key: self.victim_state_key,
            additional_victim_state_keys: self.additional_victim_state_keys,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::ApothecaryMode;

    #[test]
    fn builder_defaults() {
        let ctx = DropPlayerContextBuilder::builder().build();
        assert!(!ctx.end_turn);
        assert!(!ctx.eligible_for_safe_pair_of_hands);
        assert!(!ctx.requires_armour_break);
        assert!(!ctx.already_dropped);
        assert!(ctx.player_id.is_none());
        assert!(ctx.apothecary_mode.is_none());
    }

    #[test]
    fn builder_set_end_turn() {
        let ctx = DropPlayerContextBuilder::builder()
            .end_turn(true)
            .build();
        assert!(ctx.end_turn);
    }

    #[test]
    fn builder_set_player_id() {
        let ctx = DropPlayerContextBuilder::builder()
            .player_id("p-1")
            .build();
        assert_eq!(ctx.player_id.as_deref(), Some("p-1"));
    }

    #[test]
    fn builder_set_apothecary_mode() {
        let ctx = DropPlayerContextBuilder::builder()
            .apothecary_mode(ApothecaryMode::Defender)
            .build();
        assert_eq!(ctx.apothecary_mode, Some(ApothecaryMode::Defender));
    }

    #[test]
    fn builder_set_victim_state_key() {
        let ctx = DropPlayerContextBuilder::builder()
            .victim_state_key(VictimStateKey::OldDefenderState)
            .build();
        assert_eq!(ctx.victim_state_key, Some(VictimStateKey::OldDefenderState));
    }

    #[test]
    fn builder_from_existing() {
        let mut original = DropPlayerContext::new();
        original.end_turn = true;
        original.player_id = Some("orig-p".to_string());

        let copied = DropPlayerContextBuilder::from(&original).build();
        assert!(copied.end_turn);
        assert_eq!(copied.player_id.as_deref(), Some("orig-p"));
    }

    #[test]
    fn builder_from_then_modify() {
        let mut original = DropPlayerContext::new();
        original.end_turn = true;

        let copied = DropPlayerContextBuilder::from(&original)
            .end_turn(false)
            .requires_armour_break(true)
            .build();
        assert!(!copied.end_turn);
        assert!(copied.requires_armour_break);
    }

    #[test]
    fn builder_additional_victim_state_keys() {
        let ctx = DropPlayerContextBuilder::builder()
            .additional_victim_state_keys(vec![VictimStateKey::OldDefenderState, VictimStateKey::OldPlayerState])
            .build();
        assert_eq!(ctx.additional_victim_state_keys.len(), 2);
    }
}
