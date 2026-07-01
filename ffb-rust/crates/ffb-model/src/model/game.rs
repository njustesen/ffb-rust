use serde::{Deserialize, Serialize};
use crate::enums::{GameStatus, TurnMode, PlayerAction, Rules, Weather};
use crate::types::FieldCoordinate;
use crate::model::acting_player::ActingPlayer;
use crate::model::blitz_turn_state::BlitzTurnState;
use crate::model::field_model::FieldModel;
use crate::model::game_options::GameOptions;
use crate::model::game_result::GameResult;
use crate::model::player::PlayerId;
use crate::model::team::Team;
use crate::model::turn_data::TurnData;

/// The complete game state — primary data structure for the headless engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: u64,
    pub rules: Rules,
    pub status: GameStatus,
    pub turn_mode: TurnMode,
    pub last_turn_mode: Option<TurnMode>,
    pub half: i32,
    pub home_playing: bool,
    pub home_first_offense: bool,
    pub setup_offense: bool,
    pub weather: Weather,

    pub team_home: Team,
    pub team_away: Team,
    pub turn_data_home: TurnData,
    pub turn_data_away: TurnData,

    pub field_model: FieldModel,
    pub acting_player: ActingPlayer,
    pub game_result: GameResult,
    pub options: GameOptions,

    pub defender_id: Option<PlayerId>,
    pub defender_action: Option<PlayerAction>,
    pub thrower_id: Option<PlayerId>,
    pub thrower_action: Option<PlayerAction>,
    pub pass_coordinate: Option<FieldCoordinate>,
    pub waiting_for_opponent: bool,
    pub timeout_possible: bool,
    pub timeout_enforced: bool,
    pub concession_possible: bool,
    pub conceded_legally: bool,
    pub testing: bool,
    /// Set true when a turnover occurs (failed pickup, failed dodge, etc.) so EndPlayerAction
    /// pushes end_turn_sequence() instead of select_sequence(). Cleared after consumption.
    pub turnover: bool,
    /// Cheering Fans kickoff bonus: team gets +1 to attacker strength for ONE block.
    /// Java: GameState.getAdditionalAssist / setTeamIdsAdditionalAssist / removeAdditionalAssist.
    pub home_additional_assists: i32,
    pub away_additional_assists: i32,
    /// Java: GameState.isStalling() / stallingDetected() / resetStalling().
    /// Set true when the acting player is detected to be stalling; cleared after consumption.
    pub stalling: bool,
    /// Java: game.isAdminMode() / setAdminMode — admin-controlled game allows reconnection.
    pub admin_mode: bool,
    /// Java: GameState.getLastDefenderId / setLastDefenderId — stores defender ID for MaximumCarnage second-block.
    pub last_defender_id: Option<PlayerId>,
    /// Java: GameState.getBlitzTurnState / setBlitzTurnState — tracks activations during the Blitz! kickoff result.
    pub blitz_turn_state: Option<BlitzTurnState>,
}

impl Game {
    pub fn new(home: Team, away: Team, rules: Rules) -> Self {
        Game {
            id: 0,
            rules,
            status: GameStatus::Starting,
            turn_mode: TurnMode::StartGame,
            last_turn_mode: None,
            half: 1,
            home_playing: true,
            home_first_offense: true,
            setup_offense: false,
            weather: Weather::Nice,
            team_home: home,
            team_away: away,
            turn_data_home: TurnData::new(),
            turn_data_away: TurnData::new(),
            field_model: FieldModel::new(),
            acting_player: ActingPlayer::new(),
            game_result: GameResult::default(),
            options: GameOptions::new(),
            defender_id: None,
            defender_action: None,
            thrower_id: None,
            thrower_action: None,
            pass_coordinate: None,
            waiting_for_opponent: false,
            timeout_possible: false,
            timeout_enforced: false,
            concession_possible: false,
            conceded_legally: false,
            testing: false,
            turnover: false,
            home_additional_assists: 0,
            away_additional_assists: 0,
            stalling: false,
            admin_mode: false,
            last_defender_id: None,
            blitz_turn_state: None,
        }
    }

    /// The team currently taking their turn.
    pub fn active_team(&self) -> &Team {
        if self.home_playing { &self.team_home } else { &self.team_away }
    }

    /// The team currently waiting (not taking their turn).
    pub fn inactive_team(&self) -> &Team {
        if self.home_playing { &self.team_away } else { &self.team_home }
    }

    pub fn active_team_mut(&mut self) -> &mut Team {
        if self.home_playing { &mut self.team_home } else { &mut self.team_away }
    }

    /// TurnData for the currently-active team.
    pub fn turn_data(&self) -> &TurnData {
        if self.home_playing { &self.turn_data_home } else { &self.turn_data_away }
    }

    pub fn turn_data_mut(&mut self) -> &mut TurnData {
        if self.home_playing { &mut self.turn_data_home } else { &mut self.turn_data_away }
    }

    /// Find a player by id (searches both teams).
    pub fn player(&self, id: &str) -> Option<&crate::model::player::Player> {
        self.team_home.player(id).or_else(|| self.team_away.player(id))
    }

    /// Find a player mutably by id (searches both teams).
    pub fn player_mut(&mut self, id: &str) -> Option<&mut crate::model::player::Player> {
        if self.team_home.player(id).is_some() {
            self.team_home.player_mut(id)
        } else {
            self.team_away.player_mut(id)
        }
    }

