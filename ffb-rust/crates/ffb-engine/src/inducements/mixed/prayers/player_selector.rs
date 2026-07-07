/// 1:1 translation of `com.fumbbl.ffb.server.inducements.mixed.prayers.PlayerSelector`.
/// Java: interface PlayerSelector { List<Player<?>> selectPlayers(Team, Game, int, Set<Skill>) }
/// In Rust, modelled as a trait.
use ffb_model::model::game::Game;
use ffb_model::util::rng::GameRng;
use ffb_model::enums::SkillId;

pub trait PlayerSelector: Send + Sync {
    /// Java: selectPlayers(Team, Game, int nrOfPlayers, Set<Skill> addedSkills)
    /// Returns selected player IDs (up to `nr_of_players`).
    fn select_players(&self, game: &Game, team_id: &str, nr_of_players: i32, rng: &mut GameRng, added_skills: &[SkillId]) -> Vec<String>;
}

/// Null-object selector that selects no players.
/// Used in tests and as a placeholder when a concrete selector is not yet determined.
/// Concrete implementations: `bb2020::prayers::PlayerSelector`, `bb2020::prayers::OpponentPlayerSelector`.
#[derive(Debug, Default)]
pub struct StubPlayerSelector;

impl PlayerSelector for StubPlayerSelector {
    fn select_players(&self, _game: &Game, _team_id: &str, _nr_of_players: i32, _rng: &mut GameRng, _added_skills: &[SkillId]) -> Vec<String> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::game::Game;
    use ffb_model::enums::Rules;

    #[test]
    fn stub_selector_returns_empty() {
        use ffb_model::util::rng::GameRng;
        let selector = StubPlayerSelector;
        let game = Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2020);
        assert!(selector.select_players(&game, "team1", 1, &mut GameRng::new(0), &[]).is_empty());
    }

    #[test]
    fn player_selector_is_object_safe() {
        let _boxed: Box<dyn PlayerSelector> = Box::new(StubPlayerSelector);
    }

    #[test]
    fn stub_is_default_constructible() {
        let _ = StubPlayerSelector::default();
    }

    #[test]
    fn stub_selector_returns_empty_for_multiple_requested() {
        use ffb_model::util::rng::GameRng;
        let selector = StubPlayerSelector;
        let game = Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2020);
        assert!(selector.select_players(&game, "team1", 3, &mut GameRng::new(42), &[]).is_empty());
    }

    #[test]
    fn stub_selector_returns_empty_with_skills() {
        use ffb_model::util::rng::GameRng;
        use ffb_model::enums::SkillId;
        let selector = StubPlayerSelector;
        let game = Game::new(crate::step::framework::test_team("home", 0), crate::step::framework::test_team("away", 0), Rules::Bb2020);
        let skills = [SkillId::Block, SkillId::Dodge];
        assert!(selector.select_players(&game, "team1", 1, &mut GameRng::new(0), &skills).is_empty());
    }
}
