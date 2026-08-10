/// Translation of com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropDodge.
/// Armor roll + injury or PRONE. Arm bar player modifier is TODO.
use ffb_model::enums::{ApothecaryMode, PlayerState, SendToBoxReason, PS_PRONE};
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::model::game::Game;
use ffb_model::model::property::NamedProperties;
use ffb_mechanics::modifiers::ARMOR_CHAINSAW_3;
use ffb_mechanics::modifiers::injury_modifier_factory::InjuryModifierFactory;
use crate::injury::{InjuryContext, InjuryTypeServer, do_armor_roll, do_injury_roll_for_player};
use crate::injury::injuryType::modification_aware_injury_type_server::leak_injury_modifier;

pub struct InjuryTypeDropDodge {
    ctx: InjuryContext,
    /// Java: pArmBarPlayer. Arm bar modifier source (TODO when ArmorModifierFactory is ported).
    arm_bar_player_id: Option<String>,
    use_arm_bar_modifiers: bool,
}

impl InjuryTypeDropDodge {
    pub fn new() -> Self {
        // Java default ctor: `this(null, true)` — useArmBarModifiers defaults TRUE.
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), arm_bar_player_id: None, use_arm_bar_modifiers: true }
    }
    pub fn new_with_arm_bar(arm_bar_player_id: Option<String>, use_arm_bar_modifiers: bool) -> Self {
        Self { ctx: InjuryContext::new(ApothecaryMode::Defender), arm_bar_player_id, use_arm_bar_modifiers }
    }

    /// Java: the `avOrInjModifierSkill` search — among opponents of the defender adjacent to
    /// the dodge's FROM square (with tacklezones), plus any player standing ON the from
    /// square (Shadowing / Diving Tackle mover), find the first with a skill granting
    /// `affectsEitherArmourOrInjuryOnDodge` (Arm Bar); fall back to the diving tackler.
    fn find_av_or_inj_skill_player(&self, game: &Game, defender_id: &str, from: FieldCoordinate) -> Option<String> {
        use ffb_model::util::util_player::UtilPlayer;
        let defender_is_home = game.team_home.players.iter().any(|p| p.id == defender_id);
        let opposing = if defender_is_home { &game.team_away } else { &game.team_home };
        let mut candidates: Vec<String> = UtilPlayer::find_adjacent_players_with_tacklezones(
            game, opposing, from, false,
        ).into_iter().cloned().collect();
        // Java: shadowingOrDtPlayer = fieldModel.getPlayer(fromCoordinate) — added regardless
        // of team (the skill lookup below simply yields None for skill-less players).
        if let Some(pid) = game.field_model.player_at(from) {
            candidates.push(pid.to_owned());
        }
        for pid in candidates {
            let has_tz = game.field_model.player_state(&pid)
                .map(|s| s.has_tacklezones())
                .unwrap_or(false);
            if !has_tz { continue; }
            let has_skill = game.player(&pid)
                .map(|p| p.has_skill_property(NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_DODGE))
                .unwrap_or(false);
            if has_skill {
                return Some(pid);
            }
        }
        // Java orElseGet: the diving tackler standing on the from square.
        if let Some(ref dt) = self.arm_bar_player_id {
            if game.field_model.player_coordinate(dt) == Some(from) {
                let has_skill = game.player(dt)
                    .map(|p| p.has_skill_property(NamedProperties::AFFECTS_EITHER_ARMOUR_OR_INJURY_ON_DODGE))
                    .unwrap_or(false);
                if has_skill {
                    return Some(dt.clone());
                }
            }
        }
        None
    }
}
impl Default for InjuryTypeDropDodge { fn default() -> Self { Self::new() } }

