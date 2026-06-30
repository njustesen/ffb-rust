use crate::enums::{PlayerState, TurnMode, PS_RESERVE};
use crate::model::field_model::FieldModel;
use crate::model::game::Game;
use crate::model::player::{Player, PlayerId};
use crate::model::property::named_properties::NamedProperties;
use crate::model::team::Team;
use crate::types::FieldCoordinate;

/// 1:1 translation of com.fumbbl.ffb.util.UtilPlayer.
pub struct UtilPlayer;

impl UtilPlayer {
    pub fn new() -> Self { Self }

    /// 1:1 translation of findAdjacentPlayersWithTacklezones.
    pub fn find_adjacent_players_with_tacklezones<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
        with_start_coord: bool,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut adjacent = field_model.adjacent_on_pitch(coord);
        if with_start_coord {
            adjacent.push(coord);
        }
        let mut result = Vec::new();
        for adj_coord in adjacent {
            if let Some(id) = field_model.player_at(adj_coord) {
                if team.has_player(id) {
                    if let Some(state) = field_model.player_state(id) {
                        if state.has_tacklezones() {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// 1:1 translation of findAdjacentPlayers.
    pub fn find_adjacent_players<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj_coord in field_model.adjacent_on_pitch(coord) {
            if let Some(id) = field_model.player_at(adj_coord) {
                if team.has_player(id) {
                    result.push(id);
                }
            }
        }
        result
    }

    /// 1:1 translation of findAdjacentPronePlayers.
    pub fn find_adjacent_prone_players<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj_coord in field_model.adjacent_on_pitch(coord) {
            if let Some(id) = field_model.player_at(adj_coord) {
                if team.has_player(id) {
                    if let Some(state) = field_model.player_state(id) {
                        if state.is_prone() || state.is_stunned() {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// 1:1 translation of findPlayersWithTackleZones.
    pub fn find_players_with_tackle_zones<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
        distance: i32,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj_coord in Self::find_adjacent_coordinates(field_model, coord, distance) {
            if let Some(id) = field_model.player_at(adj_coord) {
                if team.has_player(id) {
                    if let Some(state) = field_model.player_state(id) {
                        if state.is_standing() && !state.is_distracted() {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// 1:1 translation of findBlockablePlayers.
    pub fn find_blockable_players<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
        distance: i32,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj_coord in Self::find_adjacent_coordinates(field_model, coord, distance) {
            if let Some(id) = field_model.player_at(adj_coord) {
                if team.has_player(id) {
                    if let Some(state) = field_model.player_state(id) {
                        if state.can_be_blocked() {
                            result.push(id);
                        }
                    }
                }
            }
        }
        result
    }

    /// 1:1 translation of findAdjacentBlockablePlayers.
    pub fn find_adjacent_blockable_players<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
    ) -> Vec<&'a PlayerId> {
        Self::find_blockable_players(game, team, coord, 1)
    }

    /// 1:1 translation of findPlayersOnPitchWithProperty.
    pub fn find_players_on_pitch_with_property<'a>(
        game: &'a Game,
        team: &'a Team,
        property: &str,
    ) -> Vec<&'a Player> {
        let field_model = &game.field_model;
        team.players
            .iter()
            .filter(|p| {
                p.has_skill_property(property)
                    && field_model
                        .player_coordinate(&p.id)
                        .map(|c| c.is_on_pitch())
                        .unwrap_or(false)
            })
            .collect()
    }

    /// 1:1 translation of findOtherTeam.
    pub fn find_other_team<'a>(game: &'a Game, player_id: &str) -> &'a Team {
        if game.team_home.has_player(player_id) {
            &game.team_away
        } else {
            &game.team_home
        }
    }

    /// 1:1 translation of findTacklezones.
    pub fn find_tacklezones(game: &Game, player_id: &str) -> usize {
        Self::find_tacklezone_players(game, player_id).len()
    }

    /// 1:1 translation of findTacklezonePlayers.
    pub fn find_tacklezone_players<'a>(game: &'a Game, player_id: &str) -> Vec<&'a PlayerId> {
        let other_team = Self::find_other_team(game, player_id);
        if let Some(coord) = game.field_model.player_coordinate(player_id) {
            Self::find_adjacent_players_with_tacklezones(game, other_team, coord, false)
        } else {
            Vec::new()
        }
    }

    /// 1:1 translation of findAdjacentOpposingPlayersWithProperty (4-arg overload).
    pub fn find_adjacent_opposing_players_with_property<'a>(
        game: &'a Game,
        acting_player_id: &str,
        coord: FieldCoordinate,
        property: &str,
        check_able_to_move: bool,
    ) -> Vec<&'a PlayerId> {
        Self::find_adjacent_opposing_players_with_property_ext(
            game,
            acting_player_id,
            coord,
            property,
            check_able_to_move,
            false,
        )
    }

    /// 1:1 translation of findAdjacentOpposingPlayersWithProperty (6-arg overload, requireUnusedSkill).
    pub fn find_adjacent_opposing_players_with_property_ext<'a>(
        game: &'a Game,
        acting_player_id: &str,
        coord: FieldCoordinate,
        property: &str,
        check_able_to_move: bool,
        _require_unused_skill: bool,
    ) -> Vec<&'a PlayerId> {
        // TODO: requireUnusedSkill requires UtilCards.hasUnusedSkillWithProperty — not yet translated
        let other_team = Self::find_other_team(game, acting_player_id);
        let opponents =
            Self::find_adjacent_players_with_tacklezones(game, other_team, coord, false);
        let field_model = &game.field_model;
        opponents
            .into_iter()
            .filter(|id| {
                let state = field_model.player_state(id);
                let has_tz = state.map(|s| s.has_tacklezones()).unwrap_or(false);
                let able = !check_able_to_move
                    || state.map(|s| s.is_able_to_move()).unwrap_or(false);
                let skill_ok = game.team_home
                    .player(id)
                    .or_else(|| game.team_away.player(id))
                    .map(|p| p.has_skill_property(property))
                    .unwrap_or(false);
                has_tz && skill_ok && able
            })
            .collect()
    }

    /// 1:1 translation of canFoul.
    pub fn can_foul(game: &Game, player_id: &str) -> bool {
        if let Some(coord) = game.field_model.player_coordinate(player_id) {
            let other_team = Self::find_other_team(game, player_id);
            !Self::find_adjacent_prone_players(game, other_team, coord).is_empty()
        } else {
            false
        }
    }

    /// 1:1 translation of hasBall.
    pub fn has_ball(game: &Game, player_id: &str) -> bool {
        let fm = &game.field_model;
        fm.ball_in_play
            && !fm.ball_moving
            && fm.ball_coordinate
                .zip(fm.player_coordinate(player_id))
                .map(|(b, p)| b == p)
                .unwrap_or(false)
    }

    /// 1:1 translation of isBallAvailable.
    pub fn is_ball_available(game: &Game, player_id: &str) -> bool {
        let fm = &game.field_model;
        fm.ball_in_play
            && (fm.ball_moving
                || fm.ball_coordinate
                    .zip(fm.player_coordinate(player_id))
                    .map(|(b, p)| b == p)
                    .unwrap_or(false))
    }

    /// 1:1 translation of isNextMovePossible.
    pub fn is_next_move_possible(game: &Game, jumping: bool) -> bool {
        if game.acting_player.held_in_place {
            return false;
        }
        Self::has_move_left(game, jumping)
    }

    /// 1:1 translation of hasMoveLeft.
    pub fn has_move_left(game: &Game, jumping: bool) -> bool {
        let ap = &game.acting_player;
        let player_id = match &ap.player_id {
            Some(id) => id,
            None => return false,
        };
        let state = game.field_model.player_state(player_id);
        if !state.map(|s| s.is_able_to_move()).unwrap_or(false) {
            return false;
        }
        let player = game.team_home.player(player_id).or_else(|| game.team_away.player(player_id));
        let player = match player {
            Some(p) => p,
            None => return false,
        };
        match game.turn_mode {
            TurnMode::KickoffReturn | TurnMode::PassBlock => {
                if jumping { ap.current_move < 2 } else { ap.current_move < 3 }
            }
            _ => {
                let mut extra_move = 0;
                if ap.goes_for_it {
                    extra_move = 2;
                    if player.has_skill_property(NamedProperties::CAN_MAKE_AN_EXTRA_GFI) {
                        extra_move += 1;
                    }
                    // TODO: canMakeAnExtraGfiOnce (SureFeet) requires UtilCards.hasUnusedSkillWithProperty
                    if jumping {
                        extra_move -= 1;
                    }
                }
                ap.current_move < player.movement_with_modifiers() + extra_move
            }
        }
    }

    /// 1:1 translation of isNextMoveGoingForIt.
    pub fn is_next_move_going_for_it(game: &Game) -> bool {
        let ap = &game.acting_player;
        let player_id = match &ap.player_id {
            Some(id) => id,
            None => return false,
        };
        if matches!(game.turn_mode, TurnMode::KickoffReturn | TurnMode::PassBlock) {
            return false;
        }
        let player = game.team_home.player(player_id).or_else(|| game.team_away.player(player_id));
        let player = match player {
            Some(p) => p,
            None => return false,
        };
        if ap.standing_up && !ap.has_acted && !player.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE) {
            3 >= player.movement_with_modifiers()
        } else if ap.jumping {
            (ap.current_move + 1) >= player.movement_with_modifiers()
        } else {
            ap.current_move >= player.movement_with_modifiers()
        }
    }

