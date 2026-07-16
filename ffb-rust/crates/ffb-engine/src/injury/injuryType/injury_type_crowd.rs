/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeCrowd (abstract).
///
/// Provides shared crowd-push handleInjury logic for CrowdPush/TrapDoorFall variants.
use ffb_model::enums::{ApothecaryMode, PlayerState, PS_RESERVE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub(crate) fn crowd_handle_injury(
    ctx: &mut InjuryContext, game: &Game, rng: &mut GameRng,
    attacker_id: Option<&str>, defender_id: &str,
    coord: FieldCoordinate, apo_mode: ApothecaryMode,
) {
    ctx.defender_id = Some(defender_id.to_owned());
    ctx.attacker_id = attacker_id.map(str::to_owned);
    ctx.defender_coordinate = Some(coord);
    ctx.apothecary_mode = apo_mode;
    ctx.armor_broken = true;
    // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender,
    // isStab(), isFoul(), isVomitLike())` — CrowdPush/CrowdPushForSpp (the InjuryType
    // wrapped by InjuryTypeCrowd) don't override any of isStab/isFoul/isVomitLike, so
    // all three are false (inherited InjuryType defaults).
    if let Some(defender) = game.player(defender_id) {
        let attacker = attacker_id.and_then(|aid| game.player(aid));
        let factory = InjuryModifierFactory::new(game.rules);
        for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
            ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
        }
    }
    do_injury_roll_for_player(rng, ctx, game, defender_id);
    if !ctx.is_casualty() && !ctx.is_knocked_out() {
        ctx.injury = Some(PlayerState::new(PS_RESERVE));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{ApothecaryMode, Rules};
    use ffb_model::model::game::Game;
    use crate::step::framework::test_team;
    use ffb_model::types::FieldCoordinate;

    #[test]
    fn sets_defender_and_attacker_ids() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, Some("atk1"), "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert_eq!(ctx.defender_id.as_deref(), Some("def1"));
        assert_eq!(ctx.attacker_id.as_deref(), Some("atk1"));
    }

    #[test]
    fn sets_armor_broken_true() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(ctx.armor_broken);
    }

    #[test]
    fn injury_is_set_after_call() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(ctx.injury.is_some());
    }
    #[test]
    fn sets_defender_coordinate() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        let coord = FieldCoordinate::new(3, 7);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1", coord, ApothecaryMode::Defender);
        assert_eq!(ctx.defender_coordinate, Some(coord));
    }
    #[test]
    fn sets_apothecary_mode() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Attacker);
        assert_eq!(ctx.apothecary_mode, ApothecaryMode::Attacker);
    }

    #[test]
    fn attacker_id_is_none_when_not_provided() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert_eq!(ctx.attacker_id, None);
    }

    #[test]
    fn coordinate_values_are_preserved() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        let coord = FieldCoordinate::new(5, 12);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1", coord, ApothecaryMode::Defender);
        assert_eq!(ctx.defender_coordinate, Some(coord));
    }

    #[test]
    fn injury_is_some_after_call() {
        let game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2016);
        let mut rng = GameRng::new(3);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, None, "def1",
            FieldCoordinate::new(1, 2), ApothecaryMode::Defender);
        assert!(ctx.injury.is_some());
    }

    /// isStab/isFoul/isVomitLike are all false for CrowdPush/CrowdPushForSpp, so Mighty
    /// Blow applies normally (same as a block) — proves the factory is now reached with
    /// real attacker/defender players (the earlier tests use unregistered ids, so the
    /// `game.player(..)` lookup inside `crowd_handle_injury` always misses).
    fn make_player(id: &str, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() }
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        let mut home = test_team("home", 0);
        home.players.push(make_player("attacker", attacker_skills));
        let mut away = test_team("away", 0);
        away.players.push(make_player("defender", vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow]);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, Some("attacker"), "defender",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![]);
        let mut rng = GameRng::new(1);
        let mut ctx = InjuryContext::new(ApothecaryMode::Defender);
        crowd_handle_injury(&mut ctx, &game, &mut rng, Some("attacker"), "defender",
            FieldCoordinate::new(0, 0), ApothecaryMode::Defender);
        assert!(!ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
