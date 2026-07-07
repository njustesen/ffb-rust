use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player};
use ffb_model::model::property::named_properties::NamedProperties;
use ffb_model::util::util_player::UtilPlayer;
use crate::mechanic::{Mechanic, MechanicType};
use crate::jump_mechanic::JumpMechanic as JumpMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.bb2025.JumpMechanic.
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
        let can_leap = player.has_skill_property(NamedProperties::CAN_LEAP);
        let has_prone_adjacent = coord.map(|c| self.has_prone_or_stunned_players_adjacent(game, c)).unwrap_or(false);
        (can_leap || has_prone_adjacent) && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn can_jump(&self, game: &Game, player: &Player, coordinate: FieldCoordinate) -> bool {
        let can_leap = player.has_skill_property(NamedProperties::CAN_LEAP);
        let has_prone_adjacent = self.has_prone_or_stunned_players_adjacent(game, coordinate);
        (can_leap || has_prone_adjacent) && !player.has_skill_property(NamedProperties::MOVES_RANDOMLY)
    }

    fn is_valid_jump(&self, game: &Game, player: &Player, from: FieldCoordinate, to: FieldCoordinate) -> bool {
        from != to
            && to.distance_in_steps(from) == 2
            && (player.has_skill_property(NamedProperties::CAN_LEAP) || {
                // TODO: PathFinderExtension::has_prone_or_stunned_player_on_path(game, from, to)
                let _ = game;
                false
            })
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, SkillId, Rules};
    use ffb_model::model::{Team, Player};
    use ffb_model::model::skill_def::SkillWithValue;
    use crate::jump_mechanic::JumpMechanic as JumpTrait;

    fn bare_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "human".into(), roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game() -> Game { Game::new(bare_team("home"), bare_team("away"), Rules::Bb2025) }

    fn add_player_with_leap(game: &mut Game, id: &str) {
        let p = Player {
            id: id.into(), name: id.into(), nr: 1, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        game.team_home.players.push(p);
        game.acting_player.player_id = Some(id.into());
    }

    #[test]
    fn can_still_jump_false_when_no_player() {
        let game = make_game();
        assert!(!JumpMechanic.can_still_jump(&game, &game.acting_player.clone()));
    }

    #[test]
    fn can_still_jump_true_with_leap_skill() {
        let mut game = make_game();
        add_player_with_leap(&mut game, "p1");
        assert!(JumpMechanic.can_still_jump(&game, &game.acting_player.clone()));
    }

    #[test]
    fn is_valid_jump_false_same_square() {
        let game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        let c = FieldCoordinate::new(5, 5);
        assert!(!JumpMechanic.is_valid_jump(&game, &p, c, c));
    }

    #[test]
    fn is_valid_jump_false_when_distance_not_2() {
        let game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        assert!(!JumpMechanic.is_valid_jump(&game, &p, FieldCoordinate::new(5, 5), FieldCoordinate::new(5, 8)));
    }

    #[test]
    fn is_valid_jump_true_with_leap_at_distance_2() {
        let game = make_game();
        let p = Player {
            id: "p1".into(), name: "p1".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 4, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![SkillWithValue::new(SkillId::Leap)],
            extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
};
        assert!(JumpMechanic.is_valid_jump(&game, &p, FieldCoordinate::new(5, 5), FieldCoordinate::new(7, 5)));
    }
}
