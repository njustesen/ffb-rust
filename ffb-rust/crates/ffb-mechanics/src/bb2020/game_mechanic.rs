use std::collections::HashSet;
use ffb_model::enums::{TurnMode, Weather};
use ffb_model::model::{Game, Player, PlayerStats, Roster, RosterPosition, Team, TeamResult};
use crate::mechanic::{Mechanic, MechanicType};
use crate::game_mechanic::GameMechanic as GameMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.GameMechanic.
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
    fn agility(&self) -> i32 { 2 }
    fn passing(&self) -> i32 { 0 }
    fn armour(&self) -> i32 { 5 }
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
                "You will lose D3 dedicated fans (to a minimum of 1).".into(),
                "You will lose your player award and all your winnings.".into(),
                "Some valuable players (3 or more advancements) may decide to leave your team.".into(),
            ]
        }
    }

    fn is_foul_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn is_bomb_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn is_gaze_action_allowed(&self, game: &Game, _player: &Player) -> bool {
        TurnMode::Blitz != game.turn_mode
    }

    fn declare_gaze_action_at_start(&self) -> bool { true }

    fn is_kick_team_mate_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn is_block_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn zapped_player_stats(&self) -> Box<dyn PlayerStats> { Box::new(ZappedStats) }

    fn touchdown_ends_game(&self, game: &Game) -> bool {
        game.half == 3 && game.options.is_enabled("overtimeGoldenGoal")
    }

    fn riotous_rookies_position(&self, roster: &Roster) -> Option<RosterPosition> {
        use ffb_model::enums::PlayerType;
        // Java shuffles; Rust returns the first matching position (no RNG here).
        let mut candidates: Vec<&RosterPosition> = roster.positions.iter()
            .filter(|pos| pos.quantity == 12 || pos.quantity == 16)
            .filter(|pos| pos.player_type != PlayerType::Irregular)
            .collect();
        if candidates.is_empty() {
            None
        } else {
            Some(candidates.remove(0).clone())
        }
    }

    fn is_legal_concession(&self, game: &Game, team: &Team) -> bool {
        game.turn_mode == TurnMode::Setup
            && team.players.iter()
                .filter(|p| game.field_model.player_state(&p.id)
                    .map(|s| s.can_be_set_up_next_drive())
                    .unwrap_or(false))
                .count() <= 3
    }

    fn fan_modification_name(&self) -> String { "Dedicated Fans".into() }

    fn fan_modification(&self, team_result: &TeamResult) -> i32 {
        team_result.dedicated_fans_modifier
    }

    fn fans(&self, team: &Team) -> i32 { team.dedicated_fans }

    fn audience_name(&self) -> String { "Fan Factor".into() }

    fn audience(&self, team_result: &TeamResult) -> i32 { team_result.fan_factor }

    fn weather_description(&self, weather: Weather) -> String {
        match weather {
            Weather::SwelteringHeat => "D3 random players from each team on the pitch will suffer from heat exhaustion before the next kick-off.".into(),
            Weather::VerySunny => "A -1 modifier applies to all passing rolls.".into(),
            Weather::Nice => "Perfect Fantasy Football weather.".into(),
            Weather::PouringRain => "A -1 modifier applies to all catch, intercept, or pick-up rolls.".into(),
            Weather::Blizzard => "Rushes fail on a roll of 1 or 2 and only quick or short passes can be attempted.".into(),
            _ => "No weather at all, but the intro screen shown by the client.".into(),
        }
    }

    fn enhancements_to_remove_at_end_of_turn(&self) -> HashSet<String> {
        // TODO: return WisdomOfTheWhiteDwarf enhancement name via SkillFactory
        HashSet::new()
    }

    fn enhancements_to_remove_at_end_of_turn_when_not_setting_active(&self) -> HashSet<String> {
        // TODO: Constant.getEnhancementSkillsToRemoveAtEndOfTurnWhenNotSettingActive
        HashSet::new()
    }

    fn roll_for_chef_at_start_of_half(&self) -> bool { false }

    fn allow_movement_in_end_zone(&self) -> bool { false }

    fn players_for_go_activations(&self, _game: &Game) -> bool { false }

    fn is_wisdom_available(&self, _game: &Game, _player: &Player) -> bool {
        // TODO: UtilCards + NamedProperties + UtilPlayer
        false
    }
}

impl GameMechanic {
    pub fn new() -> Self { GameMechanic }
}
