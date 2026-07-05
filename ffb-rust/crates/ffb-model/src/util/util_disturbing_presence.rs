/// 1:1 translation of com.fumbbl.ffb.util.UtilDisturbingPresence.
use crate::model::Game;
use crate::model::property::named_properties::NamedProperties;
use crate::types::FieldCoordinateBounds;
use crate::util::util_player::UtilPlayer;

pub struct UtilDisturbingPresence;

impl UtilDisturbingPresence {
    /// Count opposing players with INFLICTS_DISTURBING_PRESENCE within 3 steps of the given player.
    pub fn find_opposing_disturbing_presences(game: &Game, player_id: &str) -> i32 {
        let player_coord = match game.field_model.player_coordinate(player_id) {
            Some(c) => c,
            None => return 0,
        };
        let other_team = UtilPlayer::find_other_team(game, player_id);
        let mut count = 0i32;
        for opp in &other_team.players {
            if let Some(coord) = game.field_model.player_coordinate(&opp.id) {
                if FieldCoordinateBounds::FIELD.is_in_bounds(coord)
                    && opp.has_skill_property(NamedProperties::INFLICTS_DISTURBING_PRESENCE)
                    && player_coord.distance_in_steps(coord) <= 3
                {
                    count += 1;
                }
            }
        }
        count
    }
}

impl Default for UtilDisturbingPresence {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{Rules, SkillId, PS_STANDING, PlayerState, PlayerType, PlayerGender};
    use crate::model::{Game, Player, Team, SkillWithValue};
    use crate::types::FieldCoordinate;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(),
            roster_id: "human".into(), coach: "c".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0,
            dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![],
        }
    }

    fn minimal_player(id: &str) -> Player {
        Player {
            id: id.into(), name: id.into(), nr: 1,
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Default::default(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn no_opposing_players_returns_zero() {
        let mut g = make_game();
        g.team_home.players.push(minimal_player("h1"));
        g.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        assert_eq!(UtilDisturbingPresence::find_opposing_disturbing_presences(&g, "h1"), 0);
    }

    #[test]
    fn opposing_player_without_skill_not_counted() {
        let mut g = make_game();
        g.team_home.players.push(minimal_player("h1"));
        g.team_away.players.push(minimal_player("a1"));
        g.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_coordinate("a1", FieldCoordinate::new(6, 5));
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        g.field_model.set_player_state("a1", PlayerState(PS_STANDING));
        assert_eq!(UtilDisturbingPresence::find_opposing_disturbing_presences(&g, "h1"), 0);
    }

    #[test]
    fn opposing_player_with_skill_within_3_steps_counted() {
        let mut g = make_game();
        g.team_home.players.push(minimal_player("h1"));
        let mut a1 = minimal_player("a1");
        a1.starting_skills.push(SkillWithValue { skill_id: SkillId::DisturbingPresence, value: None });
        g.team_away.players.push(a1);
        g.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_coordinate("a1", FieldCoordinate::new(7, 5)); // 2 steps away
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        g.field_model.set_player_state("a1", PlayerState(PS_STANDING));
        assert_eq!(UtilDisturbingPresence::find_opposing_disturbing_presences(&g, "h1"), 1);
    }

    #[test]
    fn opposing_player_with_skill_beyond_3_steps_not_counted() {
        let mut g = make_game();
        g.team_home.players.push(minimal_player("h1"));
        let mut a1 = minimal_player("a1");
        a1.starting_skills.push(SkillWithValue { skill_id: SkillId::DisturbingPresence, value: None });
        g.team_away.players.push(a1);
        g.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        g.field_model.set_player_coordinate("a1", FieldCoordinate::new(10, 5)); // 5 steps away
        g.field_model.set_player_state("h1", PlayerState(PS_STANDING));
        g.field_model.set_player_state("a1", PlayerState(PS_STANDING));
        assert_eq!(UtilDisturbingPresence::find_opposing_disturbing_presences(&g, "h1"), 0);
    }
}
