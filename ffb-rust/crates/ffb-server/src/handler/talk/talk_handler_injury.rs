/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerInjury.
/// Abstract handler for /injury command — applies lasting injuries to players.
///
/// Java's `handle()` resolves `commands[1]` to an `InjuryAttribute`, asks
/// `game.getFactory(FactoryType.SERIOUS_INJURY).forAttribute(attribute)` for the
/// edition-specific `SeriousInjury`, then calls the abstract `applyInjury(...)`
/// for every `RosterPlayer` targeted by the command. Rust has no inheritance, so
/// `handle()` here only does the parts that are pure game-state lookups (parsing
/// the attribute, resolving target players) and returns `(player_id, attribute)`
/// pairs for the concrete `TalkHandlerInjuryLive`/`TalkHandlerInjuryTest` (which
/// implement Java's abstract `applyInjury`) to mutate game state with.
use std::collections::HashSet;
use ffb_model::factory::serious_injury_factory::SeriousInjuryFactory;
use ffb_model::model::game::Game;
use ffb_model::model::serious_injury::SeriousInjury as _;
use ffb_model::model::team::Team;
use ffb_model::enums::{InjuryAttribute, SeriousInjuryKind};
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerInjury {
    base: TalkHandler,
}

impl TalkHandlerInjury {
    /// Java: `TalkHandlerInjury(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/injury".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 2, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// resolves the injury attribute from `commands[1]` and returns the id of every
    /// player targeted by the command paired with that attribute, for the caller's
    /// `apply_injury` to actually apply.
    pub fn handle(&self, team: &Team, commands: &[String]) -> Vec<(String, Option<InjuryAttribute>)> {
        let attribute = Self::resolve_attribute(&commands[1]);
        self.base
            .find_players_in_command(team, commands)
            .into_iter()
            .map(|p| (p.id.clone(), attribute))
            .collect()
    }

    /// Java: the if/else chain in `handle` matching `commands[1]` to an `InjuryAttribute`
    /// ("ni" -> NI, "-ma" -> MA, "-av" -> AV, "-ag" -> AG, "-st" -> ST, "-pa" -> PA, else null).
    fn resolve_attribute(token: &str) -> Option<InjuryAttribute> {
        if token.eq_ignore_ascii_case("ni") {
            Some(InjuryAttribute::NI)
        } else if token.eq_ignore_ascii_case("-ma") {
            Some(InjuryAttribute::MA)
        } else if token.eq_ignore_ascii_case("-av") {
            Some(InjuryAttribute::AV)
        } else if token.eq_ignore_ascii_case("-ag") {
            Some(InjuryAttribute::AG)
        } else if token.eq_ignore_ascii_case("-st") {
            Some(InjuryAttribute::ST)
        } else if token.eq_ignore_ascii_case("-pa") {
            Some(InjuryAttribute::PA)
        } else {
            None
        }
    }

    /// Java: `SeriousInjuryFactory.forAttribute(InjuryAttribute)` (plus the
    /// subsequently-read `SeriousInjury.getName()`). Java resolves this through
    /// `game.getFactory(FactoryType.SERIOUS_INJURY)`, a per-game factory instance
    /// already initialized for the game's rules version; here we build and
    /// initialize one on the fly from `game.rules` (cheap, and equivalent since the
    /// factory's contents depend only on that field).
    pub fn resolve_lasting_injury(game: &Game, attr: InjuryAttribute) -> (SeriousInjuryKind, String) {
        let mut factory = SeriousInjuryFactory::new();
        factory.initialize(game);
        let injury = factory
            .for_attribute(attr)
            .expect("Java assumes forAttribute always finds a match here");
        (injury.to_kind(), injury.get_name().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
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

    fn make_team(players: Vec<ffb_model::model::player::Player>) -> Team {
        Team {
            id: "t".into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn handler() -> TalkHandlerInjury {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerInjury::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_resolves_no_attribute_for_unrecognized_token() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let commands = vec!["/injury".to_string(), "clear".to_string(), "1".to_string()];
        let result = h.handle(&team, &commands);
        assert_eq!(result, vec![("p1".to_string(), None)]);
    }

    #[test]
    fn handle_resolves_ni_attribute() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let commands = vec!["/injury".to_string(), "ni".to_string(), "1".to_string()];
        let result = h.handle(&team, &commands);
        assert_eq!(result, vec![("p1".to_string(), Some(InjuryAttribute::NI))]);
    }

    #[test]
    fn handle_resolves_dash_ma_case_insensitively() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let commands = vec!["/injury".to_string(), "-MA".to_string(), "1".to_string()];
        let result = h.handle(&team, &commands);
        assert_eq!(result, vec![("p1".to_string(), Some(InjuryAttribute::MA))]);
    }

    #[test]
    fn handle_targets_no_players_when_none_match() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1)]);
        let commands = vec!["/injury".to_string(), "ni".to_string(), "99".to_string()];
        let result = h.handle(&team, &commands);
        assert!(result.is_empty());
    }

    #[test]
    fn handle_all_keyword_targets_every_player() {
        let h = handler();
        let team = make_team(vec![make_player("p1", 1), make_player("p2", 2)]);
        let commands = vec!["/injury".to_string(), "-av".to_string(), "all".to_string()];
        let result = h.handle(&team, &commands);
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|(_, attr)| *attr == Some(InjuryAttribute::AV)));
    }

    #[test]
    fn resolve_lasting_injury_bb2025_ni_is_serious_injury() {
        let game = Game::new(
            make_team(vec![]),
            make_team(vec![]),
            ffb_model::enums::Rules::Bb2025,
        );
        let (kind, name) = TalkHandlerInjury::resolve_lasting_injury(&game, InjuryAttribute::NI);
        assert_eq!(kind, ffb_model::enums::SeriousInjuryKind::SeriousInjuryNi);
        assert_eq!(name, "Serious Injury (NI)");
    }

    #[test]
    fn resolve_lasting_injury_bb2016_ma_is_first_match_in_declaration_order() {
        let game = Game::new(
            make_team(vec![]),
            make_team(vec![]),
            ffb_model::enums::Rules::Bb2016,
        );
        let (kind, _name) = TalkHandlerInjury::resolve_lasting_injury(&game, InjuryAttribute::MA);
        assert_eq!(kind, ffb_model::enums::SeriousInjuryKind::SmashedHip);
    }
}