    /// Mark a skill as used for the given player (Java: actingPlayer.markSkillUsed / player.addUsedSkill).
    pub fn mark_skill_used(&mut self, player_id: &str, skill_id: crate::enums::SkillId) {
        if let Some(player) = self.player_mut(player_id) {
            player.used_skills.insert(skill_id);
        }
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.status, GameStatus::Finished)
    }

    /// True if `team` is the home team.
    pub fn is_home_team(&self, team_id: &str) -> bool {
        self.team_home.id == team_id
    }

    /// Returns the team id that the given player belongs to, or None if not found.
    pub fn player_team_id(&self, player_id: &str) -> Option<&str> {
        if self.team_home.player(player_id).is_some() {
            Some(&self.team_home.id)
        } else if self.team_away.player(player_id).is_some() {
            Some(&self.team_away.id)
        } else {
            None
        }
    }

    /// True if the player belongs to the currently-active team.
    pub fn is_active_team_player(&self, player_id: &str) -> bool {
        self.player_team_id(player_id) == Some(self.active_team().id.as_str())
    }

    /// Returns the current thrower (player with thrower_id), if any.
    pub fn thrower(&self) -> Option<&crate::model::player::Player> {
        self.thrower_id.as_deref().and_then(|id| self.player(id))
    }

    /// Look up a team by its id; returns None if neither team matches.
    pub fn team_by_id(&self, id: &str) -> Option<&Team> {
        if self.team_home.id == id {
            Some(&self.team_home)
        } else if self.team_away.id == id {
            Some(&self.team_away)
        } else {
            None
        }
    }

    /// Stub for Game.isActive(NamedProperties) — checks if any active player has the given skill property.
    /// TODO: implement when NamedProperties lookup is translated.
    pub fn is_active(&self, _named_property: &str) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;

    fn empty_team(id: &str) -> Team {
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
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0,
            assistant_coaches: 0,
            fan_factor: 0,
            dedicated_fans: 0,
            team_value: 0,
            treasury: 0,
            special_rules: vec![],
            players: vec![],
        }
    }

    #[test]
    fn active_team() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        assert_eq!(g.active_team().id, "home");
        assert_eq!(g.inactive_team().id, "away");
    }

    #[test]
    fn serde_round_trip() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        let json = serde_json::to_string(&g).unwrap();
        let back: Game = serde_json::from_str(&json).unwrap();
        assert_eq!(g.rules, back.rules);
        assert_eq!(g.turn_mode, back.turn_mode);
    }

    #[test]
    fn active_team_is_away_when_away_playing() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        g.home_playing = false;
        assert_eq!(g.active_team().id, "away");
        assert_eq!(g.inactive_team().id, "home");
    }

    #[test]
    fn is_finished_false_by_default() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        assert!(!g.is_finished());
    }

    #[test]
    fn is_finished_true_when_status_finished() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        g.status = GameStatus::Finished;
        assert!(g.is_finished());
    }

    #[test]
    fn is_home_team_identifies_correct_team() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        assert!(g.is_home_team("home"));
        assert!(!g.is_home_team("away"));
    }

    #[test]
    fn turn_data_returns_home_data_when_home_playing() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        g.turn_data_home.turn_nr = 3;
        g.turn_data_away.turn_nr = 7;
        assert_eq!(g.turn_data().turn_nr, 3);
    }

    #[test]
    fn turn_data_returns_away_data_when_away_playing() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        g.home_playing = false;
        g.turn_data_home.turn_nr = 3;
        g.turn_data_away.turn_nr = 7;
        assert_eq!(g.turn_data().turn_nr, 7);
    }

    #[test]
    fn player_lookup_searches_both_teams() {
        use std::collections::HashSet;
        use crate::enums::{PlayerType, PlayerGender};
        use crate::model::player::Player;
        let mut home = empty_team("home");
        home.players.push(Player {
            id: "h1".into(), name: "HomePlayer".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
        });
        let mut away = empty_team("away");
        away.players.push(Player {
            id: "a1".into(), name: "AwayPlayer".into(), nr: 1,
            position_id: "lineman".into(), player_type: PlayerType::Regular,
            gender: PlayerGender::Male, movement: 6, strength: 3, agility: 3,
            passing: 4, armour: 8, starting_skills: vec![], extra_skills: vec![],
            temporary_skills: vec![], used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0,
            career_spps: 0, race: None,
        });
        let g = Game::new(home, away, Rules::Bb2020);
        assert_eq!(g.player("h1").map(|p| p.name.as_str()), Some("HomePlayer"));
        assert_eq!(g.player("a1").map(|p| p.name.as_str()), Some("AwayPlayer"));
        assert!(g.player("x99").is_none());
    }

    #[test]
    fn mark_skill_used_inserts_into_player_used_skills() {
        use std::collections::HashSet;
        use crate::enums::{PlayerType, PlayerGender, SkillId, Rules};
        use crate::model::player::Player;
        let mut home = empty_team("home");
        home.players.push(Player {
            id: "p1".into(), name: "P".into(), nr: 1, position_id: "pos".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: HashSet::new(),
            niggling_injuries: 0, stat_injuries: vec![], current_spps: 0, career_spps: 0, race: None,
        });
        let mut g = Game::new(home, empty_team("away"), Rules::Bb2020);
        assert!(!g.player("p1").unwrap().used_skills.contains(&SkillId::BlastIt));
        g.mark_skill_used("p1", SkillId::BlastIt);
        assert!(g.player("p1").unwrap().used_skills.contains(&SkillId::BlastIt));
    }

    #[test]
    fn mark_skill_used_unknown_player_is_noop() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        // Should not panic
        g.mark_skill_used("nonexistent", crate::enums::SkillId::BlastIt);
    }
}
