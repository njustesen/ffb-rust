/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropJump.
/// Armor roll (+ Arm Bar armour-or-injury modifier) + injury or PRONE.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeDropJump {
    ctx: InjuryContext,
    /// Java: `divingTackler` (`new InjuryTypeDropJump(game.getDefender())` in bb2020/bb2025
    /// StepJump) -- the fallback Arm Bar source.
    diving_tackler_id: Option<String>,
}

impl InjuryTypeDropJump {
    pub fn new() -> Self { Self { ctx: InjuryContext::new(ApothecaryMode::Defender), diving_tackler_id: None } }
    pub fn new_with_diving_tackler(id: Option<String>) -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), diving_tackler_id: id }
    }

    /// Java `InjuryTypeDropJump.handleInjury`'s `avOrInjModifierSkill` search. Unlike DropDodge:
    /// the square is `vacatedFromCoordinate` falling back to the defender's own square; the
    /// adjacent opponents are matched on `affectsEitherArmourOrInjuryOnJump`; the player on the
    /// vacated square is added only when one was given, with no extra tacklezone filter; and the
    /// diving tackler fallback uses `...OnDodge` with no position check. Java streams a HashSet,
    /// but every candidate skill registers the same "Arm Bar" +1 modifiers, so which one is found
    /// first does not matter.
    fn has_av_or_inj_skill(&self, game: &Game, defender_id: &str, vacated: Option<FieldCoordinate>) -> bool {
        use ffb_model::util::util_player::UtilPlayer;
        let from = match vacated.or_else(|| game.field_model.player_coordinate(defender_id)) {
            Some(c) => c,
            None => return false,
        };
        let defender_is_home = game.team_home.players.iter().any(|p| p.id == defender_id);
        let opposing = if defender_is_home { &game.team_away } else { &game.team_home };
        let mut candidates: Vec<String> = UtilPlayer::find_adjacent_players_with_tacklezones(
            game, opposing, from, false,
        ).into_iter().cloned().collect();
        if let Some(v) = vacated {
            if let Some(pid) = game.field_model.player_at(v) {
                candidates.push(pid.to_owned());
            }
        }
        let found = candidates.iter().any(|pid| game.player(pid)
            .map(|p| p.has_skill_property(NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_JUMP))
            .unwrap_or(false));
        if found {
            return true;
        }
        self.diving_tackler_id.as_deref()
            .and_then(|dt| game.player(dt))
            .map(|p| p.has_skill_property(NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_DODGE))
            .unwrap_or(false)
    }
}
impl Default for InjuryTypeDropJump { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeDropJump {
    fn java_type_name(&self) -> &'static str { "dropLeap" }
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        if !self.ctx.armor_broken {
            let defender_ignores = game.player(defender_id)
                .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
                .unwrap_or(false);
            if !defender_ignores {
                if game.player(defender_id)
                    .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                    .unwrap_or(false)
                {
                    self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
                }
            } else {
                // Java: `injuryContext.addArmorModifiers(defender.getSkillWithProperty(
                //   ignoresArmourModifiersFromSkills).getArmorModifiers())` — Iron Hard Skin's
                // value-0 StaticArmourModifier, so the report NAMES the skill (H.59; chaos_dwarf
                // bb2025 seed 3 dropGfi). No armour total moves.
                self.ctx.add_armor_modifier(ffb_mechanics::modifiers::Modifier::new(
                    "Iron Hard Skin", 0, game.rules));
            }
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        // Java: avOrInjModifierSkill -- Arm Bar's armour-OR-injury mutual exclusion, skipped when
        // the defender has an unused ignoresArmourModifiersFromSkills skill. It first tries to
        // break armour (+1, re-check); if armour broke without it, the +1 goes on the injury roll
        // (chaos_troll_vs_slann_fumbbl bb2025 seed 3: Java injury ["Arm Bar"], Rust none).
        let defender_ignores = game.player(defender_id)
            .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
            .unwrap_or(false);
        let mut av_or_inj = !defender_ignores && self.has_av_or_inj_skill(game, defender_id, from_coord);
        if !self.ctx.armor_broken && av_or_inj {
            self.ctx.add_armor_modifier(ffb_mechanics::modifiers::Modifier::new("Arm Bar", 1, game.rules));
            crate::injury::recalc_armor_broken(game, &mut self.ctx, defender_id);
            av_or_inj = false;
        }
        if self.ctx.armor_broken {
            // Java: InjuryModifierFactory factory = game.getFactory(FactoryType.Factory.INJURY_MODIFIER);
            // factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender, isStab(), isFoul(),
            // isVomitLike()) — DropJump never overrides isStab/isFoul/isVomitLike (all false).
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
            if av_or_inj {
                self.ctx.add_injury_modifier(ffb_mechanics::modifiers::Modifier::new("Arm Bar", 1, game.rules));
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { true }
    /// Java: `DropJump()` constructor passes `SendToBoxReason.JUMP_FAIL`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::JumpFail) }
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
    fn armor_save_results_in_prone() {
        let mut t = InjuryTypeDropJump::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeDropJump::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn causes_turnover() { assert!(InjuryTypeDropJump::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_jump_fail() {
        assert_eq!(InjuryTypeDropJump::new().send_to_box_reason(), Some(SendToBoxReason::JumpFail));
    }
    #[test]
    fn pre_broken_skips_armor_roll() {
        let mut t = InjuryTypeDropJump::new();
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(7), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn default_equivalent_to_new() {
        let t1 = InjuryTypeDropJump::new();
        let t2 = InjuryTypeDropJump::default();
        assert_eq!(t1.ctx.armor_broken, t2.ctx.armor_broken);
        assert!(t1.ctx.injury.is_none() && t2.ctx.injury.is_none());
    }

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>, defender_armour: i32) -> Game {
        use std::collections::HashSet;
        use ffb_model::model::player::Player;
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::{PlayerType, PlayerGender};
        fn make_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> Player {
            Player { id: id.into(), name: id.into(), nr: 1,
                position_id: "lineman".into(), player_type: PlayerType::Regular,
                gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
                passing: 4, armour, starting_skills: skills.into_iter().map(SkillWithValue::new).collect(), extra_skills: vec![],
                temporary_skills: vec![], used_skills: HashSet::new(),
                niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
                is_big_guy: false,
                ..Default::default() }
        }
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", defender_armour, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_model::enums::SkillId;
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow], 2);
        let mut t = InjuryTypeDropJump::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![], 2);
        let mut t = InjuryTypeDropJump::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }

    fn place(game: &mut Game, id: &str, c: FieldCoordinate) {
        use ffb_model::enums::PS_STANDING;
        game.field_model.set_player_coordinate(id, c);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    fn make_jump_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
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

    fn arm_bar_game(jumper_av: i32, arm_bar_at: FieldCoordinate) -> Game {
        use ffb_model::enums::SkillId;
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_jump_player("jumper", jumper_av, vec![]));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_jump_player("armbar", 7, vec![SkillId::ArmBar]));
        let mut game = Game::new(home, away, Rules::Bb2025);
        place(&mut game, "jumper", FieldCoordinate::new(6, 5));
        place(&mut game, "armbar", arm_bar_at);
        game
    }

    /// Java DropJump: an opponent with Arm Bar adjacent to the jumper's square, when armour
    /// breaks on its own, adds "Arm Bar" +1 to the INJURY roll (chaos_troll_vs_slann_fumbbl
    /// bb2025 seed 3).
    #[test]
    fn arm_bar_adjacent_goes_on_injury_when_armour_breaks_alone() {
        let game = arm_bar_game(2, FieldCoordinate::new(7, 5));
        let mut t = InjuryTypeDropJump::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, None, "jumper", FieldCoordinate::new(6, 5), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert!(!t.ctx.armor_modifiers.iter().any(|m| m.name == "Arm Bar"));
        assert!(t.ctx.injury_modifiers.iter().any(|m| m.name == "Arm Bar"));
    }

    /// Armour side: a 9 against AV10 breaks with Arm Bar's +1, which is then NOT also on injury.
    #[test]
    fn arm_bar_adjacent_breaks_armour_on_a_one_short_roll() {
        let game = arm_bar_game(10, FieldCoordinate::new(7, 6));
        let mut seed = 0u64;
        loop {
            let mut probe = GameRng::new(seed);
            if probe.d6() + probe.d6() == 9 { break; }
            seed += 1;
            assert!(seed < 10_000);
        }
        let mut t = InjuryTypeDropJump::new();
        let mut rng = GameRng::new(seed);
        t.handle_injury(&game, &mut rng, None, "jumper", FieldCoordinate::new(6, 5), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken, "Arm Bar +1 must break AV10 on a 9");
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Arm Bar"));
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Arm Bar"));
    }

    #[test]
    fn no_arm_bar_without_an_adjacent_arm_bar_player() {
        let game = arm_bar_game(2, FieldCoordinate::new(10, 5));
        let mut t = InjuryTypeDropJump::new();
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, None, "jumper", FieldCoordinate::new(6, 5), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Arm Bar"));
    }
}