    /// 1:1 translation of testPlayersAbleToAct.
    pub fn test_players_able_to_act(game: &Game, team: &Team) -> bool {
        let fm = &game.field_model;
        team.players.iter().any(|p| {
            fm.player_coordinate(&p.id)
                .map(|c| c.is_on_pitch())
                .unwrap_or(false)
                && fm.player_state(&p.id)
                    .map(|s| s.is_active())
                    .unwrap_or(false)
        })
    }

    /// 1:1 translation of findPlayersInReserveOrField.
    pub fn find_players_in_reserve_or_field<'a>(game: &'a Game, team: &'a Team) -> Vec<&'a Player> {
        let fm = &game.field_model;
        team.players
            .iter()
            .filter(|p| {
                fm.player_coordinate(&p.id)
                    .map(|c| c.is_on_pitch())
                    .unwrap_or(false)
                    || fm.player_state(&p.id)
                        .map(|s| s.base() == PS_RESERVE)
                        .unwrap_or(false)
            })
            .collect()
    }

    /// 1:1 translation of hasAdjacentGazeTarget.
    pub fn has_adjacent_gaze_target(game: &Game, player_id: &str) -> bool {
        if let Some(coord) = game.field_model.player_coordinate(player_id) {
            let other_team = Self::find_other_team(game, player_id);
            !Self::find_adjacent_players_with_tacklezones(game, other_team, coord, false).is_empty()
        } else {
            false
        }
    }

