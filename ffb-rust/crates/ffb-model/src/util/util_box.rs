/// 1:1 translation of com.fumbbl.ffb.util.UtilBox (ffb-common).
///
/// Manages player placement into dugout boxes based on player state.
use crate::enums::{
    PlayerState, SendToBoxReason,
    PS_RESERVE, PS_EXHAUSTED, PS_SETUP_PREVENTED,
    PS_KNOCKED_OUT, PS_BADLY_HURT, PS_SERIOUS_INJURY, PS_RIP, PS_BANNED, PS_MISSING,
};
use crate::model::game::Game;
use crate::types::{
    FieldCoordinate,
    RSV_HOME_X, RSV_AWAY_X,
    KO_HOME_X, KO_AWAY_X,
    BH_HOME_X, BH_AWAY_X,
    SI_HOME_X, SI_AWAY_X,
    RIP_HOME_X, RIP_AWAY_X,
    BAN_HOME_X, BAN_AWAY_X,
    MNG_HOME_X, MNG_AWAY_X,
};

pub struct UtilBox;

impl UtilBox {
    pub fn new() -> Self { Self }

    /// Java: UtilBox.putPlayerIntoBox(Game, Player)
    ///
    /// Finds the correct dugout box column based on the player's current state,
    /// removes them from their current position, then places them at the first
    /// free row in that column.
    pub fn put_player_into_box(game: &mut Game, player_id: &str) {
        let home_player = game.team_home.has_player(player_id);
        let player_state: Option<PlayerState> = game.field_model.player_state(player_id);

        let box_x = match player_state.map(|s| s.base()) {
            Some(b) if b == PS_RESERVE || b == PS_EXHAUSTED || b == PS_SETUP_PREVENTED => {
                if home_player { RSV_HOME_X } else { RSV_AWAY_X }
            }
            Some(b) if b == PS_KNOCKED_OUT => {
                if home_player { KO_HOME_X } else { KO_AWAY_X }
            }
            Some(b) if b == PS_BADLY_HURT => {
                if home_player { BH_HOME_X } else { BH_AWAY_X }
            }
            Some(b) if b == PS_SERIOUS_INJURY => {
                if home_player { SI_HOME_X } else { SI_AWAY_X }
            }
            Some(b) if b == PS_RIP => {
                if home_player { RIP_HOME_X } else { RIP_AWAY_X }
            }
            Some(b) if b == PS_BANNED => {
                if home_player { BAN_HOME_X } else { BAN_AWAY_X }
            }
            Some(b) if b == PS_MISSING => {
                if home_player { MNG_HOME_X } else { MNG_AWAY_X }
            }
            _ => 0,
        };

        if box_x != 0 {
            // Remove player from current position (coordinate only, keep state).
            game.field_model.player_coordinates.remove(player_id);

            // Find the first free row in the box column.
            let mut y = 0i32;
            let mut free_coord = FieldCoordinate::new(box_x, y);
            while game.field_model.player_at(free_coord).is_some() {
                y += 1;
                free_coord = FieldCoordinate::new(box_x, y);
            }
            game.field_model.set_player_coordinate(player_id, free_coord);
        }
    }

    /// Java: UtilBox.refreshBoxes(Game)
    ///
    /// Re-packs each dugout column so its occupants are contiguous from y=0 (no gaps left by
    /// players who moved out onto the pitch). Java calls this at the end of setup, at end of turn,
    /// on ejection and on injury — without it, reserves that were boxed alongside the eleven
    /// fielded players keep their original high rows (e.g. rows 11-13) instead of collapsing to
    /// rows 0-2 after the fielded players leave, and the B&C walk-continuation scatters from the
    /// wrong box coordinate (goblin B&C Fanatic box-row divergence).
    pub fn refresh_boxes(game: &mut Game) {
        for box_x in [
            RSV_HOME_X, RSV_AWAY_X, KO_HOME_X, KO_AWAY_X, BH_HOME_X, BH_AWAY_X,
            SI_HOME_X, SI_AWAY_X, RIP_HOME_X, RIP_AWAY_X, BAN_HOME_X, BAN_AWAY_X,
            MNG_HOME_X, MNG_AWAY_X,
        ] {
            UtilBox::refresh_box(game, box_x);
        }
    }

    /// Java: UtilBox.refreshBox(Game, int) — collect the column's coordinates, sort by y, and
    /// reassign them to y = 0,1,2,… Processing in ascending-y order guarantees each target row is
    /// ≤ the original row being moved, so no unprocessed occupant is ever overwritten.
    fn refresh_box(game: &mut Game, box_x: i32) {
        let mut coords: Vec<FieldCoordinate> = game
            .field_model
            .player_coordinates
            .values()
            .filter(|c| c.x == box_x)
            .copied()
            .collect();
        coords.sort_by_key(|c| c.y);
        for (y, coord) in coords.iter().enumerate() {
            if let Some(player_id) = game.field_model.player_at(*coord).cloned() {
                game.field_model
                    .set_player_coordinate(&player_id, FieldCoordinate::new(box_x, y as i32));
            }
        }
    }

