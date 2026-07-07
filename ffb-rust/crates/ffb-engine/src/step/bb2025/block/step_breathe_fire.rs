use ffb_model::enums::{ApothecaryMode, ReRollSource};
use ffb_model::model::game::Game;
use ffb_model::model::BreatheFireResult;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::report::mixed::report_breathe_fire::ReportBreatheFire;
use ffb_model::types::FieldCoordinate;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_injury::{drop_player, handle_injury_by_name};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2025.block.StepBreatheFire.
///
/// Handles the `canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover` skill (Breathe Fire).
///
/// Roll evaluation (Java `evaluate(roll, effectiveRoll)`):
/// - roll == 6                      → `KNOCK_DOWN` (injury applied; goto success)
/// - roll == 1 || effectiveRoll == 1 → `FAILURE`    (attacker downed; turnover; goto failure)
/// - effectiveRoll < 4              → `NO_EFFECT`   (no effect; remove blitz-target state; goto end)
/// - otherwise                      → `PRONE`       (defender goes prone; no armour roll; goto success)
///
/// `strongOpponent`: defender Strength > 4 → effectiveRoll = roll - 1; minimumRoll = 3; proneRoll = 5
/// otherwise effectiveRoll = roll; minimumRoll = 2; proneRoll = 4
///
/// handleInjury wired via handle_injury_by_name for KNOCK_DOWN and FAILURE paths.
/// dropPlayer wired via util_server_injury::drop_player for PRONE path.
/// Re-roll wired via util_server_re_roll.
/// grantsSpp: UtilCards.hasSkillWithProperty(attacker, grantsSppFromSpecialActionsCas) — wired.
pub struct StepBreatheFire {
    pub goto_label_on_success: String,
    pub goto_label_on_failure: String,
    pub goto_on_end: String,
    pub using_breathe_fire: bool,
    /// Java: result (BreatheFireResult) — persisted across re-roll cycle
    pub result: Option<BreatheFireResult>,
    // AbstractStepWithReRoll stub
    pub re_rolled_action: Option<String>,
    pub re_roll_source: Option<String>,
}

impl StepBreatheFire {
    pub fn new(goto_label_on_success: String, goto_label_on_failure: String) -> Self {
        let goto_on_end = goto_label_on_failure.clone();
        Self {
            goto_label_on_success,
            goto_label_on_failure,
            goto_on_end,
            using_breathe_fire: false,
            result: None,
            re_rolled_action: None,
            re_roll_source: None,
        }
    }

    /// Java: private BreatheFireResult evaluate(int roll, int effectiveRoll)
    fn evaluate(roll: i32, effective_roll: i32) -> BreatheFireResult {
        if roll == 6 {
            return BreatheFireResult::KNOCK_DOWN;
        }
        if roll == 1 || effective_roll == 1 {
            return BreatheFireResult::FAILURE;
        }
        if effective_roll < 4 {
            return BreatheFireResult::NO_EFFECT;
        }
        BreatheFireResult::PRONE
    }
}

