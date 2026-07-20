/// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerPlayerMove.
///
/// Public methods:
///   - isValidMove (two overloads: ClientCommandMove / ClientCommandBlitzMove)
///   - updateMoveSquares(GameState, boolean jumping)
///   - fetchMoveStack (two overloads)
///   - fetchFromSquare (two overloads)
///
/// updateMoveSquares/addMoveSquare are fully ported: movesRandomly (Ball & Chain),
/// PASS_BLOCK (OnTheBallMechanic.validPassBlockMove), KICKOFF_RETURN (own-half bounds),
/// jump validity (JumpMechanic.isValidJump), dodging (ignoreTacklezonesWhenMoving +
/// UtilPlayer.findTacklezones) with DodgeModifierFactory minimum roll, jump minimum
/// roll via JumpModifierFactory, and GFI minimum roll via GoForItModifierFactory.
/// no-op: actingPlayer.isJumpsWithoutModifiers() (Java) has no Rust ActingPlayer field
///        yet — jump modifiers are always applied.
use ffb_model::enums::TurnMode;
use ffb_model::model::game::Game;
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::types::{FieldCoordinate, FieldCoordinateBounds, MoveSquare};
use ffb_model::util::util_player::UtilPlayer;
use ffb_mechanics::jump_mechanic::JumpMechanic as _;
use ffb_mechanics::on_the_ball_mechanic::OnTheBallMechanic as _;
use ffb_mechanics::modifiers::dodge_context::DodgeContext;
use ffb_mechanics::modifiers::dodge_modifier::DodgeModifier;
use ffb_mechanics::modifiers::dodge_modifier_factory::DodgeModifierFactory;
use ffb_mechanics::modifiers::go_for_it_context::GoForItContext;
use ffb_mechanics::modifiers::go_for_it_modifier::GoForItModifier;
use ffb_mechanics::modifiers::go_for_it_modifier_factory::GoForItModifierFactory;
use ffb_mechanics::modifiers::jump_context::JumpContext;
use ffb_mechanics::modifiers::jump_modifier_factory::JumpModifierFactory;

pub struct UtilServerPlayerMove;

impl UtilServerPlayerMove {
    /// Java: UtilServerPlayerMove.fetchMoveStack(FieldCoordinate[], boolean homeCommand)
    ///
    /// Transforms coordinates from the away perspective to the home perspective
    /// when homeCommand is false.
    pub fn fetch_move_stack(coordinates_to: &[FieldCoordinate], home_command: bool) -> Vec<FieldCoordinate> {
        if coordinates_to.is_empty() {
            return vec![];
        }
        if home_command {
            coordinates_to.to_vec()
        } else {
            coordinates_to.iter().map(|c| c.transform()).collect()
        }
    }

    /// Java: UtilServerPlayerMove.fetchFromSquare(FieldCoordinate, boolean homeCommand)
    ///
    /// Transforms the from-coordinate from the away perspective when homeCommand is false.
    pub fn fetch_from_square(from: FieldCoordinate, home_command: bool) -> FieldCoordinate {
        if home_command { from } else { from.transform() }
    }

    /// Java: UtilServerPlayerMove.isValidMove(GameState, coordinateFrom, homeCommand)
    ///
    /// Checks that the acting player is at coordinateFrom. Drops the command if not
    /// (command out-of-sync). In this headless port we simply check the field model.
    pub fn is_valid_move(game: &Game, coordinate_from: FieldCoordinate, _home_command: bool) -> bool {
        let acting_id = match &game.acting_player.player_id {
            Some(id) => id,
            None => return false,
        };
        match game.field_model.player_coordinate(acting_id) {
            Some(player_coord) => player_coord == coordinate_from,
            None => false,
        }
    }

