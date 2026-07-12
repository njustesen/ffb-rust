/// 1:1 translation of com.fumbbl.ffb.server.net.commands.InternalServerCommandCalculateAutomaticPlayerMarkings.
use ffb_engine::marking::auto_marking_config::AutoMarkingConfig;
use ffb_model::model::game::Game;
use super::internal_server_command::InternalServerCommand;

pub struct InternalServerCommandCalculateAutomaticPlayerMarkings {
    /// Java: `autoMarkingConfig` — a real, already-parsed config.
    pub auto_marking_config: AutoMarkingConfig,
    pub index: i32,
    /// Java: `game` — the full `Game` model. `Game` derives serde, so it round-trips via
    /// serde rather than a hand-rolled `to_json_value`/`from_json` pair (same convention as
    /// `ClientCommandLoadAutomaticPlayerMarkings`'s `game` field).
    pub game: Game,
}

impl InternalServerCommandCalculateAutomaticPlayerMarkings {
    /// Java constructor order: `(autoMarkingConfig, index, game)`.
    pub fn new(auto_marking_config: AutoMarkingConfig, index: i32, game: Game) -> Self {
        Self { auto_marking_config, index, game }
    }

    /// Java: `getAutoMarkingConfig()`.
    pub fn get_auto_marking_config(&self) -> &AutoMarkingConfig {
        &self.auto_marking_config
    }

    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// Java: `getGame()`.
    pub fn get_game(&self) -> &Game {
        &self.game
    }
}

impl InternalServerCommand for InternalServerCommandCalculateAutomaticPlayerMarkings {
    fn get_id(&self) -> &'static str {
        "internalCalculateAutomaticPlayerMarkings"
    }

    fn get_game_id(&self) -> i64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::team::Team;

    fn team(id: &str) -> Team {
        Team {
            id: id.into(), name: id.into(), race: "Human".into(), roster_id: "human".into(),
            coach: "coach".into(), rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0, cheerleaders: 0,
            assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0, team_value: 0, treasury: 0,
            special_rules: vec![], players: vec![], vampire_lord: false, necromancer: false,
        }
    }

    fn game() -> Game {
        Game::new(team("home"), team("away"), Rules::Bb2025)
    }

    fn config() -> AutoMarkingConfig {
        let mut c = AutoMarkingConfig::new();
        c.set_separator("/");
        c
    }

    #[test]
    fn construct() {
        let _ = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 0, game());
    }

    #[test]
    fn get_id() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 0, game());
        assert_eq!(c.get_id(), "internalCalculateAutomaticPlayerMarkings");
    }

    #[test]
    fn get_index() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 3, game());
        assert_eq!(c.get_index(), 3);
    }

    #[test]
    fn is_internal() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 0, game());
        assert!(c.is_internal());
    }

    #[test]
    fn get_auto_marking_config() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 0, game());
        assert_eq!(c.get_auto_marking_config().get_separator(), "/");
    }

    #[test]
    fn get_game_returns_carried_game() {
        let c = InternalServerCommandCalculateAutomaticPlayerMarkings::new(config(), 0, game());
        assert_eq!(c.get_game().team_home.id, "home");
    }
}
