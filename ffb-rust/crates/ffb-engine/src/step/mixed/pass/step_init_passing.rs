/// 1:1 translation of `com.fumbbl.ffb.server.step.mixed.pass.StepInitPassing`.
///
/// Initialization step of the pass sequence.  Waits for a CLIENT_PASS,
/// CLIENT_HAND_OVER, or CLIENT_ACTING_PLAYER / CLIENT_END_TURN command and sets
/// up the thrower, catcher and pass coordinate on the game object accordingly.
///
/// Init parameters (mandatory): GOTO_LABEL_ON_END.
/// Optional init: TARGET_COORDINATE, CATCHER_ID.
/// Sets: CATCHER_ID, END_TURN, END_PLAYER_ACTION (all published for downstream).
///
/// Java: `@RulesCollection(BB2020, BB2025)`, extends `AbstractStep`.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::step::framework::{Step, StepOutcome, StepId, StepParameter};

/// Java: `StepInitPassing` (mixed/pass, BB2020 + BB2025).
#[derive(Debug, Default)]
pub struct StepInitPassing {
    /// Java: `fGotoLabelOnEnd`
    pub goto_label_on_end: String,
    /// Java: `fCatcherId`
    pub catcher_id: Option<String>,
    /// Java: `fEndTurn`
    pub end_turn: bool,
    /// Java: `fEndPlayerAction`
    pub end_player_action: bool,
}

impl StepInitPassing {
    pub fn new() -> Self { Self::default() }

    fn execute_step(&mut self, game: &mut Game) -> StepOutcome {
        // Java: if (game.getThrower() == null || game.getThrowerAction() == null) { return; }
        if game.thrower_id.is_none() || game.thrower_action.is_none() {
            return StepOutcome::cont();
        }

        // Java: catcher publish
        let mut outcome = StepOutcome::next();
        if let Some(ref cid) = self.catcher_id {
            outcome = outcome.publish(StepParameter::CatcherId(Some(cid.clone())));
        }

        if self.end_turn {
            outcome = outcome.publish(StepParameter::EndTurn(true));
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .with_events(outcome.events)
                .with_published(outcome.published);
        }

        if self.end_player_action {
            outcome = outcome.publish(StepParameter::EndPlayerAction(true));
            return StepOutcome::goto(&self.goto_label_on_end.clone())
                .with_events(outcome.events)
                .with_published(outcome.published);
        }

        // Java: if suffering blood lust and not fed → goto end
        if game.acting_player.suffering_blood_lust {
            // Java: !actingPlayer.hasFed() — use has_acted proxy (TODO: add has_fed)
            let has_fed = false; // stub
            if !has_fed {
                return StepOutcome::goto(&self.goto_label_on_end.clone())
                    .with_events(outcome.events)
                    .with_published(outcome.published);
            }
        }

        // Java: pass/hand-over setup (range ruler, hasPassed, concessionPossible, etc.)
        // These mutations are on game state — perform them then advance.
        // Simplified: just mark hasPassed on actingPlayer via has_moved proxy.
        // client-only: full range ruler logic — RangeRuler is client-side display
        game.acting_player.has_moved = true; // proxy for hasPassed

        outcome
    }
}

// Extension to add published parameters to a StepOutcome (mirrors the helper in step_trap_door)
trait WithPublished {
    fn with_published(self, params: Vec<StepParameter>) -> Self;
}

impl WithPublished for StepOutcome {
    fn with_published(mut self, params: Vec<StepParameter>) -> Self {
        self.published.extend(params);
        self
    }
}

impl Step for StepInitPassing {
    fn id(&self) -> StepId { StepId::InitPassing }

    fn start(&mut self, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game)
    }

    fn handle_command(&mut self, action: &Action, game: &mut Game, _rng: &mut GameRng) -> StepOutcome {
        match action {
            Action::Pass { coord } => {
                // Java: CLIENT_PASS
                game.pass_coordinate = Some(*coord);
                // Java: catcher = fieldModel.getPlayer(passCoordinate)
                self.catcher_id = game.field_model.player_at(*coord).cloned();
                // Java: setThrowerId / setThrowerAction (use actingPlayer defaults)
                if game.thrower_id.is_none() {
                    game.thrower_id = game.acting_player.player_id.clone();
                }
                if game.thrower_action.is_none() {
                    game.thrower_action = game.acting_player.player_action;
                }
                self.execute_step(game)
            }
            Action::EndTurn => {
                // Java: CLIENT_END_TURN → fEndTurn = true
                self.end_turn = true;
                self.execute_step(game)
            }
            Action::ActivatePlayer {player_id, .. } if player_id.is_empty() => {
                // Java: CLIENT_ACTING_PLAYER with null playerId → fEndPlayerAction = true
                self.end_player_action = true;
                self.execute_step(game)
            }
            _ => StepOutcome::cont(),
        }
    }

    fn set_parameter(&mut self, param: &StepParameter) -> bool {
        match param {
            StepParameter::GotoLabelOnEnd(v)    => { self.goto_label_on_end = v.clone(); true }
            StepParameter::CatcherId(v)          => { self.catcher_id = v.clone(); true }
            StepParameter::EndTurn(v)            => { self.end_turn = *v; true }
            StepParameter::EndPlayerAction(v)    => { self.end_player_action = *v; true }
            _ => false,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::{test_team, StepAction};
    use ffb_model::enums::{Rules, PlayerAction};
    use ffb_model::model::game::Game;
    use ffb_model::util::rng::GameRng;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn id_is_init_passing() {
        assert_eq!(StepInitPassing::new().id(), StepId::InitPassing);
    }

    #[test]
    fn no_thrower_waits_for_command() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::Continue);
    }

    #[test]
    fn end_turn_set_goes_to_label() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.handle_command(&Action::EndTurn, &mut game, &mut rng);
        assert_eq!(out.action, StepAction::GotoLabel);
        let has_end_turn = out.published.iter().any(|p| matches!(p, StepParameter::EndTurn(true)));
        assert!(has_end_turn);
    }

    #[test]
    fn end_player_action_via_activate_empty_goes_to_label() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        // Java: CLIENT_ACTING_PLAYER with null playerId → endPlayerAction
        let out = step.handle_command(
            &Action::ActivatePlayer {player_id: "".into(),
                player_action: crate::action::PlayerActionChoice::Move,
                block_defender_id: None },
            &mut game,
            &mut rng,
        );
        assert_eq!(out.action, StepAction::GotoLabel);
    }

    #[test]
    fn with_thrower_and_no_flags_returns_next() {
        let mut step = StepInitPassing::new();
        step.goto_label_on_end = "end".into();
        let mut game = make_game();
        game.thrower_id = Some("p1".into());
        game.thrower_action = Some(PlayerAction::Pass);
        let mut rng = GameRng::new(0);
        let out = step.start(&mut game, &mut rng);
        assert_eq!(out.action, StepAction::NextStep);
    }
}
