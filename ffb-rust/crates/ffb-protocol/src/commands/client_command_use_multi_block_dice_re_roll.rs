/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseMultiBlockDiceReRoll`.
/// Sent to re-roll specific dice in a multi-block action.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseMultiBlockDiceReRoll {
    /// Java: `diceIndexes`
    pub dice_indexes: Vec<i32>,
}

impl ClientCommandUseMultiBlockDiceReRoll {
    pub fn new() -> Self { Self::default() }
    pub fn with_indexes(dice_indexes: Vec<i32>) -> Self { Self { dice_indexes } }
    pub fn get_dice_indexes(&self) -> &[i32] { &self.dice_indexes }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn indexes_stored() {
        let cmd = ClientCommandUseMultiBlockDiceReRoll::with_indexes(vec![0, 2]);
        assert_eq!(cmd.get_dice_indexes(), &[0, 2]);
    }
    #[test]
    fn default_empty() {
        assert!(ClientCommandUseMultiBlockDiceReRoll::new().dice_indexes.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseMultiBlockDiceReRoll::default()).is_empty());
    }

}
