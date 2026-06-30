/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeCrowdPush.
use ffb_model::enums::{ApothecaryMode, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer};
use crate::injury::injuryType::injury_type_crowd::crowd_handle_injury;

pub struct InjuryTypeCrowdPush { ctx: InjuryContext }
impl InjuryTypeCrowdPush { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeCrowdPush { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeCrowdPush {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        crowd_handle_injury(&mut self.ctx, game, rng, attacker_id, defender_id, coord, apo_mode);
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
        let mut t = InjuryTypeCrowdPush::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn injury_is_reserve_or_ko_or_casualty() {
        for seed in 1..=10u64 {
            let mut t = InjuryTypeCrowdPush::new(); let mut rng = GameRng::new(seed);
            t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
            let b = t.ctx.injury.expect("injury set").base();
            assert!(b == PS_RESERVE || t.ctx.is_knocked_out() || t.ctx.is_casualty(), "seed={seed}");
        }
    }
}
