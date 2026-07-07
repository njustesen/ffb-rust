use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::prompts::AgentPrompt;
use ffb_model::util::rng::GameRng;
use crate::action::{Action, PlayerActionChoice};
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};
use crate::step::util_server_steps;

/// Initialises the player-selection phase (BB2020): waits for `ActivatePlayer` or `EndTurn`
/// commands, then dispatches to the appropriate action sequence via GOTO_LABEL or NEXT_STEP.
///
/// Java executeStep routing:
///   end_turn → GotoLabel(end) + publish EndTurn + CheckForgo
///   end_player_action → GotoLabel(end) + publish EndPlayerAction
///   dispatch_player_action set + acting player present:
///     publish DispatchPlayerAction
///     if standing_up && !force_goto → NextStep (proceed to JumpUp/StandUp)
///     else → GotoLabel(end)
///   otherwise → Continue (waiting for command)
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2020.shared.StepInitSelecting`.
pub struct StepInitSelecting {
    /// Java: fGotoLabelOnEnd (init param)
    pub goto_label_on_end: String,
    /// Java: fDispatchPlayerAction
    pub dispatch_player_action: Option<PlayerAction>,
    /// Java: fEndTurn
    pub end_turn: bool,
    /// Java: fEndPlayerAction
    pub end_player_action: bool,
    /// Java: forceGotoOnDispatch
    pub force_goto_on_dispatch: bool,
}

impl StepInitSelecting {
    pub fn new(goto_label_on_end: String) -> Self {
        Self {
            goto_label_on_end,
            dispatch_player_action: None,
            end_turn: false,
            end_player_action: false,
            force_goto_on_dispatch: false,
        }
    }
}

impl Step for StepInitSelecting {
    fn id(&self) -> StepId { StepId::InitSelecting }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        // Java start() only updates persistence — it does NOT call executeStep().
        // Emit the activation prompt so the agent knows which players are available.
        let team = if game.home_playing { &game.team_home } else { &game.team_away };
        let eligible: Vec<_> = team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
            .map(|p| (p.id.clone(), vec![PlayerAction::Move]))
            .collect();
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::EndTurn => {
                self.end_turn = true;
            }
            Action::ActivatePlayer { player_id, player_action, block_defender_id } => {
                let pa = pac_to_player_action(*player_action);
                util_server_steps::change_player_action(game, player_id, pa, false);
                if let Some(def_id) = block_defender_id {
                    game.defender_id = Some(def_id.clone());
                }
                // Block/Blitz variants: go directly to label (forceGotoOnDispatch)
                self.force_goto_on_dispatch = matches!(
                    player_action,
                    PlayerActionChoice::Block | PlayerActionChoice::Blitz
                );
                self.dispatch_player_action = Some(pa);
            }
            _ => {}
        }
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::EndTurn(v) => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v) => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

impl StepInitSelecting {
    fn execute_step(&self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        let label = &self.goto_label_on_end;

        if self.end_turn {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndTurn(true))
                .publish(StepParameter::CheckForgo(true));
        }
        if self.end_player_action {
            return StepOutcome::goto(label)
                .publish(StepParameter::EndPlayerAction(true));
        }
        if let Some(dispatch) = self.dispatch_player_action {
            if game.acting_player.player_id.is_some() {
                let standing_up = game.acting_player.standing_up;
                let outcome = StepOutcome::next()
                    .publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                if standing_up && !self.force_goto_on_dispatch {
                    return outcome;
                } else {
                    return StepOutcome::goto(label)
                        .publish(StepParameter::DispatchPlayerAction(Some(dispatch)));
                }
            }
        }
        // Waiting: build activation prompt
        let team = if game.home_playing { &game.team_home } else { &game.team_away };
        let eligible: Vec<_> = team.players.iter()
            .filter(|p| game.field_model.player_coordinate(&p.id).is_some())
            .map(|p| (p.id.clone(), vec![PlayerAction::Move]))
            .collect();
        StepOutcome::cont()
            .with_prompt(AgentPrompt::ActivatePlayer { eligible_players: eligible })
    }
}

/// Maps `PlayerActionChoice` (Rust engine action) to `PlayerAction` (model enum).
fn pac_to_player_action(pac: PlayerActionChoice) -> PlayerAction {
    match pac {
        PlayerActionChoice::Move => PlayerAction::Move,
        PlayerActionChoice::Blitz => PlayerAction::Blitz,
        PlayerActionChoice::Block => PlayerAction::Block,
        PlayerActionChoice::Stab => PlayerAction::Stab,
        PlayerActionChoice::Foul => PlayerAction::Foul,
        PlayerActionChoice::Pass => PlayerAction::Pass,
        PlayerActionChoice::HandOff => PlayerAction::HandOver,
        PlayerActionChoice::StandUp => PlayerAction::StandUp,
        PlayerActionChoice::StandUpBlitz => PlayerAction::StandUpBlitz,
        PlayerActionChoice::ThrowTeamMate => PlayerAction::ThrowTeamMate,
        PlayerActionChoice::KickTeamMate => PlayerAction::KickTeamMate,
        PlayerActionChoice::HypnoticGaze => PlayerAction::Gaze,
        PlayerActionChoice::ThrowBomb => PlayerAction::ThrowBomb,
        PlayerActionChoice::Swoop => PlayerAction::Swoop,
        PlayerActionChoice::Punt => PlayerAction::Punt,
        PlayerActionChoice::BreatheFire => PlayerAction::BreatheFire,
        PlayerActionChoice::ProjectileVomit => PlayerAction::ProjectileVomit,
        PlayerActionChoice::SecureTheBall => PlayerAction::SecureTheBall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2020)
    }

    #[test]
    fn start_returns_cont() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn set_parameter_end_turn_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndTurn(true)));
        assert!(step.end_turn);
    }

    #[test]
    fn set_parameter_end_player_action_accepted() {
        let mut step = StepInitSelecting::new("end".into());
        assert!(step.set_parameter(&StepParameter::EndPlayerAction(true)));
        assert!(step.end_player_action);
    }

    #[test]
    fn start_emits_activate_player_prompt() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end".into());
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(matches!(out.prompt, Some(AgentPrompt::ActivatePlayer { .. })));
    }

    #[test]
    fn end_turn_returns_goto_label_with_end_turn_param() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn end_player_action_returns_goto_label() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("end_label".into());
        step.end_player_action = true;
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn activate_player_move_sets_dispatch_and_returns_goto_label() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end_label".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Move,
            block_defender_id: None,
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end_label"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::DispatchPlayerAction(_))));
    }

    #[test]
    fn activate_player_block_sets_force_goto_on_dispatch() {
        let mut game = make_game();
        game.acting_player.player_id = Some("p1".into());
        let mut step = StepInitSelecting::new("end".into());
        let action = Action::ActivatePlayer {
            player_id: "p1".into(),
            player_action: PlayerActionChoice::Block,
            block_defender_id: Some("def".into()),
        };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert!(step.force_goto_on_dispatch);
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn pac_to_player_action_all_variants() {
        assert_eq!(pac_to_player_action(PlayerActionChoice::Move), PlayerAction::Move);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Block), PlayerAction::Block);
        assert_eq!(pac_to_player_action(PlayerActionChoice::Foul), PlayerAction::Foul);
        assert_eq!(pac_to_player_action(PlayerActionChoice::HandOff), PlayerAction::HandOver);
    }

    #[test]
    fn end_turn_command_publishes_end_turn_and_check_forgo() {
        let mut game = make_game();
        let mut step = StepInitSelecting::new("lbl".into());
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("lbl"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::CheckForgo(true))));
    }

    #[test]
    fn report_list_empty_after_headless_start() {
        // Java's addReport calls (ReportFumblerooskie, ReportSkillUse for GAIN_HAIL_MARY,
        // ADD_BLOCK_DIE) are all in client-dialog paths that headless does not exercise.
        // Verify no spurious reports are emitted on plain start().
        let mut game = make_game();
        let mut step = StepInitSelecting::new("lbl".into());
        step.start(&mut game, &mut GameRng::new(0));
        assert!(game.report_list.is_empty(),
            "headless start() should emit no reports");
    }
}
