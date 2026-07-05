/// 1:1 translation of com.fumbbl.ffb.server.model.change.ChompRemovalObserver.
///
/// Java: @RulesCollection(BB2025). Implements ConditionalModelChangeObserver.
/// Handles FIELD_MODEL_SET_PLAYER_COORDINATE and FIELD_MODEL_SET_PLAYER_STATE model changes
/// to remove or update Chomp markers on the field model.
///
/// headless: ReportChompRemoved not yet ported — chomp removal updates field state
/// but does not emit reports until the report system is in place.
use ffb_model::enums::ModelChangeId;
use ffb_model::model::game::Game;
use crate::model::change::conditional_model_change_observer::ConditionalModelChangeObserver;

pub struct ChompRemovalObserver;

impl ChompRemovalObserver {
    pub fn new() -> Self { Self }
}

impl Default for ChompRemovalObserver {
    fn default() -> Self { Self::new() }
}

impl ConditionalModelChangeObserver for ChompRemovalObserver {
    fn get_name(&self) -> &str { "ChompRemovalObserver" }

    /// Java: void next(GameState gameState, ModelChange modelChange).
    ///
    /// FIELD_MODEL_SET_PLAYER_COORDINATE: if player moved to a box coordinate → removeChomps;
    /// otherwise → updateChomps (evict chompees no longer adjacent).
    ///
    /// FIELD_MODEL_SET_PLAYER_STATE: if player no longer has tackle zones → removeChomps
    /// (prone/stunned/KO players can't maintain a chomp).
    ///
    /// Reports ReportChompRemoved for each evicted chompee — headless: report system not yet ported.
    fn next(&self, key: Option<&str>, change_id: ModelChangeId, game: &mut Game) {
        let player_id = match key {
            Some(id) => id,
            None => return,
        };

        let removed: Vec<(String, bool)> = match change_id {
            ModelChangeId::FieldModelSetPlayerCoordinate => {
                let coord = match game.field_model.player_coordinate(player_id) {
                    Some(c) => c,
                    None => return,
                };
                if coord.is_box_coordinate() {
                    game.field_model.remove_chomps(player_id)
                } else {
                    game.field_model.update_chomps(player_id)
                }
            }
            ModelChangeId::FieldModelSetPlayerState => {
                let state = match game.field_model.player_state(player_id) {
                    Some(s) => s,
                    None => return,
                };
                if !state.has_tacklezones() {
                    game.field_model.remove_chomps(player_id)
                } else {
                    return;
                }
            }
            _ => return,
        };

        // headless: emit ReportChompRemoved for each chompee_id in removed — report system not yet ported
        let _ = removed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING, PS_PRONE, PlayerState};
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn get_name_returns_class_name() {
        assert_eq!(ChompRemovalObserver::new().get_name(), "ChompRemovalObserver");
    }

    #[test]
    fn next_with_no_key_does_not_panic() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        obs.next(None, ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
    }

    #[test]
    fn next_unrelated_change_does_not_panic() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        obs.next(Some("p-1"), ModelChangeId::GameSetActingTeam, &mut game);
    }

    #[test]
    fn coordinate_change_to_box_removes_all_chomps() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        // Set state first so add_chomp can apply the chomped bit
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        // chomper p1 had chomped p2
        game.field_model.add_chomp("p1", "p2");
        // move p1 to a box coordinate (RSV_HOME_X = -1, a reserve dugout column)
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(-1, 3));
        assert!(game.field_model.player_coordinate("p1").unwrap().is_box_coordinate());

        obs.next(Some("p1"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);

        // p2 should no longer be chomped
        let state = game.field_model.player_state("p2").unwrap();
        assert!(!state.is_chomped(), "chompee should no longer be chomped after chomper enters box");
    }

    #[test]
    fn coordinate_change_to_pitch_calls_update_chomps() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        // Set states first so add_chomp can apply the chomped bit
        game.field_model.set_player_state("p2", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("p3", PlayerState::new(PS_STANDING));
        // p1 chomped p2 who is adjacent, and p3 who is far away
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(5, 5));
        game.field_model.set_player_coordinate("p2", FieldCoordinate::new(6, 5)); // adjacent
        game.field_model.set_player_coordinate("p3", FieldCoordinate::new(15, 5)); // far
        game.field_model.add_chomp("p1", "p2");
        game.field_model.add_chomp("p1", "p3");

        obs.next(Some("p1"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);

        // p3 (non-adjacent) should lose chomp; p2 (adjacent) should keep it
        let p3_state = game.field_model.player_state("p3").unwrap();
        assert!(!p3_state.is_chomped(), "non-adjacent chompee should lose chomp on update");
        let p2_state = game.field_model.player_state("p2").unwrap();
        assert!(p2_state.is_chomped(), "adjacent chompee should keep chomp on update");
    }

    #[test]
    fn state_change_prone_removes_chomps() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        // Set state first so add_chomp applies the chomped bit
        game.field_model.set_player_state("victim", PlayerState::new(PS_STANDING));
        game.field_model.add_chomp("chomper", "victim");
        // Now set chomper to prone (no tackle zones) — triggers the observer condition
        game.field_model.set_player_state("chomper", PlayerState::new(PS_PRONE));

        obs.next(Some("chomper"), ModelChangeId::FieldModelSetPlayerState, &mut game);

        let state = game.field_model.player_state("victim").unwrap();
        assert!(!state.is_chomped(), "chompee should lose chomp when chomper goes prone");
    }

    #[test]
    fn state_change_standing_keeps_chomps() {
        let obs = ChompRemovalObserver::new();
        let mut game = make_game();
        // Set states first so add_chomp can apply the chomped bit
        game.field_model.set_player_state("victim", PlayerState::new(PS_STANDING));
        game.field_model.set_player_state("chomper", PlayerState::new(PS_STANDING));
        game.field_model.add_chomp("chomper", "victim");

        obs.next(Some("chomper"), ModelChangeId::FieldModelSetPlayerState, &mut game);

        // still has tackle zones → chomps remain
        let state = game.field_model.player_state("victim").unwrap();
        assert!(state.is_chomped(), "chompee should keep chomp when chomper stands");
    }

    #[test]
    fn default_creates_instance() {
        let _obs = ChompRemovalObserver::default();
    }
}
