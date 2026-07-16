/// 1:1 translation of `com.fumbbl.ffb.server.step.action.block.StepStab` (COMMON rules), which
/// is a thin shell dispatching to one of three per-edition skill-hook classes:
/// `com.fumbbl.ffb.server.skillbehaviour.{bb2016,bb2020,bb2025}.StabBehaviour`.
///
/// Handles the Stab special action: the attacker rolls armor against the defender instead of
/// blocking normally.  Only runs when USING_STAB = true was set by a preceding step.
///
/// Per-edition behaviour (matched on `game.rules`, per the Phase ABI convention of hand-inlining
/// per-edition skill-hook logic directly into the shared step rather than routing through the
/// (unused, for this family of skills) generic `SkillBehaviour`/`StepModifier` hook dispatch):
///
/// - **bb2016**: gate = `hasSkill(actingPlayer, Stab)`. `InjuryTypeStab(useInjuryModifiers=false)`.
///   On armor break: also calls `dropPlayer` and publishes its params. Publishes the raw
///   `InjuryResult`. `GOTO_LABEL` to `goToLabelOnSuccess` on success; else `NEXT_STEP`.
/// - **bb2020**: gate = `hasUnusedSkillWithProperty(actingPlayer, canPerformArmourRollInsteadOfBlock)`.
///   `InjuryTypeStab(useInjuryModifiers=true)`. Publishes a `DropPlayerContext` (embedding the
///   goto label and `requiresArmourBreak=true`) instead of a raw `InjuryResult`. Always `NEXT_STEP`.
/// - **bb2025**: same gate as bb2020, plus picks `InjuryTypeStabForSpp` vs `InjuryTypeStab`
///   (both `useInjuryModifiers=true`) based on `grantsSppFromSpecialActionsCas`. Same
///   `DropPlayerContext` publish shape and unconditional `NEXT_STEP` as bb2020.
///
/// Needs GOTO_LABEL_ON_SUCCESS init parameter.
/// Expects USING_STAB parameter from a preceding step.
use ffb_model::enums::{ApothecaryMode, Rules, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::drop_player_context::DropPlayerContext;
use crate::injury::injuryType::injury_type_stab::InjuryTypeStab;
use crate::injury::injuryType::injury_type_stab_for_spp::InjuryTypeStabForSpp;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player_no_sph, handle_injury};

pub struct StepStab {
    /// Java: state.goToLabelOnSuccess — GOTO_LABEL_ON_SUCCESS init parameter.
    pub goto_label_on_success: String,
    /// Java: state.usingStab — set by USING_STAB parameter from a preceding step.
    pub using_stab: Option<bool>,
}

impl StepStab {
    pub fn new() -> Self {
        Self {
            goto_label_on_success: String::new(),
            using_stab: None,
        }
    }
}

impl Default for StepStab {
    fn default() -> Self { Self::new() }
}

impl Step for StepStab {
    fn id(&self) -> StepId { StepId::Stab }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::UsingStab(v)          => { self.using_stab = Some(*v); true }
            _ => false,
        }
    }
}

impl StepStab {
    /// Java: `StabBehaviour.handleExecuteStepHook` — dispatches to the per-edition logic below.
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        // Java: `(state.usingStab != null) && state.usingStab` — checked by all three editions
        // before their own skill gate.
        if self.using_stab != Some(true) {
            return StepOutcome::next();
        }

