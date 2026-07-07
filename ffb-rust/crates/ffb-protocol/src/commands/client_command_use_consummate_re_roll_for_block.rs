/// 1:1 translation of ClientCommandUseConsummateReRollForBlock (Java field: proIndex).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseConsummateReRollForBlock {
    pub pro_index: i32,
}

impl ClientCommandUseConsummateReRollForBlock {
    pub fn new(pro_index: i32) -> Self {
        Self { pro_index }
    }

    pub fn get_pro_index(&self) -> i32 {
        self.pro_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_index() {
        let cmd = ClientCommandUseConsummateReRollForBlock::new(3);
        assert_eq!(cmd.get_pro_index(), 3);
    }

    #[test]
    fn default_is_zero() {
        let cmd = ClientCommandUseConsummateReRollForBlock::default();
        assert_eq!(cmd.get_pro_index(), 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseConsummateReRollForBlock::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseConsummateReRollForBlock::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseConsummateReRollForBlock::default());
        assert!(s.contains("ClientCommandUseConsummateReRollForBlock"));
    }
}