    /// 1:1 translation of canGaze(Game, Player, NamedProperties.inflictsConfusion).
    /// Returns true if the player can declare a Gaze (Hypnotic Gaze) action:
    ///   - has an unused skill with the "inflictsConfusion" property
    ///   - is in an active player state
    ///   - either the mechanic allows gaze at start OR has an adjacent opponent with a tacklezone
    ///
    /// Note: `declareGazeActionAtStart` is not yet wired (requires GameMechanic factory). This
    /// implementation conservatively checks only the adjacent-opponent branch.
    pub fn can_gaze(game: &Game, player_id: &str) -> bool {
        use crate::model::property::named_properties::NamedProperties;
        let player = match game.player(player_id) {
            Some(p) => p,
            None => return false,
        };
        // Java: player.getSkillWithProperty(inflictsConfusion) == null || player.isUsed(skill)
        if !player.has_skill_property(NamedProperties::INFLICTS_CONFUSION) {
            return false;
        }
        // Java: !playerState.isActive()
        let is_active = game.field_model.player_state(player_id)
            .map(|s| s.is_active())
            .unwrap_or(false);
        if !is_active {
            return false;
        }
        // Java: mechanic.declareGazeActionAtStart() || hasAdjacentEnemyWithTacklezone
        // declareGazeActionAtStart() not yet wired → conservatively use adjacent check.
        Self::has_adjacent_gaze_target(game, player_id)
    }

    /// Returns all coordinates within `distance` from `coord` that are on the pitch
    /// (Chebyshev distance — matches Java FieldModel.findAdjacentCoordinates).
    fn find_adjacent_coordinates(
        field_model: &FieldModel,
        coord: FieldCoordinate,
        distance: i32,
    ) -> Vec<FieldCoordinate> {
        if distance == 1 {
            return field_model.adjacent_on_pitch(coord);
        }
        let mut result = Vec::new();
        for dx in -distance..=distance {
            for dy in -distance..=distance {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = coord.x + dx;
                let ny = coord.y + dy;
                let c = FieldCoordinate::new(nx, ny);
                if c.is_on_pitch() {
                    result.push(c);
                }
            }
        }
        result
    }
}

