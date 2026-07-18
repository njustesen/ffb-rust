/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeSaboteur.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_KNOCKED_OUT};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::SoundId;
use crate::injury::{InjuryContext, InjuryTypeServer};

pub struct InjuryTypeSaboteur { ctx: InjuryContext }
impl InjuryTypeSaboteur { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeSaboteur { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeSaboteur {
    fn handle_injury(&mut self, _game: &Game, _rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        self.ctx.armor_roll = None;
        self.ctx.injury_roll = None;
        self.ctx.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        // Java: `injuryContext.setSound(SoundId.KO)` — previously never set.
        self.ctx.sound = Some(SoundId::KO);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    /// Java: `Saboteur()` constructor passes `SendToBoxReason.SABOTEUR`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Saboteur) }
    /// Java: `Saboteur.canUseApo()` — false (unlike the base `InjuryType` default of true; this
    /// was previously missing entirely).
    fn can_use_apo(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeSaboteur::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_ps_knocked_out() {
        let mut t = InjuryTypeSaboteur::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_KNOCKED_OUT));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeSaboteur::new().falling_down_causes_turnover()); }
    #[test]
    fn context_stores_attacker_and_defender() {
        let mut t = InjuryTypeSaboteur::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("saboteur"), "victim", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("saboteur"));
        assert_eq!(t.ctx.defender_id.as_deref(), Some("victim"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeSaboteur::new();
        let t2 = InjuryTypeSaboteur::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }
    #[test]
    fn send_to_box_reason_is_saboteur() {
        assert_eq!(InjuryTypeSaboteur::new().send_to_box_reason(), Some(SendToBoxReason::Saboteur));
    }
    #[test]
    fn cannot_use_apo() {
        // Regression test: Java `Saboteur.canUseApo()` returns false, unlike the `InjuryType`
        // base default of true (this was previously missing entirely).
        assert!(!InjuryTypeSaboteur::new().can_use_apo());
    }
    #[test]
    fn sets_ko_sound() {
        let mut t = InjuryTypeSaboteur::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.sound, Some(ffb_model::model::SoundId::KO));
    }
}
