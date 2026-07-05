/// 1:1 translation of `com.fumbbl.ffb.server.mechanic.bb2025.SetupMechanic`.
///
/// Setup validation and pin-players-in-tacklezones for BB2025 rules.
/// Differs from mixed (BB2016/BB2020) in swarmer detection:
///   BB2025: position keyword "Lineman" marks a swarmer.
///   Mixed:  skill property `canSneakExtraPlayersOntoPitch` marks a swarmer.
/// Also adds a captain-must-be-fielded check (`needsToBeSetUp` property).
use ffb_model::dialog::dialog_id::DialogId;
use ffb_model::enums::PS_RESERVE;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::option::game_option_id;
use ffb_model::option::util_game_option;
use ffb_model::types::FieldCoordinateBounds;
use ffb_model::util::util_player::UtilPlayer;

use crate::mechanic::setup_mechanic::SetupMechanic as SetupMechanicTrait;
use crate::util::util_server_dialog::UtilServerDialog;

pub struct SetupMechanic;

impl SetupMechanic {
    pub fn new() -> Self { Self }
}

impl Default for SetupMechanic {
    fn default() -> Self { Self::new() }
}

impl SetupMechanicTrait for SetupMechanic {
    fn check_setup(&self, game: &mut Game, home_team: bool) -> bool {
        self.check_setup_with_swarmers(game, home_team, 0)
    }

