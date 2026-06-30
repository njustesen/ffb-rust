use std::collections::HashSet;
use ffb_model::enums::{TurnMode, Weather};
use ffb_model::model::{Game, Player, PlayerStats, Roster, RosterPosition, Team, TeamResult};
use crate::mechanic::{Mechanic, MechanicType};

/// 1:1 translation of com.fumbbl.ffb.mechanics.GameMechanic.
pub trait GameMechanic: Mechanic {
    fn get_type(&self) -> MechanicType { MechanicType::GAME }

    fn concession_dialog_messages(&self, legal_concession: bool) -> Vec<String>;
    fn is_foul_action_allowed(&self, turn_mode: TurnMode) -> bool;
    fn is_bomb_action_allowed(&self, turn_mode: TurnMode) -> bool;
    fn is_gaze_action_allowed(&self, game: &Game, player: &Player) -> bool;
    fn declare_gaze_action_at_start(&self) -> bool;
    fn is_kick_team_mate_action_allowed(&self, turn_mode: TurnMode) -> bool;
    fn is_block_action_allowed(&self, turn_mode: TurnMode) -> bool;
    fn zapped_player_stats(&self) -> Box<dyn PlayerStats>;
    fn touchdown_ends_game(&self, game: &Game) -> bool;
    fn riotous_rookies_position(&self, roster: &Roster) -> Option<RosterPosition>;
    fn is_legal_concession(&self, game: &Game, team: &Team) -> bool;
    fn fan_modification_name(&self) -> String;
    fn fan_modification(&self, team_result: &TeamResult) -> i32;
    fn fans(&self, team: &Team) -> i32;
    fn audience_name(&self) -> String;
    fn audience(&self, team_result: &TeamResult) -> i32;
    fn weather_description(&self, weather: Weather) -> String;
    fn enhancements_to_remove_at_end_of_turn(&self) -> HashSet<String>;
    fn enhancements_to_remove_at_end_of_turn_when_not_setting_active(&self) -> HashSet<String>;
    fn roll_for_chef_at_start_of_half(&self) -> bool;
    fn allow_movement_in_end_zone(&self) -> bool;
    fn players_for_go_activations(&self, game: &Game) -> bool;
    fn is_wisdom_available(&self, game: &Game, player: &Player) -> bool;
}