impl Default for UtilPlayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{PlayerType, PlayerGender, Rules};
    use crate::model::game::Game;
    use crate::model::player::Player;
    use crate::model::team::Team;

    // PS_STANDING(0x1) | BIT_ACTIVE(0x100) = 0x101
    const ACTIVE_STANDING: PlayerState = PlayerState(0x101);
    // PS_PRONE(0x3) | BIT_ACTIVE(0x100) = 0x103
    const ACTIVE_PRONE: PlayerState = PlayerState(0x103);

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![],
            players: vec![],
        }
    }

    fn minimal_player(id: &str) -> Player {
        Player {
            id: id.into(),
            name: id.into(),
            nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6,
            strength: 3,
            agility: 3,
            passing: 4,
            armour: 8,
            starting_skills: vec![],
            extra_skills: vec![],
            temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
        }
    }

    fn minimal_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, state: PlayerState) {
        let p = minimal_player(id);
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn find_other_team_home_player_returns_away() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        let other = UtilPlayer::find_other_team(&game, "h1");
        assert_eq!(other.id, "away");
    }

    #[test]
    fn find_other_team_away_player_returns_home() {
        let mut game = minimal_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        let other = UtilPlayer::find_other_team(&game, "a1");
        assert_eq!(other.id, "home");
    }

    #[test]
    fn find_adjacent_players_with_tacklezones_returns_standing_adjacent() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h2", FieldCoordinate::new(10, 10), ACTIVE_STANDING);
        let center = FieldCoordinate::new(5, 5);
        let team = game.team_home.clone();
        let results = UtilPlayer::find_adjacent_players_with_tacklezones(
            &game, &team, center, false,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "h1");
    }

    #[test]
    fn find_tacklezones_counts_opposing_adjacent_standing() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a1", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a2", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_tacklezones(&game, "h1"), 2);
    }

    #[test]
    fn find_tacklezones_zero_when_no_adjacent_opponents() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 10), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_tacklezones(&game, "h1"), 0);
    }

    #[test]
    fn can_foul_true_when_adjacent_prone_opponent() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a1", FieldCoordinate::new(6, 5), ACTIVE_PRONE);
        assert!(UtilPlayer::can_foul(&game, "h1"));
    }

    #[test]
    fn can_foul_false_when_no_prone_adjacent() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a1", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        assert!(!UtilPlayer::can_foul(&game, "h1"));
    }

    #[test]
    fn is_next_move_possible_false_when_held_in_place() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.held_in_place = true;
        assert!(!UtilPlayer::is_next_move_possible(&game, false));
    }

    #[test]
    fn has_move_left_true_when_moves_remaining() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.current_move = 5;
        assert!(UtilPlayer::has_move_left(&game, false));
    }

    #[test]
    fn has_move_left_false_when_all_moves_used() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.current_move = 6;
        assert!(!UtilPlayer::has_move_left(&game, false));
    }

    #[test]
    fn has_move_left_goes_for_it_allows_two_extra() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        game.acting_player.player_id = Some("h1".into());
        game.acting_player.current_move = 7;
        game.acting_player.goes_for_it = true;
        assert!(UtilPlayer::has_move_left(&game, false));
    }

    #[test]
    fn has_ball_false_when_ball_not_at_player() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(FieldCoordinate::new(10, 10));
        assert!(!UtilPlayer::has_ball(&game, "h1"));
    }

    #[test]
    fn has_ball_true_when_ball_at_player() {
        let mut game = minimal_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, true, "h1", coord, ACTIVE_STANDING);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(coord);
        assert!(UtilPlayer::has_ball(&game, "h1"));
    }

    // ── can_gaze ──────────────────────────────────────────────────────────────

    fn add_hypnotic_gaze_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate, state: PlayerState) {
        use crate::model::skill_def::SkillWithValue;
        use crate::enums::SkillId;
        let mut p = minimal_player(id);
        p.starting_skills.push(SkillWithValue::new(SkillId::HypnoticGaze));
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, state);
    }

    #[test]
    fn can_gaze_false_when_no_hypnotic_gaze_skill() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        // away opponent adjacent
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert!(!UtilPlayer::can_gaze(&game, "h1"));
    }

    #[test]
    fn can_gaze_false_when_no_adjacent_opponent() {
        let mut game = minimal_game();
        add_hypnotic_gaze_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        // no away players
        assert!(!UtilPlayer::can_gaze(&game, "h1"));
    }

    #[test]
    fn can_gaze_true_when_active_with_skill_and_adjacent_opponent() {
        let mut game = minimal_game();
        add_hypnotic_gaze_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert!(UtilPlayer::can_gaze(&game, "h1"));
    }

    #[test]
    fn can_gaze_false_when_player_not_active() {
        use crate::enums::PS_PRONE;
        let mut game = minimal_game();
        let prone = PlayerState::new(PS_PRONE);
        add_hypnotic_gaze_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), prone);
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert!(!UtilPlayer::can_gaze(&game, "h1"));
    }
}
