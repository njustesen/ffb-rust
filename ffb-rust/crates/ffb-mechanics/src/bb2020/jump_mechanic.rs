use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_cards::UtilCards;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::jump_mechanic::JumpMechanic as JumpMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2020.JumpMechanic.
pub struct JumpMechanic;

impl JumpMechanic {
    pub fn new() -> Self { Self }
}

impl Default for JumpMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for JumpMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::JUMP }
}

impl JumpMechanicTrait for JumpMechanic {
    fn is_available_as_next_move(&self, game: &Game, acting_player: &ActingPlayer, jumping: bool) -> bool {
        self.can_still_jump(game, acting_player) && UtilPlayer::is_next_move_possible(game, jumping)
    }

    fn can_still_jump(&self, game: &Game, acting_player: &ActingPlayer) -> bool {
        let player_id = match &acting_player.player_id {
            Some(id) => id,
            None => return false,
        };
        let player = match game.player(player_id) {
            Some(p) => p,
            None => return false,
        };
        let coord = game.field_model.player_coordinate(player_id);
        let has_unused_leap = UtilCards::has_unused_skill_with_property(player, NamedProperties::CAN_LEAP);
        // BB2020: also can jump if not yet jumped AND there's a prone/stunned adjacent player
        let has_prone_adjacent = coord.map(|c| self.has_prone_or_stunned_players_adjacent(game, c)).unwrap_or(false);
        let can_leap_this_turn = has_unused_leap || (!acting_player.jumping && has_prone_adjacent);
        can_leap_this_turn && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn can_jump(&self, game: &Game, player: &Player, coordinate: FieldCoordinate) -> bool {
        let can_leap = player.has_skill_property(NamedProperties::CAN_LEAP);
        let has_prone_adjacent = self.has_prone_or_stunned_players_adjacent(game, coordinate);
        (can_leap || has_prone_adjacent) && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn is_valid_jump(&self, game: &Game, player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        from != to
            && to.distance_in_steps(from) == 2
            && (player.has_skill_property(NamedProperties::CAN_LEAP)
                || self.has_prone_or_stunned_player_on_path(game, from, to))
    }
}

impl JumpMechanic {
    fn has_prone_or_stunned_players_adjacent(&self, game: &Game, coordinate: FieldCoordinate) -> bool {
        game.field_model.adjacent_on_pitch(coordinate).iter().any(|&adj| {
            if let Some(id) = game.field_model.player_at(adj) {
                if let Some(state) = game.field_model.player_state(id) {
                    return state.is_prone_or_stunned() || state.is_stunned();
                }
            }
            false
        })
    }

    fn has_prone_or_stunned_player_on_path(&self, game: &Game, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        self.find_possible_path_squares(from, to).iter().any(|&sq| {
            if let Some(id) = game.field_model.player_at(sq) {
                if let Some(state) = game.field_model.player_state(id) {
                    return state.is_stunned() || state.is_prone_or_stunned();
                }
            }
            false
        })
    }

    fn find_possible_path_squares(&self, from: FieldCoordinate, to: FieldCoordinate) -> Vec<FieldCoordinate> {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let x_variances = Self::dimension_variance(dx);
        let y_variances = Self::dimension_variance(dy);
        let mut coords = Vec::new();
        for &xv in &x_variances {
            for &yv in &y_variances {
                let nx = from.x + xv;
                let ny = from.y + yv;
                if nx >= 0 && nx <= 25 && ny >= 0 && ny <= 14 {
                    coords.push(FieldCoordinate { x: nx, y: ny });
                }
            }
        }
        coords
    }

    fn dimension_variance(diff: i32) -> Vec<i32> {
        match diff.abs() {
            2 => vec![diff / 2],
            1 => vec![diff, 0],
            0 => vec![0],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerState, PS_PRONE, Rules};
    use ffb_model::model::{Game, Team};
    use ffb_model::model::skill_def::SkillWithValue;
    use ffb_model::enums::SkillId;

    fn empty_team() -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
        }
    }

    fn minimal_game() -> Game { Game::new(empty_team(), empty_team(), Rules::Bb2025) }
    fn bare_player() -> Player { Player::default() }

    #[test]
    fn is_valid_jump_false_same_square() {
        let game = minimal_game();
        let player = bare_player();
        let m = JumpMechanic::new();
        let c = FieldCoordinate::new(5, 5);
        assert!(!m.is_valid_jump(&game, &player, c, c));
    }

    #[test]
    fn is_valid_jump_false_distance_not_2() {
        let game = minimal_game();
        let player = bare_player();
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(5, 6);
        assert!(!m.is_valid_jump(&game, &player, from, to));
    }

    #[test]
    fn is_valid_jump_true_with_leap_skill() {
        let game = minimal_game();
        let mut player = bare_player();
        player.starting_skills.push(SkillWithValue::new(SkillId::Leap));
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        assert!(m.is_valid_jump(&game, &player, from, to));
    }

    #[test]
    fn is_valid_jump_false_no_leap_no_prone_on_path() {
        let game = minimal_game();
        let player = bare_player();
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        assert!(!m.is_valid_jump(&game, &player, from, to));
    }

    #[test]
    fn is_valid_jump_true_when_prone_player_on_path() {
        let mut game = minimal_game();
        let player = bare_player();
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        // Path square: (6, 5)
        game.field_model.set_player_coordinate("prone1", FieldCoordinate::new(6, 5));
        game.field_model.set_player_state("prone1", PlayerState::new(PS_PRONE));
        assert!(m.is_valid_jump(&game, &player, from, to));
    }

    #[test]
    fn path_squares_straight_jump() {
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 5);
        let squares = m.find_possible_path_squares(from, to);
        assert_eq!(squares, vec![FieldCoordinate::new(6, 5)]);
    }

    #[test]
    fn path_squares_knights_move() {
        let m = JumpMechanic::new();
        let from = FieldCoordinate::new(5, 5);
        let to = FieldCoordinate::new(7, 6);
        let mut squares = m.find_possible_path_squares(from, to);
        squares.sort_by_key(|c| (c.x, c.y));
        assert_eq!(squares, vec![FieldCoordinate::new(6, 5), FieldCoordinate::new(6, 6)]);
    }
}
