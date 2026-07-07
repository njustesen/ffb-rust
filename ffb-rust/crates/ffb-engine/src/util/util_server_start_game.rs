// 1:1 translation of com.fumbbl.ffb.server.util.UtilServerStartGame.
//
// Translated methods:
//   add_default_game_options(game) — sets game options for standalone/dev mode
//
// Skipped (touch DB / WebSocket / SessionManager / SequenceGenerator):
//   joinGameAsPlayerAndCheckIfReadyToStart,
//   sendServerJoin, sendUserSettings, startGame

use ffb_model::model::game::Game;

/// Option key constants — mirror Java's GameOptionId names.
pub mod opt {
    pub const PITCH_URL: &str = "PITCH_URL";
    pub const WIZARD_AVAILABLE: &str = "WIZARD_AVAILABLE";
    pub const INDUCEMENT_IGORS_MAX: &str = "INDUCEMENT_IGORS_MAX";
    pub const INDUCEMENT_APOS_MAX: &str = "INDUCEMENT_APOS_MAX";
    pub const MVP_NOMINATIONS: &str = "MVP_NOMINATIONS";
    pub const RULESVERSION: &str = "RULESVERSION";
    pub const INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG: &str =
        "INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG";
    pub const INDUCEMENT_PRAYERS_MAX: &str = "INDUCEMENT_PRAYERS_MAX";
    pub const INDUCEMENT_PRAYERS_COST: &str = "INDUCEMENT_PRAYERS_COST";
    pub const CLAW_DOES_NOT_STACK: &str = "CLAW_DOES_NOT_STACK";
    pub const ALLOW_STAFF_ON_BOTH_TEAMS: &str = "ALLOW_STAFF_ON_BOTH_TEAMS";
    pub const ALLOW_BALL_AND_CHAIN_RE_ROLL: &str = "ALLOW_BALL_AND_CHAIN_RE_ROLL";
    pub const ENABLE_TACKLEZONE_OVERLAYS: &str = "ENABLE_TACKLEZONE_OVERLAYS";
    pub const INDUCEMENT_BRIBES_REDUCED_MAX: &str = "INDUCEMENT_BRIBES_REDUCED_MAX";
    pub const INDUCEMENT_CHEFS_REDUCED_MAX: &str = "INDUCEMENT_CHEFS_REDUCED_MAX";
    pub const MB_STACKS_AGAINST_CHAINSAW: &str = "MB_STACKS_AGAINST_CHAINSAW";
    pub const FORCE_TREASURY_TO_PETTY_CASH: &str = "FORCE_TREASURY_TO_PETTY_CASH";
    pub const PETTY_CASH_AFFECTS_TV: &str = "PETTY_CASH_AFFECTS_TV";
}

pub struct UtilServerStartGame;

impl UtilServerStartGame {
    /// Java: UtilServerStartGame.addDefaultGameOptions(GameState pGameState)
    ///
    /// Populates `game.options` with the default option values used in standalone
    /// (development) mode.  Only the options that are commented-in in the Java
    /// source are set here.
    pub fn add_default_game_options(game: &mut Game) {
        game.options.set(
            opt::PITCH_URL,
            "http://localhost:2224/icons/pitches/fumbblcup.zip",
        );
        game.options.set(opt::WIZARD_AVAILABLE, "true");
        game.options.set(opt::INDUCEMENT_IGORS_MAX, "9");
        game.options.set(opt::INDUCEMENT_APOS_MAX, "9");
        game.options.set(opt::MVP_NOMINATIONS, "6");
        game.options.set(opt::RULESVERSION, "BB2025");
        game.options.set(opt::INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG, "false");
        game.options.set(opt::INDUCEMENT_PRAYERS_MAX, "3");
        game.options.set(opt::INDUCEMENT_PRAYERS_COST, "30000");
        game.options.set(opt::CLAW_DOES_NOT_STACK, "false");
        game.options.set(opt::ALLOW_STAFF_ON_BOTH_TEAMS, "true");
        game.options.set(opt::ALLOW_BALL_AND_CHAIN_RE_ROLL, "true");
        game.options.set(opt::ENABLE_TACKLEZONE_OVERLAYS, "true");
        game.options.set(opt::INDUCEMENT_BRIBES_REDUCED_MAX, "5");
        game.options.set(opt::INDUCEMENT_CHEFS_REDUCED_MAX, "5");
        game.options.set(opt::MB_STACKS_AGAINST_CHAINSAW, "true");
        game.options.set(opt::FORCE_TREASURY_TO_PETTY_CASH, "false");
        game.options.set(opt::PETTY_CASH_AFFECTS_TV, "true");
    }
}

impl Default for UtilServerStartGame {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::opt;
    use ffb_model::enums::Rules;
    use ffb_model::model::game::Game;
    use ffb_model::model::team::Team;

    fn empty_team(id: &str) -> Team {
        Team {
            id: id.into(),
            name: id.into(),
            race: "Human".into(),
            roster_id: "human".into(),
            coach: "coach".into(),
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
            players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn empty_game() -> Game {
        Game::new(empty_team("home"), empty_team("away"), Rules::Bb2025)
    }

    #[test]
    fn add_default_game_options_sets_rulesversion() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert_eq!(game.options.get(opt::RULESVERSION), Some("BB2025"));
    }

    #[test]
    fn add_default_game_options_sets_wizard_available() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert!(game.options.is_enabled(opt::WIZARD_AVAILABLE));
    }

    #[test]
    fn add_default_game_options_sets_mvp_nominations() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert_eq!(game.options.get_int(opt::MVP_NOMINATIONS), Some(6));
    }

    #[test]
    fn add_default_game_options_prayers_not_available_for_underdog() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert!(!game.options.is_enabled(opt::INDUCEMENT_PRAYERS_AVAILABLE_FOR_UNDERDOG));
    }

    #[test]
    fn add_default_game_options_bribes_reduced_max_is_5() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert_eq!(game.options.get_int(opt::INDUCEMENT_BRIBES_REDUCED_MAX), Some(5));
    }

    #[test]
    fn add_default_game_options_allow_staff_on_both_teams() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert!(game.options.is_enabled(opt::ALLOW_STAFF_ON_BOTH_TEAMS));
    }

    #[test]
    fn add_default_game_options_claw_does_not_stack_false() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert!(!game.options.is_enabled(opt::CLAW_DOES_NOT_STACK));
    }

    #[test]
    fn add_default_game_options_prayer_cost() {
        let mut game = empty_game();
        UtilServerStartGame::add_default_game_options(&mut game);
        assert_eq!(game.options.get_int(opt::INDUCEMENT_PRAYERS_COST), Some(30000));
    }
}
