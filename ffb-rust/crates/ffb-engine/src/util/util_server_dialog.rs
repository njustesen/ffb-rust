// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerDialog.
//
// Both Java methods (showDialog / hideDialog) call UtilServerTimer which
// reaches into the live server's WebSocket timer layer.  The pure-model
// side-effects — setting `game.waiting_for_opponent` — are translated here.
// The timer calls are skipped because the Rust engine has no WebSocket layer.
// The `dialog_parameter` field has not yet been added to the Rust Game struct;
// the corresponding logic is left as a TODO comment.
//
// Translated:
//   show_dialog(game, stop_turn_timer: bool)
//   hide_dialog(game)
//
// Skipped (touch WebSocket timer):
//   UtilServerTimer.stopTurnTimer, UtilServerTimer.startTurnTimer

use ffb_model::model::game::Game;

pub struct UtilServerDialog;

impl UtilServerDialog {
    /// Java: UtilServerDialog.showDialog(GameState, IDialogParameter, boolean stopTurnTimer)
    ///
    /// Sets `game.waiting_for_opponent` when `stop_turn_timer` is true.
    /// (The `dialog_parameter` field is not yet modelled in the Rust Game struct.)
    pub fn show_dialog(game: &mut Game, stop_turn_timer: bool) {
        // game.dialog_parameter = Some(dialog_parameter);  // TODO: add field
        if stop_turn_timer {
            game.waiting_for_opponent = true;
            // UtilServerTimer::stop_turn_timer — skipped (WebSocket layer)
        }
    }

    /// Java: UtilServerDialog.hideDialog(GameState)
    ///
    /// Clears `dialog_parameter` and resets `waiting_for_opponent`.
    pub fn hide_dialog(game: &mut Game) {
        // game.dialog_parameter = None;  // TODO: add field
        game.waiting_for_opponent = false;
        // UtilServerTimer::start_turn_timer — skipped (WebSocket layer)
    }
}

impl Default for UtilServerDialog {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
        }
    }

    fn empty_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn show_dialog_with_stop_timer_sets_waiting_for_opponent() {
        let mut game = empty_game();
        assert!(!game.waiting_for_opponent);
        UtilServerDialog::show_dialog(&mut game, true);
        assert!(game.waiting_for_opponent);
    }

    #[test]
    fn show_dialog_without_stop_timer_leaves_waiting_for_opponent_unchanged() {
        let mut game = empty_game();
        game.waiting_for_opponent = false;
        UtilServerDialog::show_dialog(&mut game, false);
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn hide_dialog_clears_waiting_for_opponent() {
        let mut game = empty_game();
        game.waiting_for_opponent = true;
        UtilServerDialog::hide_dialog(&mut game);
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn hide_dialog_idempotent_when_already_false() {
        let mut game = empty_game();
        game.waiting_for_opponent = false;
        UtilServerDialog::hide_dialog(&mut game);
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn show_then_hide_round_trips_waiting_for_opponent() {
        let mut game = empty_game();
        UtilServerDialog::show_dialog(&mut game, true);
        assert!(game.waiting_for_opponent);
        UtilServerDialog::hide_dialog(&mut game);
        assert!(!game.waiting_for_opponent);
    }
}
