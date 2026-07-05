/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandBlockChoice`.
/// Sent when the attacker selects which block die result to use.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandBlockChoice {
    /// Java: `fDiceIndex` — index of the chosen die result.
    pub dice_index: i32,
}

impl ClientCommandBlockChoice {
    pub fn new(dice_index: i32) -> Self {
        Self { dice_index }
    }

    /// Java: `getDiceIndex()`
    pub fn get_dice_index(&self) -> i32 { self.dice_index }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_index_stored() {
        let cmd = ClientCommandBlockChoice::new(2);
        assert_eq!(cmd.get_dice_index(), 2);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandBlockChoice::default();
        assert_eq!(cmd.dice_index, 0);
    }
}