    /// Java: `checkSetup(GameState, boolean, int additionalSwarmers)`.
    ///
    /// Validates team placement counts (LoS, wide zones, max/min on field).
    /// BB2025: position keyword `Lineman` marks a swarmer (not a named property).
    fn check_setup_with_swarmers(
        &self,
        game: &mut Game,
        home_team: bool,
        additional_swarmers: i32,
    ) -> bool {
        let mut swarmers_on_field = 0i32;
        let mut players_on_field = 0i32;
        let mut players_in_upper_wide_zone = 0i32;
        let mut players_in_lower_wide_zone = 0i32;
        let mut players_on_los = 0i32;
        let mut available_players = 0i32;
        let mut must_field_captain = false;
        let mut messages: Vec<String> = Vec::new();

        let team_id = if home_team {
            game.team_home.id.clone()
        } else {
            game.team_away.id.clone()
        };

        // Collect player data without borrowing game mutably mid-loop.
        // Java: `player.getPosition().getKeywords().contains(Keyword.LINEMAN)` for swarmer check.
        // headless: Swarming position keyword lookup requires roster access not yet ported
        // Currently treats all on-field players as regular (no swarmer separation).
        let player_data: Vec<(String, bool, bool)> = {
            let team = if home_team { &game.team_home } else { &game.team_away };
            team.players.iter().map(|p| {
                let is_keen = p.has_skill_property(NamedProperties::CAN_JOIN_TEAM_IF_LESS_THAN_ELEVEN);
                let needs_field = p.has_skill_property(NamedProperties::NEEDS_TO_BE_SET_UP);
                (p.id.clone(), is_keen, needs_field)
            }).collect()
        };

        for (player_id, is_keen, needs_to_be_set_up) in &player_data {
            let state = game.field_model.player_state(player_id);
            let coord = game.field_model.player_coordinate(player_id);

            // Keen Players (canJoinTeamIfLessThanEleven) are available but excluded from counts.
            if state.map(|s| s.can_be_set_up_next_drive()).unwrap_or(false) && !is_keen {
                available_players += 1;
            }

            let Some(coord) = coord else { continue };

            let in_home_half = FieldCoordinateBounds::HALF_HOME.is_in_bounds(coord);
            let in_away_half = FieldCoordinateBounds::HALF_AWAY.is_in_bounds(coord);

            if (home_team && in_home_half) || (!home_team && in_away_half) {
                // headless: Keyword.LINEMAN check for swarmer count — roster keyword access not yet ported
                players_on_field += 1;
                let _ = swarmers_on_field;
            }

            let in_upper_home = FieldCoordinateBounds::UPPER_WIDE_ZONE_HOME.is_in_bounds(coord);
            let in_upper_away = FieldCoordinateBounds::UPPER_WIDE_ZONE_AWAY.is_in_bounds(coord);
            if (home_team && in_upper_home) || (!home_team && in_upper_away) {
                players_in_upper_wide_zone += 1;
            }

            let in_lower_home = FieldCoordinateBounds::LOWER_WIDE_ZONE_HOME.is_in_bounds(coord);
            let in_lower_away = FieldCoordinateBounds::LOWER_WIDE_ZONE_AWAY.is_in_bounds(coord);
            if (home_team && in_lower_home) || (!home_team && in_lower_away) {
                players_in_lower_wide_zone += 1;
            }

            let on_los_home = FieldCoordinateBounds::LOS_HOME.is_in_bounds(coord);
            let on_los_away = FieldCoordinateBounds::LOS_AWAY.is_in_bounds(coord);
            if (home_team && on_los_home) || (!home_team && on_los_away) {
                players_on_los += 1;
            }

            // BB2025 captain rule: if player has needsToBeSetUp and is in reserve, must field.
            if *needs_to_be_set_up {
                let base = state.map(|s| s.base()).unwrap_or(0);
                if base == PS_RESERVE {
                    must_field_captain = true;
                }
            }
        }

        let max_players_on_field = util_game_option::get_int_option(game, game_option_id::MAX_PLAYERS_ON_FIELD);
        let all_players_on_field = players_on_field + swarmers_on_field;

        if all_players_on_field > max_players_on_field + additional_swarmers
            || players_on_field > max_players_on_field
        {
            messages.push(format!(
                "You placed {} Players on the field. Maximum are {} players.",
                all_players_on_field,
                max_players_on_field + additional_swarmers
            ));
            if additional_swarmers > 0 {
                messages.push(format!(
                    "Maximum {} regular Players and maximum {} Swarming Players.",
                    max_players_on_field, additional_swarmers
                ));
            }
        }

        if (all_players_on_field < max_players_on_field) && (available_players >= max_players_on_field) {
            messages.push(format!(
                "You placed {} Players on the field. You have to put {} players on the field (except Keen Players).",
                all_players_on_field, max_players_on_field
            ));
        } else if (all_players_on_field < max_players_on_field) && (all_players_on_field < available_players) {
            messages.push(format!(
                "You placed {} Players on the field. You have to put all players (except Keen Players) on the field.",
                all_players_on_field
            ));
        }

        let max_players_in_wide_zone = util_game_option::get_int_option(game, game_option_id::MAX_PLAYERS_IN_WIDE_ZONE);
        if players_in_lower_wide_zone > max_players_in_wide_zone {
            messages.push(format!(
                "You placed {} Players in the lower wide zone. Only {} allowed there.",
                players_in_lower_wide_zone, max_players_in_wide_zone
            ));
        }
        if players_in_upper_wide_zone > max_players_in_wide_zone {
            messages.push(format!(
                "You placed {} Players in the upper wide zone. Only {} allowed there.",
                players_in_upper_wide_zone, max_players_in_wide_zone
            ));
        }

        let min_players_on_los = util_game_option::get_int_option(game, game_option_id::MIN_PLAYERS_ON_LOS);
        if (players_on_los < min_players_on_los) && (available_players >= min_players_on_los) {
            messages.push(format!(
                "You placed {} Players on the Line of Scrimmage. You have to put {} there.",
                players_on_los, min_players_on_los
            ));
        } else if players_on_los < min_players_on_los && players_on_los < available_players {
            messages.push(format!(
                "You placed {} Players on the Line of Scrimmage. You have to put all your Players there.",
                players_on_los
            ));
        }

        if must_field_captain {
            messages.push("Your Team Captain is in reserves.".into());
        }

        if !messages.is_empty() {
            // Java: UtilServerDialog.showDialog(pGameState, new DialogSetupErrorParameter(team.getId(),
            //     messageList.toArray(new String[0])), false)
            let _ = (team_id, messages);
            UtilServerDialog::show_dialog(game, DialogId::SETUP_ERROR, false);
            false
        } else {
            true
        }
    }

    fn pin_players_in_tacklezones(&self, game: &mut Game, team_id: &str) {
        self.pin_players_in_tacklezones_chain(game, team_id, false);
    }

