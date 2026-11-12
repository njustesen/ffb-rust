/// 1:1 translation of `com.fumbbl.ffb.option.UtilGameOption`.
use crate::model::game::Game;
use crate::option::game_option_id::GameOptionId as GameOptionEnum;

pub fn is_option_enabled(game: &Game, option_id: &str) -> bool {
    // Java: `getOptions().getOptionWithDefault(pOptionId)` materializes the factory
    // default when the option was never explicitly set, then calls `isEnabled()`.
    // A boolean option whose factory default is `true` (e.g.
    // `animalSavageryLashOutEndsActivation`, `clawDoesNotStack`) must therefore read
    // as enabled even when the game never stored an explicit value.
    match GameOptionEnum::for_name(option_id) {
        Some(id) => {
            let value = game.options.get_option_with_default(id).get_value_as_string();
            matches!(value.as_str(), "true" | "1" | "yes")
        }
        // Unknown id: no factory default to fall back to (mirrors Java's null option).
        None => game.options.is_enabled(option_id),
    }
}

pub fn get_int_option(game: &Game, option_id: &str) -> i32 {
    // Java: `(GameOptionInt) getOptions().getOptionWithDefault(pOptionId)` → `getValue()`, so an
    // option the game never stored reads its FACTORY default, exactly as `is_option_enabled` does.
    // Reading 0 instead made e.g. MAX_NR_OF_CARDS (factory 5) look disabled, so bb2016
    // StepBuyCards skipped its whole card phase and never reported `cardsBought` (BACKLOG §I.9).
    match GameOptionEnum::for_name(option_id) {
        Some(id) => game.options.get_option_with_default(id)
            .get_value_as_string().parse().unwrap_or(0),
        // Unknown id: no factory default to fall back to (mirrors Java's null option → 0).
        None => game.options.get_int(option_id).unwrap_or(0),
    }
}

pub fn get_str_option<'a>(game: &'a Game, option_id: &str, default: &'a str) -> &'a str {
    game.options.get(option_id).unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use crate::model::game::Game;
    use crate::model::team::Team;

    fn make_game() -> Game {
        let home = Team {
            id: "home".into(), name: "Home".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        };
        let away = home.clone();
        Game::new(home, away, Rules::Bb2025)
    }

    #[test]
    fn get_int_option_missing_returns_factory_default() {
        // Java getOptionWithDefault: an unset real option reads its GameOptionFactory default.
        let game = make_game();
        assert_eq!(get_int_option(&game, "maxPlayersOnField"), 11);
        assert_eq!(get_int_option(&game, "maxNrOfCards"), 5);
        assert_eq!(get_int_option(&game, "inducementDummy"), 0);
    }

    #[test]
    fn get_int_option_unknown_id_returns_zero() {
        let game = make_game();
        assert_eq!(get_int_option(&game, "noSuchOption"), 0);
    }

    #[test]
    fn get_int_option_set_value() {
        let mut game = make_game();
        game.options.set("maxPlayersOnField", "11");
        assert_eq!(get_int_option(&game, "maxPlayersOnField"), 11);
    }

    #[test]
    fn is_option_enabled_missing_is_false() {
        let game = make_game();
        assert!(!is_option_enabled(&game, "testMode"));
    }

    #[test]
    fn is_option_enabled_set_true() {
        let mut game = make_game();
        game.options.set("testMode", "true");
        assert!(is_option_enabled(&game, "testMode"));
    }
    #[test]
    fn get_str_option_missing_returns_default() {
        let game = make_game();
        assert_eq!(get_str_option(&game, "MISSING", "fallback"), "fallback");
    }

    /// Regression: an unset real option whose `GameOptionFactory` default is `true` must read as
    /// enabled — mirroring Java `UtilGameOption.isOptionEnabled` → `getOptionWithDefault(...)`.
    /// `animalSavageryLashOutEndsActivation` defaults to `true`; the previous implementation only
    /// consulted the stored value and wrongly returned `false`, so an Animal Savagery lash-out
    /// during a Foul failed to end the activation and the foul resolved anyway (2 extra dice).
    #[test]
    fn is_option_enabled_unset_real_option_uses_true_factory_default() {
        use crate::option::game_option_id::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION;
        let game = make_game();
        assert!(is_option_enabled(&game, ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION));
    }

    /// An explicitly-stored value still overrides the factory default in both directions.
    #[test]
    fn is_option_enabled_explicit_false_overrides_true_default() {
        use crate::option::game_option_id::ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION;
        let mut game = make_game();
        game.options.set(ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION, "false");
        assert!(!is_option_enabled(&game, ANIMAL_SAVAGERY_LASH_OUT_ENDS_ACTIVATION));
    }
}
