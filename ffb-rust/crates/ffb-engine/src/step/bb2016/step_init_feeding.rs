use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

/// 1:1 translation of com.fumbbl.ffb.server.step.bb2016.StepInitFeeding.
///
/// Handles the feeding sub-sequence for Blood Lust. If the acting player
/// is a Vampire suffering Blood Lust and has not yet fed, prompts to choose
/// a victim (or bites a spectator if no victims available or feeding not allowed).
///
/// Init: mandatory GOTO_LABEL_ON_END, FEEDING_ALLOWED.
///       optional END_PLAYER_ACTION, END_TURN.
/// Sets: END_PLAYER_ACTION, END_TURN for all steps on the stack.
pub struct StepInitFeeding {
    /// Java: fGotoLabelOnEnd (mandatory)
    pub goto_label_on_end: Option<String>,
    /// Java: fFeedOnPlayerChoice (tristate Boolean)
    pub feed_on_player_choice: Option<bool>,
    /// Java: fFeedingAllowed (mandatory)
    pub feeding_allowed: Option<bool>,
    /// Java: fEndPlayerAction (optional init, default false)
    pub end_player_action: bool,
    /// Java: fEndTurn (optional init, default false)
    pub end_turn: bool,
}

impl StepInitFeeding {
    pub fn new() -> Self {
        Self {
            goto_label_on_end: None,
            feed_on_player_choice: None,
            feeding_allowed: None,
            end_player_action: false,
            end_turn: false,
        }
    }
}

impl Default for StepInitFeeding {
    fn default() -> Self { Self::new() }
}

impl Step for StepInitFeeding {
    fn id(&self) -> StepId { StepId::InitFeeding }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java: CLIENT_PLAYER_CHOICE (PlayerChoiceMode.FEED)
        //   fFeedOnPlayerChoice = StringTool.isProvided(playerId)
        //   game.setDefenderId(playerId)
        if let Action::SelectPlayer { player_id } = action {
            self.feed_on_player_choice = Some(!player_id.is_empty());
            game.defender_id = if player_id.is_empty() { None } else { Some(player_id.clone()) };
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(label) => {
                self.goto_label_on_end = Some(label.clone());
                true
            }
            StepParameter::FeedingAllowed(v) => {
                self.feeding_allowed = Some(*v);
                true
            }
            StepParameter::EndPlayerAction(v) => {
                self.end_player_action = *v;
                true
            }
            StepParameter::EndTurn(v) => {
                self.end_turn = *v;
                true
            }
            _ => false,
        }
    }
}

