/// 1:1 translation of com.fumbbl.ffb.util.UtilBox (ffb-common).
///
/// Manages player placement into dugout boxes based on player state.
use crate::enums::{
    PlayerState,
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
    /// Clears and re-sorts box contents so all players in each dugout column
    /// are packed contiguously from y=0. Stub — not critical for current parity phase.
    pub fn refresh_boxes(_game: &mut Game) {
        // TODO(refreshBoxes): full implementation — clear then re-pack each dugout column.
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
}