    /// Java: UtilServerPlayerMove.updateMoveSquares(GameState, boolean jumping)
    ///
    /// Recomputes which squares the acting player can legally move to and stores
    /// them in FieldModel.move_squares — including per-square minimum dodge and
    /// GFI rolls (via addMoveSquare), which StepInitMoving reads to set
    /// actingPlayer.dodging / goesForIt.
    pub fn update_move_squares(game: &mut Game, jumping: bool) {
        // Java: if (actingPlayer.getPlayer() != null) { fieldModel.clearMoveSquares(); ... }
        let acting_id = match game.acting_player.player_id.clone() {
            Some(id) => id,
            None => return,
        };
        game.field_model.move_squares.clear();

        let player_coord = match game.field_model.player_coordinate(&acting_id) {
            Some(c) => c,
            None => return,
        };

        let action_is_moving = game.acting_player.player_action
            .map(|a| a.is_moving())
            .unwrap_or(false);

        // Java: actingPlayer.getPlayerAction().isMoving()
        //       && UtilPlayer.isNextMovePossible(game, jumping)
        //       && FieldCoordinateBounds.FIELD.isInBounds(playerCoordinate)
        if !action_is_moving
            || !UtilPlayer::is_next_move_possible(game, jumping)
            || !FieldCoordinateBounds::FIELD.is_in_bounds(player_coord)
        {
            return;
        }

        // Java: if (actingPlayer.getPlayer().hasSkillProperty(NamedProperties.movesRandomly))
        let moves_randomly = game.player(&acting_id)
            .map(|p| p.has_skill_property(NamedProperties::MOVES_RANDOMLY))
            .unwrap_or(false);
        if moves_randomly {
            for x in [-1i32, 1] {
                let move_coordinate = player_coord.add(x, 0);
                if FieldCoordinateBounds::FIELD.is_in_bounds(move_coordinate) {
                    Self::add_move_square(game, jumping, move_coordinate);
                }
            }
            for y in [-1i32, 1] {
                let move_coordinate = player_coord.add(0, y);
                if FieldCoordinateBounds::FIELD.is_in_bounds(move_coordinate) {
                    Self::add_move_square(game, jumping, move_coordinate);
                }
            }
            return;
        }

        // Java: steps=2 for jump, 1 otherwise. findAdjacentCoordinates(playerCoordinate, FIELD, steps, false)
        let adjacent: Vec<FieldCoordinate> = if jumping {
            let mut coords = Vec::new();
            for dx in -2i32..=2 {
                for dy in -2i32..=2 {
                    if dx == 0 && dy == 0 { continue; }
                    let c = player_coord.add(dx, dy);
                    if FieldCoordinateBounds::FIELD.is_in_bounds(c) { coords.push(c); }
                }
            }
            coords
        } else {
            game.field_model.adjacent_on_pitch(player_coord)
        };

        // Java: validPassBlockCoordinates / canStillJump / onTheBallMechanic (computed before the loop)
        let valid_pass_block_coordinates = ffb_model::util::passing::find_valid_pass_block_end_coordinates(game);
        let jump_mechanic = crate::mechanic::jump_mechanic_for(game.rules);
        let can_still_jump = jump_mechanic.can_still_jump(game, &game.acting_player);
        let on_the_ball_mechanic = crate::mechanic::on_the_ball_mechanic_for(game.rules);

        for coordinate in adjacent {
            if game.field_model.player_at(coordinate).is_some() { continue; }
            match game.turn_mode {
                // Java: TurnMode.PASS_BLOCK → onTheBallMechanic.validPassBlockMove(...)
                TurnMode::PassBlock => {
                    let distance = coordinate.distance_in_steps(player_coord);
                    let acting = game.acting_player.clone();
                    if on_the_ball_mechanic.valid_pass_block_move(
                        game, &acting, player_coord, coordinate,
                        &valid_pass_block_coordinates, can_still_jump, distance,
                    ) {
                        Self::add_move_square(game, jumping, coordinate);
                    }
                }
                // Java: TurnMode.KICKOFF_RETURN → own-half bounds check
                TurnMode::KickoffReturn => {
                    let bounds = if game.home_playing {
                        FieldCoordinateBounds::HALF_HOME
                    } else {
                        FieldCoordinateBounds::HALF_AWAY
                    };
                    if bounds.is_in_bounds(coordinate) {
                        Self::add_move_square(game, jumping, coordinate);
                    }
                }
                _ => Self::add_move_square(game, jumping, coordinate),
            }
        }
    }