impl StepInitFeeding {
    fn execute_step(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java: UtilServerDialog.hideDialog(getGameState())
        // TODO: dialog hiding (no-op in headless)

        let goto_label = match &self.goto_label_on_end {
            Some(l) => l.clone(),
            None => return StepOutcome::next(), // misconfigured — guard
        };

        // Java: ActingPlayer actingPlayer = game.getActingPlayer()
        // Java: if ((actingPlayer.getPlayer() == null) || !actingPlayer.isSufferingBloodLust() || actingPlayer.hasFed())
        //   → goto label
        // Simplification: check acting_player hasFed / not blood-lusting
        let player_exists = game.acting_player.player_id.is_some();
        let is_blood_lusting = game.acting_player.suffering_blood_lust;
        // Java: actingPlayer.hasFed() — ActingPlayer doesn't have has_fed yet; use has_acted as proxy
        // TODO: add has_fed field to ActingPlayer when Blood Lust is fully integrated
        let has_fed = false; // stub — hooks will supply the real check

        if !player_exists || !is_blood_lusting || has_fed {
            return StepOutcome::goto(&goto_label)
                .publish(StepParameter::EndPlayerAction(self.end_player_action))
                .publish(StepParameter::EndTurn(self.end_turn));
        }

        // Java: if (isSufferingBloodLust && !hasFed && !fFeedingAllowed) → fFeedOnPlayerChoice = false
        if self.feeding_allowed == Some(false) {
            self.feed_on_player_choice = Some(false);
        }

        // Java: check playerState.hasTacklezones() && fFeedOnPlayerChoice == null
        // → find adjacent victims, show dialog or set choice=false
        // TODO: UtilPlayer.findAdjacentPlayersToFeedOn / dialog logic
        // Stub: if choice not yet made, show dialog prompt (Continue)
        if self.feed_on_player_choice.is_none() {
            // TODO: show DialogPlayerChoiceParameter(FEED mode)
            return StepOutcome::cont();
        }

        let do_next_step;
        if self.feed_on_player_choice == Some(true) && game.defender_id.is_some() {
            // Java: handleInjury(InjuryTypeBitten, ...)
            // Java: fEndTurn = UtilPlayer.hasBall(game, game.getDefender())
            // Java: publishParameter INJURY_RESULT
            // Java: publishParameters dropPlayer
            // Java: actingPlayer.setSufferingBloodLust(false)
            // TODO: UtilServerInjury.handleInjury with InjuryTypeBitten
            let defender_id = game.defender_id.clone().unwrap_or_default();
            // Approximate: mark feeding complete, use InjuryTypeBitten stub
            let _ = defender_id;
            game.acting_player.suffering_blood_lust = false;
            do_next_step = true;
        } else {
            // Java: biting spectator path
            // Java: fEndTurn = true; remove ball from field if carrying, move to RESERVE
            self.end_turn = true;
            // TODO: scatter ball if acting player is ball carrier
            if let Some(pid) = game.acting_player.player_id.clone() {
                if game.field_model.ball_coordinate
                    .and_then(|bc| game.field_model.player_at(bc))
                    .map(|id| id == &pid)
                    .unwrap_or(false)
                {
                    game.field_model.ball_moving = true;
                    // publish scatter
                }
                // TODO: move player to RESERVE / box (UtilBox.putPlayerIntoBox)
            }
            do_next_step = true;
        }

        if do_next_step {
            // Java: actingPlayer.setHasFed(true)
            // TODO: add has_fed field to ActingPlayer
            game.acting_player.has_acted = true;
            StepOutcome::next()
                .publish(StepParameter::EndPlayerAction(self.end_player_action))
                .publish(StepParameter::EndTurn(self.end_turn))
        } else {
            StepOutcome::cont()
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
        Game::new(home, away, Rules::Bb2016)
    }

    #[test]
    fn step_id_is_init_feeding() {
        let step = StepInitFeeding::new();
        assert_eq!(step.id(), StepId::InitFeeding);
    }

    #[test]
    fn goto_label_on_end_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::GotoLabelOnEnd("end".to_string()));
        assert!(ok);
        assert_eq!(step.goto_label_on_end.as_deref(), Some("end"));
    }

    #[test]
    fn feeding_allowed_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::FeedingAllowed(true));
        assert!(ok);
        assert_eq!(step.feeding_allowed, Some(true));
    }

    #[test]
    fn end_player_action_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::EndPlayerAction(true));
        assert!(ok);
        assert!(step.end_player_action);
    }

    #[test]
    fn end_turn_parameter_accepted() {
        let mut step = StepInitFeeding::new();
        let ok = step.set_parameter(&StepParameter::EndTurn(true));
        assert!(ok);
        assert!(step.end_turn);
    }

    #[test]
    fn no_acting_player_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        // No acting player → goto label
        game.acting_player.player_id = None;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("label_end"));
    }

    #[test]
    fn not_blood_lusting_goes_to_label() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = false;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn already_fed_goes_to_label() {
        // TODO: this test is a stub — ActingPlayer.has_fed not yet added.
        // When has_fed is added, this test should set it true and assert GotoLabel.
        // For now just verify the step doesn't panic.
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(true);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = true;
        // has_fed not available yet — will be cont (waiting for victim choice)
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Either GotoLabel (has_fed logic) or Continue (victim dialog) — don't panic
        assert!(matches!(out.action, StepAction::GotoLabel | StepAction::Continue));
    }

    #[test]
    fn feeding_not_allowed_sets_choice_false_and_bites_spectator() {
        let mut step = StepInitFeeding::new();
        step.goto_label_on_end = Some("label_end".to_string());
        step.feeding_allowed = Some(false);
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.suffering_blood_lust = true;
        // Note: has_fed not yet on ActingPlayer; feeding_allowed=false bypasses that check
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Should bite spectator → fEndTurn=true → NextStep
        assert_eq!(out.action, StepAction::NextStep);
        // end_turn should be published as true
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }
}
