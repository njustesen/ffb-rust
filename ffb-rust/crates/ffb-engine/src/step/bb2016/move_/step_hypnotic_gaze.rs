use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::mechanics::minimum_roll_hypnotic_gaze;
use crate::dice_interpreter::DiceInterpreter;

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.move.StepHypnoticGaze.
///
/// Resolves the HYPNOTIC_GAZE skill (inflictsConfusion property).
///
/// Init params: GOTO_LABEL_ON_END (mandatory).
///
/// Logic:
/// - doGaze = (playerAction==GAZE) && (defender!=null) && (defender.team != actingTeam)
/// - If !doGaze → NEXT_STEP, clear defenderId
/// - If re-rolling: useReRoll or → gotoEndLabel
/// - Else: check gazeSkill present && !hasCancelSkill → markSkillUsed, roll, check success
///   - Success: set defender state to hypnotized
///   - Failure: offer TRR if available (gotoEndLabel=false)
/// - If gotoEndLabel: publish END_PLAYER_ACTION(true), GOTO_LABEL(fGotoLabelOnEnd), clear defender
///
/// Sets stepParameter END_PLAYER_ACTION for all steps on the stack.
///
/// DEFERRED(gazeModifiers): GazeModifierFactory / modifier collection not yet ported.
/// DEFERRED(cancelsSkill): UtilCards.hasSkillToCancelProperty not yet ported.
/// DEFERRED(sound): SoundId.HYPNO not yet ported.
pub struct StepHypnoticGaze {
    /// Java: fGotoLabelOnEnd
    pub goto_label_on_end: String,
    /// Java: AbstractStepWithReRoll fields
    pub re_roll_state: ReRollState,
}

impl StepHypnoticGaze {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            re_roll_state: ReRollState::new(),
        }
    }
}

impl Default for StepHypnoticGaze {
    fn default() -> Self { Self::new(String::new()) }
}

impl Step for StepHypnoticGaze {
    fn id(&self) -> StepId { StepId::HypnoticGaze }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            _ => false,
        }
    }
}

impl StepHypnoticGaze {
    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = game.acting_player.player_id.clone();
        let player_action = game.acting_player.player_action;

        // Java: doGaze = (playerAction==GAZE) && (defender!=null) && (defender.team != actingTeam)
        let defender_id = game.defender_id.clone();
        let do_gaze_initial = player_action == Some(PlayerAction::Gaze)
            && defender_id.is_some();
        // TODO(teamCheck): defender.team != actingTeam check not yet ported

        if !do_gaze_initial {
            game.defender_id = None;
            return StepOutcome::next();
        }

        let mut goto_end_label = true;

        // Check if re-rolling
        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "HYPNOTIC_GAZE").unwrap_or(false);

        let do_gaze = if already_rerolled {
            let pid = player_id.as_deref().unwrap_or("").to_owned();
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt
                .as_ref()
                .map(|s| use_reroll(game, s, &pid))
                .unwrap_or(false);
            consumed
        } else {
            // Java: gazeSkill.isPresent() && !hasSkillToCancelProperty
            // TODO(cancelsSkill): hasSkillToCancelProperty not yet ported
            let has_gaze_skill = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::INFLICTS_CONFUSION))
                .unwrap_or(false);
            has_gaze_skill
        };

        if do_gaze {
            // Java: actingPlayer.markSkillUsed(gazeSkill)
            // TODO: markSkillUsed not yet ported

            let roll = rng.d6();

            // TODO(gazeModifiers): use GazeModifierFactory — stub minimum roll
            let agility = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.agility as i32)
                .unwrap_or(3);
            let minimum_roll = minimum_roll_hypnotic_gaze(agility, &[]);
            let successful = DiceInterpreter::is_skill_roll_successful(roll, minimum_roll);

            if successful {
                // Java: if (!oldVictimState.isConfused() && !oldVictimState.isHypnotized())
                //           setPlayerState(defender, oldState.changeHypnotized(true))
                if let Some(def_id) = defender_id.as_deref() {
                    if let Some(old_state) = game.field_model.player_state(def_id) {
                        if !old_state.is_confused() && !old_state.is_hypnotized() {
                            game.field_model.set_player_state(def_id, old_state.change_hypnotized(true));
                        }
                    }
                }
            } else if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("HYPNOTIC_GAZE"));

                if let Some(prompt) = ask_for_reroll_if_available(game, "HYPNOTIC_GAZE", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ffb_model::enums::ReRollSource::new("TRR"));
                    return StepOutcome::cont().with_prompt(prompt);
                }
                // No re-roll available — fall through to gotoEndLabel
            }
        }

        if goto_end_label {
            let label = self.goto_label_on_end.clone();
            game.defender_id = None;
            StepOutcome::goto(&label)
                .publish(StepParameter::EndPlayerAction(true))
        } else {
            StepOutcome::next()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn no_gaze_action_returns_next_step() {
        let mut game = make_game();
        // PlayerAction is None — not gazing
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_action_but_no_defender_returns_next_step() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        // No defender set
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_with_defender_but_no_gaze_skill_goes_to_end_label() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("victim".into());
        // No player with gaze skill → do_gaze=false → gotoEndLabel
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn gaze_clears_defender_id_on_end() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("victim".into());
        let mut step = StepHypnoticGaze::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn no_gaze_clears_defender_id() {
        let mut game = make_game();
        game.defender_id = Some("victim".into());
        let mut step = StepHypnoticGaze::new("end".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.defender_id.is_none());
    }

    #[test]
    fn set_parameter_goto_label_on_end_accepted() {
        let mut step = StepHypnoticGaze::new("old".into());
        assert!(step.set_parameter(&StepParameter::GotoLabelOnEnd("new".into())));
        assert_eq!(step.goto_label_on_end, "new");
    }

    #[test]
    fn end_player_action_published_on_goto() {
        let mut game = make_game();
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("victim".into());
        let mut step = StepHypnoticGaze::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn unrecognised_parameter_returns_false() {
        let mut step = StepHypnoticGaze::new("end".into());
        assert!(!step.set_parameter(&StepParameter::EndTurn(true)));
    }
}
