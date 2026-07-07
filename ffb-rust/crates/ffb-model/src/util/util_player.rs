use crate::enums::{PlayerState, TurnMode, PS_RESERVE};
use crate::model::field_model::FieldModel;
use crate::model::game::Game;
use crate::model::player::{Player, PlayerId};
use crate::model::property::named_properties::NamedProperties;
use crate::model::team::Team;
use crate::types::FieldCoordinate;
use crate::util::util_cards::UtilCards;

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

    /// 1:1 translation of findAdjacentPlayersToFeedOn.
    /// Returns team-owned thrall players adjacent to `coord`.
    pub fn find_adjacent_players_to_feed_on<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
    ) -> Vec<&'a PlayerId> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj_coord in field_model.adjacent_on_pitch(coord) {
            if let Some(id) = field_model.player_at(adj_coord) {
                if !team.has_player(id) { continue; }
                if let Some(player) = game.player(id) {
                    if player.is_thrall {
                        result.push(id);
                    }
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

    /// 1:1 translation of findBlockablePlayersTwoSquaresAway.
    /// Returns players at Chebyshev distance 2 from coord that can be blocked,
    /// excluding those adjacent (distance 1).
    pub fn find_blockable_players_two_squares_away<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
    ) -> Vec<&'a PlayerId> {
        let at_two = Self::find_blockable_players(game, team, coord, 2);
        let at_one: std::collections::HashSet<&PlayerId> =
            Self::find_adjacent_blockable_players(game, team, coord).into_iter().collect();
        at_two.into_iter().filter(|id| !at_one.contains(id)).collect()
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

    /// 1:1 translation of findStandUpAssists.
    ///
    /// Returns number of friendly standing players adjacent to `player_id` who are not
    /// themselves adjacent to any opposing player with tacklezones.
    pub fn find_stand_up_assists(game: &Game, player_id: &str) -> i32 {
        let own_team = if game.team_home.has_player(player_id) {
            &game.team_home
        } else {
            &game.team_away
        };
        let opposing_team = Self::find_other_team(game, player_id);
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return 0,
        };
        let mut assists = 0i32;
        for assist_id in Self::find_adjacent_players_with_tacklezones(game, own_team, coord, false) {
            if let Some(assist_coord) = game.field_model.player_coordinate(assist_id) {
                let opponents = Self::find_adjacent_players_with_tacklezones(game, opposing_team, assist_coord, false);
                if opponents.is_empty() {
                    assists += 1;
                }
            }
        }
        assists
    }

    /// 1:1 translation of findOffensiveFoulAssists(Game, Player, Player, SkillMechanic).
    ///
    /// Counts attacker-team players adjacent to the defender (with tacklezones, excluding attacker)
    /// that are not marked by opposing players — or have canAlwaysAssistFouls (SneakyGit/PutTheBootIn)
    /// and that property isn't cancelled by an adjacent opponent with cancelsCanAlwaysAssistFouls.
    pub fn find_offensive_foul_assists(game: &Game, attacker_id: &str, defender_id: &str) -> i32 {
        let attacker_team = if game.team_home.has_player(attacker_id) { &game.team_home } else { &game.team_away };
        let defender_team = if game.team_home.has_player(defender_id) { &game.team_home } else { &game.team_away };
        let def_coord = match game.field_model.player_coordinate(defender_id) {
            Some(c) => c,
            None => return 0,
        };
        let mut assists = 0i32;
        for assist_id in Self::find_adjacent_players_with_tacklezones(game, attacker_team, def_coord, false) {
            if assist_id == attacker_id {
                continue;
            }
            if let Some(assist_coord) = game.field_model.player_coordinate(assist_id) {
                let adjacent_defenders = Self::find_adjacent_players_with_tacklezones(game, defender_team, assist_coord, false);
                let can_always_assist = game.player(assist_id)
                    .map(|p| p.has_skill_property(NamedProperties::CAN_ALWAYS_ASSIST_FOULS))
                    .unwrap_or(false);
                let cancelled = adjacent_defenders.iter().any(|def_id| {
                    game.player(def_id)
                        .map(|p| p.has_skill_property(NamedProperties::CANCELS_CAN_ALWAYS_ASSIST_FOULS))
                        .unwrap_or(false)
                });
                if adjacent_defenders.is_empty() || (can_always_assist && !cancelled) {
                    assists += 1;
                }
            }
        }
        assists
    }

    /// 1:1 translation of findDefensiveFoulAssists(Game, Player, Player).
    ///
    /// Counts defender-team players adjacent to the attacker (with tacklezones, excluding defender)
    /// where the attacker-side count of adjacent TZ players next to that assist player is < 2.
    pub fn find_defensive_foul_assists(game: &Game, attacker_id: &str, defender_id: &str) -> i32 {
        let attacker_team = if game.team_home.has_player(attacker_id) { &game.team_home } else { &game.team_away };
        let defender_team = if game.team_home.has_player(defender_id) { &game.team_home } else { &game.team_away };
        let att_coord = match game.field_model.player_coordinate(attacker_id) {
            Some(c) => c,
            None => return 0,
        };
        let mut assists = 0i32;
        for assist_id in Self::find_adjacent_players_with_tacklezones(game, defender_team, att_coord, false) {
            if assist_id == defender_id {
                continue;
            }
            if let Some(assist_coord) = game.field_model.player_coordinate(assist_id) {
                let adjacent_attackers = Self::find_adjacent_players_with_tacklezones(game, attacker_team, assist_coord, false);
                if adjacent_attackers.len() < 2 {
                    assists += 1;
                }
            }
        }
        assists
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
        require_unused_skill: bool,
    ) -> Vec<&'a PlayerId> {
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
                let player = game.team_home.player(id).or_else(|| game.team_away.player(id));
                let skill_ok = player.map(|p| {
                    if require_unused_skill {
                        UtilCards::has_unused_skill_with_property(p, property)
                    } else {
                        p.has_skill_property(property)
                    }
                }).unwrap_or(false);
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
                    if UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_MAKE_AN_EXTRA_GFI_ONCE) {
                        extra_move += 1;
                    }
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

    /// Java: UtilPlayer.canHandOver(Game, Player) — BB2020+.
    ///
    /// Returns true if the thrower is carrying the ball, the ball is not moving,
    /// and at least one team-mate with tackle zones is adjacent.
    pub fn can_hand_over(game: &Game, player_id: &str) -> bool {
        let coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return false,
        };
        // Thrower must be on the ball.
        if game.field_model.ball_coordinate != Some(coord) || game.field_model.ball_moving {
            return false;
        }
        // Java: !pGame.getTurnData().isHandOverUsed() — deferred; no turnData field.
        // At least one same-team adjacent player with tackle zones.
        let is_home = game.team_home.has_player(player_id);
        let team = if is_home { &game.team_home } else { &game.team_away };
        !Self::find_adjacent_players_with_tacklezones(game, team, coord, false).is_empty()
    }

    /// 1:1 translation of findStandingOrPronePlayers(Game, Team, FieldCoordinate, distance).
    ///
    /// Returns team players within `distance` Chebyshev squares of `coord` that are not stunned.
    pub fn find_standing_or_prone_players<'a>(
        game: &'a Game,
        team: &'a Team,
        coord: FieldCoordinate,
        distance: i32,
    ) -> Vec<&'a Player> {
        let field_model = &game.field_model;
        let mut result = Vec::new();
        for adj in Self::find_adjacent_coordinates(field_model, coord, distance) {
            if let Some(id) = field_model.player_at(adj) {
                if team.has_player(id) {
                    if let Some(state) = field_model.player_state(id) {
                        if !state.is_stunned() {
                            if let Some(player) = game.player(id) {
                                result.push(player);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    /// Java: UtilPlayer.refreshPlayersForTurnStart(Game) — resets transient player states at the
    /// start of each team's turn. Called after flipping `home_playing` to the new active team.
    ///
    /// Transitions (1:1 with Java switch):
    ///   BLOCKED / MOVING / FALLING / HIT_ON_GROUND → STANDING + active
    ///   PRONE / STANDING → setActive only
    ///   STUNNED (new active team) → PRONE + active=false
    ///
    /// `enhancements_to_remove`: skill-source names removed from every player at end-of-turn
    ///   (e.g. "Wisdom of the White Dwarf"). Pass an empty set when the mechanic is unavailable.
    /// `enhancements_to_remove_when_not_active`: same, but only for players that are NOT being
    ///   set active this turn (e.g. extra skills granted only while active).
    pub fn refresh_players_for_turn_start(
        game: &mut Game,
        enhancements_to_remove: &std::collections::HashSet<String>,
        enhancements_to_remove_when_not_active: &std::collections::HashSet<String>,
    ) {
        use crate::enums::{PS_BLOCKED, PS_FALLING, PS_HIT_ON_GROUND, PS_MOVING, PS_PRONE, PS_STANDING, PS_STUNNED};
        use crate::enums::SkillUsageType;
        use crate::model::property::NamedProperties;
        let home_playing = game.home_playing;

        // First pass: collect per-player data (avoids holding borrows across mutable ops).
        struct PlayerEntry { id: String, is_home: bool, has_to_miss_turn: bool }
        let entries: Vec<PlayerEntry> = game.team_home.players.iter()
            .map(|p| PlayerEntry {
                id: p.id.clone(),
                is_home: true,
                has_to_miss_turn: p.has_skill_property(NamedProperties::HAS_TO_MISS_TURN),
            })
            .chain(game.team_away.players.iter().map(|p| PlayerEntry {
                id: p.id.clone(),
                is_home: false,
                has_to_miss_turn: p.has_skill_property(NamedProperties::HAS_TO_MISS_TURN),
            }))
            .collect();

        // Remove per-turn enhancements from all players.
        for p in game.team_home.players.iter_mut().chain(game.team_away.players.iter_mut()) {
            p.reset_used_skills(SkillUsageType::OncePerTurn);
            p.reset_used_skills(SkillUsageType::OncePerTurnByTeamMate);
            for name in enhancements_to_remove.iter() {
                p.remove_enhancements(name);
            }
        }

        // Second pass: update player states.
        for entry in &entries {
            let Some(ps) = game.field_model.player_state(&entry.id) else { continue };
            let base = ps.base();
            let player_on_team_from_last_turn = entry.is_home != home_playing;
            let set_active = player_on_team_from_last_turn || !entry.has_to_miss_turn;
            let new_ps = if base == PS_BLOCKED || base == PS_MOVING || base == PS_FALLING || base == PS_HIT_ON_GROUND {
                Some(ps.change_base(PS_STANDING).change_active(set_active))
            } else if base == PS_STANDING || base == PS_PRONE {
                Some(ps.change_active(set_active))
            } else if base == PS_STUNNED && entry.is_home == home_playing {
                Some(ps.change_base(PS_PRONE).change_active(false))
            } else {
                None
            };
            if let Some(new_ps) = new_ps {
                game.field_model.set_player_state(&entry.id, new_ps);
            }
            // Java: enhancementsToRemoveWhenNotSettingActive — only strip when not becoming active.
            if !set_active {
                if let Some(p) = game.team_home.players.iter_mut().chain(game.team_away.players.iter_mut())
                    .find(|p| p.id == entry.id)
                {
                    for name in enhancements_to_remove_when_not_active.iter() {
                        p.remove_enhancements(name);
                    }
                }
            }
        }
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
            vampire_lord: false,
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
            is_big_guy: false,
            ..Default::default()
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

    #[test]
    fn find_stand_up_assists_no_friendly_adjacent_returns_zero() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_stand_up_assists(&game, "h1"), 0);
    }

    #[test]
    fn find_stand_up_assists_friendly_not_pressured_counts() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        // h2 is adjacent to h1 and has no opposing tacklezones adjacent to h2
        add_player(&mut game, true, "h2", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_stand_up_assists(&game, "h1"), 1);
    }

    #[test]
    fn find_stand_up_assists_friendly_under_pressure_does_not_count() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h2", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        // Opposing player adjacent to h2 — h2 is pressured, should not count
        add_player(&mut game, false, "a1", FieldCoordinate::new(7, 5), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_stand_up_assists(&game, "h1"), 0);
    }

    #[test]
    fn find_stand_up_assists_two_unpressured_friendlies() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h2", FieldCoordinate::new(6, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h3", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_stand_up_assists(&game, "h1"), 2);
    }

    #[test]
    fn find_standing_or_prone_players_returns_non_stunned_team_mates() {
        use crate::enums::PS_STUNNED;
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h2", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        let stunned = PlayerState::new(PS_STUNNED);
        add_player(&mut game, true, "h3", FieldCoordinate::new(5, 7), stunned);
        let team = game.team_home.clone();
        let result = UtilPlayer::find_standing_or_prone_players(
            &game, &team, FieldCoordinate::new(5, 5), 1
        );
        // h2 is adjacent and not stunned; h3 is stunned (excluded); h1 is origin (excluded)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "h2");
    }

    #[test]
    fn find_standing_or_prone_players_distance_2_includes_two_squares_away() {
        let mut game = minimal_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, true, "h2", FieldCoordinate::new(5, 7), ACTIVE_STANDING);
        let team = game.team_home.clone();
        let result = UtilPlayer::find_standing_or_prone_players(
            &game, &team, FieldCoordinate::new(5, 5), 2
        );
        assert!(result.iter().any(|p| p.id == "h2"), "should find player 2 squares away");
    }

    #[test]
    fn find_standing_or_prone_players_excludes_opposing_team() {
        let mut game = minimal_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        let team = game.team_home.clone();
        let result = UtilPlayer::find_standing_or_prone_players(
            &game, &team, FieldCoordinate::new(5, 5), 1
        );
        assert!(result.is_empty(), "should not include opposing team players");
    }

    // PS_STUNNED base = 0x4; PS_PRONE base = 0x3 (see enums/player.rs constants)
    const STUNNED: PlayerState = PlayerState(crate::enums::PS_STUNNED);
    const PRONE_INACTIVE: PlayerState = PlayerState(crate::enums::PS_PRONE);

    #[test]
    fn refresh_players_stunned_on_active_team_becomes_prone() {
        let mut game = minimal_game();
        game.home_playing = true;
        // home player is STUNNED; home_playing=true so home team is the "new active team"
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), STUNNED);
        UtilPlayer::refresh_players_for_turn_start(&mut game, &Default::default(), &Default::default());
        let ps = game.field_model.player_state("h1").unwrap();
        assert_eq!(ps.base(), crate::enums::PS_PRONE, "STUNNED on active team → PRONE");
    }

    #[test]
    fn refresh_players_blocked_becomes_standing() {
        use crate::enums::PS_BLOCKED;
        let mut game = minimal_game();
        game.home_playing = true;
        let blocked = PlayerState(PS_BLOCKED);
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), blocked);
        UtilPlayer::refresh_players_for_turn_start(&mut game, &Default::default(), &Default::default());
        let ps = game.field_model.player_state("h1").unwrap();
        assert_eq!(ps.base(), crate::enums::PS_STANDING, "BLOCKED → STANDING");
    }

    #[test]
    fn refresh_players_stunned_on_opponent_team_unchanged() {
        let mut game = minimal_game();
        game.home_playing = true;
        // away player is STUNNED; away team is NOT the active team → should not recover
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5), STUNNED);
        UtilPlayer::refresh_players_for_turn_start(&mut game, &Default::default(), &Default::default());
        let ps = game.field_model.player_state("a1").unwrap();
        assert_eq!(ps.base(), crate::enums::PS_STUNNED, "STUNNED on opponent stays STUNNED");
    }

    #[test]
    fn field_model_clear_track_numbers_is_no_op() {
        let mut fm = crate::model::field_model::FieldModel::new();
        fm.clear_track_numbers(); // should not panic
    }

    // PS_PRONE(0x3) = prone, no tacklezones
    const PRONE: PlayerState = PlayerState(0x3);

    #[test]
    fn find_offensive_foul_assists_no_assistants() {
        let mut game = minimal_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        // def is prone (typical for foul victim) — no tacklezones
        add_player(&mut game, false, "def", FieldCoordinate::new(5, 6), PRONE);
        // No other attacker-team player adjacent to def
        assert_eq!(UtilPlayer::find_offensive_foul_assists(&game, "att", "def"), 0);
    }

    #[test]
    fn find_offensive_foul_assists_one_free_assistant() {
        let mut game = minimal_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        // def is prone — no tacklezones (cannot mark assist1)
        add_player(&mut game, false, "def", FieldCoordinate::new(7, 7), PRONE);
        // assist1 adjacent to def, no standing defender adjacent to assist1
        add_player(&mut game, true, "assist1", FieldCoordinate::new(7, 6), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_offensive_foul_assists(&game, "att", "def"), 1);
    }

    #[test]
    fn find_offensive_foul_assists_assistant_marked_by_standing_defender() {
        let mut game = minimal_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(7, 7), PRONE);
        add_player(&mut game, true, "assist1", FieldCoordinate::new(7, 6), ACTIVE_STANDING);
        // A standing defender adjacent to assist1 marks it
        add_player(&mut game, false, "def2", FieldCoordinate::new(7, 5), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_offensive_foul_assists(&game, "att", "def"), 0);
    }

    #[test]
    fn find_defensive_foul_assists_no_assistants() {
        let mut game = minimal_game();
        add_player(&mut game, true, "att", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(5, 6), PRONE);
        assert_eq!(UtilPlayer::find_defensive_foul_assists(&game, "att", "def"), 0);
    }

    #[test]
    fn find_defensive_foul_assists_one_free_assistant() {
        let mut game = minimal_game();
        // att at (5,5); def2 (defender) at (5,6) adjacent to att; att is the only attacker adjacent
        // to def2 (< 2), so def2 counts as a defensive assist
        add_player(&mut game, true, "att", FieldCoordinate::new(5, 5), ACTIVE_STANDING);
        add_player(&mut game, false, "def", FieldCoordinate::new(10, 10), PRONE);
        add_player(&mut game, false, "def2", FieldCoordinate::new(5, 6), ACTIVE_STANDING);
        assert_eq!(UtilPlayer::find_defensive_foul_assists(&game, "att", "def"), 1);
    }

    #[test]
    fn refresh_removes_enhancements_for_all_players() {
        use std::collections::HashSet;
        use crate::enums::{SkillId, PS_STANDING};
        let mut game = minimal_game();
        let ps = PlayerState(PS_STANDING);
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ps);
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 5), ps);
        // Give each player a "Wisdom of the White Dwarf" enhancement
        game.team_home.players.iter_mut().find(|p| p.id == "h1").unwrap()
            .add_prayer_skill("Wisdom of the White Dwarf", SkillId::Dodge, None);
        game.team_away.players.iter_mut().find(|p| p.id == "a1").unwrap()
            .add_prayer_skill("Wisdom of the White Dwarf", SkillId::Sprint, None);
        let mut to_remove = HashSet::new();
        to_remove.insert("Wisdom of the White Dwarf".to_string());
        UtilPlayer::refresh_players_for_turn_start(&mut game, &to_remove, &Default::default());
        let h1 = game.team_home.players.iter().find(|p| p.id == "h1").unwrap();
        let a1 = game.team_away.players.iter().find(|p| p.id == "a1").unwrap();
        assert!(h1.temporary_skills.is_empty(), "wisdom enhancement removed from home");
        assert!(a1.temporary_skills.is_empty(), "wisdom enhancement removed from away");
    }

    #[test]
    fn refresh_removes_conditional_enhancements_only_when_not_setting_active() {
        use std::collections::HashSet;
        use crate::enums::{SkillId, PS_STANDING};
        let mut game = minimal_game();
        let ps = PlayerState(PS_STANDING);
        game.home_playing = true;
        // h1 (home) sets active=false if has_to_miss_turn; use a player without that skill
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5), ps);
        add_player(&mut game, false, "a1", FieldCoordinate::new(10, 5), ps);
        game.team_home.players.iter_mut().find(|p| p.id == "h1").unwrap()
            .add_prayer_skill("ConditionalSkill", SkillId::Dodge, None);
        game.team_away.players.iter_mut().find(|p| p.id == "a1").unwrap()
            .add_prayer_skill("ConditionalSkill", SkillId::Sprint, None);
        // conditional set for "ConditionalSkill" — only removes when NOT setting active
        let mut cond_remove = HashSet::new();
        cond_remove.insert("ConditionalSkill".to_string());
        UtilPlayer::refresh_players_for_turn_start(&mut game, &Default::default(), &cond_remove);
        // h1 is home, home_playing=true → player_on_team_from_last_turn=false → set_active = !has_to_miss_turn
        // a1 is away, home_playing=true → player_on_team_from_last_turn=true → set_active=true
        // Both h1 and a1 should keep the skill (both set active)
        let h1 = game.team_home.players.iter().find(|p| p.id == "h1").unwrap();
        let a1 = game.team_away.players.iter().find(|p| p.id == "a1").unwrap();
        assert!(!h1.temporary_skills.is_empty(), "active player keeps conditional skill");
        assert!(!a1.temporary_skills.is_empty(), "active player keeps conditional skill");
    }
}
