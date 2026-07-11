/// 1:1 translation of com.fumbbl.ffb.server.handler.talk.TalkHandlerBox.
/// Abstract handler for /box command — moves players to dugout with given status.
///
/// Java's `handle()` looks up the edition's `RollMechanic` via
/// `game.getFactory(FactoryType.Factory.MECHANIC).forName(Mechanic.Type.ROLL.name())`;
/// the Rust port picks the concrete `RollMechanic` impl directly from `game.rules`
/// (no generic factory-by-name lookup exists yet in this crate).
///
/// Java's `mechanic.rollCasualty(gameState.getDiceRoller())` uses the server
/// `DiceRoller`, but the ported `RollMechanic` trait (ffb-engine) already takes
/// `&mut GameRng` instead (established elsewhere in ffb-engine, not invented here) —
/// the caller passes the driver's `GameRng` explicitly.
///
/// Java's `SeriousInjuryFactory.dead()` (for the "rip" command) has a direct
/// Rust equivalent: `SeriousInjuryKind::Dead`, so no factory lookup is needed.
///
/// Java's `handle()` also calls `UtilServerGame.syncGameModel(...)` at the end
/// — that's the caller's responsibility once sync-to-client infra exists (see
/// `talk_handler_activated.rs` for the same documented adaptation).
use std::collections::HashSet;
use ffb_engine::injury::InjuryContext;
use ffb_engine::mechanic::roll_mechanic::RollMechanic as RollMechanicTrait;
use ffb_engine::mechanic::bb2016::roll_mechanic::RollMechanic as RollMechanicBb2016;
use ffb_engine::mechanic::bb2020::roll_mechanic::RollMechanic as RollMechanicBb2020;
use ffb_engine::mechanic::bb2025::roll_mechanic::RollMechanic as RollMechanicBb2025;
use ffb_model::enums::{
    ApothecaryMode, PlayerState, Rules, SeriousInjuryKind,
    PS_BADLY_HURT, PS_BANNED, PS_KNOCKED_OUT, PS_RESERVE, PS_RIP, PS_SERIOUS_INJURY,
};
use ffb_model::model::game::Game;
use ffb_model::model::team::Team;
use ffb_model::util::rng::GameRng;
use crate::handler::talk::command_adapter::CommandAdapter;
use crate::handler::talk::talk_handler::TalkHandler;
use crate::handler::talk::talk_requirements::{Client, Environment, Privilege};

pub struct TalkHandlerBox {
    base: TalkHandler,
}

impl TalkHandlerBox {
    /// Java: `TalkHandlerBox(CommandAdapter, Client, Environment, Privilege...)`.
    pub fn new(
        command_adapter: &dyn CommandAdapter,
        required_client: Client,
        required_env: Environment,
        requires_one_privilege_of: HashSet<Privilege>,
    ) -> Self {
        let mut commands = HashSet::new();
        commands.insert("/box".to_string());
        let commands = command_adapter.decorate_commands(commands);
        Self {
            base: TalkHandler::new(commands, 2, required_client, required_env, requires_one_privilege_of),
        }
    }

    pub fn base(&self) -> &TalkHandler { &self.base }

    fn mechanic_for(rules: Rules) -> Box<dyn RollMechanicTrait> {
        match rules {
            Rules::Bb2016 => Box::new(RollMechanicBb2016::new()),
            Rules::Bb2020 => Box::new(RollMechanicBb2020::new()),
            _ => Box::new(RollMechanicBb2025::new()),
        }
    }

