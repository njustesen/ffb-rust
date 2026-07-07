/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandClearSketches`.
/// Instructs the client to clear all sketches from the field view.
/// Java: no fields — command carries no payload.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandClearSketches;

impl ServerCommandClearSketches {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_be_created() {
        let _ = ServerCommandClearSketches::new();
    }

    #[test]
    fn default_same_as_new() { let _ = ServerCommandClearSketches::default(); }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandClearSketches::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandClearSketches::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandClearSketches::default());
        assert!(s.contains("ServerCommandClearSketches"));
    }
}
