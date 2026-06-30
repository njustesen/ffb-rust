use std::collections::HashSet;
use ffb_model::enums::{PlayerAction, TurnMode, Weather};
use ffb_model::model::{Game, Player, PlayerStats, Roster, RosterPosition, Team, TeamResult};
use crate::mechanic::{Mechanic, MechanicType};
use crate::game_mechanic::GameMechanic as GameMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2016.GameMechanic.
pub struct GameMechanic;

impl Default for GameMechanic {
    fn default() -> Self { GameMechanic }
}

impl Mechanic for GameMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::GAME }
}

struct ZappedStats;
impl PlayerStats for ZappedStats {
    fn move_stat(&self) -> i32 { 5 }
    fn strength(&self) -> i32 { 1 }
    fn agility(&self) -> i32 { 4 }
    fn passing(&self) -> i32 { 0 }
    fn armour(&self) -> i32 { 4 }
}

impl GameMechanicTrait for GameMechanic {
    fn concession_dialog_messages(&self, legal_concession: bool) -> Vec<String> {
        if legal_concession {
            vec![
                "Do you want to concede this game?".into(),
                "The concession will have no negative consequences at this point.".into(),
            ]
        } else {
            vec![
                "Do you want to concede this game?".into(),
                "Your fan factor will decrease by 1.".into(),
                "You will lose your player award and all your winnings.".into(),
                "Some valuable players (SPP 51+) may decide to leave your team.".into(),
            ]
        }
    }

    fn is_foul_action_allowed(&self, _turn_mode: TurnMode) -> bool { true }

    fn is_bomb_action_allowed(&self, _turn_mode: TurnMode) -> bool { true }

    fn is_gaze_action_allowed(&self, game: &Game, player: &Player) -> bool {
        game.acting_player.player_action == Some(PlayerAction::Move)
    }

    fn declare_gaze_action_at_start(&self) -> bool { false }

    fn is_kick_team_mate_action_allowed(&self, _turn_mode: TurnMode) -> bool { true }

    fn is_block_action_allowed(&self, _turn_mode: TurnMode) -> bool { true }

    fn zapped_player_stats(&self) -> Box<dyn PlayerStats> { Box::new(ZappedStats) }

    fn touchdown_ends_game(&self, game: &Game) -> bool { game.half == 3 }

    fn riotous_rookies_position(&self, _roster: &Roster) -> Option<RosterPosition> {
        // TODO: roster.get_riotous_position() — needs riotous_position_id on Roster
        None
    }

    fn is_legal_concession(&self, _game: &Game, _team: &Team) -> bool {
        // TODO: UtilPlayer::find_players_in_reserve_or_field(game, team).len() <= 2
        false
    }

    fn fan_modification_name(&self) -> String { "Fan Factor".into() }

    fn fan_modification(&self, team_result: &TeamResult) -> i32 {
        team_result.fan_factor_modifier
    }

    fn fans(&self, team: &Team) -> i32 { team.fan_factor }

    fn audience_name(&self) -> String { "Fame".into() }

    fn audience(&self, team_result: &TeamResult) -> i32 { team_result.fame }

    fn weather_description(&self, weather: Weather) -> String {
        match weather {
            Weather::SwelteringHeat => "Each player on the pitch may suffer from heat exhaustion on a roll of 1 before the next kick-off.".into(),
            Weather::VerySunny => "A -1 modifier applies to all passing rolls.".into(),
            Weather::Nice => "Perfect Fantasy Football weather.".into(),
            Weather::PouringRain => "A -1 modifier applies to all catch, intercept, or pick-up rolls.".into(),
            Weather::Blizzard => "Going For It fails on a roll of 1 or 2 and only quick or short passes can be attempted.".into(),
            _ => "No weather at all, but the intro screen shown by the client.".into(),
        }
    }

    fn enhancements_to_remove_at_end_of_turn(&self) -> HashSet<String> { HashSet::new() }

    fn enhancements_to_remove_at_end_of_turn_when_not_setting_active(&self) -> HashSet<String> { HashSet::new() }

    fn roll_for_chef_at_start_of_half(&self) -> bool { true }

    fn allow_movement_in_end_zone(&self) -> bool { true }

    fn players_for_go_activations(&self, _game: &Game) -> bool { false }

    fn is_wisdom_available(&self, _game: &Game, _player: &Player) -> bool { false }
}

impl GameMechanic {
    pub fn new() -> Self { GameMechanic }
}