    /// Java: `handle(FantasyFootballServer, GameState, String[], Team, Session)` —
    /// puts each referenced player into the box with the requested status;
    /// returns the info message(s) Java would have sent via
    /// `communication.sendPlayerTalk`.
    pub fn handle(&self, game: &mut Game, rng: &mut GameRng, commands: &[String], team: &Team) -> Vec<String> {
        let mechanic = Self::mechanic_for(game.rules);
        let is_home = game.is_home_team(&team.id);
        let mut info = Vec::new();
        let player_ids: Vec<String> = self.base.find_players_in_command(team, commands)
            .into_iter().map(|p| p.id.clone()).collect();

        for player_id in player_ids {
            let status = commands[1].to_ascii_lowercase();
            let (state, box_name, serious_injury): (PlayerState, &str, Option<SeriousInjuryKind>) = match status.as_str() {
                "rsv" => (PlayerState::new(PS_RESERVE), "Reserve", None),
                "ko" => (PlayerState::new(PS_KNOCKED_OUT), "Knocked Out", None),
                "bh" => (PlayerState::new(PS_BADLY_HURT), "Badly Hurt", None),
                "si" => {
                    let roll = mechanic.roll_casualty(rng);
                    // Java `new InjuryContext()` doesn't set an apothecary mode either
                    // (field stays null); the mode is unused by interpretSeriousInjuryRoll.
                    let ctx = InjuryContext::new(ApothecaryMode::Home);
                    let serious_injury = mechanic.interpret_serious_injury_roll_explicit(game, &ctx, roll);
                    (PlayerState::new(PS_SERIOUS_INJURY), "Serious Injury", serious_injury)
                }
                "rip" => (PlayerState::new(PS_RIP), "RIP", Some(SeriousInjuryKind::Dead)),
                "ban" => (PlayerState::new(PS_BANNED), "Banned", None),
                _ => break,
            };

            {
                let player_result = game.game_result.team_result_mut(is_home).player_result_mut(&player_id);
                player_result.serious_injury = serious_injury;
                player_result.serious_injury_decay = None;
            }
            info.push(self.base.put_player_into_box(game, &player_id, state, box_name));
        }
        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::{PlayerType, PlayerGender};
    use ffb_model::types::FieldCoordinate;
    use ffb_model::model::player::Player;
    use crate::handler::talk::identity_command_adapter::IdentityCommandAdapter;
    use std::collections::HashSet as Set;

    fn make_player(id: &str, nr: i32) -> Player {
        Player {
            id: id.into(), name: format!("Player{nr}"), nr, position_id: "lineman".into(),
            player_type: PlayerType::Regular, gender: PlayerGender::Male,
            movement: 6, strength: 3, agility: 3, passing: 4, armour: 8,
            starting_skills: vec![], extra_skills: vec![], temporary_skills: vec![],
            used_skills: Set::new(), niggling_injuries: 0, stat_injuries: vec![],
            current_spps: 0, career_spps: 0, race: None, is_big_guy: false,
            ..Default::default()
        }
    }

    fn make_team(id: &str, players: Vec<Player>) -> Team {
        Team {
            id: id.into(), name: "Team".into(), race: "Human".into(), roster_id: "human".into(),
            coach: "Coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players, vampire_lord: false, necromancer: false,
        }
    }

    fn handler() -> TalkHandlerBox {
        let adapter = IdentityCommandAdapter::new();
        TalkHandlerBox::new(&adapter, Client::Player, Environment::None, HashSet::new())
    }

    #[test]
    fn construct() { let _ = handler(); }

    #[test]
    fn handle_reserve_moves_player_into_box() {
        let h = handler();
        let home = make_team("home", vec![make_player("h1", 1)]);
        let mut game = Game::new(home.clone(), make_team("away", vec![]), Rules::Bb2025);
        game.field_model.set_player_coordinate("h1", FieldCoordinate::new(5, 5));
        let mut rng = GameRng::new(1);
        let commands = vec!["/box".to_string(), "rsv".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &home);
        assert_eq!(info.len(), 1);
        assert!(info[0].contains("moved into box Reserve"));
        assert!(game.field_model.player_coordinate("h1").unwrap().is_box_coordinate());
    }

    #[test]
    fn handle_ko_sets_knocked_out_state() {
        let h = handler();
        let home = make_team("home", vec![make_player("h1", 1)]);
        let mut game = Game::new(home.clone(), make_team("away", vec![]), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let commands = vec!["/box".to_string(), "ko".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &home);
        assert!(info[0].contains("Knocked Out"));
        assert_eq!(game.field_model.player_state("h1").unwrap().base(), PS_KNOCKED_OUT);
    }

    #[test]
    fn handle_rip_sets_dead_serious_injury() {
        let h = handler();
        let home = make_team("home", vec![make_player("h1", 1)]);
        let mut game = Game::new(home.clone(), make_team("away", vec![]), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let commands = vec!["/box".to_string(), "rip".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &home);
        assert!(info[0].contains("RIP"));
        let pr = game.game_result.team_result(true).player_result("h1").unwrap();
        assert_eq!(pr.serious_injury, Some(SeriousInjuryKind::Dead));
    }

    #[test]
    fn handle_si_rolls_and_records_serious_injury() {
        let h = handler();
        let home = make_team("home", vec![make_player("h1", 1)]);
        let mut game = Game::new(home.clone(), make_team("away", vec![]), Rules::Bb2025);
        let mut rng = GameRng::new(7);
        let commands = vec!["/box".to_string(), "si".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &home);
        assert!(info[0].contains("Serious Injury"));
        assert_eq!(game.field_model.player_state("h1").unwrap().base(), PS_SERIOUS_INJURY);
    }

    #[test]
    fn handle_unrecognized_status_stops_without_info() {
        let h = handler();
        let home = make_team("home", vec![make_player("h1", 1)]);
        let mut game = Game::new(home.clone(), make_team("away", vec![]), Rules::Bb2025);
        let mut rng = GameRng::new(1);
        let commands = vec!["/box".to_string(), "bogus".to_string(), "1".to_string()];
        let info = h.handle(&mut game, &mut rng, &commands, &home);
        assert!(info.is_empty());
    }
}