    /// Java: UtilBox.putAllPlayersIntoBox(game).
    /// Moves all canBeSetUpNextDrive players to RESERVE and places them in the box.
    pub fn put_all_players_into_box(game: &mut Game) {
        UtilBox::refresh_boxes(game);
        let all_ids: Vec<String> = game.team_home.players.iter()
            .chain(game.team_away.players.iter())
            .map(|p| p.id.clone())
            .collect();
        for id in all_ids {
            let can_setup = game.field_model.player_state(&id)
                .map(|s| s.can_be_set_up_next_drive())
                .unwrap_or(false);
            if can_setup {
                if let Some(state) = game.field_model.player_state(&id) {
                    game.field_model.set_player_state(&id, state.change_base(PS_RESERVE));
                }
                UtilBox::put_player_into_box(game, &id);
            }
        }
    }

    /// Java: the boxing loop of `GameCache.addTeamToGame` (`ffb-server GameCache.java:271-297`).
    /// When a team joins the game — before it starts — Java gives EVERY player a real dugout
    /// coordinate: RESERVE (or MISSING, if recovering from a lasting injury) plus a box slot via
    /// `putPlayerIntoBox`. Setup then moves only the fielded players OUT; the un-fielded reserves
    /// stay boxed. The headless parity harness runs this via `HeadlessGameSetup` → `addTeamToGame`,
    /// but the synchronous engine constructors build the game directly, so they replicate it here.
    /// Without it a reserve keeps a `None` coordinate; the state hash is blind to that (nr>11), but
    /// the B&C walk-continuation reads the reserve column's occupancy (so the Fanatic's crowd-push
    /// box row diverges from Java). The reserves MUST be PS_RESERVE (not standing): the agent's
    /// legal-action adjacency scans are on-pitch-guarded, and the block/blitz scans additionally
    /// gate on `has_tacklezones` (false for RESERVE), so a boxed reserve never leaks into scoring.
    pub fn box_all_players_at_game_start(game: &mut Game) {
        let players: Vec<(String, bool, bool, i32)> = game
            .team_home
            .players
            .iter()
            .map(|p| (p.id.clone(), true, p.recovering_injury.is_some(), p.current_spps))
            .chain(
                game.team_away
                    .players
                    .iter()
                    .map(|p| (p.id.clone(), false, p.recovering_injury.is_some(), p.current_spps)),
            )
            .collect();
        for (player_id, home_team, has_recovering_injury, current_spps) in players {
            if has_recovering_injury {
                game.field_model.set_player_state(&player_id, PlayerState::new(PS_MISSING));
                let tr = if home_team { &mut game.game_result.home } else { &mut game.game_result.away };
                tr.player_result_mut(&player_id).send_to_box_reason = Some(SendToBoxReason::Mng);
            } else {
                game.field_model.set_player_state(&player_id, PlayerState::new(PS_RESERVE));
            }
            UtilBox::put_player_into_box(game, &player_id);
            if current_spps > 0 {
                let tr = if home_team { &mut game.game_result.home } else { &mut game.game_result.away };
                tr.player_result_mut(&player_id).current_spps = current_spps;
            }
        }
    }
}

impl Default for UtilBox {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use crate::enums::{Rules, PlayerType, PlayerGender};
    use crate::model::player::Player;
    use crate::model::team::Team;

