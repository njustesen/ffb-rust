use std::collections::HashSet;
use ffb_model::enums::{TurnMode, Weather};
use ffb_model::model::{Game, Player, PlayerStats, Roster, RosterPosition, Team, TeamResult};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::game_mechanic::GameMechanic as GameMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.GameMechanic.
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
                "You will loose all SPP earned during this game".into(),
            ]
        }
    }

    fn is_foul_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn is_bomb_action_allowed(&self, turn_mode: TurnMode) -> bool {
        TurnMode::Blitz != turn_mode
    }

    fn is_gaze_action_allowed(&self, game: &Game, player: &Player) -> bool {
        let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
        let can_declare = !player_state.is_prone()
            || game.options.is_enabled("allowSpecialActionsFromProne");
        can_declare && TurnMode::Blitz != game.turn_mode
    }

    fn declare_gaze_action_at_start(&self) -> bool { true }

    fn is_kick_team_mate_action_allowed(&self, _turn_mode: TurnMode) -> bool { true }

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
        // TODO: shuffle with an RNG source for full parity.
        let mut candidates: Vec<&RosterPosition> = roster.positions.iter()
            .filter(|pos| (pos.quantity == 12 || pos.quantity == 16))
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
                .count() < 3
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

    fn players_for_go_activations(&self, game: &Game) -> bool {
        game.options.is_enabled("enableStallingCheck")
    }

    fn is_wisdom_available(&self, game: &Game, player: &Player) -> bool {
        // Java: check hasUnusedSkillWithProperty(canGrantSkillsToTeamMates) first.
        if !UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_GRANT_SKILLS_TO_TEAM_MATES) {
            return false;
        }
        // Java: find team-mates within 2 squares (not stunned, active) that lack a grantable skill.
        let coord = match game.field_model.player_coordinate(&player.id) {
            Some(c) => c,
            None => return false,
        };
        let is_home = game.team_home.has_player(&player.id);
        let team = if is_home { &game.team_home } else { &game.team_away };
        let team_mates = UtilPlayer::find_standing_or_prone_players(game, team, coord, 2);
        // TODO: Constant.getGrantAbleSkills via SkillFactory — check that teammate lacks a grantable skill
        // For now: return true if any active team-mate is present within 2 squares.
        team_mates.iter().any(|tm| {
            game.field_model.player_state(&tm.id)
                .map(|s| s.is_active())
                .unwrap_or(false)
        })
    }
}

impl GameMechanic {
    pub fn new() -> Self { GameMechanic }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Weather;
    use ffb_model::model::Team;
    use crate::game_mechanic::GameMechanic as GameTrait;

    fn bare_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 4, dedicated_fans: 9, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
        }
    }

    #[test]
    fn is_foul_not_allowed_during_blitz() {
        assert!(!GameMechanic.is_foul_action_allowed(TurnMode::Blitz));
    }

    #[test]
    fn is_foul_allowed_during_regular() {
        assert!(GameMechanic.is_foul_action_allowed(TurnMode::Regular));
    }

    #[test]
    fn is_ktm_allowed_during_blitz() {
        // bb2025: KTM is allowed even during blitz (unlike bb2020)
        assert!(GameMechanic.is_kick_team_mate_action_allowed(TurnMode::Blitz));
    }

    #[test]
    fn fan_modification_name_is_dedicated_fans() {
        assert_eq!(GameMechanic.fan_modification_name(), "Dedicated Fans");
    }

    #[test]
    fn fans_returns_dedicated_fans() {
        let team = bare_team("home");
        assert_eq!(GameMechanic.fans(&team), 9);
    }

    #[test]
    fn roll_for_chef_is_false() {
        assert!(!GameMechanic.roll_for_chef_at_start_of_half());
    }

    #[test]
    fn concession_dialog_illegal_has_five_messages_in_bb2025() {
        // bb2025 adds "You will loose all SPP earned during this game"
        let msgs = GameMechanic.concession_dialog_messages(false);
        assert_eq!(msgs.len(), 5);
    }
}
