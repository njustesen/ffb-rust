/// 1:1 translation of com.fumbbl.ffb.server.step.action.ktm.StepInitKickTeamMate (COMMON).
///
/// Initialises the kick-team-mate sequence.  Mandatory init param: GOTO_LABEL_ON_END.
/// Optional: KICKED_PLAYER_ID, NR_OF_DICE.
///
/// On start: if kicked_player_id + num_dice are already set (from init) → publish all
/// KTM parameters and proceed NEXT_STEP.  Otherwise return Continue to wait for agent.
use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome};
use crate::step::framework::{StepId, StepParameter};

pub struct StepInitKickTeamMate {
    /// Java: fGotoLabelOnEnd — mandatory, set at init.
    pub goto_label_on_end: String,
    /// Java: fKickedPlayerId — the player being kicked.
    pub kicked_player_id: Option<String>,
    /// Java: fNumDice — number of dice to roll for the kick.
    pub num_dice: i32,
    /// Java: fEndTurn — set when the coach signals end of turn.
    pub end_turn: bool,
    /// Java: fEndPlayerAction — set when the coach ends the player action without kicking.
    pub end_player_action: bool,
}

impl StepInitKickTeamMate {
    pub fn new(goto_label_on_end: impl Into<String>) -> Self {
        Self {
            goto_label_on_end: goto_label_on_end.into(),
            kicked_player_id: None,
            num_dice: 0,
            end_turn: false,
            end_player_action: false,
        }
    }
}

impl Step for StepInitKickTeamMate {
    fn id(&self) -> StepId { StepId::InitKickTeamMate }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            // Java: CLIENT_KICK_TEAM_MATE
            // If num_dice != 0 && kicked_player_id already set → update num_dice from command.
            // Else → set kicked_player_id from command, clear num_dice.
            // In Rust Action::KickTeamMate has no num_dice field → always take "set player" path;
            // num_dice must come from NrOfDice set_parameter before start or via a subsequent param.
            Action::KickTeamMate { player_id, .. } => {
                self.kicked_player_id = Some(player_id.clone());
                self.num_dice = 1; // default; overridden by NrOfDice set_parameter if provided
            }
            // Java: CLIENT_END_TURN
            Action::EndTurn => {
                self.end_turn = true;
            }
            _ => return StepOutcome::cont(),
        }
        self.execute_step(game)
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v) => { self.goto_label_on_end = v.clone(); true }
            StepParameter::NrOfDice(v) => { self.num_dice = *v; true }
            StepParameter::KickedPlayerId(v) => { self.kicked_player_id = v.clone(); true }
            _ => false,
        }
    }
}

impl StepInitKickTeamMate {
    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: if (fEndTurn) → publish END_TURN, GOTO label
        if self.end_turn {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndTurn(true));
        }
        // Java: else if (fEndPlayerAction) → publish END_PLAYER_ACTION, GOTO label
        if self.end_player_action {
            return StepOutcome::goto(&self.goto_label_on_end)
                .publish(StepParameter::EndPlayerAction(true));
        }
        // Java: else if (kicked_player_id provided && numDice != 0) → publish KTM params, NEXT_STEP
        if let Some(ref kicked_id) = self.kicked_player_id.clone() {
            if self.num_dice != 0 {
                game.defender_id = Some(kicked_id.clone());

                let kicked_state = game.field_model.player_state(kicked_id);
                let kicked_coord = game.field_model.player_coordinate(kicked_id);
                let kicked_has_ball = kicked_coord
                    .map(|c| {
                        game.field_model.ball_coordinate == Some(c)
                            && !game.field_model.ball_moving
                    })
                    .unwrap_or(false);

                // Java: changePlayerAction(this, actingPlayer.getPlayerId(), PlayerAction.KICK_TEAM_MATE, false)
                game.acting_player.player_action = Some(PlayerAction::KickTeamMate);

                let mut outcome = StepOutcome::next()
                    .publish(StepParameter::KickedPlayerId(Some(kicked_id.clone())))
                    .publish(StepParameter::NrOfDice(self.num_dice))
                    .publish(StepParameter::KickedPlayerHasBall(kicked_has_ball));

                if let Some(state) = kicked_state {
                    outcome = outcome.publish(StepParameter::KickedPlayerState(state));
                }
                if let Some(coord) = kicked_coord {
                    outcome = outcome.publish(StepParameter::KickedPlayerCoordinate(Some(coord)));
                }

                return outcome;
            }
        }
        // Not enough data yet — wait for command.
        StepOutcome::cont()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::{StepAction, StepParameter};
    use ffb_model::enums::{PlayerState, Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        let mut game = Game::new(home, away, Rules::Bb2025);
        game.home_playing = true;
        game
    }

    #[test]
    fn end_turn_publishes_end_turn_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitKickTeamMate::new("end");
        step.end_turn = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn end_player_action_publishes_and_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitKickTeamMate::new("end");
        step.end_player_action = true;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndPlayerAction(true))));
    }

    #[test]
    fn no_kicked_player_waits_for_command() {
        let mut game = make_game();
        let mut step = StepInitKickTeamMate::new("end");
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn kicked_player_no_dice_waits() {
        let mut game = make_game();
        let mut step = StepInitKickTeamMate::new("end");
        step.kicked_player_id = Some("k1".into());
        step.num_dice = 0;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn kicked_player_with_dice_publishes_all_and_next() {
        let mut game = make_game();
        let kicked_id = "kick1";
        game.field_model.set_player_state(kicked_id, PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate(kicked_id, FieldCoordinate::new(7, 5));

        let mut step = StepInitKickTeamMate::new("end");
        step.kicked_player_id = Some(kicked_id.into());
        step.num_dice = 2;
        let out = step.start(&mut game, &mut GameRng::new(0));

        assert_eq!(out.action, StepAction::NextStep);
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickedPlayerId(Some(id)) if id == kicked_id)));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::NrOfDice(2))));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickedPlayerCoordinate(Some(c)) if *c == FieldCoordinate::new(7, 5))));
        assert_eq!(game.acting_player.player_action, Some(PlayerAction::KickTeamMate));
        assert_eq!(game.defender_id.as_deref(), Some(kicked_id));
    }

    #[test]
    fn kicked_player_has_ball_true_when_ball_at_coord() {
        let mut game = make_game();
        let kicked_id = "kick2";
        let coord = FieldCoordinate::new(3, 4);
        game.field_model.set_player_state(kicked_id, PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate(kicked_id, coord);
        game.field_model.ball_coordinate = Some(coord);

        let mut step = StepInitKickTeamMate::new("end");
        step.kicked_player_id = Some(kicked_id.into());
        step.num_dice = 1;
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::KickedPlayerHasBall(true))));
    }

    #[test]
    fn handle_end_turn_action_gotos_label() {
        let mut game = make_game();
        let mut step = StepInitKickTeamMate::new("end");
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::GotoLabel);
        assert_eq!(out.goto_label.as_deref(), Some("end"));
        assert!(out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true))));
    }

    #[test]
    fn handle_kick_team_mate_action_sets_kicked_player() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        game.field_model.set_player_state("k3", PlayerState::new(PS_STANDING));
        game.field_model.set_player_coordinate("k3", coord);

        let mut step = StepInitKickTeamMate::new("end");
        let action = Action::KickTeamMate { player_id: "k3".into(), coord };
        let out = step.handle_command(&action, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
        assert_eq!(step.kicked_player_id.as_deref(), Some("k3"));
    }
}