    /// Java: UtilServerPlayerMove.addMoveSquare(GameState, boolean jumping, FieldCoordinate)
    ///
    /// Computes the square's minimum dodge roll (0 = no dodge needed) and minimum
    /// GFI roll (0 = no GFI needed) and adds the MoveSquare to the field model.
    fn add_move_square(game: &mut Game, jumping: bool, coordinate: FieldCoordinate) {
        let Some(acting_id) = game.acting_player.player_id.clone() else { return; };
        let Some(player_coord) = game.field_model.player_coordinate(&acting_id) else { return; };

        // Java: if (jumping && !jumpMechanic.isValidJump(game, player, playerCoordinate, coordinate)) return;
        if jumping {
            let mechanic = crate::mechanic::jump_mechanic_for(game.rules);
            let valid = game.player(&acting_id)
                .map(|p| mechanic.is_valid_jump(game, p, player_coord, coordinate))
                .unwrap_or(false);
            if !valid { return; }
        }

        // Java: dodging = !actingPlayer.getPlayer().hasSkillProperty(ignoreTacklezonesWhenMoving)
        //                 && (UtilPlayer.findTacklezones(game, actingPlayer.getPlayer()) > 0)
        let dodging = game.player(&acting_id)
            .map(|p| !p.has_skill_property(NamedProperties::IGNORE_TACKLEZONES_WHEN_MOVING))
            .unwrap_or(false)
            && UtilPlayer::find_tacklezones(game, &acting_id) > 0;

        let go_for_it: bool;
        let mut minimum_roll_dodge = 0i32;
        if jumping {
            // Java: minimumRollDodge = mechanic.minimumRollJump(player, jumpModifiers)
            // no-op: actingPlayer.isJumpsWithoutModifiers() not ported — modifiers always applied
            if let Some(player) = game.player(&acting_id) {
                let ctx = JumpContext::new(game, player, player_coord, coordinate);
                let factory = JumpModifierFactory::for_rules(game.rules);
                let mods = factory.find_applicable(&ctx);
                let accumulated: i32 = mods.iter().map(|m| m.get_modifier()).sum();
                let count = mods.len() as i32;
                let skill_mods = factory.find_skill_modifiers(&ctx, accumulated, count);
                let total: i32 = mods.iter().map(|m| m.get_modifier())
                    .chain(skill_mods.iter().map(|m| m.get_modifier()))
                    .sum();
                minimum_roll_dodge = (player.agility_with_modifiers() + total).max(2);
            }
            // Java: goForIt for jump — standing-up players without canStandUpForFree
            //       compare 3 + distance, others currentMove + distance, vs movement.
            let (movement, can_stand_free) = game.player(&acting_id)
                .map(|p| (p.movement_with_modifiers(), p.has_skill_property(NamedProperties::CAN_STAND_UP_FOR_FREE)))
                .unwrap_or((0, false));
            let distance = player_coord.distance_in_steps(coordinate);
            let ap = &game.acting_player;
            go_for_it = if ap.standing_up && !ap.has_acted && !can_stand_free {
                (3 + distance) > movement
            } else {
                (ap.current_move + distance) > movement
            };
        } else {
            // Java: goForIt = UtilPlayer.isNextMoveGoingForIt(game)
            go_for_it = UtilPlayer::is_next_move_going_for_it(game);
            if dodging {
                // Java: DodgeModifierFactory.findModifiers(DodgeContext) →
                //       mechanic.minimumRollDodge(game, player, dodgeModifiers)
                let acting = game.acting_player.clone();
                let ctx = DodgeContext::new(game, &acting, player_coord, coordinate);
                let factory = DodgeModifierFactory::for_rules(game.rules);
                let mods = factory.find_applicable(&ctx);
                let skill_mods = factory.find_skill_modifiers(&ctx);
                let all: Vec<&DodgeModifier> = mods.iter().copied().chain(skill_mods.iter()).collect();
                let agility = game.player(&acting_id).map(|p| p.agility as i32).unwrap_or(3);
                minimum_roll_dodge = DodgeModifierFactory::minimum_roll(agility, &all);
            }
        }

        // Java: if (goForIt) minimumRollGoForIt = DiceInterpreter.minimumRollGoingForIt(goForItModifiers)
        let mut minimum_roll_gfi = 0i32;
        if go_for_it {
            let factory = GoForItModifierFactory::for_rules(game.rules);
            minimum_roll_gfi = if let Some(player) = game.player(&acting_id) {
                let ctx = GoForItContext::new(game, player);
                let mods = factory.find_applicable(&ctx);
                let card_mods = factory.find_card_modifiers(&ctx);
                let all: Vec<&GoForItModifier> = mods.iter().copied().chain(card_mods.iter()).collect();
                GoForItModifierFactory::minimum_roll_going_for_it(&all)
            } else {
                2
            };
        }

        game.field_model.add_move_square(MoveSquare::new(coordinate, minimum_roll_dodge, minimum_roll_gfi));
    }
}

