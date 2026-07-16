/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypePilingOnInjury.
/// Piling On injury roll only (armor already broken).
/// turnover=false, no apo, stun treated as KO = false.
use ffb_model::enums::ApothecaryMode;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::niggling_injury_modifier;
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
        // Java: ((InjuryModifierFactory) game.getFactory(...)).getNigglingInjuryModifier(pDefender)
        //         .ifPresent(injuryContext::addInjuryModifier);
        if let Some(defender) = game.player(defender_id) {
            if let Some(m) = niggling_injury_modifier(defender.niggling_injuries) {
                self.ctx.add_injury_modifier(m);
            }
        }
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
    #[test]
    fn injury_context_returns_context() {
        let t = InjuryTypePilingOnInjury::new();
        assert_eq!(t.injury_context().apothecary_mode, ApothecaryMode::Defender);
    }
    #[test]
    fn sets_defender_id_after_handle_injury() {
        let mut t = InjuryTypePilingOnInjury::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&make_game(), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("p1"));
    }

    fn game_with_niggling_defender(niggling_injuries: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2016)
    }

    #[test]
    fn niggling_injured_defender_gets_niggling_injury_modifier() {
        // Java: InjuryTypePilingOnInjury.handleInjury calls only
        // factory.getNigglingInjuryModifier(pDefender), not the full findInjuryModifiers.
        let mut t = InjuryTypePilingOnInjury::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_niggling_defender(1), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "1 Niggling Injury"));
    }

    #[test]
    fn non_niggling_defender_gets_no_niggling_injury_modifier() {
        let mut t = InjuryTypePilingOnInjury::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_niggling_defender(0), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")));
    }
}
