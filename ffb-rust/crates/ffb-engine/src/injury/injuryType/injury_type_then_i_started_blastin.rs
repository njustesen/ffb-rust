/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeThenIStartedBlastin.
/// ModificationAware: armor roll + injury roll. savedByArmour -> None (no injury set, like Stab/Chainsaw).
use ffb_model::enums::{ApothecaryMode, SendToBoxReason};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury, leak_injury_modifier};

pub struct InjuryTypeThenIStartedBlastin { ctx: InjuryContext }
impl InjuryTypeThenIStartedBlastin { pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender) } } }
impl Default for InjuryTypeThenIStartedBlastin { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeThenIStartedBlastin {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    // Java: `ThenIStartedBlastin` does not override `fallingDownCausesTurnover()`, so the
    // `InjuryType` base default (`true`) applies. Regression fix: was previously inverted to
    // `false` here with no basis in the Java source.
    /// Java: `ThenIStartedBlastin.isCausedByOpponent()` → true. Was previously missing
    /// (defaulted to `false`).
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `ThenIStartedBlastin` constructed with
    /// `super("startedBlastin", false, SendToBoxReason.THEN_I_STARTED_BLASTIN)`. Was previously
    /// missing (defaulted to `None`).
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::ThenIStartedBlastin) }
    /// Java: `InjuryTypeThenIStartedBlastin`'s constructor calls
    /// `super.setFailedArmourPlacesProne(false)`. Was previously missing (defaulted to the
    /// trait's `true`).
    fn failed_armour_places_prone(&self) -> bool { false }
}
impl ModificationAwareInjuryType for InjuryTypeThenIStartedBlastin {
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, _attacker_id: Option<&str>, defender_id: &str, _roll: bool) {
        do_armor_roll(game, rng, &mut self.ctx, defender_id);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(),
        // isFoul(), isVomitLike())` — ThenIStartedBlastin does not override isStab/isFoul/
        // isVomitLike (all default false in InjuryType).
        if let Some(defender) = game.player(defender_id) {
            let attacker = attacker_id.and_then(|aid| game.player(aid));
            let factory = InjuryModifierFactory::new(game.rules);
            for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
            }
        }
        do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
    }
    fn saved_by_armour(&mut self) {
        // No injury set when armor holds (Java: savedByArmour = null / no-op)
        self.ctx.injury = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    fn game_with_armor(armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(Player { id: "p1".into(), name: "p1".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
    ..Default::default() });
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }
    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }
    #[test]
    fn armor_save_leaves_no_injury() {
        let mut t = InjuryTypeThenIStartedBlastin::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.armor_broken); assert!(t.ctx.injury.is_none());
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeThenIStartedBlastin::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert!(t.ctx.injury.is_some());
    }
    #[test]
    fn causes_turnover_by_default() {
        // Java: `ThenIStartedBlastin` does not override `fallingDownCausesTurnover()`, so the
        // `InjuryType` base default (`true`) applies. Regression test for a previously-inverted
        // override.
        assert!(InjuryTypeThenIStartedBlastin::new().falling_down_causes_turnover());
    }
    #[test]
    fn is_caused_by_opponent_is_true() {
        assert!(InjuryTypeThenIStartedBlastin::new().is_caused_by_opponent());
    }
    #[test]
    fn send_to_box_reason_is_then_i_started_blastin() {
        assert_eq!(
            InjuryTypeThenIStartedBlastin::new().send_to_box_reason(),
            Some(ffb_model::enums::SendToBoxReason::ThenIStartedBlastin)
        );
    }
    #[test]
    fn failed_armour_places_prone_is_false() {
        assert!(!InjuryTypeThenIStartedBlastin::new().failed_armour_places_prone());
    }
    #[test]
    fn context_stores_defender_id() {
        let mut t = InjuryTypeThenIStartedBlastin::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "blastin_target", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.defender_id.as_deref(), Some("blastin_target"));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeThenIStartedBlastin::new();
        let t2 = InjuryTypeThenIStartedBlastin::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn make_player(id: &str, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 7, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() }
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        // MightyBlow applies when isStab/isFoul/isVomitLike are all false — ThenIStartedBlastin's
        // defaults (InjuryType base class) — unlike DirtyPlayer, which requires isFoul=true.
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![ffb_model::enums::SkillId::MightyBlow]);
        let mut t = InjuryTypeThenIStartedBlastin::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![]);
        let mut t = InjuryTypeThenIStartedBlastin::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
