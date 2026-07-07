/// 1:1 translation of `com.fumbbl.ffb.option.UtilGameOption`.
use crate::model::game::Game;

pub fn is_option_enabled(game: &Game, option_id: &str) -> bool {
    game.options.is_enabled(option_id)
}

pub fn get_int_option(game: &Game, option_id: &str) -> i32 {
    game.options.get_int(option_id).unwrap_or(0)
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
    fn get_int_option_missing_returns_zero() {
        let game = make_game();
        assert_eq!(get_int_option(&game, "maxPlayersOnField"), 0);
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
}
