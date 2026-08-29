/// 1:1 translation of `com.fumbbl.ffb.server.step.bb2025.shared.StallingExtension`.
///
/// Helper logic for stalling detection and penalty, shared across stalling-related steps.
use ffb_model::model::game::Game;
use ffb_model::enums::ApothecaryMode;
use crate::drop_player_context::{DropPlayerContext, SteadyFootingContext};
use crate::step::util_server_injury::handle_injury_by_name;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds};
use ffb_model::util::pathfinding::path_finder_with_pass_block_support::PathFinderWithPassBlockSupport;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;

pub struct StallingExtension;

impl StallingExtension {
    pub fn new() -> Self { Self }

    /// Java `isConsideredStalling(Game, Player<?> player)`.
    ///
    /// A player is considered stalling when ALL of the following hold:
    ///  1. They are carrying the ball.
    ///  2. None of their skills has a "roll-at-activation" property (appliesConfusion,
    ///     needsToRollForActionBlockingIsEasier, needsToRollForActionButKeepsTacklezone,
    ///     becomesImmovable) — those players may be unable to move.
    ///  3. No opponent with a tacklezone is adjacent (clear path).
    ///  4. They have an open path to the enemy end-zone (pathfinder check).
    ///
    /// The pathfinder (PathFinderWithPassBlockSupport) and the skill property checks are not
    /// yet translated, so this implementation is a conservative stub: it always returns false
    /// (no false positives). TODO: port pathfinder and skill-property lookups.
    pub fn is_considered_stalling(&self, game: &Game, player_id: &str) -> bool {
        // Guard 1: must have the ball.
        if !UtilPlayer::has_ball(game, player_id) {
            return false;
        }
        // Guard 2: skip players whose skill set includes a "roll at activation" property.
        // Java: checks appliesConfusion, needsToRollForActionBlockingIsEasier,
        //       needsToRollForActionButKeepsTacklezone, becomesImmovable.
        let has_activation_roll = game.player(player_id).map(|p| {
            p.has_skill_property(NamedProperties::APPLIES_CONFUSION)
                || p.has_skill_property(NamedProperties::NEEDS_TO_ROLL_FOR_ACTION_BLOCKING_IS_EASIER)
                || p.has_skill_property(NamedProperties::NEEDS_TO_ROLL_FOR_ACTION_BUT_KEEPS_TACKLEZONE)
                || p.has_skill_property(NamedProperties::BECOMES_IMMOVABLE)
        }).unwrap_or(false);
        if has_activation_roll {
            return false;
        }
        // Guard 3: no adjacent opponent with tacklezones.
        let opponent = UtilPlayer::find_other_team(game, player_id);
        if let Some(coord) = game.field_model.player_coordinate(player_id) {
            let tacklers = UtilPlayer::find_adjacent_players_with_tacklezones(
                game, opponent, coord, false,
            );
            if !tacklers.is_empty() {
                return false;
            }
        } else {
            return false;
        }
        // Guard 4: an open path to the enemy end-zone.
        //
        // Java `hasOpenPathToEndzone`: the endzone the player is ATTACKING (away's for a home
        // player), every square of it as the target set, and
        // `PathFinderWithPassBlockSupport.getShortestPath(game, endZoneCoordinates, player, 0)` --
        // `0` because this asks whether the carrier could have scored with a FULL move, not from
        // wherever he has already got to.
        //
        // This was a stub returning `false`, on the note that the pathfinder was not translated.
        // It is: `ffb_model::util::pathfinding::path_finder_with_pass_block_support`, with
        // `get_shortest_path_for_player` taking exactly Java's four arguments. So the whole BB2025
        // stalling rule was dead -- `StepForgoneStalling` ran, found its carrier and asked this,
        // which always said no. Java rolls a d6 for such a carrier and Rust rolled nothing, which
        // shifts every later die by one (bb2025 seed 26 step 25 under uniform sampling: identical
        // boards, rng_calls 46 against 45).
        let player = match game.player(player_id) {
            Some(p) => p,
            None => return false,
        };
        let attacking_away_endzone = game.team_home.has_player(player_id);
        let bounds = if attacking_away_endzone {
            FieldCoordinateBounds::ENDZONE_AWAY
        } else {
            FieldCoordinateBounds::ENDZONE_HOME
        };
        let end_coords: std::collections::HashSet<FieldCoordinate> = (0..=14)
            .map(|y| FieldCoordinate::new(if attacking_away_endzone { 25 } else { 0 }, y))
            .filter(|c| bounds.is_in_bounds(*c))
            .collect();
        PathFinderWithPassBlockSupport::new()
            .get_shortest_path_for_player(game, &end_coords, player, 0)
            .is_some()
    }

