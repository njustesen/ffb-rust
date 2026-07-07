/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandIllegalProcedure`.
/// Sent when a coach invokes the Illegal Procedure ruling (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandIllegalProcedure;

impl ClientCommandIllegalProcedure {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _ = ClientCommandIllegalProcedure::new();
    }

    #[test]
    fn default_same_as_new() {
        let _ = ClientCommandIllegalProcedure::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandIllegalProcedure::new()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandIllegalProcedure::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandIllegalProcedure::default());
        assert!(s.contains("ClientCommandIllegalProcedure"));
    }
}