        let defender_id = match game.defender_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        match game.rules {
            Rules::Bb2016 => self.execute_bb2016(game, rng, &player_id, &defender_id),
            Rules::Bb2020 => self.execute_bb2020_bb2025(game, rng, &player_id, &defender_id, false),
            Rules::Bb2025 | Rules::Common => self.execute_bb2020_bb2025(game, rng, &player_id, &defender_id, true),
        }
    }

    /// Java: `bb2016.StabBehaviour.handleExecuteStepHook`.
    fn execute_bb2016(&self, game: &mut Game, rng: &mut GameRng, player_id: &str, defender_id: &str) -> StepOutcome {
        let has_skill = game.player(player_id).map(|p| p.has_skill(SkillId::Stab)).unwrap_or(false);
        if !has_skill {
            return StepOutcome::next();
        }

        let defender_coord = game.field_model.player_coordinate(defender_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        let mut injury_type = InjuryTypeStab::new(false);
        let injury_result = handle_injury(
            game, rng, &mut injury_type,
            Some(player_id), defender_id,
            defender_coord, None, None,
            ApothecaryMode::Defender,
        );

        let mut outcome = StepOutcome::goto(&self.goto_label_on_success);
        if injury_result.injury_context().is_armor_broken() {
            for param in drop_player_no_sph(game, defender_id) {
                outcome = outcome.publish(param);
            }
        }
        outcome.publish(StepParameter::InjuryResult(Box::new(injury_result)))
    }

    /// Java: `bb2020.StabBehaviour.handleExecuteStepHook` / `bb2025.StabBehaviour.handleExecuteStepHook`.
    /// `grants_spp_check` selects between BB2020 (never checks) and BB2025 (checks
    /// `grantsSppFromSpecialActionsCas` to pick `InjuryTypeStabForSpp` over `InjuryTypeStab`).
    fn execute_bb2020_bb2025(
        &self,
        game: &mut Game,
        rng: &mut GameRng,
        player_id: &str,
        defender_id: &str,
        grants_spp_check: bool,
    ) -> StepOutcome {
        let has_property = game.player(player_id)
            .map(|p| UtilCards::has_unused_skill_with_property(p, NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK))
            .unwrap_or(false);

        if !has_property {
            return StepOutcome::next();
        }

        let defender_coord = game.field_model.player_coordinate(defender_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        let grants_spp = grants_spp_check
            && game.player(player_id)
                .map(|p| UtilCards::has_skill_with_property(p, NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS))
                .unwrap_or(false);

        let injury_result = if grants_spp {
            let mut injury_type = InjuryTypeStabForSpp::new(true);
            handle_injury(game, rng, &mut injury_type, Some(player_id), defender_id, defender_coord, None, None, ApothecaryMode::Defender)
        } else {
            let mut injury_type = InjuryTypeStab::new(true);
            handle_injury(game, rng, &mut injury_type, Some(player_id), defender_id, defender_coord, None, None, ApothecaryMode::Defender)
        };

        // Java: `new DropPlayerContext(injuryResultDefender, false, true, state.goToLabelOnSuccess,
        // game.getDefenderId(), ApothecaryMode.DEFENDER, true)`.
        let mut dpc = DropPlayerContext::with_injury(injury_result, defender_id.to_owned(), ApothecaryMode::Defender, true);
        dpc.label = Some(self.goto_label_on_success.clone());
        dpc.requires_armour_break = true;

        StepOutcome::next().publish(StepParameter::DropPlayerContext(Box::new(dpc)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::enums::PlayerState;

    fn add_player(
        team: &mut ffb_model::model::team::Team,
        id: &str,
        nr: i32,
        skills: Vec<SkillId>,
        armour: i32,
    ) {
        team.players.push(ffb_model::model::player::Player {
            id: id.into(),
            name: id.into(),
            nr,
            position_id: "pos".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4,
            armour,
            starting_skills: skills
                .into_iter()
                .map(|s| SkillWithValue { skill_id: s, value: None })
                .collect(),
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        });
    }

    fn make_stab_game(rules: Rules, attacker_skills: Vec<SkillId>, defender_armour: i32) -> Game {
        let mut home = test_team("home", 0);
        let mut away = test_team("away", 0);
        add_player(&mut home, "att", 1, attacker_skills, 8);
        add_player(&mut away, "def", 2, vec![], defender_armour);
        let mut game = Game::new(home, away, rules);
        game.home_playing = true;
        game.acting_player.player_id = Some("att".into());
        game.defender_id = Some("def".into());
        game.field_model.set_player_coordinate("att", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("def", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("att", PlayerState::new(PS_STANDING).change_active(true));
        game.field_model.set_player_state("def", PlayerState::new(PS_STANDING));
        game
    }

    fn seed_for_2d6(target_sum: i32) -> u64 {
        for s in 0u64..100_000 {
            let mut rng = GameRng::new(s);
            if rng.d6() + rng.d6() == target_sum { return s; }
        }
        panic!("no seed for 2d6={}", target_sum);
    }

    // ── shared gate checks (using_stab / no acting player) ──────────────────

    #[test]
    fn not_using_stab_returns_next() {
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        // using_stab = None (not set) → NEXT_STEP
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn using_stab_false_returns_next() {
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        step.using_stab = Some(false);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn no_acting_player_returns_next() {
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 8);
        game.acting_player.player_id = None;
        let mut step = StepStab::new();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn no_defender_returns_next() {
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 8);
        game.defender_id = None;
        let mut step = StepStab::new();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    // ── bb2016: hasSkill gate, InjuryTypeStab(false), raw InjuryResult, separate dropPlayer ──

    #[test]
    fn bb2016_no_stab_skill_returns_next() {
        let mut game = make_stab_game(Rules::Bb2016, vec![], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
    }

    #[test]
    fn bb2016_armor_not_broken_goes_to_label_without_defender_prone() {
        let seed = seed_for_2d6(8); // 8 <= 10 → not broken
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 10);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_OK".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        assert_eq!(outcome.goto_label.as_deref(), Some("STAB_OK"));
        assert_ne!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        // Raw InjuryResult published (not a DropPlayerContext) with armor not broken.
        let has_injury_result = outcome.published.iter().any(|p| {
            if let StepParameter::InjuryResult(r) = p {
                !r.injury_context().is_armor_broken() && r.injury_context().get_injury_roll().is_none()
            } else { false }
        });
        assert!(has_injury_result);
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
    }

    #[test]
    fn bb2016_armor_broken_drops_defender_and_rolls_injury() {
        let seed = seed_for_2d6(12); // 12 > 2 → breaks
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 2);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::GotoLabel);
        // Java: separate UtilServerInjury.dropPlayer call places the defender PRONE.
        assert_eq!(game.field_model.player_state("def").unwrap().base(), PS_PRONE);
        let injury_ok = outcome.published.iter().any(|p| {
            if let StepParameter::InjuryResult(r) = p {
                r.injury_context().is_armor_broken() && r.injury_context().get_injury_roll().is_some()
            } else { false }
        });
        assert!(injury_ok);
    }

    #[test]
    fn bb2016_use_injury_modifiers_false_skips_niggling_modifier() {
        // Java: bb2016 constructs `new InjuryTypeStab(false)` — niggling/skill injury modifiers
        // never apply, unlike bb2020/bb2025's `new InjuryTypeStab(true)`.
        let seed = seed_for_2d6(12);
        let mut game = make_stab_game(Rules::Bb2016, vec![SkillId::Stab], 2);
        game.team_away.players.iter_mut().find(|p| p.id == "def").unwrap().niggling_injuries = 1;
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        let injury_result = outcome.published.iter().find_map(|p| {
            if let StepParameter::InjuryResult(r) = p { Some(r) } else { None }
        }).expect("InjuryResult should be published");
        assert!(injury_result.injury_context().injury_modifiers.is_empty(),
            "bb2016 must not apply injury modifiers, got {:?}", injury_result.injury_context().injury_modifiers);
    }

    // ── bb2020/bb2025: property gate, DropPlayerContext publish, unconditional NEXT_STEP ────

    #[test]
    fn bb2020_missing_property_returns_next_and_publishes_nothing() {
        let mut game = make_stab_game(Rules::Bb2020, vec![], 8);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.is_empty());
    }

    #[test]
    fn bb2020_has_property_publishes_drop_player_context_and_always_next_step() {
        // Even on a "successful" stab, bb2020/bb2025 always NEXT_STEP — the goto label is
        // embedded in the DropPlayerContext and consumed later by StepHandleDropPlayerContext.
        let seed = seed_for_2d6(12); // armor breaks
        let mut game = make_stab_game(Rules::Bb2020, vec![SkillId::Stab], 2);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
        let dpc = outcome.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(c) = p { Some(c) } else { None }
        }).expect("DropPlayerContext should be published");
        assert_eq!(dpc.label.as_deref(), Some("STAB_HURT"));
        assert!(dpc.requires_armour_break);
        assert_eq!(dpc.player_id.as_deref(), Some("def"));
        assert_eq!(dpc.apothecary_mode, Some(ApothecaryMode::Defender));
        assert!(dpc.eligible_for_safe_pair_of_hands);
        // No raw InjuryResult published directly (only wrapped inside the context).
        assert!(!outcome.published.iter().any(|p| matches!(p, StepParameter::InjuryResult(_))));
    }

    #[test]
    fn bb2020_armor_not_broken_still_publishes_context_and_next_step() {
        let seed = seed_for_2d6(8);
        let mut game = make_stab_game(Rules::Bb2020, vec![SkillId::Stab], 10);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_OK".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        assert_eq!(outcome.action, StepAction::NextStep);
        assert!(outcome.published.iter().any(|p| matches!(p, StepParameter::DropPlayerContext(_))));
    }

    // ── bb2025: grantsSppFromSpecialActionsCas picks InjuryTypeStabForSpp ────────────────────

    #[test]
    fn bb2025_without_violent_innovator_uses_plain_injury_type_stab() {
        // Java: `Stab` is constructed with `worthSpps=false` — plain InjuryTypeStab never
        // awards a casualty SPP, unlike InjuryTypeStabForSpp (`worthSpps=true`).
        let seed = seed_for_2d6(12);
        let mut game = make_stab_game(Rules::Bb2025, vec![SkillId::Stab], 2);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        let dpc = outcome.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(c) = p { Some(c) } else { None }
        }).expect("DropPlayerContext should be published");
        let injury_result = dpc.injury_result.as_ref().expect("injury result present");
        assert!(!injury_result.injury_context().is_worth_spps);
        assert!(injury_result.injury_context().is_caused_by_opponent);
    }

    #[test]
    fn bb2025_with_violent_innovator_uses_injury_type_stab_for_spp() {
        let seed = seed_for_2d6(12);
        let mut game = make_stab_game(Rules::Bb2025, vec![SkillId::Stab, SkillId::ViolentInnovator], 2);
        let mut step = StepStab::new();
        step.goto_label_on_success = "STAB_HURT".into();
        step.using_stab = Some(true);
        let outcome = step.start(&mut game, &mut GameRng::new(seed));
        let dpc = outcome.published.iter().find_map(|p| {
            if let StepParameter::DropPlayerContext(c) = p { Some(c) } else { None }
        }).expect("DropPlayerContext should be published");
        let injury_result = dpc.injury_result.as_ref().expect("injury result present");
        assert!(injury_result.injury_context().is_worth_spps);
        assert!(injury_result.injury_context().is_caused_by_opponent);
    }

    #[test]
    fn set_parameter_stores_fields() {
        let mut step = StepStab::new();
        assert!(step.set_parameter(&StepParameter::GotoLabelOnSuccess("G".into())));
        assert!(step.set_parameter(&StepParameter::UsingStab(true)));
        assert_eq!(step.goto_label_on_success, "G");
        assert_eq!(step.using_stab, Some(true));
        assert!(!step.set_parameter(&StepParameter::EndPlayerAction(true)));
    }
}
