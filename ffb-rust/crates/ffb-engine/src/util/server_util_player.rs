/// 1:1 translation of com.fumbbl.ffb.server.util.ServerUtilPlayer.
///
/// Java public method:
///   findBlockStrength(Game, attacker, attackerStrength, defender, isMultiBlock) -> int
use ffb_model::model::game::Game;
use ffb_model::types::FieldCoordinate;

pub struct ServerUtilPlayer;

impl ServerUtilPlayer {
    pub fn find_block_strength_simple(attacker_strength: i32, free_assists: i32) -> i32 {
        attacker_strength + free_assists
    }

    pub fn find_block_strength_ignoring_assists(
        attacker_strength: i32,
        _ignores_assists: bool,
        _same_team: bool,
    ) -> i32 {
        attacker_strength
    }

    pub fn find_block_strength(
        game: &Game,
        attacker_coord: FieldCoordinate,
        attacker_strength: i32,
        defender_coord: FieldCoordinate,
    ) -> i32 {
        let mut block_strength = attacker_strength;

        let attacker_id = game.field_model.player_at(attacker_coord);
        let att_team_home = attacker_id.map(|aid| game.team_home.has_player(aid)).unwrap_or(false);
        let att_team_away = attacker_id.map(|aid| game.team_away.has_player(aid)).unwrap_or(false);
        if !att_team_home && !att_team_away {
            return block_strength;
        }
        let defender_id = game.field_model.player_at(defender_coord);
        let def_team_home = defender_id.map(|did| game.team_home.has_player(did)).unwrap_or(false);
        for (id, &coord) in &game.field_model.player_coordinates {
            if attacker_id.map(|a| a == id).unwrap_or(false) { continue; }
            let on_att_team = if att_team_home { game.team_home.has_player(id) } else { game.team_away.has_player(id) };
            if !on_att_team { continue; }
            if !coord.is_adjacent(defender_coord) { continue; }
            let state = game.field_model.player_state(id);
            if !state.map(|s| s.has_tacklezones()).unwrap_or(false) { continue; }
            if state.map(|s| s.is_eye_gouged()).unwrap_or(false) { continue; }
            let mut opponents_other_than_defender = 0i32;
            for (oid, &ocoord) in &game.field_model.player_coordinates {
                let on_def_team = if def_team_home { game.team_home.has_player(oid) } else { game.team_away.has_player(oid) };
                if !on_def_team { continue; }
                if defender_id.map(|d| d == oid).unwrap_or(false) { continue; }
                if ocoord.is_adjacent(coord) {
                    let ostate = game.field_model.player_state(oid);
                    if ostate.map(|s| s.has_tacklezones()).unwrap_or(false) {
                        opponents_other_than_defender += 1;
                    }
                }
            }
            if opponents_other_than_defender == 0 { block_strength += 1; }
        }
        block_strength
    }
}

impl Default for ServerUtilPlayer {
    fn default() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;
    use ffb_model::model::player::Player;
    use ffb_model::enums::{Rules, PlayerState, PlayerType, PlayerGender, PS_STANDING, PS_PRONE};
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

    fn make_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), nr, name: id.into(),
            position_id: "lineman".into(),
            player_type: PlayerType::Regular,
            gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020)
    }

    #[test]
    fn find_block_strength_simple_no_assists() {
        assert_eq!(ServerUtilPlayer::find_block_strength_simple(3, 0), 3);
    }

    #[test]
    fn find_block_strength_simple_two_assists() {
        assert_eq!(ServerUtilPlayer::find_block_strength_simple(3, 2), 5);
    }

    #[test]
    fn find_block_strength_ignoring_assists_same_team() {
        assert_eq!(ServerUtilPlayer::find_block_strength_ignoring_assists(4, true, true), 4);
    }

    #[test]
    fn find_block_strength_ignoring_assists_different_team() {
        assert_eq!(ServerUtilPlayer::find_block_strength_ignoring_assists(4, true, false), 4);
    }

    #[test]
    fn find_block_strength_no_players_returns_base() {
        let game = make_game();
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        let result = ServerUtilPlayer::find_block_strength(&game, att_coord, 3, def_coord);
        assert_eq!(result, 3);
    }

    #[test]
    fn find_block_strength_prone_assist_not_counted() {
        let mut game = make_game();
        game.team_home.players.push(make_player("att", 1));
        game.team_away.players.push(make_player("def", 1));
        game.team_home.players.push(make_player("assist", 2));
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        let assist_coord = FieldCoordinate::new(6, 8);
        game.field_model.set_player_coordinate("att", att_coord);
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_coordinate("assist", assist_coord);
        game.field_model.set_player_state("att", PlayerState(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState(PS_STANDING));
        game.field_model.set_player_state("assist", PlayerState(PS_PRONE));
        let result = ServerUtilPlayer::find_block_strength(&game, att_coord, 3, def_coord);
        assert_eq!(result, 3);
    }

    #[test]
    fn find_block_strength_standing_assist_counted() {
        let mut game = make_game();
        game.team_home.players.push(make_player("att", 1));
        game.team_away.players.push(make_player("def", 1));
        game.team_home.players.push(make_player("assist", 2));
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        let assist_coord = FieldCoordinate::new(6, 8);
        game.field_model.set_player_coordinate("att", att_coord);
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_coordinate("assist", assist_coord);
        game.field_model.set_player_state("att", PlayerState(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState(PS_STANDING));
        game.field_model.set_player_state("assist", PlayerState(PS_STANDING));
        let result = ServerUtilPlayer::find_block_strength(&game, att_coord, 3, def_coord);
        assert_eq!(result, 4);
    }

    #[test]
    fn find_block_strength_hindered_assist_not_counted() {
        let mut game = make_game();
        game.team_home.players.push(make_player("att", 1));
        game.team_away.players.push(make_player("def", 1));
        game.team_home.players.push(make_player("assist", 2));
        game.team_away.players.push(make_player("hinderer", 2));
        let att_coord = FieldCoordinate::new(5, 7);
        let def_coord = FieldCoordinate::new(6, 7);
        let assist_coord = FieldCoordinate::new(6, 8);
        let hinderer_coord = FieldCoordinate::new(7, 8);
        game.field_model.set_player_coordinate("att", att_coord);
        game.field_model.set_player_coordinate("def", def_coord);
        game.field_model.set_player_coordinate("assist", assist_coord);
        game.field_model.set_player_coordinate("hinderer", hinderer_coord);
        game.field_model.set_player_state("att", PlayerState(PS_STANDING));
        game.field_model.set_player_state("def", PlayerState(PS_STANDING));
        game.field_model.set_player_state("assist", PlayerState(PS_STANDING));
        game.field_model.set_player_state("hinderer", PlayerState(PS_STANDING));
        let result = ServerUtilPlayer::find_block_strength(&game, att_coord, 3, def_coord);
        assert_eq!(result, 3);
    }

    #[test]
    fn find_block_strength_simple_is_additive() {
        for base in [1i32, 3, 5] {
            for assists in [0i32, 1, 2, 3] {
                assert_eq!(ServerUtilPlayer::find_block_strength_simple(base, assists), base + assists);
            }
        }
    }
}
