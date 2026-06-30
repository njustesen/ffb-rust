/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFumbledKtmApoKo.
/// Armor always broken. Injury roll (no special modifier filter). stunIsTreatedAsKo=true.
use ffb_model::enums::{ApothecaryMode, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll};

pub struct InjuryTypeFumbledKtmApoKo { ctx: InjuryContext }
impl InjuryTypeFumbledKtmApoKo { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeFumbledKtmApoKo { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFumbledKtmApoKo {
    fn handle_injury(&mut self, _game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, _from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        self.ctx.armor_broken = true;
        do_injury_roll(rng, &mut self.ctx);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn stun_is_treated_as_ko(&self) -> bool { true }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn make_game() -> Game { Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2025) }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_always_broken_and_injury_set() {
        let mut t = InjuryTypeFumbledKtmApoKo::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test] fn stun_is_ko() { assert!(InjuryTypeFumbledKtmApoKo::new().stun_is_treated_as_ko()); }
}