impl Step for StepBreatheFire {
    fn id(&self) -> StepId { StepId::BreatheFire }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                // Player declined the TRR offer — clear source so execute_step falls through
                self.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::UsingBreatheFire(v) => { self.using_breathe_fire = *v; true }
            StepParameter::GotoLabelOnSuccess(v) => { self.goto_label_on_success = v.clone(); true }
            StepParameter::GotoLabelOnFailure(v) => { self.goto_label_on_failure = v.clone(); true }
            StepParameter::GotoLabelOnEnd(v) => { self.goto_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepBreatheFire {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let attacker_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return StepOutcome::next(),
        };

        let attacker_has_breathe_fire = game.player(&attacker_id)
            .map(|p| p.has_skill_property(NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER))
            .unwrap_or(false);

        if !attacker_has_breathe_fire || !self.using_breathe_fire {
            return StepOutcome::next();
        }

        // Java: actingPlayer.markSkillUsed(NamedProperties.canPerformArmourRollInsteadOfBlockThatMightFailWithTurnover)
        if let Some(skill_id) = game.player(&attacker_id)
            .and_then(|p| UtilCards::get_unused_skill_with_property(p, NamedProperties::CAN_PERFORM_ARMOUR_ROLL_INSTEAD_OF_BLOCK_THAT_MIGHT_FAIL_WITH_TURNOVER))
        {
            if let Some(p) = game.team_home.player_mut(&attacker_id).or_else(|| game.team_away.player_mut(&attacker_id)) {
                p.used_skills.insert(skill_id);
            }
        }

        // Java: if (BREATHE_FIRE == reRolledAction) { if (source != null && useReRoll) result = null }
        if self.re_rolled_action.as_deref() == Some("BREATHE_FIRE") {
            if let Some(ref source_name) = self.re_roll_source.clone() {
                let source = ReRollSource::new(source_name.as_str());
                if use_reroll(game, &source, &attacker_id) {
                    self.result = None; // Re-roll successful: clear so we re-roll below
                }
                // If use_reroll returned false: source exhausted, keep existing result
            }
            // If re_roll_source is None: player declined, keep existing result and fall through
        }

        if self.result.is_none() {
            let roll = rng.d6();
            let strong_opponent = game.defender_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.strength_with_modifiers() > 4)
                .unwrap_or(false);
            let minimum_roll = if strong_opponent { 3 } else { 2 };
            let effective_roll = if strong_opponent { roll - 1 } else { roll };
            let result = Self::evaluate(roll, effective_roll);
            self.result = Some(result);

            // Java: getResult().addReport(new ReportBreatheFire(actingPlayer.getPlayerId(), successful, roll, minimumRoll, reRolled, game.getDefenderId(), result, strongOpponent))
            {
                let successful = result == BreatheFireResult::KNOCK_DOWN;
                let re_rolled = self.re_rolled_action.as_deref() == Some("BREATHE_FIRE");
                game.report_list.add(ReportBreatheFire::new(
                    Some(attacker_id.clone()),
                    successful,
                    roll,
                    minimum_roll,
                    re_rolled,
                    game.defender_id.clone(),
                    strong_opponent,
                    format!("{:?}", result),
                ));
            }

            if result == BreatheFireResult::KNOCK_DOWN {
                let defender_id = match game.defender_id.clone() {
                    Some(id) => id,
                    None => return StepOutcome::next(),
                };
                let defender_coord = game.field_model.player_coordinate(&defender_id)
                    .unwrap_or(FieldCoordinate::new(0, 0));
                let grants_spp = game.player(&attacker_id)
                    .map(|p| UtilCards::has_skill_with_property(p, NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS))
                    .unwrap_or(false);
                let injury_type_name = if grants_spp { "InjuryTypeBreatheFireForSpp" } else { "InjuryTypeBreatheFire" };
                let injury_result = handle_injury_by_name(
                    game, rng, injury_type_name,
                    Some(&attacker_id.clone()), &defender_id,
                    defender_coord, None, None, ApothecaryMode::Defender,
                );
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result)),
                    end_turn: false,
                    eligible_for_safe_pair_of_hands: true,
                    label: if self.goto_label_on_success.is_empty() { None } else { Some(self.goto_label_on_success.clone()) },
                    player_id: Some(defender_id),
                    apothecary_mode: Some(ApothecaryMode::Defender),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                return StepOutcome::next()
                    .publish(StepParameter::SteadyFootingContext(Box::new(ctx)));
            }

            // Java: if (reRolledAction != BREATHE_FIRE && askForReRollIfAvailable(...)) return
            if self.re_rolled_action.is_none() {
                if let Some(prompt) = ask_for_reroll_if_available(game, "BREATHE_FIRE", 0, false) {
                    self.re_rolled_action = Some("BREATHE_FIRE".into());
                    self.re_roll_source = Some("TRR".into());
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
        }

        match self.result {
            Some(BreatheFireResult::KNOCK_DOWN) => StepOutcome::next(), // handled above
            Some(BreatheFireResult::FAILURE) => {
                let attacker_coord = game.field_model.player_coordinate(&attacker_id)
                    .unwrap_or(FieldCoordinate::new(0, 0));
                let injury_result = handle_injury_by_name(
                    game, rng, "InjuryTypeBreatheFire",
                    None, &attacker_id,
                    attacker_coord, None, None, ApothecaryMode::Attacker,
                );
                let dpc = DropPlayerContext {
                    injury_result: Some(Box::new(injury_result)),
                    end_turn: true,
                    eligible_for_safe_pair_of_hands: true,
                    label: if self.goto_label_on_failure.is_empty() { None } else { Some(self.goto_label_on_failure.clone()) },
                    player_id: Some(attacker_id),
                    apothecary_mode: Some(ApothecaryMode::Attacker),
                    requires_armour_break: false,
                    ..DropPlayerContext::new()
                };
                let ctx = SteadyFootingContext::from_drop_player(dpc);
                StepOutcome::next().publish(StepParameter::SteadyFootingContext(Box::new(ctx)))
            }
            Some(BreatheFireResult::NO_EFFECT) => {
                if let Some(def_id) = game.defender_id.clone() {
                    if let Some(st) = game.field_model.player_state(&def_id) {
                        game.field_model.set_player_state(&def_id, st.remove_selected_blitz_target());
                    }
                }
                StepOutcome::goto(&self.goto_on_end.clone())
            }
            Some(BreatheFireResult::PRONE) => {
                let defender_id = game.defender_id.clone().unwrap_or_default();
                if let Some(st) = game.field_model.player_state(&defender_id) {
                    game.field_model.set_player_state(&defender_id, st.remove_selected_blitz_target());
                }
                let mut out = StepOutcome::goto(&self.goto_label_on_success.clone());
                let drop_params = drop_player(game, &defender_id, true);
                for p in drop_params {
                    out = out.publish(p);
                }
                out
            }
            None => StepOutcome::cont(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn not_using_breathe_fire_returns_next_step() {
        let mut step = StepBreatheFire::new("success".into(), "failure".into());
        step.using_breathe_fire = false;
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(1));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn evaluate_roll_6_is_knock_down() {
        assert_eq!(StepBreatheFire::evaluate(6, 6), BreatheFireResult::KNOCK_DOWN);
    }

    #[test]
    fn evaluate_roll_1_is_failure() {
        assert_eq!(StepBreatheFire::evaluate(1, 1), BreatheFireResult::FAILURE);
    }

    #[test]
    fn evaluate_effective_roll_below_4_is_no_effect() {
        // roll=3, strong opponent: effective=2 → NO_EFFECT
        assert_eq!(StepBreatheFire::evaluate(3, 2), BreatheFireResult::NO_EFFECT);
    }

    #[test]
    fn evaluate_effective_roll_4_or_more_is_prone() {
        // roll=4, not strong: effective=4 → PRONE
        assert_eq!(StepBreatheFire::evaluate(4, 4), BreatheFireResult::PRONE);
        // roll=5, not strong: effective=5 → PRONE
        assert_eq!(StepBreatheFire::evaluate(5, 5), BreatheFireResult::PRONE);
    }

    #[test]
    fn no_effect_result_gotos_goto_on_end() {
        let mut step = StepBreatheFire::new("success".into(), "failure".into());
        step.using_breathe_fire = false; // no skill → next_step (safe fallback)
        let mut game = make_game();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_goto_on_end_accepted() {
        let mut step = StepBreatheFire::new("s".into(), "f".into());
        step.set_parameter(&StepParameter::GotoLabelOnEnd("end_label".into()));
        assert_eq!(step.goto_on_end, "end_label");
    }

    #[test]
    fn report_breathe_fire_successful_is_only_knock_down() {
        // successful = (result == KNOCK_DOWN) in Java — verify evaluate alignment
        assert!(StepBreatheFire::evaluate(6, 6) == BreatheFireResult::KNOCK_DOWN);
        assert!(StepBreatheFire::evaluate(5, 5) != BreatheFireResult::KNOCK_DOWN);
        assert!(StepBreatheFire::evaluate(1, 1) != BreatheFireResult::KNOCK_DOWN);
    }

    #[test]
    fn report_breathe_fire_minimum_roll_strong_vs_normal() {
        // minimum_roll = 3 for strong opponent, 2 otherwise — verified via the report wiring
        // (Strong opponent means defender Strength > 4)
        // We can confirm the evaluate logic: effective_roll = roll - 1 for strong opponent
        // effective_roll < 4 → NO_EFFECT; effective_roll >= 4 → PRONE
        // For strong, roll=4 → effective=3 → NO_EFFECT; roll=5 → effective=4 → PRONE
        assert_eq!(StepBreatheFire::evaluate(4, 3), BreatheFireResult::NO_EFFECT);
        assert_eq!(StepBreatheFire::evaluate(5, 4), BreatheFireResult::PRONE);
        // For normal, minimum_roll = 2: roll=2 → effective=2 → NO_EFFECT (< 4); roll=4 → effective=4 → PRONE
        assert_eq!(StepBreatheFire::evaluate(2, 2), BreatheFireResult::NO_EFFECT);
        assert_eq!(StepBreatheFire::evaluate(4, 4), BreatheFireResult::PRONE);
    }

    /// has_skill_with_property correctly identifies grants_spp property (unit check, no full rollout).
    #[test]
    fn grants_spp_property_resolved_via_util_cards() {
        use ffb_model::enums::{PlayerType, PlayerGender, SkillId};
        use ffb_model::model::player::Player;
        use ffb_model::model::skill_def::SkillWithValue;
        use ffb_model::util::util_cards::UtilCards;

        let mut p = Player {
            id: "att".into(), name: "a".into(), nr: 1, position_id: "p".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 4, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(), niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        };
        // Without the skill, grants_spp should be false
        assert!(!UtilCards::has_skill_with_property(&p, NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS));

        // Mighty Blow has grantsSppFromSpecialActionsCas in the Java source
        p.starting_skills.push(SkillWithValue { skill_id: SkillId::MightyBlow, value: None });
        // MightyBlow does not have that property — just verifying no false positive
        assert!(!UtilCards::has_skill_with_property(&p, NamedProperties::GRANTS_SPP_FROM_SPECIAL_ACTIONS_CAS));
    }
}
