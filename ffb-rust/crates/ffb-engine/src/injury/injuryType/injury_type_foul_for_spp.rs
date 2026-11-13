/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeFoulForSpp.
/// ModificationAware: foul armor roll (foul-assist + blatant-foul modifiers) + injury roll.
/// savedByArmour -> PRONE (default). isFoul=true, isStab=false.
use ffb_model::enums::{ApothecaryMode, PS_PRONE, SendToBoxReason};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use crate::injury::{InjuryContext, InjuryTypeServer};
use crate::injury::injuryType::modification_aware_injury_type_server::{ModificationAwareInjuryType, modification_aware_handle_injury};
use crate::injury::injuryType::injury_type_foul::{foul_armour_roll, foul_injury_roll};

pub struct InjuryTypeFoulForSpp { ctx: InjuryContext, use_chainsaw: bool }
impl InjuryTypeFoulForSpp {
    pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw: false } }
    pub fn new_with_chainsaw(use_chainsaw: bool) -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), use_chainsaw } }
}
impl Default for InjuryTypeFoulForSpp { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeFoulForSpp {
    fn java_type_name(&self) -> &'static str { "foulForSpp" }
    fn java_class_name(&self) -> &'static str { "FoulForSpp" }
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        modification_aware_handle_injury(self, game, rng, attacker_id, defender_id, coord, from_coord, old_ctx, apo_mode);
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    /// Java: `FoulForSpp.isWorthSpps()` — true.
    fn is_worth_spps(&self) -> bool { true }
    /// Java: `FoulForSpp.isCausedByOpponent()` — true (overridden, unlike base `Foul`).
    fn is_caused_by_opponent(&self) -> bool { true }
    /// Java: `FoulForSpp.shouldPlayFallSound()` — overridden to `false`.
    fn should_play_fall_sound(&self) -> bool { false }
    /// Java: `FoulForSpp()` constructor passes `SendToBoxReason.FOULED`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::Fouled) }
}
impl ModificationAwareInjuryType for InjuryTypeFoulForSpp {
    // Java `InjuryTypeFoulForSpp.armourRoll`/`injuryRoll` are line-for-line copies of
    // `InjuryTypeFoul`'s. The Rust port had drifted to a simplified copy that added a hardcoded
    // "Dirty Player +1" armour modifier before rolling, where Java adds the factory's "Dirty Player"
    // only when the base roll did not break armour (snotling bb2020 seed 51 @1.0, injury
    // foulForSpp: armorModifiers ["Dirty Player +1"] vs Java ["Dirty Player"]). Share the body.
    fn armour_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str, roll: bool) {
        foul_armour_roll(&mut self.ctx, self.use_chainsaw, game, rng, attacker_id, defender_id, roll);
    }
    fn injury_roll(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str) {
        foul_injury_roll(&mut self.ctx, game, rng, attacker_id, defender_id);
    }
    // savedByArmour: default PRONE
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId};
    use ffb_mechanics::modifiers::{ARMOR_CHAINSAW_3, Modifier};
    fn has_dp_armor(t: &InjuryTypeFoulForSpp) -> bool {
        t.ctx.armor_modifiers.iter().any(|m| m.name.starts_with("Dirty Player"))
    }

    /// Real `InjuryModifierFactory`-sourced Dirty Player injury modifier is named "Dirty Player"
    /// (not the pre-Phase-ABJ placeholder constant `INJURY_DIRTY_PLAYER_1` = "Dirty Player +1").
    fn dirty_player_injury_modifier(rules: Rules) -> Modifier {
        Modifier::new("Dirty Player", 1, rules)
    }

    fn make_player(id: &str, armour: i32, skills: Vec<SkillId>) -> ffb_model::model::player::Player {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        Player { id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default() }
    }

    fn game_with_armor(armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("p1", armour, vec![]));
        Game::new(home, crate::step::framework::test_team("away", 0), Rules::Bb2025)
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    fn coord() -> FieldCoordinate { FieldCoordinate::new(5, 5) }

    #[test]
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeFoulForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeFoulForSpp::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn should_play_fall_sound_is_false() {
        assert!(!InjuryTypeFoulForSpp::new().should_play_fall_sound());
    }
    #[test]
    fn send_to_box_reason_is_fouled() {
        use ffb_model::enums::SendToBoxReason;
        assert_eq!(InjuryTypeFoulForSpp::new().send_to_box_reason(), Some(SendToBoxReason::Fouled));
    }
    #[test]
    fn dirty_player_adds_armor_modifier() {
        // AV 13 is never broken by the base roll, so Java's second pass adds the factory's
        // "Dirty Player" (named without a "+1" suffix, like InjuryTypeFoul).
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 13);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Dirty Player"),
            "got {:?}", t.ctx.armor_modifiers);
    }
    #[test]
    fn dirty_player_not_added_when_base_roll_breaks_armour() {
        // Java adds the skill-based armour modifiers only `if (!injuryContext.isArmorBroken())`;
        // AV 2 is always broken by the base roll, so Dirty Player stays free for the injury roll.
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!has_dp_armor(&t));
    }
    #[test]
    fn no_dirty_player_no_armor_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!has_dp_armor(&t));
    }
    #[test]
    fn dirty_player_adds_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::DirtyPlayer], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }
    #[test]
    fn no_dirty_player_no_injury_modifier() {
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.ctx.armor_broken = true;
        t.injury_roll(&game, &mut rng, Some("attacker"), "defender");
        assert!(!t.ctx.injury_modifiers.contains(&dirty_player_injury_modifier(game.rules)));
    }
    #[test]
    fn chainsaw_foul_adds_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoulForSpp::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }

    #[test]
    fn iron_hard_skin_defender_blocks_chainsaw_modifier() {
        // Same armourRoll IHS gate as InjuryTypeFoul (added for parity with the Java twin).
        use ffb_model::model::SkillWithValue;
        let mut game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        game.team_away.players[0].extra_skills.push(SkillWithValue::new(SkillId::IronHardSkin));
        let mut t = InjuryTypeFoulForSpp::new_with_chainsaw(true);
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
    #[test]
    fn non_chainsaw_foul_no_chainsaw_modifier() {
        let game = game_with_attacker_and_defender(vec![SkillId::Chainsaw], 2);
        let mut t = InjuryTypeFoulForSpp::new();
        let mut rng = GameRng::new(1);
        t.armour_roll(&game, &mut rng, Some("attacker"), "defender", true);
        assert!(!t.ctx.armor_modifiers.contains(&ARMOR_CHAINSAW_3));
    }
}
