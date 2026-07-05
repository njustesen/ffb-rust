/// 1:1 translation of com.fumbbl.ffb.server.model.change.ConditionalModelChangeObserver.
///
/// Java: interface INamedObject + void next(GameState, ModelChange).
/// Rust: trait with default `get_name()` and abstract `next()`.
use ffb_model::enums::ModelChangeId;
use ffb_model::model::game::Game;

pub trait ConditionalModelChangeObserver: Send + Sync {
    /// Java: default String getName() { return getClass().getSimpleName(); }
    fn get_name(&self) -> &str;

    /// Java: void next(GameState gameState, ModelChange modelChange).
    /// key = modelChange.getKey() (typically a player ID).
    fn next(&self, key: Option<&str>, change_id: ModelChangeId, game: &mut Game);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::step::framework::test_team;
    use ffb_model::enums::Rules;

    struct DummyObserver;

    impl ConditionalModelChangeObserver for DummyObserver {
        fn get_name(&self) -> &str { "DummyObserver" }
        fn next(&self, _key: Option<&str>, _change_id: ModelChangeId, _game: &mut Game) {}
    }

    fn make_game() -> Game {
        Game::new(test_team("home", 0), test_team("away", 0), Rules::Bb2025)
    }

    #[test]
    fn dummy_observer_get_name() {
        let obs = DummyObserver;
        assert_eq!(obs.get_name(), "DummyObserver");
    }

    #[test]
    fn dummy_observer_next_does_not_panic() {
        let obs = DummyObserver;
        let mut game = make_game();
        obs.next(Some("player-1"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
    }
}
