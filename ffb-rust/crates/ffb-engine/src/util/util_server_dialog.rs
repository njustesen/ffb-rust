// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerDialog.
//
// Translated:
//   show_dialog(game, dialog_id, stop_turn_timer: bool)
//   hide_dialog(game)
//
// Skipped (touch WebSocket timer):
//   UtilServerTimer.stopTurnTimer, UtilServerTimer.startTurnTimer

use ffb_model::dialog::dialog_id::DialogId;
use ffb_model::model::game::Game;

pub struct UtilServerDialog;

impl UtilServerDialog {
    /// Java: UtilServerDialog.showDialog(GameState, IDialogParameter, boolean stopTurnTimer)
    pub fn show_dialog(game: &mut Game, dialog_id: DialogId, stop_turn_timer: bool) {
        game.dialog_id = Some(dialog_id);
        if stop_turn_timer {
            game.waiting_for_opponent = true;
            // UtilServerTimer::stop_turn_timer — skipped (WebSocket layer)
        }
    }

    /// Java: UtilServerDialog.hideDialog(GameState)
    pub fn hide_dialog(game: &mut Game) {
        game.dialog_id = None;
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
            vampire_lord: false,
        }
    }

    fn empty_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn show_dialog_sets_dialog_id() {
        let mut game = empty_game();
        assert!(game.dialog_id.is_none());
        UtilServerDialog::show_dialog(&mut game, DialogId::RE_ROLL, false);
        assert_eq!(game.dialog_id, Some(DialogId::RE_ROLL));
    }

    #[test]
    fn show_dialog_with_stop_timer_sets_waiting_for_opponent() {
        let mut game = empty_game();
        UtilServerDialog::show_dialog(&mut game, DialogId::SKILL_USE, true);
        assert!(game.waiting_for_opponent);
        assert_eq!(game.dialog_id, Some(DialogId::SKILL_USE));
    }

    #[test]
    fn show_dialog_without_stop_timer_leaves_waiting_for_opponent_unchanged() {
        let mut game = empty_game();
        game.waiting_for_opponent = false;
        UtilServerDialog::show_dialog(&mut game, DialogId::SKILL_USE, false);
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn hide_dialog_clears_dialog_id_and_waiting() {
        let mut game = empty_game();
        UtilServerDialog::show_dialog(&mut game, DialogId::RE_ROLL, true);
        UtilServerDialog::hide_dialog(&mut game);
        assert!(game.dialog_id.is_none());
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn hide_dialog_idempotent_when_already_hidden() {
        let mut game = empty_game();
        UtilServerDialog::hide_dialog(&mut game);
        assert!(game.dialog_id.is_none());
        assert!(!game.waiting_for_opponent);
    }

    #[test]
    fn show_then_hide_round_trips() {
        let mut game = empty_game();
        UtilServerDialog::show_dialog(&mut game, DialogId::USE_APOTHECARY, true);
        assert!(game.waiting_for_opponent);
        UtilServerDialog::hide_dialog(&mut game);
        assert!(!game.waiting_for_opponent);
        assert!(game.dialog_id.is_none());
    }
}
