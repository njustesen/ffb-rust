/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseProReRollForBlock`.
/// Sent when Pro skill re-roll is used for a specific block die.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseProReRollForBlock {
    /// Java: `proIndex`
    pub pro_index: i32,
}

impl ClientCommandUseProReRollForBlock {
    pub fn new(pro_index: i32) -> Self { Self { pro_index } }
    pub fn get_pro_index(&self) -> i32 { self.pro_index }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn index_stored() {
        assert_eq!(ClientCommandUseProReRollForBlock::new(2).get_pro_index(), 2);
    }
    #[test]
    fn default_zero() {
        assert_eq!(ClientCommandUseProReRollForBlock::default().pro_index, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseProReRollForBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseProReRollForBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseProReRollForBlock::default());
        assert!(s.contains("ClientCommandUseProReRollForBlock"));
    }
}
