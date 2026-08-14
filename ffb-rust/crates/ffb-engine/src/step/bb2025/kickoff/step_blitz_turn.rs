use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use crate::action::Action;
use crate::mechanic::mixed::setup_mechanic::SetupMechanic;
use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::step::framework::{Step, StepOutcome, SequenceStep};
use crate::step::framework::{StepId, StepParameter};
use crate::step::generator::bb2025::Select;
use crate::step::generator::bb2025::select::SelectParams;

/// Executes the Charge/Blitz! kickoff result turn.
///
/// Java logic:
///  1. If TurnMode is already BLITZ (second entry, after the blitz turn completes):
///     set TurnMode back to KICKOFF → NEXT_STEP.
///  2. Otherwise (first entry):
///     - Find the blitzing team (home_playing ? home : away).
///     - Call SetupMechanic.pinPlayersInTacklezones to deactivate pinned players.
///     - Set TurnMode = BLITZ.
///     - Start the blitz turn timer (TODO — timer infrastructure not ported).
///     - game.startTurn() — now implemented.
///     - Push `this` step back onto the stack (so we re-enter on the second call).
///     - Push a Select sequence for the blitzing team.
///     - NEXT_STEP.
///
/// TurnMode transitions, `pinPlayersInTacklezones`, `startTurn()`, and Select push all implemented.
///
/// Mirrors Java `com.fumbbl.ffb.server.step.bb2025.kickoff.StepBlitzTurn`.
pub struct StepBlitzTurn;

impl StepBlitzTurn {
    pub fn new() -> Self { Self }
}

impl Default for StepBlitzTurn {
    fn default() -> Self { Self::new() }
}

impl Step for StepBlitzTurn {
    fn id(&self) -> StepId { StepId::BlitzTurn }

    fn start(&mut self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn handle_command(&mut self, _action: &Action, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        self.execute_step(game, rng)
    }

    fn set_parameter(&mut self, _param: &StepParameter) -> bool { false }
}

impl StepBlitzTurn {
    fn execute_step(&self, game: &mut Game, rng: &mut GameRng) -> StepOutcome {
        // Java routes by @RulesCollection: `bb2020/StepBlitzTurn` is a materially different class
        // from `bb2025/kickoff/StepBlitzTurn` — it rolls a d3 for the activation LIMIT
        // (`limit = roll + 3`), records a `BlitzTurnState(limit, availablePlayers)`, and reports
        // ACTIVATIONS_EXHAUSTED when the blitzing team has no active players. BB2025's has none of
        // that. Only BB2020's kickoff table can produce a BLITZ result, so this step is reachable
        // solely through the BB2020 path — and taking the BB2025 body there cost the d3
        // (bb2020 human seed 23: Java rng 10 = `StepBlitzTurn.executeStep:87` d3=3; Rust went
        // straight to the d8 ball bounce).
        if game.rules == ffb_model::enums::Rules::Bb2020 {
            let mut bb2020 = crate::step::bb2020::step_blitz_turn::StepBlitzTurn::new();
            return bb2020.start(game, rng);
        }
        if game.turn_mode == TurnMode::Blitz {
            // Second entry: blitz turn is over, return to kickoff.
            game.turn_mode = TurnMode::Kickoff;
        } else {
            // First entry: set up the blitz turn for the kicking team.
            let blitzing_team_id = if game.home_playing {
                game.team_home.id.clone()
            } else {
                game.team_away.id.clone()
            };
            SetupMechanic::new().pin_players_in_tacklezones_chain(game, &blitzing_team_id, true);
            // no-op: headless engine has no turn timer; stopTurnTimer/startTurnTimer are server-only
            game.start_turn();
            game.turn_mode = TurnMode::Blitz;
            // Java: pushCurrentStepOnStack(); Select.pushSequence(gameState, true)
            let self_seq = vec![SequenceStep::new(StepId::BlitzTurn)];
            let select_seq = Select::build_sequence(&SelectParams {
                update_persistence: true,
                is_blitz_move: false,
                ..Default::default()
            });
            return StepOutcome::next().push_seq(self_seq).push_seq(select_seq);
        }
        StepOutcome::next()
    }
}

#[cfg(test)]
mod tests {
    /// Java routes StepBlitzTurn by @RulesCollection and BB2020's class is materially different:
    /// it rolls a d3 for the activation limit (`limit = roll + 3`) and records a BlitzTurnState.
    /// Only BB2020's kickoff table produces a BLITZ result, so running the BB2025 body there
    /// silently dropped that d3 and desynced the shared dice stream from the very first kickoff
    /// (bb2020 human seed 23: Java rng 10 = `StepBlitzTurn.executeStep:87` d3).
    #[test]
    fn bb2020_blitz_turn_rolls_the_activation_limit_d3() {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerState, PS_STANDING};
        use ffb_model::types::FieldCoordinate;
        let mut game = Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2020);
        game.home_playing = true;
        game.team_home.players.push(Player {
            id: "h1".into(), name: "h1".into(), nr: 1, ..Default::default() });
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        // Java counts `getPlayerState(player).isActive()`; without the active bit the team has no
        // available players and Java's `availablePlayers == 0` arm skips the roll entirely.
        game.field_model.set_player_state("h1", PlayerState::new(PS_STANDING).change_active(true));

        let mut step = StepBlitzTurn::new();
        let mut rng = GameRng::new(11);
        step.start(&mut game, &mut rng);

        assert_eq!(rng.call_count, 1, "BB2020 rolls exactly one d3 for the activation limit");
        assert_eq!(game.turn_mode, TurnMode::Blitz);
        let st = game.blitz_turn_state.as_ref().expect("BB2020 records a BlitzTurnState");
        assert!(st.limit >= 4 && st.limit <= 6, "limit = d3 + 3, got {}", st.limit);
    }

    /// The BB2025 body must stay dice-free — it has no activation limit at all.
    #[test]
    fn bb2025_blitz_turn_rolls_nothing() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let mut rng = GameRng::new(11);
        step.start(&mut game, &mut rng);
        assert_eq!(rng.call_count, 0);
        assert!(game.blitz_turn_state.is_none());
    }

    use super::*;
    use crate::step::framework::test_team;
    use crate::step::framework::StepAction;
    use ffb_model::enums::Rules;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        let home = test_team("home", 0);
        let away = test_team("away", 0);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn first_entry_sets_blitz_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepBlitzTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Blitz);
    }

    #[test]
    fn second_entry_restores_kickoff_mode() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Blitz;
        let mut step = StepBlitzTurn::new();
        step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(game.turn_mode, TurnMode::Kickoff);
    }

    #[test]
    fn start_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn handle_command_returns_next_step() {
        let mut game = make_game();
        let mut step = StepBlitzTurn::new();
        let out = step.handle_command(&Action::Acknowledge, &mut game, &mut GameRng::new(0));
        assert_eq!(out.action, StepAction::NextStep);
    }

    #[test]
    fn set_parameter_returns_false() {
        let mut step = StepBlitzTurn::new();
        assert!(!step.set_parameter(&StepParameter::EndTurn(false)));
    }

    #[test]
    fn first_entry_pushes_self_seq_and_select_seq() {
        let mut game = make_game();
        game.turn_mode = TurnMode::Kickoff;
        let mut step = StepBlitzTurn::new();
        let out = step.start(&mut game, &mut GameRng::new(0));
        // Java: pushCurrentStepOnStack() + Select.pushSequence → two pushed sequences
        assert_eq!(out.pushes.len(), 2, "must push self_seq + select_seq");
        assert_eq!(out.pushes[0][0].step_id, StepId::BlitzTurn);
    }
}