impl InjuryTypeServer for InjuryTypeDropDodge {
    fn handle_injury(&mut self, game: &Game, rng: &mut GameRng, attacker_id: Option<&str>, defender_id: &str,
        coord: FieldCoordinate, from_coord: Option<FieldCoordinate>, _old_ctx: Option<&InjuryContext>, apo_mode: ApothecaryMode) {
        self.ctx.defender_id = Some(defender_id.to_owned());
        self.ctx.attacker_id = attacker_id.map(str::to_owned);
        self.ctx.defender_coordinate = Some(coord);
        self.ctx.apothecary_mode = apo_mode;
        let defender_ignores = game.player(defender_id)
            .map(|p| p.has_unused_skill_with_property(NamedProperties::IGNORES_ARMOUR_MODIFIERS_FROM_SKILLS))
            .unwrap_or(false);
        if !self.ctx.armor_broken {
            if !defender_ignores {
                if game.player(defender_id)
                    .map(|p| p.has_skill_property(NamedProperties::BLOCKS_LIKE_CHAINSAW))
                    .unwrap_or(false)
                {
                    self.ctx.add_armor_modifier(ARMOR_CHAINSAW_3);
                }
            }
            do_armor_roll(game, rng, &mut self.ctx, defender_id);
        }
        // Java: avOrInjModifierSkill — Arm Bar's armour-OR-injury mutual exclusion. An
        // opponent with Arm Bar adjacent to the dodge's FROM square first tries to break
        // armour (+1, re-check); if the armour broke WITHOUT it, the +1 shifts to the
        // injury roll instead (chaos seed 3 i=15: home2's fall, armour [4,5]=9 vs AV10 →
        // Arm Bar +1 → broken → injury 8 → KO; Rust left it prone and fell 2 dice behind).
        let mut av_or_inj_skill_player: Option<String> = None;
        if self.use_arm_bar_modifiers && !defender_ignores {
            if let Some(from) = from_coord {
                av_or_inj_skill_player = self.find_av_or_inj_skill_player(game, defender_id, from);
            }
        }
        if !self.ctx.armor_broken && av_or_inj_skill_player.is_some() {
            self.ctx.add_armor_modifier(ffb_mechanics::modifiers::Modifier::new("Arm Bar", 1, game.rules));
            crate::injury::recalc_armor_broken(game, &mut self.ctx, defender_id);
            av_or_inj_skill_player = None;
        }
        if self.ctx.armor_broken {
            // Java: `factory.findInjuryModifiers(game, injuryContext, pAttacker, pDefender,
            // isStab(), isFoul(), isVomitLike())` — DropDodge doesn't override any of
            // isStab/isFoul/isVomitLike, so all three are false (inherited InjuryType defaults).
            if let Some(defender) = game.player(defender_id) {
                let attacker = attacker_id.and_then(|aid| game.player(aid));
                let factory = InjuryModifierFactory::new(game.rules);
                for m in factory.find_injury_modifiers(game, attacker, defender, false, false, false) {
                    self.ctx.add_injury_modifier(leak_injury_modifier(m.as_ref(), attacker, defender, game.rules));
                }
            }
            // Java: if (avOrInjModifierSkill != null) add its INJURY modifiers — the armour
            // broke on its own, so Arm Bar's +1 applies to the injury roll instead.
            if av_or_inj_skill_player.is_some() {
                self.ctx.add_injury_modifier(ffb_mechanics::modifiers::Modifier::new("Arm Bar", 1, game.rules));
            }
            do_injury_roll_for_player(rng, &mut self.ctx, game, defender_id);
        }
        else { self.ctx.injury = Some(PlayerState::new(PS_PRONE)); }
    }
    fn injury_context(&self) -> &InjuryContext { &self.ctx }
    fn injury_context_mut(&mut self) -> &mut InjuryContext { &mut self.ctx }
    fn falling_down_causes_turnover(&self) -> bool { true }
    /// Java: `DropDodge()` constructor passes `SendToBoxReason.DODGE_FAIL`.
    fn send_to_box_reason(&self) -> Option<SendToBoxReason> { Some(SendToBoxReason::DodgeFail) }
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
        let mut t = InjuryTypeDropDodge::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(13), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert_eq!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn armor_break_results_in_injury_roll() {
        let mut t = InjuryTypeDropDodge::new(); let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(2), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken); assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }
    #[test]
    fn causes_turnover() { assert!(InjuryTypeDropDodge::new().falling_down_causes_turnover()); }
    #[test]
    fn send_to_box_reason_is_dodge_fail() {
        assert_eq!(InjuryTypeDropDodge::new().send_to_box_reason(), Some(SendToBoxReason::DodgeFail));
    }

    #[test]
    fn new_with_arm_bar_stores_arm_bar_player() {
        let t = InjuryTypeDropDodge::new_with_arm_bar(Some("arm_bar".into()), true);
        assert_eq!(t.arm_bar_player_id.as_deref(), Some("arm_bar"));
        assert!(t.use_arm_bar_modifiers);
    }

    #[test]
    fn pre_broken_armor_skips_armor_roll() {
        let mut t = InjuryTypeDropDodge::new();
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.handle_injury(&game_with_armor(7), &mut rng, None, "p1", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken);
        assert_ne!(t.ctx.injury.map(|s| s.base()), Some(PS_PRONE));
    }

    /// isStab/isFoul/isVomitLike are all false for DropDodge, so Mighty Blow applies
    /// normally — proves the factory is now reached.
    fn make_player(id: &str, armour: i32, skills: Vec<ffb_model::enums::SkillId>) -> ffb_model::model::player::Player {
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

    fn game_with_attacker_and_defender(attacker_skills: Vec<ffb_model::enums::SkillId>) -> Game {
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("attacker", 7, attacker_skills));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("defender", 7, vec![]));
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn mighty_blow_adds_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        use ffb_model::enums::SkillId;
        let game = game_with_attacker_and_defender(vec![SkillId::MightyBlow]);
        let mut t = InjuryTypeDropDodge::new();
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }

    /// Regression (chaos seed 3 i=15): an opponent with Arm Bar adjacent to the dodge's
    /// FROM square adds +1 to the fall armour roll when the unmodified roll failed —
    /// armour [4,5]=9 vs AV10 must BREAK with the Arm Bar +1 (Java armour-or-injury
    /// mutual exclusion, armour side).
    #[test]
    fn arm_bar_adjacent_to_from_square_breaks_armour() {
        use ffb_model::enums::{SkillId, PlayerState, PS_STANDING};
        let mut home = crate::step::framework::test_team("home", 0);
        home.players.push(make_player("faller", 10, vec![]));
        let mut away = crate::step::framework::test_team("away", 0);
        away.players.push(make_player("armbar", 7, vec![SkillId::ArmBar]));
        let mut game = Game::new(home, away, Rules::Bb2025);
        let from = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("faller", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("faller", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("armbar", FieldCoordinate::new(5, 6)); // adjacent to FROM
        game.field_model.set_player_state("armbar", PlayerState::new(PS_STANDING));

        // Find an rng seed whose first two d6 sum to exactly AV-1 (9 vs AV10: fails raw,
        // breaks with +1). Deterministic scan keeps the test seed-stable.
        let mut seed = 0u64;
        loop {
            let mut probe = GameRng::new(seed);
            if probe.d6() + probe.d6() == 9 { break; }
            seed += 1;
            assert!(seed < 10_000, "no suitable seed found");
        }
        let mut rng = GameRng::new(seed);
        let mut t = InjuryTypeDropDodge::new();
        t.handle_injury(&game, &mut rng, None, "faller", FieldCoordinate::new(6, 5),
            Some(from), None, ApothecaryMode::Defender);
        assert!(t.ctx.armor_broken, "Arm Bar +1 must break AV10 on a 9 roll");
        assert!(t.ctx.armor_modifiers.iter().any(|m| m.name == "Arm Bar"));
        assert!(!t.ctx.injury_modifiers.iter().any(|m| m.name == "Arm Bar"),
            "used on armour → must NOT also apply to injury");

        // Without the from coordinate, no Arm Bar applies and the 9 stays unbroken.
        let mut rng2 = GameRng::new(seed);
        let mut t2 = InjuryTypeDropDodge::new();
        t2.handle_injury(&game, &mut rng2, None, "faller", FieldCoordinate::new(6, 5),
            None, None, ApothecaryMode::Defender);
        assert!(!t2.ctx.armor_broken);
    }

    #[test]
    fn no_mighty_blow_no_injury_modifier() {
        use ffb_mechanics::modifiers::Modifier;
        let game = game_with_attacker_and_defender(vec![]);
        let mut t = InjuryTypeDropDodge::new();
        t.ctx.armor_broken = true;
        let mut rng = GameRng::new(1);
        t.handle_injury(&game, &mut rng, Some("attacker"), "defender", coord(), None, None, ApothecaryMode::Defender);
        assert!(!t.ctx.injury_modifiers.contains(&Modifier::new("Mighty Blow", 1, game.rules)));
    }
}