    fn make_player(id: &str) -> Player {
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
            used_skills: HashSet::new(),
            niggling_injuries: 0,
            stat_injuries: vec![],
            current_spps: 0,
            career_spps: 0,
            race: None,
            is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(id: &str, player_ids: &[&str]) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "Coach".into(),
            rerolls: 0,
            apothecaries: 0,
            bribes: 0,
            master_chefs: 0,
            prayers_to_nuffle: 0,
            bloodweiser_kegs: 0,
            riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: player_ids.iter().map(|id| make_player(id)).collect(),
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn make_game_with_players(home_ids: &[&str], away_ids: &[&str]) -> Game {
        let home = make_team("home", home_ids);
        let away = make_team("away", away_ids);
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn put_player_into_box_ko_home_team() {
        let mut game = make_game_with_players(&["h1"], &[]);
        // Place player on pitch first
        let on_pitch = FieldCoordinate::new(5, 5);
        game.field_model.set_player_coordinate("h1", on_pitch);
        game.field_model.set_player_state("h1", PlayerState::new(PS_KNOCKED_OUT));

        UtilBox::put_player_into_box(&mut game, "h1");

        let coord = game.field_model.player_coordinate("h1").expect("player should be in box");
        assert_eq!(coord.x, KO_HOME_X, "KO home player should be at KO_HOME_X");
        assert_eq!(coord.y, 0);
    }

    #[test]
    fn put_player_into_box_rip_away_team() {
        let mut game = make_game_with_players(&[], &["a1"]);
        let on_pitch = FieldCoordinate::new(10, 7);
        game.field_model.set_player_coordinate("a1", on_pitch);
        game.field_model.set_player_state("a1", PlayerState::new(PS_RIP));

        UtilBox::put_player_into_box(&mut game, "a1");

        let coord = game.field_model.player_coordinate("a1").expect("player should be in box");
        assert_eq!(coord.x, RIP_AWAY_X, "RIP away player should be at RIP_AWAY_X");
        assert_eq!(coord.y, 0);
    }

    #[test]
    fn put_player_into_box_reserve_stacks() {
        let mut game = make_game_with_players(&["h1", "h2"], &[]);
        // Place both players on pitch
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(1, 1));
        game.field_model.set_player_state("h1", PlayerState::new(PS_RESERVE));
        game.field_model.set_player_coordinate("h2", FieldCoordinate::new(2, 2));
        game.field_model.set_player_state("h2", PlayerState::new(PS_RESERVE));

        UtilBox::put_player_into_box(&mut game, "h1");
        UtilBox::put_player_into_box(&mut game, "h2");

        let c1 = game.field_model.player_coordinate("h1").unwrap();
        let c2 = game.field_model.player_coordinate("h2").unwrap();
        assert_eq!(c1.x, RSV_HOME_X);
        assert_eq!(c2.x, RSV_HOME_X);
        // Both should be in the reserve column — one at y=0, the other at y=1
        let ys: std::collections::HashSet<i32> = [c1.y, c2.y].iter().copied().collect();
        assert!(ys.contains(&0));
        assert!(ys.contains(&1));
    }

    #[test]
    fn put_player_into_box_badly_hurt_away_team() {
        let mut game = make_game_with_players(&[], &["a1"]);
        game.field_model.set_player_coordinate("a1", FieldCoordinate::new(7, 3));
        game.field_model.set_player_state("a1", PlayerState::new(PS_BADLY_HURT));

        UtilBox::put_player_into_box(&mut game, "a1");

        let coord = game.field_model.player_coordinate("a1").expect("player should be in box");
        assert_eq!(coord.x, BH_AWAY_X, "BH away player should be at BH_AWAY_X");
        assert_eq!(coord.y, 0);
    }

    #[test]
    fn put_player_into_box_unknown_state_leaves_player_unplaced() {
        // A player with no state set should not be moved (box_x == 0 branch).
        let mut game = make_game_with_players(&["h1"], &[]);
        // Do NOT set a state — player_state returns None → box_x = 0.
        let original = FieldCoordinate::new(6, 6);
        game.field_model.set_player_coordinate("h1", original);

        UtilBox::put_player_into_box(&mut game, "h1");

        // Player coordinate should be unchanged (still at original position).
        let coord = game.field_model.player_coordinate("h1").expect("player should still have a coordinate");
        assert_eq!(coord, original);
    }

    // Java UtilBox.refreshBox: after fielded players leave, the reserve column collapses so the
    // remaining occupants are contiguous from y=0. Mirrors the real bug: three reserves boxed at
    // rows 11,12,13 (alongside eleven now-fielded players) must repack to rows 0,1,2.
    #[test]
    fn refresh_boxes_collapses_gaps_in_reserve_column() {
        let mut game = make_game_with_players(&["h12", "h13", "h14"], &[]);
        game.field_model.set_player_coordinate("h12", FieldCoordinate::new(RSV_HOME_X, 11));
        game.field_model.set_player_coordinate("h13", FieldCoordinate::new(RSV_HOME_X, 12));
        game.field_model.set_player_coordinate("h14", FieldCoordinate::new(RSV_HOME_X, 13));

        UtilBox::refresh_boxes(&mut game);

        assert_eq!(game.field_model.player_coordinate("h12").unwrap(), FieldCoordinate::new(RSV_HOME_X, 0));
        assert_eq!(game.field_model.player_coordinate("h13").unwrap(), FieldCoordinate::new(RSV_HOME_X, 1));
        assert_eq!(game.field_model.player_coordinate("h14").unwrap(), FieldCoordinate::new(RSV_HOME_X, 2));
    }

    // Java GameCache.addTeamToGame: every player is RESERVE + boxed at game start; the box columns
    // fill contiguously from y=0 in squad-number order.
    #[test]
    fn box_all_players_at_game_start_boxes_every_player() {
        let mut game = make_game_with_players(&["h1", "h2", "h3"], &["a1", "a2"]);
        UtilBox::box_all_players_at_game_start(&mut game);

        for id in ["h1", "h2", "h3"] {
            let c = game.field_model.player_coordinate(id).expect("home player boxed");
            assert_eq!(c.x, RSV_HOME_X);
            assert_eq!(game.field_model.player_state(id).unwrap().base(), PS_RESERVE);
        }
        for id in ["a1", "a2"] {
            let c = game.field_model.player_coordinate(id).expect("away player boxed");
            assert_eq!(c.x, RSV_AWAY_X);
        }
        let mut rows: Vec<i32> = ["h1", "h2", "h3"].iter()
            .map(|id| game.field_model.player_coordinate(id).unwrap().y).collect();
        rows.sort();
        assert_eq!(rows, vec![0, 1, 2]);
    }
}
