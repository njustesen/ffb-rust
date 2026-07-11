/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerInjuryTest.
/// Test variant of TalkHandlerInjury — applies injury to player's lasting injury list.
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::enums::InjuryAttribute;
use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_handler_injury::TalkHandlerInjury;
use crate::handler::talk::talk_requirements::{Client, Environment};

pub struct TalkHandlerInjuryTest {
    base: TalkHandlerInjury,
}

impl TalkHandlerInjuryTest {
    /// Java: `TalkHandlerInjuryTest()`.
    pub fn new() -> Self {
        let adapter = IdentityCommandAdapter::new();
        Self {
            base: TalkHandlerInjury::new(&adapter, Client::Player, Environment::TestGame, Default::default()),
        }
    }

    pub fn base(&self) -> &TalkHandler { self.base.base() }

    /// Java: `handle` — delegates to TalkHandlerInjury with test game settings.
    pub fn handle(&self, team: &Team, commands: &[String]) -> Vec<(String, Option<InjuryAttribute>)> {
        self.base.handle(team, commands)
    }

    /// Java: `applyInjury(FantasyFootballServer, GameState, RosterPlayer, SeriousInjury)` —
    /// adds the injury to the player's lasting-injury list and recalculates stats, then
    /// returns the info message Java would have sent via `communication.sendPlayerTalk`.
    ///
    /// Java calls `lastingInjury.getName()` unconditionally (no null-check), so a `None`
    /// attribute here mirrors that latent NPE via `.expect(...)`.
    pub fn apply_injury(&self, game: &mut Game, player_id: &str, attribute: Option<InjuryAttribute>) -> Vec<String> {
        let attr = attribute.expect("Java calls lastingInjury.getName() unconditionally here too");
        let (kind, name) = TalkHandlerInjury::resolve_lasting_injury(game, attr);

        // Java: `player.addLastingInjury(lastingInjury)`.
        if let Some(player) = game.player_mut(player_id) {
            player.stat_injuries.push(kind);
        }

        // Java: `player.updatePosition(player.getPosition(), true, game.getRules(), game.getId())`
        // — recalculates stats from position + lasting injuries via StatsMechanic. ffb-server
        // has no dependency on ffb-mechanics yet, so the recalculation itself is unwired.
        Self::update_position_stats(game, player_id);

        let player_name = game.player(player_id).map(|p| p.name.clone()).unwrap_or_default();
        vec![format!("Player {player_name} suffers injury {name}.")]
    }

    fn update_position_stats(_game: &mut Game, _player_id: &str) {
        todo!("Phase ZV: needs RosterPlayer::update_position (stat recalculation via StatsMechanic) wiring — ffb-server has no dependency on ffb-mechanics yet")
    }
}

impl Default for TalkHandlerInjuryTest {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender, Rules};
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
    fn construct() { let _ = TalkHandlerInjuryTest::new(); }

    #[test]
    #[should_panic(expected = "Java calls lastingInjury.getName")]
    fn apply_injury_without_attribute_panics_like_java_npe() {
        let h = TalkHandlerInjuryTest::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let _ = h.apply_injury(&mut game, "p1", None);
    }

    #[test]
    #[should_panic(expected = "Phase ZV: needs RosterPlayer::update_position")]
    fn apply_injury_with_attribute_hits_unwired_stats_recalculation() {
        // The `SeriousInjuryFactory::for_attribute` lookup is wired up (see
        // `talk_handler_injury.rs::resolve_lasting_injury`); this now panics further
        // down, in `update_position_stats`, which still needs `RosterPlayer::update_position`
        // (StatsMechanic wiring) — out of scope for this fix (ffb-server has no
        // dependency on ffb-mechanics yet).
        let h = TalkHandlerInjuryTest::new();
        let home = make_team("home", vec![make_player("p1", 1)]);
        let mut game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let _ = h.apply_injury(&mut game, "p1", Some(InjuryAttribute::NI));
    }

    #[test]
    fn resolve_lasting_injury_is_wired_up_before_the_stats_recalculation_stub() {
        // Confirms the injury-factory lookup itself (this task's blocker) now succeeds
        // and produces the expected kind/name, independent of the still-unported
        // stats-recalculation step exercised by the should_panic test above.
        let home = make_team("home", vec![make_player("p1", 1)]);
        let game = Game::new(home, make_team("away", vec![]), Rules::Bb2025);
        let (kind, name) = TalkHandlerInjury::resolve_lasting_injury(&game, InjuryAttribute::NI);
        assert_eq!(kind, ffb_model::enums::SeriousInjuryKind::SeriousInjuryNi);
        assert_eq!(name, "Serious Injury (NI)");
    }
}
