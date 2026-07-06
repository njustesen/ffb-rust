/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2020.ttm.StepRightStuff`.
///
/// Step in TTM sequence to handle skill RIGHT_STUFF (landing roll).
/// - If player state is FALLING (thrown out of bounds) or on a box coordinate:
///   skip landing roll, publish END_TURN + THROWN_PLAYER_COORDINATE(null).
/// - Restore player state to `old_player_state` (BB2020 change) and sync ball.
/// - If FUMBLE + kickedPlayer → fumbled-KTM injury (no landing roll).
/// - If drop_thrown_player == false: roll landing (minimumRollRightStuff + modifiers).
///   - Success + has ball → touchdown check.
///   - Success without ball on ball square → SCATTER_BALL.
///   - Failure → re-roll if available.
/// - If roll fails or drop_thrown_player: TTMLanding / FumbledKtm injury.
///
/// BB2020 differences vs BB2016:
///  - Restores player state via `old_player_state` (OLD_DEFENDER_STATE), not thrownPlayerState.
///  - Publishes THROWN_PLAYER_STATE (oldPlayerState) after restoring.
///  - Adds `passResult` and `kickedPlayer` fields.
///  - Adds `goToOnSuccess` label (GOTO_LABEL_ON_SUCCESS).
///  - On fumbled KTM → InjuryTypeFumbledKtm instead of InjuryTypeTTMLanding.
///  - Uses `playerCoordinate.isBoxCoordinate()` guard (trapdoor).
///
/// RightStuffModifierFactory + AgilityMechanic.minimumRollRightStuff → wired.
/// UtilServerInjury.handleInjury wired for both TTMLanding and FumbledKtm paths.
/// SppMechanic.addCompletion wired for accurate TTM pass (non-kicked player).
/// `isBoxCoordinate()` guard wired — skips landing roll when player is in the dugout.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::{PlayerState, PassResult as ModelPassResult, PS_FALLING, ApothecaryMode, ReRollSource};
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter, CatchScatterThrowInMode};
use ffb_mechanics::modifiers::right_stuff_modifier_factory::RightStuffModifierFactory;
use ffb_mechanics::modifiers::right_stuff_context::RightStuffContext;
use ffb_mechanics::pass_result::PassResult as MechanicPassResult;
use crate::dice_interpreter::DiceInterpreter;
use crate::injury::injuryType::injury_type_ttm_landing::InjuryTypeTTMLanding;
use crate::injury::injuryType::injury_type_fumbled_ktm::InjuryTypeFumbledKtm;
use crate::step::util_server_injury;
use crate::step::util_server_steps::check_touchdown;
use crate::step::abstract_step_with_re_roll::{ReRollState, find_skill_reroll_source};
use crate::step::util_server_re_roll::{ask_for_reroll_if_available, use_reroll};

/// Java: `StepRightStuff` (bb2020/ttm).
pub struct StepRightStuff {
    /// Java: `fThrownPlayerHasBall`
    thrown_player_has_ball: Option<bool>,
    /// Java: `fThrownPlayerId`
    thrown_player_id: Option<String>,
    /// Java: `fDropThrownPlayer`
    drop_thrown_player: bool,
    /// Java: `passResult` — BB2020 addition.
    pass_result: Option<ModelPassResult>,
    /// Java: `kickedPlayer` — BB2020 addition.
    kicked_player: bool,
    /// Java: `goToOnSuccess` — BB2020 addition.
    goto_on_success: Option<String>,
    /// Java: `oldPlayerState` (OLD_DEFENDER_STATE) — BB2020 addition.
    old_player_state: Option<PlayerState>,
    /// Java: AbstractStepWithReRoll fields
    re_roll_state: ReRollState,
    /// Cached roll value (0 = not yet rolled).
    roll: i32,
}

impl StepRightStuff {
    pub fn new() -> Self {
        Self {
            thrown_player_has_ball: None,
            thrown_player_id: None,
            drop_thrown_player: false,
            pass_result: None,
            kicked_player: false,
            goto_on_success: None,
            old_player_state: None,
            re_roll_state: ReRollState::new(),
            roll: 0,
        }
    }

    fn execute_step(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        let player_id = match &self.thrown_player_id {
            Some(id) => id.clone(),
            None     => return StepOutcome::next(),
        };
        let has_ball = self.thrown_player_has_ball.unwrap_or(false);
        let player_coord = game.field_model.player_coordinate(&player_id);

        // BB2020: skip landing roll when FALLING or on a box coordinate.
        let is_falling = game.field_model.player_state(&player_id)
            .map(|s| s.base() == PS_FALLING)
            .unwrap_or(false);
        let is_box_coord = player_coord.map(|c| c.is_box_coordinate()).unwrap_or(false);

        if is_falling || is_box_coord {
            return StepOutcome::next()
                .publish(StepParameter::EndTurn(has_ball))
                .publish(StepParameter::ThrownPlayerCoordinate(None));
        }

        // BB2020: restore player state to old_player_state before the roll.
        if let Some(old) = self.old_player_state {
            game.field_model.set_player_state(&player_id, old);
        }
        // Publish restored state so downstream steps see the right value.
        let out_state = self.old_player_state.unwrap_or_default();

        // Sync ball to player coordinate when holding ball.
        if has_ball {
            if let Some(coord) = player_coord {
                game.field_model.ball_coordinate = Some(coord);
            }
        }

        // BB2020: fumbled KTM path.
        let fumbled_ktm = self.pass_result == Some(ModelPassResult::Fumble) && self.kicked_player;

        let already_rerolled = self.re_roll_state.re_rolled_action
            .as_ref().map(|a| a.name == "RIGHT_STUFF").unwrap_or(false);
        let mut do_roll = !self.drop_thrown_player && !fumbled_ktm;

        // Java: if (reRolledAction == RIGHT_STUFF) { if (source == null || !useReRoll) doRoll = false; }
        if do_roll && already_rerolled {
            let source_opt = self.re_roll_state.re_roll_source.clone();
            let consumed = source_opt.as_ref().map(|s| use_reroll(game, s, &player_id)).unwrap_or(false);
            if !consumed {
                do_roll = false;
            }
        }

        if do_roll {
            let minimum_roll = if let Some(player) = game.player(&player_id) {
                let factory = RightStuffModifierFactory::for_rules(game.rules);
                let mechanic_pass_result = self.pass_result.map(|r| match r {
                    ModelPassResult::Fumble | ModelPassResult::MissedCatch => MechanicPassResult::FUMBLE,
                    ModelPassResult::Inaccurate => MechanicPassResult::INACCURATE,
                    ModelPassResult::WildlyInaccurate => MechanicPassResult::WILDLY_INACCURATE,
                    _ => MechanicPassResult::ACCURATE,
                });
                let ctx = RightStuffContext::new_full(game, player, mechanic_pass_result, None);
                let mods = factory.find_applicable(&ctx);
                RightStuffModifierFactory::minimum_roll(player.agility as i32, &mods)
            } else {
                4
            };
            if self.roll == 0 {
                self.roll = rng.d6();
            }
            let successful = DiceInterpreter::is_skill_roll_successful(self.roll, minimum_roll);
            if successful {
                // Java: if (passResult == ACCURATE && !kickedPlayer && thrower != null)
                //   spp.addCompletion(additionalCompletionSppTeams, playerResult(thrower))
                if self.pass_result == Some(ModelPassResult::Complete) && !self.kicked_player {
                    if let Some(ref thrower_id) = game.thrower_id.clone() {
                        let is_home = game.team_home.has_player(thrower_id);
                        let team_result = if is_home {
                            &mut game.game_result.home
                        } else {
                            &mut game.game_result.away
                        };
                        team_result.player_result_mut(thrower_id).completions += 1;
                    }
                }
                let success_label = self.goto_on_success.as_deref().unwrap_or("");
                let mut out = StepOutcome::goto(success_label)
                    .publish(StepParameter::ThrownPlayerState(out_state))
                    .publish(StepParameter::ThrownPlayerCoordinate(None));
                if has_ball {
                    if check_touchdown(game) {
                        out = out.publish(StepParameter::EndTurn(true));
                    }
                } else {
                    let ball_coord = game.field_model.ball_coordinate;
                    if player_coord.is_some() && player_coord == ball_coord {
                        game.field_model.ball_moving = true;
                        out = out.publish(StepParameter::CatchScatterThrowInMode(CatchScatterThrowInMode::ScatterBall));
                    }
                }
                return out;
            }

            // Failure: offer re-roll if not yet re-rolled
            if !already_rerolled {
                use ffb_model::model::re_rolled_action::ReRolledAction;
                self.re_roll_state.re_rolled_action = Some(ReRolledAction::new("RIGHT_STUFF"));

                let skill_source = find_skill_reroll_source(game, "RIGHT_STUFF");
                if let Some(source) = skill_source {
                    use_reroll(game, &source, &player_id);
                    self.re_roll_state.re_roll_source = Some(source);
                    self.roll = 0;
                    return self.execute_step(game, rng);
                }

                if let Some(prompt) = ask_for_reroll_if_available(game, "RIGHT_STUFF", minimum_roll, false) {
                    self.re_roll_state.re_roll_source = Some(ReRollSource::new("TRR"));
                    self.roll = 0;
                    return StepOutcome::cont().with_prompt(prompt);
                }
            }
            // Failed roll, no re-roll available: fall through to drop path.
        }

        // Drop path (drop_thrown_player == true OR fumbled_ktm OR failed roll).
        // Java: UtilServerInjury.handleInjury(fumbledKtm → FumbledKtm else TTMLanding).
        let coord = game.field_model.player_coordinate(&player_id)
            .unwrap_or(ffb_model::types::FieldCoordinate::new(0, 0));
        let ir = if fumbled_ktm {
            let mut injury_type = InjuryTypeFumbledKtm::new();
            util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &player_id, coord, None, None,
                ApothecaryMode::Defender,
            )
        } else {
            let mut injury_type = InjuryTypeTTMLanding::new();
            util_server_injury::handle_injury(
                game, rng, &mut injury_type,
                None, &player_id, coord, None, None,
                ApothecaryMode::ThrownPlayer,
            )
        };
        ir.apply_to(game);
        let mut out = StepOutcome::next()
            .publish(StepParameter::ThrownPlayerState(out_state))
            .publish(StepParameter::ThrownPlayerCoordinate(None));
        if has_ball {
            out = out.publish(StepParameter::EndTurn(true));
        }
        out
    }
}

impl Default for StepRightStuff {
    fn default() -> Self { Self::new() }
}

impl Step for StepRightStuff {
    fn id(&self) -> StepId { StepId::RightStuff }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::UseReRoll { use_reroll: true } => {
                self.execute_step(game, rng)
            }
            Action::UseReRoll { use_reroll: false } => {
                self.re_roll_state.re_roll_source = None;
                self.execute_step(game, rng)
            }
            _ => self.execute_step(game, rng),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::ThrownPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::ThrownPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            StepParameter::DropThrownPlayer(v)    => { self.drop_thrown_player = *v; true }
            StepParameter::PassResultParam(v)     => { self.pass_result = Some(*v); true }
            StepParameter::IsKickedPlayer(v)      => { self.kicked_player = *v; true }
            StepParameter::GotoLabelOnSuccess(s)  => { self.goto_on_success = Some(s.clone()); true }
            StepParameter::OldDefenderState(v)    => { self.old_player_state = Some(*v); true }
            // Also accept kicked-player aliases.
            StepParameter::KickedPlayerHasBall(v) => { self.thrown_player_has_ball = Some(*v); true }
            StepParameter::KickedPlayerId(v)      => { self.thrown_player_id = v.clone(); true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{StepAction, test_team};
    use ffb_model::enums::Rules;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020)
    }

    #[test]
    fn id_is_right_stuff() {
        assert_eq!(StepRightStuff::new().id(), StepId::RightStuff);
    }

    #[test]
    fn no_thrown_player_returns_next() {
        let mut game = make_game();
        let out = StepRightStuff::new().start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.action, StepAction::NextStep));
    }

    #[test]
    fn drop_thrown_player_publishes_coordinate_null() {
        let mut game = make_game();
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.drop_thrown_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }

    #[test]
    fn set_parameter_drop_thrown_player() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::DropThrownPlayer(true)));
        assert!(step.drop_thrown_player);
    }

    #[test]
    fn set_parameter_pass_result() {
        let mut step = StepRightStuff::new();
        assert!(step.set_parameter(&StepParameter::PassResultParam(ModelPassResult::Fumble)));
        assert_eq!(step.pass_result, Some(ModelPassResult::Fumble));
    }

    #[test]
    fn set_parameter_old_defender_state() {
        use ffb_model::enums::{PlayerState, PS_STANDING};
        let mut step = StepRightStuff::new();
        let state = PlayerState::new(PS_STANDING);
        assert!(step.set_parameter(&StepParameter::OldDefenderState(state)));
        assert_eq!(step.old_player_state, Some(state));
    }

    #[test]
    fn fumbled_ktm_drops_player_without_roll() {
        let mut game = make_game();
        let mut step = StepRightStuff::new();
        step.thrown_player_id = Some("p1".into());
        step.pass_result = Some(ModelPassResult::Fumble);
        step.kicked_player = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Fumbled KTM → no landing roll → drop path → NEXT_STEP.
        assert!(matches!(out.action, StepAction::NextStep));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::ThrownPlayerCoordinate(None))));
    }
}
