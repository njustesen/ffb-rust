use std::collections::HashSet;
use ffb_model::model::{Game, Player};

/// 1:1 translation of com.fumbbl.ffb.modifiers.GoForItContext.
pub struct GoForItContext<'a> {
    pub game: &'a Game,
    pub player: &'a Player,
    pub teams_with_moles_under_pitch: HashSet<String>,
}

impl<'a> GoForItContext<'a> {
    pub fn new(game: &'a Game, player: &'a Player) -> Self {
        Self {
            game,
            player,
            teams_with_moles_under_pitch: HashSet::new(),
        }
    }

    pub fn new_with_moles(
        game: &'a Game,
        player: &'a Player,
        teams_with_moles_under_pitch: HashSet<String>,
    ) -> Self {
        Self { game, player, teams_with_moles_under_pitch }
    }

    pub fn get_game(&self) -> &Game { self.game }
    pub fn get_player(&self) -> &Player { self.player }
    pub fn get_teams_with_moles_under_pitch(&self) -> &HashSet<String> {
        &self.teams_with_moles_under_pitch
    }
}
