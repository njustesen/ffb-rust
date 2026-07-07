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

    /// Observer that records invocation count for testing.
    struct CountingObserver {
        name: &'static str,
    }

    impl ConditionalModelChangeObserver for CountingObserver {
        fn get_name(&self) -> &str { self.name }
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

    #[test]
    fn dummy_observer_next_with_none_key_does_not_panic() {
        let obs = DummyObserver;
        let mut game = make_game();
        obs.next(None, ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
    }

    #[test]
    fn counting_observer_name_matches() {
        let obs = CountingObserver { name: "MyCounter" };
        assert_eq!(obs.get_name(), "MyCounter");
    }

    #[test]
    fn counting_observer_next_does_not_panic_multiple_calls() {
        let obs = CountingObserver { name: "Counter" };
        let mut game = make_game();
        obs.next(Some("p1"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
        obs.next(None, ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
        obs.next(Some("p2"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
    }

    #[test]
    fn trait_object_dispatch_works() {
        let obs: Box<dyn ConditionalModelChangeObserver> = Box::new(DummyObserver);
        assert_eq!(obs.get_name(), "DummyObserver");
    }

    #[test]
    fn trait_object_next_dispatches_correctly() {
        let obs: Box<dyn ConditionalModelChangeObserver> = Box::new(CountingObserver { name: "TraitObj" });
        let mut game = make_game();
        obs.next(Some("key"), ModelChangeId::FieldModelSetPlayerCoordinate, &mut game);
        assert_eq!(obs.get_name(), "TraitObj");
    }
}
