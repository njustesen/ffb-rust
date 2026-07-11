use ffb_model::enums::TurnMode;

/// 1:1 translation of com.fumbbl.ffb.server.admin.GameStateService.
pub struct GameStateService;

impl GameStateService {
    pub fn new() -> Self {
        Self
    }

    /// Resets a game to a fresh "select" state: clears the acting player, resets `TurnMode` to
    /// `REGULAR`, clears timeout/blitz state, and clears any active target-selection.
    ///
    /// Java additionally clears the engine's step stack and pushes a fresh
    /// `Select.SequenceParams` sequence (`gameState.getStepStack().clear()` +
    /// `generator.pushSequence(...)` + `gameState.startNextStep()`). `DriverGameState`'s step
    /// stack is private to `ffb-engine` with no public "clear + push Select" entry point
    /// exposed yet, so that half of the reset is not performed here — this ports the `Game`
    /// field resets only.
    pub fn reset_step_stack(&self, game_state: &mut crate::game_state::GameState) -> Result<(), String> {
        let game = match game_state.get_game_mut() {
            Some(game) => game,
            None => return Err("game not started".to_string()),
        };

        game.acting_player.clear();

        game.turn_mode = TurnMode::Regular;
        game.last_turn_mode = None;
        game.timeout_enforced = false;
        game.blitz_turn_state = None;

        if let Some(target_selection_state) = game.field_model.target_selection_state.clone() {
            if let Some(player_id) = target_selection_state.get_selected_player_id() {
                if let Some(player_state) = game.field_model.player_state(player_id) {
                    game.field_model
                        .set_player_state(player_id, player_state.remove_all_target_selections());
                }
            }
            game.field_model.target_selection_state = None;
        }

        Ok(())
    }
}

impl Default for GameStateService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct() {
        let _ = GameStateService::new();
    }

    #[test]
    fn reset_step_stack_errors_when_game_not_started() {
        let mut game_state = crate::game_state::GameState::new(1);
        let service = GameStateService::new();
        assert!(service.reset_step_stack(&mut game_state).is_err());
    }
}