impl Default for UtilServerPlayerMove {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use ffb_model::enums::{Rules, PlayerAction, PlayerState, PS_STANDING};
    use ffb_model::types::FieldCoordinate;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    /// Adds a real Player to a team (required since update_move_squares now runs the
    /// Java-faithful UtilPlayer::is_next_move_possible / find_tacklezones guards).
    fn add_player(game: &mut Game, home: bool, id: &str, coord: FieldCoordinate) {
        use ffb_model::model::player::Player;
        use ffb_model::enums::{PlayerType, PlayerGender};
        let team = if home { &mut game.team_home } else { &mut game.team_away };
        team.players.push(Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 9,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        });
        game.field_model.set_player_coordinate(id, coord);
        game.field_model.set_player_state(id, PlayerState(PS_STANDING).change_active(true));
    }

    // -- fetch_move_stack --

    #[test]
    fn fetch_move_stack_home_command_unchanged() {
        let coords = vec![FieldCoordinate::new(5, 7), FieldCoordinate::new(6, 7)];
        let result = UtilServerPlayerMove::fetch_move_stack(&coords, true);
        assert_eq!(result, coords);
    }

    #[test]
    fn fetch_move_stack_away_command_transforms() {
        let c = FieldCoordinate::new(10, 7);
        let result = UtilServerPlayerMove::fetch_move_stack(&[c], false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], c.transform());
    }

    #[test]
    fn fetch_move_stack_empty_input() {
        let result = UtilServerPlayerMove::fetch_move_stack(&[], true);
        assert!(result.is_empty());
    }

    // -- fetch_from_square --

    #[test]
    fn fetch_from_square_home_unchanged() {
        let c = FieldCoordinate::new(8, 5);
        assert_eq!(UtilServerPlayerMove::fetch_from_square(c, true), c);
    }

    #[test]
    fn fetch_from_square_away_transforms() {
        let c = FieldCoordinate::new(10, 7);
        assert_eq!(UtilServerPlayerMove::fetch_from_square(c, false), c.transform());
    }

    // -- is_valid_move --

    #[test]
    fn is_valid_move_true_when_player_at_coord() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        assert!(UtilServerPlayerMove::is_valid_move(&game, coord, true));
    }

    #[test]
    fn is_valid_move_false_when_player_elsewhere() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        let other = FieldCoordinate::new(6, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        assert!(!UtilServerPlayerMove::is_valid_move(&game, other, true));
    }

    #[test]
    fn is_valid_move_false_when_no_acting_player() {
        let game = make_game();
        let coord = FieldCoordinate::new(5, 7);
        assert!(!UtilServerPlayerMove::is_valid_move(&game, coord, true));
    }

    // -- update_move_squares --

    #[test]
    fn update_move_squares_no_acting_player_is_noop() {
        let mut game = make_game();
        game.acting_player.player_id = None;
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_not_moving_action_is_noop() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("p1", coord);
        game.field_model.set_player_state("p1", PlayerState(PS_STANDING));
        game.acting_player.player_action = Some(PlayerAction::Block);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_move_action_populates_squares() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        add_player(&mut game, true, "p1", coord);
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(!game.field_model.move_squares.is_empty());
    }

    #[test]
    fn update_move_squares_occupied_squares_excluded() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        let blocker_coord = FieldCoordinate::new(11, 7);
        add_player(&mut game, true, "p1", coord);
        game.acting_player.player_id = Some("p1".to_string());
        game.field_model.set_player_coordinate("blocker", blocker_coord);
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        assert!(game.field_model.get_move_square(blocker_coord).is_none());
    }

    #[test]
    fn update_move_squares_clears_previous_squares() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        add_player(&mut game, true, "p1", coord);
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        // Add a stale square.
        game.field_model.add_move_square(MoveSquare::new(FieldCoordinate::new(0, 0), 0, 0));
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        // Stale (0,0) is not adjacent to (10,7), so it should be gone.
        assert!(game.field_model.get_move_square(FieldCoordinate::new(0, 0)).is_none());
    }

    #[test]
    fn update_move_squares_no_adjacent_opponent_squares_not_dodging() {
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        add_player(&mut game, true, "p1", coord);
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        let square = game.field_model.get_move_square(FieldCoordinate::new(11, 7)).unwrap();
        assert!(!square.is_dodging(), "no tackle zones → no dodge required");
    }

    #[test]
    fn update_move_squares_leaving_tacklezone_marks_squares_dodging() {
        // Java: addMoveSquare — dodging = findTacklezones(actingPlayer) > 0, i.e. leaving a
        // square adjacent to a standing opponent requires a dodge to EVERY destination.
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        add_player(&mut game, true, "p1", coord);
        add_player(&mut game, false, "opp1", FieldCoordinate::new(11, 7));
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        let away_square = game.field_model.get_move_square(FieldCoordinate::new(9, 7)).unwrap();
        assert!(away_square.is_dodging(), "leaving a tackle zone requires a dodge");
        assert!(away_square.minimum_roll_dodge >= 2, "minimum dodge roll must be at least 2");
    }

    #[test]
    fn update_move_squares_gfi_square_has_minimum_roll() {
        // current_move == movement → next move is a GFI (minimum roll 2, no dodging)
        let mut game = make_game();
        let coord = FieldCoordinate::new(10, 7);
        add_player(&mut game, true, "p1", coord);
        game.acting_player.player_id = Some("p1".to_string());
        game.acting_player.player_action = Some(PlayerAction::Move);
        game.acting_player.current_move = 6; // movement is 6
        game.acting_player.goes_for_it = true; // grants +2 move in has_move_left
        UtilServerPlayerMove::update_move_squares(&mut game, false);
        let square = game.field_model.get_move_square(FieldCoordinate::new(11, 7)).unwrap();
        assert!(square.is_going_for_it(), "square beyond MA must be a GFI square");
        assert_eq!(square.minimum_roll_gfi, 2);
    }
}
