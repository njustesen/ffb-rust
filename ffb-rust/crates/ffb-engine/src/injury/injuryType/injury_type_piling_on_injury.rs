/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypePilingOnInjury.
/// Piling On injury roll only (armor already broken).
/// turnover=false, no apo, stun treated as KO = false.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player};

pub struct InjuryTypePilingOnInjury { ctx: InjuryContext }
impl InjuryTypePilingOnInjury {
    pub fn new() -> Self {
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        ctx.armor_broken = true;
        Self { ctx }
    }
}
impl Default for InjuryTypePilingOnInjury { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypePilingOnInjury {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    fn can_use_apo(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, PS_PRONE};
    fn make_game() -> Game {
        Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_already_broken_and_injury_rolled() {
        let mut t = InjuryTypePilingOnInjury::new();
        assert!(t.ctx.armor_broken);
        let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn no_apo() { assert!(!InjuryTypePilingOnInjury::new().can_use_apo()); }
    #[test]
    fn no_turnover() { assert!(!InjuryTypePilingOnInjury::new().falling_down_causes_turnover()); }
}
