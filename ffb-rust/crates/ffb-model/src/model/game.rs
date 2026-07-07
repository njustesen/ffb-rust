use serde::{Deserialize, Serialize};
use crate::dialog::dialog_id::DialogId;
use crate::enums::{GameStatus, TurnMode, PlayerAction, Rules, Weather};
use crate::model::property::NamedProperties;
use crate::types::FieldCoordinate;
use crate::model::acting_player::ActingPlayer;
use crate::model::blitz_turn_state::BlitzTurnState;
use crate::model::field_model::FieldModel;
use crate::model::game_options::GameOptions;
use crate::model::game_result::GameResult;
use crate::model::player::PlayerId;
use crate::model::team::Team;
use crate::model::turn_data::TurnData;
use crate::model::prayer_state::PrayerState;
use crate::model::zapped_player::ZappedPlayer;
use crate::model::zapped_position::ZappedPosition;
use crate::report::report_list::ReportList;

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
    /// Java: GameState.getPassState().getOriginalBombardier() — player ID of the bombardier
    /// who fired a bomb this turn; set by bomb-pass steps, cleared at start_turn().
    pub original_bombardier: Option<PlayerId>,
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
    /// Java: GameState.getPrayerState() — prayer effects active this game.
    pub prayer_state: PrayerState,
    /// Java: GameState.getKickingSwarmers / setKickingSwarmers — tracks how many
    /// kicking-team swarming players were placed during the Swarming kickoff result.
    pub kicking_swarmers: i32,
    /// Java: GameState.activeEffects.shadowers — list of player IDs that have shadowed
    /// this move (may contain duplicates; count per ID = shadowingCount for that player).
    pub active_shadowers: Vec<String>,
    /// Java: GameState.getGame().getDialogParameter().getId() — which dialog is currently shown.
    /// Set by UtilServerDialog.showDialog(), cleared by hideDialog().
    pub dialog_id: Option<DialogId>,
    /// Java: GameState.getReportList() — all reports emitted this turn/game.
    /// Populated by step methods via report structs in ffb-model/src/report/.
    #[serde(skip)]
    pub report_list: ReportList,

    /// Java: GameState.zappedPlayerIds + team.addPlayer(zappedPlayer) —
    /// stores the original player snapshot before ZAP stats were applied.
    /// Players are restored (unzapped) at end of drive/half.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zapped_players: Vec<ZappedPlayer>,
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
            original_bombardier: None,
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
            prayer_state: PrayerState::new(),
            kicking_swarmers: 0,
            active_shadowers: Vec::new(),
            dialog_id: None,
            report_list: ReportList::new(),
            zapped_players: Vec::new(),
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

    /// Java: GameState.shadowingCount(playerId) — count of how many times a player has shadowed.
    pub fn shadowing_count(&self, player_id: &str) -> usize {
        self.active_shadowers.iter().filter(|id| id.as_str() == player_id).count()
    }

    /// Java: GameState.addShadower(playerId) — record that a player shadowed this move.
    pub fn add_shadower(&mut self, player_id: &str) {
        self.active_shadowers.push(player_id.to_owned());
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

    /// Java: `Game.isActive(ISkillProperty)` — true if any active card in either team's
    /// InducementSet has the given named property in its globalProperties().
    pub fn is_active(&self, named_property: &str) -> bool {
        let home = self.turn_data_home.inducement_set.get_active_cards();
        let away = self.turn_data_away.inducement_set.get_active_cards();
        home.into_iter().chain(away)
            .any(|card| Self::card_global_properties(card).contains(&named_property))
    }

    /// Static mapping of BB2016 card names → their globalProperties().
    /// Java: anonymous Card subclasses override globalProperties() per card.
    fn card_global_properties(card_name: &str) -> &'static [&'static str] {
        match card_name {
            "Blatant Foul" => &[NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL],
            "Greased Shoes" => &[NamedProperties::SET_GFI_ROLL_TO_FIVE],
            "Spiked Ball" => &[NamedProperties::DROPPED_BALL_CAUSES_ARMOUR_ROLL],
            _ => &[],
        }
    }

    /// Java: GameState.addZappedPlayer(player) + team.addPlayer(zappedPlayer) —
    /// saves the original player, then overwrites the in-team player with ZAP stats and skills.
    pub fn add_zapped_player(&mut self, zapped: ZappedPlayer) {
        let id = zapped.original_player.id.clone();
        // Apply zap stats to the live player in the team.
        if let Some(p) = self.player_mut(&id) {
            p.movement = zapped.position.movement;
            p.strength = zapped.position.strength;
            p.agility = zapped.position.agility;
            p.passing = zapped.position.passing;
            p.armour = zapped.position.armour;
            p.starting_skills = ZappedPosition::get_skills();
            p.extra_skills.clear();
            p.temporary_skills.clear();
            p.zapped = true;
        }
        self.zapped_players.push(zapped);
    }

    /// Java: GameState.removeZappedPlayer(player) — called after unzapping restores the original.
    pub fn remove_zapped_player(&mut self, player_id: &str) {
        self.zapped_players.retain(|z| z.original_player.id != player_id);
    }

    /// Java: GameState.isZapped(player).
    pub fn is_zapped_player(&self, player_id: &str) -> bool {
        self.zapped_players.iter().any(|z| z.original_player.id == player_id)
    }

    /// Java: end-of-drive unzap loop in StepEndTurn — restores all zapped players to their original stats.
    /// Returns the IDs of all players that were unzapped.
    pub fn unzap_all_players(&mut self) -> Vec<String> {
        let zapped = std::mem::take(&mut self.zapped_players);
        let mut ids = Vec::with_capacity(zapped.len());
        for z in zapped {
            let id = z.original_player.id.clone();
            if let Some(p) = self.player_mut(&id) {
                *p = z.original_player;
            }
            ids.push(id);
        }
        ids
    }

    /// Java: `Game.startTurn()` — resets per-turn state on both teams' TurnData and clears acting player.
    /// Called at the start of each team turn (end-turn, blitz turn, etc.).
    pub fn start_turn(&mut self) {
        self.pass_coordinate = None;
        self.original_bombardier = None;
        self.acting_player.clear();
        self.turn_data_home.reset_for_turn();
        self.turn_data_away.reset_for_turn();
        self.thrower_id = None;
        self.thrower_action = None;
        self.defender_id = None;
        self.defender_action = None;
        self.waiting_for_opponent = false;
        self.timeout_possible = false;
        self.timeout_enforced = false;
        self.concession_possible = true;
        self.last_defender_id = None;
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
            vampire_lord: false,
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
            is_big_guy: false,
            ..Default::default()
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
            is_big_guy: false,
            ..Default::default()
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
            is_big_guy: false,
            ..Default::default()
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

    #[test]
    fn start_turn_clears_per_turn_state() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2020);
        use crate::types::FieldCoordinate;
        g.pass_coordinate = Some(FieldCoordinate::new(5, 7));
        g.thrower_id = Some("p1".into());
        g.thrower_action = Some(crate::enums::PlayerAction::Pass);
        g.defender_id = Some("p2".into());
        g.defender_action = Some(crate::enums::PlayerAction::Blitz);
        g.waiting_for_opponent = true;
        g.timeout_possible = true;
        g.timeout_enforced = true;
        g.concession_possible = false;
        g.last_defender_id = Some("p3".into());
        g.turn_data_home.blitz_used = true;
        g.turn_data_away.foul_used = true;

        g.start_turn();

        assert!(g.pass_coordinate.is_none());
        assert!(g.thrower_id.is_none());
        assert!(g.thrower_action.is_none());
        assert!(g.defender_id.is_none());
        assert!(g.defender_action.is_none());
        assert!(!g.waiting_for_opponent);
        assert!(!g.timeout_possible);
        assert!(!g.timeout_enforced);
        assert!(g.concession_possible);
        assert!(g.last_defender_id.is_none());
        assert!(!g.turn_data_home.blitz_used);
        assert!(!g.turn_data_away.foul_used);
    }

    #[test]
    fn is_active_returns_false_with_no_active_cards() {
        let g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        assert!(!g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
    }

    #[test]
    fn is_active_returns_true_when_blatant_foul_active_on_home() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        g.turn_data_home.inducement_set.add_available_card("Blatant Foul");
        g.turn_data_home.inducement_set.activate_card("Blatant Foul");
        assert!(g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
    }

    #[test]
    fn is_active_returns_true_when_blatant_foul_active_on_away() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        g.turn_data_away.inducement_set.add_available_card("Blatant Foul");
        g.turn_data_away.inducement_set.activate_card("Blatant Foul");
        assert!(g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
    }

    #[test]
    fn is_active_greased_shoes_sets_gfi_property() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        g.turn_data_home.inducement_set.add_available_card("Greased Shoes");
        g.turn_data_home.inducement_set.activate_card("Greased Shoes");
        assert!(g.is_active(NamedProperties::SET_GFI_ROLL_TO_FIVE));
        assert!(!g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
    }

    #[test]
    fn card_deactivated_is_no_longer_active() {
        let mut g = Game::new(empty_team("home"), empty_team("away"), Rules::Bb2016);
        g.turn_data_home.inducement_set.add_available_card("Blatant Foul");
        g.turn_data_home.inducement_set.activate_card("Blatant Foul");
        assert!(g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
        g.turn_data_home.inducement_set.deactivate_card("Blatant Foul");
        assert!(!g.is_active(NamedProperties::FOUL_BREAKS_ARMOUR_WITHOUT_ROLL));
    }
}
