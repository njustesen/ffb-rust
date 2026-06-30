/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeKTMInjury.
/// Armor always broken. Injury roll. If STUNNED result, upgrade to KNOCKED_OUT.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_PRONE, PS_STUNNED, PS_KNOCKED_OUT};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll};

pub struct InjuryTypeKTMInjury { ctx: InjuryContext }
impl InjuryTypeKTMInjury { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeKTMInjury { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeKTMInjury {
    fn handle_injury(&mut self, _game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        do_injury_roll(rng, &mut self.ctx);
        // KTM injuries: STUNNED is upgraded to KNOCKED_OUT
        if self.ctx.injury.map(|s| s.base() == PS_STUNNED).unwrap_or(false) {
            self.ctx.injury = Some(PlayerState::new(PS_KNOCKED_OUT));
        }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken() {
        let mut t = InjuryTypeKTMInjury::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
    }
    #[test]
    fn stunned_becomes_ko() {
        // Seed chosen so injury roll produces STUNNED (2–7), then verify upgrade.
        // We'll just run and check invariant: result is never STUNNED.
        for seed in 1..=20u64 {
            let mut t = InjuryTypeKTMInjury::new(); let mut rng = GameRng::new(seed);
            t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
            assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_STUNNED), "seed={seed}: STUNNED should be upgraded");
        }
    }
    #[test]
    fn not_prone() {
        let mut t = InjuryTypeKTMInjury::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
}
