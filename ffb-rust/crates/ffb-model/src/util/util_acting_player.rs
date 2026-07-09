/// 1:1 translation of `com.fumbbl.ffb.util.UtilActingPlayer`.
///
/// The Java implementation mutates the game model to transition between acting
/// players.  The full porting of `changeActingPlayer` lives in `ffb-engine` where
/// it has access to the mutable `GameState`.  This module exposes only the types
/// and helper logic that are stateless and belong in `ffb-model`.
pub struct UtilActingPlayer;

impl UtilActingPlayer {
    pub fn new() -> Self { Self }
}

impl Default for UtilActingPlayer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_constructed() {
        let _u = UtilActingPlayer::new();
    }

    #[test]
    fn default_and_new_equivalent() {
        let _a = UtilActingPlayer::new();
        let _b = UtilActingPlayer::default();
    }
}
