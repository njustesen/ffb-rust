use ffb_model::enums::{PlayerAction, SkillId};
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::rng::GameRng;
use ffb_model::util::util_cards::UtilCards;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::abstract_step_with_re_roll::ReRollState;
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};
use ffb_mechanics::mechanics::minimum_roll_base_bb2016;
use ffb_mechanics::modifiers::bb2016::gaze_modifier_collection::GazeModifierCollection;
use ffb_mechanics::modifiers::gaze_modifier_context::GazeModifierContext;
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
/// DEFERRED(sound-client): SoundId.HYPNO not yet ported.
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
        let defender_on_other_team = defender_id.as_deref()
            .map(|def| !game.is_active_team_player(def))
            .unwrap_or(false);
        let do_gaze_initial = player_action == Some(PlayerAction::Gaze)
            && defender_id.is_some()
            && defender_on_other_team;

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
            // Java: gazeSkill.isPresent() && !hasSkillToCancelProperty(actingPlayer, inflictsConfusion)
            let has_gaze_skill = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| p.has_skill_property(NamedProperties::INFLICTS_CONFUSION))
                .unwrap_or(false);
            let has_cancel = player_id.as_deref()
                .and_then(|id| game.player(id))
                .map(|p| UtilCards::has_skill_to_cancel_property(p, NamedProperties::INFLICTS_CONFUSION))
                .unwrap_or(false);
            has_gaze_skill && !has_cancel
        };

        if do_gaze {
            // Java: actingPlayer.markSkillUsed(gazeSkill)
            if let Some(pid) = player_id.as_deref() {
                if let Some(p) = game.team_home.player_mut(pid).or_else(|| game.team_away.player_mut(pid)) {
                    p.used_skills.insert(SkillId::HypnoticGaze);
                }
            }

            let roll = rng.d6();

            let player = player_id.as_deref().and_then(|id| game.player(id));
            let agility = player.map(|p| p.agility_with_modifiers()).unwrap_or(3);
            let gaze_col = GazeModifierCollection::new();
            let modifier_total: i32 = if let Some(p) = player {
                let ctx = GazeModifierContext::new(game, p);
                gaze_col.find_applicable(&ctx).iter().map(|m| m.get_modifier()).sum()
            } else {
                0
            };
            let minimum_roll = minimum_roll_base_bb2016(agility, modifier_total);
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
    use ffb_model::enums::{Rules, SkillId};
    use ffb_model::model::player::Player;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2016)
    }

    fn setup_gaze_game() -> (Game, StepHypnoticGaze) {
        let mut game = make_game();
        let mut attacker = Player::default();
        attacker.id = "a1".into();
        attacker.agility = 3;
        attacker.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        game.team_home.players.push(attacker);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(5, 5));

        let mut defender = Player::default();
        defender.id = "d1".into();
        game.team_away.players.push(defender);
        game.field_model.set_player_coordinate("d1", FieldCoordinate::new(6, 5));

        game.home_playing = true;
        game.acting_player.player_id = Some("a1".into());
        game.acting_player.player_action = Some(PlayerAction::Gaze);
        game.defender_id = Some("d1".into());

        let step = StepHypnoticGaze::new("end".into());
        (game, step)
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

    #[test]
    fn defender_on_same_team_skips_gaze() {
        // Defender is on home team (same as acting player) → do_gaze_initial = false
        let (mut game, mut step) = setup_gaze_game();
        // Override defender to be on home team
        let mut same_team_def = Player::default();
        same_team_def.id = "d_same".into();
        game.team_home.players.push(same_team_def);
        game.field_model.set_player_coordinate("d_same", FieldCoordinate::new(6, 5));
        game.defender_id = Some("d_same".into());

        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn gaze_marks_skill_used_on_gazer() {
        let (mut game, mut step) = setup_gaze_game();
        game.turn_data_home.rerolls = 0;
        step.start(&mut game, &mut GameRng::new(0));
        let used = game.team_home.player("a1")
            .map(|p| p.used_skills.contains(&SkillId::HypnoticGaze))
            .unwrap_or(false);
        assert!(used);
    }

    #[test]
    fn gaze_with_opposite_team_defender_goes_to_end_label() {
        let (mut game, mut step) = setup_gaze_game();
        game.turn_data_home.rerolls = 0;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }

    #[test]
    fn ball_and_chain_cancels_hypnotic_gaze() {
        // BallAndChain has cancelsInflictsConfusion — player with both skills cannot gaze.
        // Since do_gaze=false but do_gaze_initial=true, gaze is attempted but fails skill check,
        // goto_end_label=true → GotoLabel (end player action).
        let (mut game, mut step) = setup_gaze_game();
        game.turn_data_home.rerolls = 0;
        game.team_home.player_mut("a1").unwrap()
            .starting_skills.push(SkillWithValue::new(SkillId::BallAndChain));
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.goto_label.as_deref(), Some("end"));
    }
}
