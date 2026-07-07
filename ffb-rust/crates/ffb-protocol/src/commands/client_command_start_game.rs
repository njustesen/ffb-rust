/// 1:1 translation of ClientCommandStartGame (Java: no fields).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandStartGame;

impl ClientCommandStartGame {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandStartGame::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandStartGame::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandStartGame::default()).is_empty());
    }

}
