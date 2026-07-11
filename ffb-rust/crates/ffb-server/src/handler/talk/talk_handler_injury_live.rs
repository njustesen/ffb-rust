/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerInjuryLive.
/// Live variant of TalkHandlerInjury — applies injury to PlayerResult (game result tracking).
use std::collections::HashSet;
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::InjuryAttribute;
use crate::handler::talk::decorating_command_adapter::DecoratingCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_injury::TalkHandlerInjury;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerInjuryLive {
    base: TalkHandlerInjury,
}

impl TalkHandlerInjuryLive {
    /// Java: `TalkHandlerInjuryLive()`.
    pub fn new() -> Self {
        let adapter = DecoratingCommandAdapter::new();
        let mut privileges = HashSet::new();
        privileges.insert(Privilege::EditState);
        Self {
            base: TalkHandlerInjury::new(&adapter, Client::Spec, Environment::None, privileges),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerInjury with live game settings.
    pub fn handle(&self, team: &Team, commands: &[String]) -> Vec<(String, Option<InjuryAttribute>)> {
        self.base.handle(team, commands)
    }

    /// Java: `applyInjury(FantasyFootballServer, GameState, RosterPlayer, SeriousInjury)` —
    /// updates the player's `PlayerResult` serious injury fields. Returns the info message
    /// Java would have sent via `communication.sendPlayerTalk` (see `talk_handler.rs` doc
    /// for why `sendAddPlayer`/`sendPlayerTalk` calls return strings instead of sending).
    pub fn apply_injury(&self, game: &mut Game, player_id: &str, attribute: Option<InjuryAttribute>) -> Vec<String> {
        let player_name = game.player(player_id).map(|p| p.name.clone()).unwrap_or_default();
        let is_home = game.team_home.player(player_id).is_some();
        // Resolved before taking the mutable `player_result` borrow below, since it needs
        // `game` immutably (Java has no such conflict — one mutable object, freely re-read).
        let resolved = attribute.map(|attr| TalkHandlerInjury::resolve_lasting_injury(game, attr));
        let player_result = game.game_result.team_result_mut(is_home).player_result_mut(player_id);

        let mut info = Vec::new();
        match resolved {
            None => {
                info.push(format!("Removing injuries from player {player_name}."));
                player_result.serious_injury_decay = None;
                player_result.serious_injury = None;
            }
            Some((kind, name)) => {
                if player_result.serious_injury.is_none() {
                    player_result.serious_injury = Some(kind);
                    info.push(format!("Player {player_name} suffers a injury: {name}."));
                } else if player_result.serious_injury_decay.is_none() {
                    player_result.serious_injury_decay = Some(kind);
                    info.push(format!("Player {player_name} suffers a second injury: {name}."));
                }
            }
        }
        info
    }
}

impl Default for TalkHandlerInjuryLive {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};
    use ffb_model::model::team::Team;
    use std::collections::HashSet as Set;

    fn make_player(id: &str, nr: i32) -> ffb_model::model::player::Player {
        ffb_model::model::player::Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<ffb_model::model::player::Player>) -> Team {
        Team {
            id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    #[test]
    fn construct() { let _ = TalkHandlerInjuryLive::new(); }

    #[test]
    fn apply_injury_none_removes_existing_injuries() {
        let h = TalkHandlerInjuryLive::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        {
            let pr = game.game_result.team_result_mut(true).player_result_mut("p1");
            pr.serious_injury = Some(ffb_model::enums::SeriousInjuryKind::HeadInjuryAv);
        }
        let info = h.apply_injury(&mut game, "p1", None);
        assert_eq!(info, vec!["Removing injuries from player Player1.".to_string()]);
        let pr = game.game_result.team_result_mut(true).player_result_mut("p1");
        assert!(pr.serious_injury.is_none());
        assert!(pr.serious_injury_decay.is_none());
    }

    #[test]
    fn apply_injury_none_on_clean_player_is_noop_message_only() {
        let h = TalkHandlerInjuryLive::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let info = h.apply_injury(&mut game, "p1", None);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("Removing injuries"));
    }

    #[test]
    fn apply_injury_with_attribute_sets_serious_injury_and_reports_it() {
        let h = TalkHandlerInjuryLive::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let info = h.apply_injury(&mut game, "p1", Some(InjuryAttribute::NI));
        assert_eq!(info, vec!["Player Player1 suffers a injury: Serious Injury (NI).".to_string()]);
        let pr = game.game_result.team_result_mut(true).player_result_mut("p1");
        assert_eq!(pr.serious_injury, Some(ffb_model::enums::SeriousInjuryKind::SeriousInjuryNi));
        assert!(pr.serious_injury_decay.is_none());
    }

    #[test]
    fn apply_injury_second_attribute_sets_decay_slot() {
        let h = TalkHandlerInjuryLive::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let _ = h.apply_injury(&mut game, "p1", Some(InjuryAttribute::NI));
        let info = h.apply_injury(&mut game, "p1", Some(InjuryAttribute::AV));
        assert_eq!(info, vec!["Player Player1 suffers a second injury: Head Injury (-AV).".to_string()]);
        let pr = game.game_result.team_result_mut(true).player_result_mut("p1");
        assert_eq!(pr.serious_injury, Some(ffb_model::enums::SeriousInjuryKind::SeriousInjuryNi));
        assert_eq!(pr.serious_injury_decay, Some(ffb_model::enums::SeriousInjuryKind::HeadInjuryAv));
    }
}
