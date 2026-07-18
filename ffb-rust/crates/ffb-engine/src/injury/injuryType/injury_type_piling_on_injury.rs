/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypePilingOnInjury.
/// Piling On injury roll only (armor already broken).
/// turnover=false, no apo, stun treated as KO = false.
use ffb_model::enums::{ApothecaryMode, SendToBoxReason};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::niggling_injury_modifier;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use ffb_model::option::{game_option_id, util_game_option::is_option_enabled};
use crate::injury::{InjuryContext, InjuryTypeServer, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

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
            // Java: `if (!UtilGameOption.isOptionEnabled(game, GameOptionId.PILING_ON_DOES_NOT_STACK))
            // { availableSkill = attacker skills with affectsEitherArmourOrInjuryOnBlock (only Mighty
            // Blow has this property); if none of the (carried-over) armor modifiers is already
            // registered to that property, add the skill's injury modifiers }`. This was previously
            // missing entirely (Mighty Blow's injury bonus never applied during a Piling-On-Injury
            // reroll); wired via the shared factory call, matching every other injury type's
            // Mighty-Blow handling. The old-context armor-modifier either/or check is a pre-existing,
            // separately-tracked gap in `InjuryModifierFactory` (see its doc comment on
            // `skill_to_injury_modifier`), not reimplemented here.
            if !is_option_enabled(game, game_option_id::PILING_ON_DOES_NOT_STACK) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers_without_niggling(game, attacker, defender, false, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { false }
    fn can_use_apo(&self) -> bool { false }
    /// Java: `PilingOnInjury()` constructor passes `SendToBoxReason.PILED_ON`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::PiledOn) }
    /// Java: `PilingOnInjury()` constructor passes `isWorthSpps=true`.
    fn is_worth_spps(&self) -> bool { true }
    /// Java: `PilingOnInjury.isCausedByOpponent()` — true.
    fn is_caused_by_opponent(&self) -> bool { true }
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
    fn send_to_box_reason_is_piled_on() {
        assert_eq!(InjuryTypePilingOnInjury::new().send_to_box_reason(), Some(SendToBoxReason::PiledOn));
    }
    #[test]
    fn is_worth_spps_and_caused_by_opponent() {
        let t = InjuryTypePilingOnInjury::new();
        assert!(t.is_worth_spps());
        assert!(t.is_caused_by_opponent());
    }
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
        // Java: InjuryTypePilingOnInjury.handleInjury always calls
        // factory.getNigglingInjuryModifier(pDefender) directly (not the with-niggling
        // findInjuryModifiers variant).
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

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn make_player(id: &str, skills: Vec<ffb_model::enums::SkillId>) -> Player {
            Player { id: id.into(), name: id.into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour: 8, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default() }
        }
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier_during_piling_on_injury_reroll() {
        // Regression test: Java's InjuryTypePilingOnInjury.handleInjury applies the attacker's
        // Mighty-Blow-family injury bonus (via the affectsEitherArmourOrInjuryOnBlock skill
        // search) when PILING_ON_DOES_NOT_STACK is disabled — this was previously never wired in.
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow]);
        let mut t = InjuryTypePilingOnInjury::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"),
            "expected Mighty Blow injury modifier, got {:?}", t.ctx.injury_modifiers);
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![]);
        let mut t = InjuryTypePilingOnInjury::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"));
    }

    #[test]
    fn piling_on_does_not_stack_suppresses_mighty_blow_but_not_niggling() {
        use ffb_model::enums::SkillId;
        let mut game = game_with_attacker_and_defender(vec![SkillId::MightyBlow]);
        game.options.set(game_option_id::PILING_ON_DOES_NOT_STACK, "true");
        game.team_away.players[0].niggling_injuries = 1;
        let mut t = InjuryTypePilingOnInjury::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Mighty Blow"),
            "PILING_ON_DOES_NOT_STACK must suppress the Mighty Blow injury modifier");
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name.contains("Niggling")),
            "niggling modifier is always added regardless of PILING_ON_DOES_NOT_STACK");
    }
}
