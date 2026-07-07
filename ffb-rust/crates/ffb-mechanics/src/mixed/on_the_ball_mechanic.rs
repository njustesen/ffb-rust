use std::collections::HashSet;
use ffb_model::types::FieldCoordinate;
use ffb_model::model::{ActingPlayer, Game, Player, Team};
use ffb_model::model::property::named_properties::NamedProperties;
use crate::mechanic::{Mechanic, MechanicType};
use crate::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallMechanicTrait;

/// 1:1 translation of com.fumbbl.ffb.mechanics.mixed.OnTheBallMechanic.
pub struct OnTheBallMechanic;

impl OnTheBallMechanic {
    pub fn new() -> Self { Self }
}

impl Default for OnTheBallMechanic {
    fn default() -> Self { Self::new() }
}

impl Mechanic for OnTheBallMechanic {
    fn get_type(&self) -> MechanicType { MechanicType::ON_THE_BALL }
}

impl OnTheBallMechanicTrait for OnTheBallMechanic {
    fn find_pass_blockers(&self, game: &Game, team: &Team, _check_can_reach: bool) -> HashSet<String> {
        let mut pass_blockers = HashSet::new();
        for player in &team.players {
            let player_state = game.field_model.player_state(&player.id).unwrap_or_default();
            if player.has_skill_property(NamedProperties::CAN_MOVE_WHEN_OPPONENT_PASSES) && player_state.has_tacklezones() {
                pass_blockers.insert(player.id.clone());
            }
        }
        pass_blockers
    }

    fn valid_pass_block_move(&self, _game: &Game, acting_player: &ActingPlayer, _from_coordinate: FieldCoordinate, _to_coordinate: FieldCoordinate, _valid_pass_block_coordinates: &HashSet<FieldCoordinate>, _can_still_jump: bool, distance: i32) -> bool {
        distance + acting_player.current_move <= 3
    }

    fn display_string_pass_interference(&self) -> String {
        "On The Ball".to_string()
    }

    fn pass_interference_dialog_description(&self) -> Vec<String> {
        vec!["You may move your players with ON THE BALL skill up to 3 squares.".to_string()]
    }

    fn pass_interference_status_description(&self) -> String {
        "Waiting for coach to move players with \"On The Ball\".".to_string()
    }

    fn display_string_kick_off_interference(&self) -> String {
        self.display_string_pass_interference()
    }

    fn has_reached_valid_position(&self, _game: &Game, _player: &Player) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::on_the_ball_mechanic::OnTheBallMechanic as OnTheBallTrait;

    #[test]
    fn display_string_is_on_the_ball() {
        assert_eq!(OnTheBallMechanic.display_string_pass_interference(), "On The Ball");
    }

    #[test]
    fn kick_off_interference_matches_pass_interference() {
        assert_eq!(
            OnTheBallMechanic.display_string_kick_off_interference(),
            OnTheBallMechanic.display_string_pass_interference()
        );
    }

    #[test]
    fn dialog_description_has_one_entry() {
        assert_eq!(OnTheBallMechanic.pass_interference_dialog_description().len(), 1);
    }

    #[test]
    fn pass_interference_status_description_is_non_empty() {
        let s = OnTheBallMechanic.pass_interference_status_description();
        assert!(!s.is_empty());
        assert!(s.contains("On The Ball") || s.contains("on the ball") || s.contains("coach"));
    }

    #[test]
    fn mechanic_type_is_on_the_ball() {
        assert_eq!(crate::mechanic::Mechanic::get_type(&OnTheBallMechanic), MechanicType::ON_THE_BALL);
    }

    #[test]
    fn valid_pass_block_move_within_3_squares_is_valid() {
        use ffb_model::model::ActingPlayer;
        use ffb_model::types::FieldCoordinate;
        use std::collections::HashSet;
        use ffb_model::enums::Rules;
        use ffb_model::model::{Game, Team};
        let home = Team { id: "h".into(), name: "h".into(), race: "h".into(), roster_id: "h".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false };
        let game = Game::new(home.clone(), home, Rules::Bb2016);
        let acting_player = ActingPlayer { current_move: 0, ..Default::default() };
        let valid = OnTheBallMechanic.valid_pass_block_move(
            &game, &acting_player,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5),
            &HashSet::new(), false, 3
        );
        assert!(valid); // 3 + 0 <= 3
    }

    #[test]
    fn valid_pass_block_move_exceeding_3_squares_is_invalid() {
        use ffb_model::model::ActingPlayer;
        use ffb_model::types::FieldCoordinate;
        use std::collections::HashSet;
        use ffb_model::enums::Rules;
        use ffb_model::model::{Game, Team};
        let home = Team { id: "h".into(), name: "h".into(), race: "h".into(), roster_id: "h".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0, prayers_to_nuffle: 0,
            bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0, assistant_coaches: 0,
            fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false };
        let game = Game::new(home.clone(), home, Rules::Bb2016);
        let acting_player = ActingPlayer { current_move: 1, ..Default::default() };
        let valid = OnTheBallMechanic.valid_pass_block_move(
            &game, &acting_player,
            FieldCoordinate::new(5, 5), FieldCoordinate::new(6, 5),
            &HashSet::new(), false, 3
        );
        assert!(!valid); // 3 + 1 > 3
    }
}