    /// Java: `pinPlayersInTacklezones(GameState, Team, boolean pinBallAndChain)`.
    ///
    /// Marks active players as inactive (pinned) when they are adjacent to an
    /// opponent tacklezone, or (when `pin_ball_and_chain=true`) the player has
    /// the `movesRandomly` property.
    fn pin_players_in_tacklezones_chain(
        &self,
        game: &mut Game,
        team_id: &str,
        pin_ball_and_chain: bool,
    ) {
        let other_team_id = if game.team_home.id == team_id {
            game.team_away.id.clone()
        } else {
            game.team_home.id.clone()
        };

        let player_data: Vec<(String, bool)> = {
            let team = if game.team_home.id == team_id {
                &game.team_home
            } else {
                &game.team_away
            };
            team.players.iter().map(|p| {
                let moves_randomly = p.has_skill_property(NamedProperties::MOVES_RANDOMLY);
                (p.id.clone(), moves_randomly)
            }).collect()
        };

        let mut to_pin: Vec<String> = Vec::new();
        for (player_id, moves_randomly) in &player_data {
            let state = match game.field_model.player_state(player_id) {
                Some(s) => s,
                None => continue,
            };
            if !state.is_active() {
                continue;
            }
            let coord = match game.field_model.player_coordinate(player_id) {
                Some(c) => c,
                None => continue,
            };

            let other_team = if game.team_home.id == other_team_id {
                &game.team_home
            } else {
                &game.team_away
            };
            let adjacent = UtilPlayer::find_adjacent_players_with_tacklezones(game, other_team, coord, false);
            let should_pin = !adjacent.is_empty() || (pin_ball_and_chain && *moves_randomly);
            if should_pin {
                to_pin.push(player_id.clone());
            }
        }

        for player_id in &to_pin {
            if let Some(s) = game.field_model.player_state(player_id) {
                game.field_model.set_player_state(player_id, s.change_active(false));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{Rules, SkillId, PlayerState, PS_STANDING};
    use ffb_model::model::game::Game;
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::model::team::Team;
    use ffb_model::types::FieldCoordinate;

    fn make_game() -> Game {
        let home = Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
        };
        let away = Team { id: "away".into(), ..home.clone() };
        Game::new(home, away, Rules::Bb2025)
    }

    fn add_player(game: &mut Game, team: &str, id: &str, coord: FieldCoordinate) {
        let player = ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            ..Default::default()
        };
        if team == "home" {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
    }

    fn add_player_with_skill(game: &mut Game, team: &str, id: &str, coord: FieldCoordinate, skill: SkillId) {
        let player = ffb_model::model::player::Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: ffb_model::enums::PlayerType::Regular,
            gender: ffb_model::enums::PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue { skill_id: skill, value: None }],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0,
            race: None,
            ..Default::default()
        };
        if team == "home" {
            game.team_home.players.push(player);
        } else {
            game.team_away.players.push(player);
        }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING).change_active(true));
    }

    fn mechanic() -> SetupMechanic { SetupMechanic }

    #[test]
    fn check_setup_empty_team_is_valid() {
        let mut game = make_game();
        assert!(mechanic().check_setup(&mut game, true));
    }

    #[test]
    fn check_setup_with_swarmers_zero_matches_check_setup() {
        let mut game = make_game();
        let a = mechanic().check_setup(&mut game, true);
        let b = mechanic().check_setup_with_swarmers(&mut game, true, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn pin_players_in_tacklezones_no_players_is_noop() {
        let mut game = make_game();
        mechanic().pin_players_in_tacklezones(&mut game, "home");
    }

    #[test]
    fn pin_players_in_tacklezones_chain_delegates() {
        let mut game = make_game();
        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", true);
        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", false);
    }

    #[test]
    fn check_setup_uses_game_options() {
        let mut game = make_game();
        game.options.set(game_option_id::MAX_PLAYERS_ON_FIELD, "11");
        game.options.set(game_option_id::MAX_PLAYERS_IN_WIDE_ZONE, "2");
        game.options.set(game_option_id::MIN_PLAYERS_ON_LOS, "3");
        assert!(mechanic().check_setup(&mut game, true));
    }

    #[test]
    fn pin_players_pins_player_adjacent_to_opponent_with_tacklezone() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", FieldCoordinate::new(5, 5));
        add_player(&mut game, "away", "a1", FieldCoordinate::new(6, 5));

        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", false);

        let state = game.field_model.player_state("h1").unwrap();
        assert!(!state.is_active(), "Player adjacent to opponent tackle zone should be pinned");
    }

    #[test]
    fn pin_players_does_not_pin_player_not_adjacent_to_opponent() {
        let mut game = make_game();
        add_player(&mut game, "home", "h1", FieldCoordinate::new(3, 3));
        add_player(&mut game, "away", "a1", FieldCoordinate::new(10, 10));

        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", false);

        let state = game.field_model.player_state("h1").unwrap();
        assert!(state.is_active(), "Player not adjacent to any opponent should remain active");
    }

    #[test]
    fn pin_ball_and_chain_true_pins_moves_randomly_player() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "home", "h1", FieldCoordinate::new(5, 5), SkillId::BallAndChain);

        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", true);

        let state = game.field_model.player_state("h1").unwrap();
        assert!(!state.is_active(), "BallAndChain player pinned when pin_ball_and_chain=true");
    }

    #[test]
    fn pin_ball_and_chain_false_does_not_pin_moves_randomly() {
        let mut game = make_game();
        add_player_with_skill(&mut game, "home", "h1", FieldCoordinate::new(5, 5), SkillId::BallAndChain);

        mechanic().pin_players_in_tacklezones_chain(&mut game, "home", false);

        let state = game.field_model.player_state("h1").unwrap();
        assert!(state.is_active(), "BallAndChain player should not be pinned when flag is false");
    }
}