    /// Java `handleStaller(IStep, Player<?> player)`.
    ///
    /// Rolls a d6; if the result >= current turn number (and turn <= 6) the
    /// stalling player is hit by a rock (injury applied). Always marks the team
    /// result as stalled (reduces winnings by 1).
    ///
    /// The injury machinery (UtilServerInjury, InjuryTypeThrowARockStalling,
    /// SteadyFootingContext, DropPlayerCommand, Animation) is not yet translated,
    /// so the rock-hit branch is a TODO stub that never executes (conservative).
    pub fn handle_staller(
        &self,
        game: &mut Game,
        player_id: &str,
        turn_nr: i32,
        rng: &mut ffb_model::util::rng::GameRng,
    ) -> (ffb_model::events::GameEvent, Option<SteadyFootingContext>) {
        let roll: i32;
        let successful: bool;

        if turn_nr > 6 {
            roll = 0;
            successful = false;
        } else {
            roll = rng.die(6);
            successful = roll >= turn_nr;
        }

        // Mark team as having stalled (reduces winnings).
        let home_has_player = game.team_home.has_player(player_id);
        if home_has_player {
            game.game_result.home.stalled = true;
        } else {
            game.game_result.away.stalled = true;
        }

        let event = ffb_model::events::GameEvent::ThrowAtStallingPlayer {
            player_id: player_id.to_string(),
            roll,
            success: successful,
        };
        if !successful {
            return (event, None);
        }

        // Java, the `if (successful)` branch: pick the rock's starting square on whichever
        // touchline the player is nearer, then injure him with `InjuryTypeThrowARockStalling` and
        // hand the drop to the SteadyFooting pipeline.
        //
        // This was a stub whose note said the branch was "unreachable in headless (stalling
        // detection disabled)". That was true until ITER65 switched the rule on, and then the roll
        // happened with none of its consequences: Java rolled 12 dice at bb2025 seed 82 step 94 --
        // the staller d6, an x-coordinate, armour, injury, casualty -- where Rust rolled 4.
        let player_coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return (event, None),
        };
        // Java: `rollXCoordinate()` is `rollDice(26) - 1`, i.e. a d26 giving x in [0, 25] --
        // NOT a d24. (bb2020's twin rolls `die(24)` and is wrong about this; its branch does not
        // fire in the current gate, so it is left alone rather than fixed blind.)
        let x = rng.die(26) - 1;
        // Java: FieldCoordinateBounds.UPPER_HALF is (0,0)..(25,7).
        let _start_coord = if player_coord.y <= 7 {
            FieldCoordinate::new(x, 0)
        } else {
            FieldCoordinate::new(x, 14)
        };
        // The animation and syncGameModel that follow in Java are client-side only.

        let injury_result = handle_injury_by_name(
            game,
            rng,
            "InjuryTypeThrowARockStalling",
            None,
            player_id,
            player_coord,
            None,
            None,
            ApothecaryMode::HitPlayer,
        );
        let dpc = DropPlayerContext {
            injury_result: Some(Box::new(injury_result)),
            end_turn: false,
            eligible_for_safe_pair_of_hands: true,
            label: None,
            player_id: Some(player_id.to_string()),
            apothecary_mode: Some(ApothecaryMode::HitPlayer),
            requires_armour_break: false,
            ..DropPlayerContext::new()
        };
        (event, Some(SteadyFootingContext::from_drop_player(dpc)))
    }
}

impl Default for StallingExtension {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::{Rules, PS_STANDING};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::util::rng::GameRng;

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender, PlayerState};
        let p = Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 9, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: std::collections::HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        };
        if home { game.team_home.players.push(p); } else { game.team_away.players.push(p); }
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
    }

    #[test]
    fn is_considered_stalling_false_without_ball() {
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let ext = StallingExtension::new();
        assert!(!ext.is_considered_stalling(&game, "h1"));
    }

    #[test]
    fn is_considered_stalling_false_when_adjacent_opponent_with_tz() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 5);
        add_player(&mut game, true, "h1", coord);
        game.field_model.ball_in_play = true;
        game.field_model.ball_coordinate = Some(coord);
        // Place adjacent opponent with tacklezone (standing)
        add_player(&mut game, false, "a1", FieldCoordinate::new(6, 5));
        let ext = StallingExtension::new();
        assert!(!ext.is_considered_stalling(&game, "h1"));
    }

    #[test]
    fn handle_staller_marks_home_team_stalled() {
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let ext = StallingExtension::new();
        let mut rng = GameRng::new(42);
        let _ = ext.handle_staller(&mut game, "h1", 3, &mut rng);
        assert!(game.game_result.home.stalled);
        assert!(!game.game_result.away.stalled);
    }

    #[test]
    fn handle_staller_marks_away_team_stalled() {
        let mut game = make_game();
        add_player(&mut game, false, "a1", FieldCoordinate::new(5, 5));
        let ext = StallingExtension::new();
        let mut rng = GameRng::new(42);
        let _ = ext.handle_staller(&mut game, "a1", 3, &mut rng);
        assert!(game.game_result.away.stalled);
        assert!(!game.game_result.home.stalled);
    }

    #[test]
    fn handle_staller_after_turn_6_no_roll_needed() {
        // Turn 7+ always unsuccessful — just ensures no panic
        let mut game = make_game();
        add_player(&mut game, true, "h1", FieldCoordinate::new(5, 5));
        let ext = StallingExtension::new();
        let mut rng = GameRng::new(0);
        ext.handle_staller(&mut game, "h1", 7, &mut rng);
        assert!(game.game_result.home.stalled);
    }
}
