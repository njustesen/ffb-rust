pub trait IGameIdListener {
    fn set_game_id(&mut self, game_id: i64);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockListener {
        game_id: i64,
    }

    impl IGameIdListener for MockListener {
        fn set_game_id(&mut self, game_id: i64) {
            self.game_id = game_id;
        }
    }

    #[test]
    fn test_set_game_id() {
        let mut listener = MockListener { game_id: 0 };
        listener.set_game_id(42);
        assert_eq!(listener.game_id, 42);
    }

    #[test]
    fn test_set_game_id_zero() {
        let mut listener = MockListener { game_id: 99 };
        listener.set_game_id(0);
        assert_eq!(listener.game_id, 0);
    }
}
