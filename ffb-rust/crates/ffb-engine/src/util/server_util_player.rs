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
        let mut def_team_home = defender_id.map(|did| game.team_home.has_player(did)).unwrap_or(false);
        // Java ServerUtilPlayer.findBlockStrength same-team clauses (a Ball & Chain player's
        // compulsory block on a TEAM-MATE):
        //   if (ignoresAssists && sameTeam) return attackerStrength;
        //   if (attacker.hasSkillProperty(flipSameTeamOpponentToOtherTeam) && sameTeam)
        //       defenderTeam = otherTeam(defender);
        // "team-mates assist b&c ... to gain maximum block dice" — the marked-check for the
        // assisted player's helpers flips to the OTHER team, so his own team-mates count as
        // unmarked assists. Missing both, a Fanatic's same-team block rolled 2 dice where Java
        // rolls 3 (goblin bb2025 seed 64 step 111), silently shifting the stream by one die.
        {
            use ffb_model::model::property::NamedProperties;
            let same_team = attacker_id.zip(defender_id).map(|(a, d)| {
                game.team_home.has_player(a) == game.team_home.has_player(d)
            }).unwrap_or(false);
            if same_team {
                let assisted = attacker_id.and_then(|aid| game.player(aid));
                let opponent = defender_id.and_then(|did| game.player(did));
                // Edition-aware property reads: Ball & Chain registers ignoreBlockAssists only in
                // bb2020/bb2025 and the flip only in bb2016 — the edition-agnostic union would make
                // the ignores clause swallow the bb2016 flip.
                let ignores = assisted.map(|p| p.has_skill_property_in(game.rules, NamedProperties::IGNORE_BLOCK_ASSISTS)).unwrap_or(false)
                    || opponent.map(|p| p.has_skill_property_in(game.rules, NamedProperties::IGNORE_BLOCK_ASSISTS)).unwrap_or(false);
                if ignores {
                    return block_strength;
                }
                let flips = assisted.map(|p| p.has_skill_property_in(game.rules, NamedProperties::FLIP_SAME_TEAM_OPPONENT_TO_OTHER_TEAM)).unwrap_or(false);
                if flips {
                    def_team_home = !def_team_home;
                }
            }
        }
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

    /// Java's same-team clauses: a Ball & Chain attacker (`flipSameTeamOpponentToOtherTeam`)
    /// blocking a TEAM-MATE flips the marked-team, so his own teammates count as unmarked
    /// assists — without the flip the same-team helpers "mark" each other and the assist is lost
    /// (goblin bb2025 seed 64 step 111: a 3-die Fanatic block became 2 dice).
    #[test]
    fn same_team_ball_and_chain_block_flips_assist_marking() {
        use ffb_model::model::SkillWithValue;
        use ffb_model::enums::SkillId;
        let mut game = make_game();
        let mut fanatic = make_player("bc", 1);
        fanatic.extra_skills.push(SkillWithValue::new(SkillId::BallAndChain));
        game.team_home.players.push(fanatic);
        game.team_home.players.push(make_player("victim", 2));
        game.team_home.players.push(make_player("helper", 3));
        for (id, x, y) in [("bc", 5, 5), ("victim", 6, 5), ("helper", 6, 6)] {
            game.field_model.set_player_coordinate(id, FieldCoordinate::new(x, y));
            game.field_model.set_player_state(id, PlayerState::new(PS_STANDING));
        }
        // bb2020/bb2025 Ball & Chain registers ignoreBlockAssists: the same-team clause
        // short-circuits to BARE strength — no assists on either side (this is what makes a
        // ST7 Fanatic's team-mate block 3 dice in bb2025; goblin seed 64 step 111).
        let st = ServerUtilPlayer::find_block_strength(
            &game, FieldCoordinate::new(5, 5), 7, FieldCoordinate::new(6, 5));
        assert_eq!(st, 7, "bb2020+ B&C same-team block ignores assists entirely");

        // bb2016's B&C has NO ignoreBlockAssists but DOES have the flip: the marked-team for the
        // helper becomes AWAY (empty), so the same-team helper is an unmarked assist: 7 + 1.
        game.rules = Rules::Bb2016;
        let st16 = ServerUtilPlayer::find_block_strength(
            &game, FieldCoordinate::new(5, 5), 7, FieldCoordinate::new(6, 5));
        assert_eq!(st16, 8, "bb2016 B&C same-team block counts the teammate assist via the flip");
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
