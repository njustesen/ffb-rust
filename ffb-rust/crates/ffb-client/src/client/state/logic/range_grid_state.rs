//! 1:1 translation of `com.fumbbl.ffb.client.state.logic.RangeGridState`.

use ffb_model::enums::PlayerAction;
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;

use crate::client::fantasy_football_client::FantasyFootballClient;
use crate::client::state::logic::interaction::interaction_result::InteractionResult;

/// 1:1 translation of the `RangeGridState` class.
pub struct RangeGridState {
    show_range_grid: bool,
    throw_team_mate: bool,
}

impl RangeGridState {
    /// java: `public RangeGridState(FantasyFootballClient client, boolean throwTeamMate)`
    ///
    /// The Java constructor stores `client` on the instance; here it is passed explicitly
    /// to the methods that need it instead (matches the `LogicModule` convention).
    pub fn new(throw_team_mate: bool) -> Self {
        Self {
            show_range_grid: false,
            throw_team_mate,
        }
    }

    /// java: `public boolean isShowRangeGrid()`
    pub fn is_show_range_grid(&self) -> bool {
        self.show_range_grid
    }

    /// java: `public void setShowRangeGrid(boolean showRangeGrid)`
    pub fn set_show_range_grid(&mut self, show_range_grid: bool) {
        self.show_range_grid = show_range_grid;
    }

    /// java: `public InteractionResult refreshRangeGrid()`
    pub fn refresh_range_grid(&self, game: &Game) -> InteractionResult {
        if self.show_range_grid {
            let acting_player = &game.acting_player;
            let action = acting_player.player_action;
            if !self.throw_team_mate
                || action == Some(PlayerAction::ThrowTeamMate)
                || action == Some(PlayerAction::ThrowTeamMateMove)
                || action == Some(PlayerAction::KickTeamMate)
                || action == Some(PlayerAction::KickTeamMateMove)
                || action == Some(PlayerAction::ThrowBomb)
            {
                let acting_player_coordinate: Option<FieldCoordinate> = acting_player
                    .player_id
                    .as_deref()
                    .and_then(|id| game.field_model.player_coordinate(id));
                if let Some(coordinate) = acting_player_coordinate {
                    return InteractionResult::perform().with_coordinate(coordinate);
                }
            }
        }
        InteractionResult::reset()
    }

    /// java: `public InteractionResult refreshSettings()`
    ///
    /// java: `client.getProperty(CommonProperty.SETTING_RANGEGRID)` — `getProperty()` is
    /// `abstract` on `FantasyFootballClient` with no in-scope body (see that module's own
    /// doc comment); documented gap, so this always takes the "no setting" branch.
    pub fn refresh_settings(&mut self, _client: &FantasyFootballClient, game: &Game) -> InteractionResult {
        // java: gap — see doc comment above; `range_grid_setting_property` is unavailable.
        InteractionResult::ignore()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn make_team(id: &str) -> Team {
        Team {
            id: id.to_string(),
            name: id.to_string(),
            race: "human".into(),
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
            special_rules: Vec::new(),
            players: Vec::new(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(make_team("home"), make_team("away"), Rules::Bb2025)
    }

    #[test]
    fn new_defaults_to_hidden() {
        let state = RangeGridState::new(false);
        assert!(!state.is_show_range_grid());
    }

    #[test]
    fn set_show_range_grid_toggles_flag() {
        let mut state = RangeGridState::new(false);
        state.set_show_range_grid(true);
        assert!(state.is_show_range_grid());
    }

    #[test]
    fn refresh_range_grid_resets_when_hidden() {
        let state = RangeGridState::new(false);
        let game = make_game();
        assert_eq!(state.refresh_range_grid(&game).get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Reset);
    }

    #[test]
    fn refresh_range_grid_resets_without_acting_player_coordinate() {
        let mut state = RangeGridState::new(false);
        state.set_show_range_grid(true);
        let game = make_game();
        // No acting player set up, so no coordinate is available -> falls through to reset.
        assert_eq!(state.refresh_range_grid(&game).get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Reset);
    }

    #[test]
    fn refresh_range_grid_performs_when_shown_and_not_gated_by_ttm() {
        let mut state = RangeGridState::new(false);
        state.set_show_range_grid(true);
        let mut game = make_game();
        let mut player = ffb_model::model::player::Player::default();
        player.id = "p1".to_string();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(2, 3));
        game.acting_player.player_id = Some("p1".to_string());
        let result = state.refresh_range_grid(&game);
        assert_eq!(result.get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Perform);
        assert_eq!(result.get_coordinate(), Some(FieldCoordinate::new(2, 3)));
    }

    #[test]
    fn refresh_range_grid_gated_by_ttm_requires_matching_action() {
        let mut state = RangeGridState::new(true);
        state.set_show_range_grid(true);
        let mut game = make_game();
        let mut player = ffb_model::model::player::Player::default();
        player.id = "p1".to_string();
        game.team_home.players.push(player);
        game.field_model.set_player_coordinate("p1", FieldCoordinate::new(2, 3));
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        // Move isn't a throw-team-mate-family action, so this should reset, not perform.
        assert_eq!(state.refresh_range_grid(&game).get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Reset);
        game.acting_player.player_action = Some(PlayerAction::ThrowTeamMate);
        assert_eq!(state.refresh_range_grid(&game).get_kind(), crate::client::state::logic::interaction::interaction_result::Kind::Perform);
    }
}
