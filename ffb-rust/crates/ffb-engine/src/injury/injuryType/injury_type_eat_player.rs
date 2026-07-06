/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeEatPlayer.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_RIP};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer};

pub struct InjuryTypeEatPlayer { ctx: InjuryContext }
impl InjuryTypeEatPlayer { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeEatPlayer { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeEatPlayer {
    fn handle_injury(&mut self, _game: &Game, _rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        self.ctx.injury = Some(PlayerState::new(PS_RIP));
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeEatPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_ps_rip() {
        let mut t = InjuryTypeEatPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_RIP));
    }
    #[test]
    fn does_not_cause_turnover() { assert!(!InjuryTypeEatPlayer::new().falling_down_causes_turnover()); }
    #[test]
    fn context_stores_attacker_and_defender() {
        let mut t = InjuryTypeEatPlayer::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, Some("eater"), "eaten", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.attacker_id.as_deref(), Some("eater"));
        assert_eq!(t.ctx.defender_id.as_deref(), Some("eaten"));
    }
}
