/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandClearSketches`.
/// Sent when a client clears all field sketches (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandClearSketches;

impl ClientCommandClearSketches {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandClearSketches::new(); }

    #[test]
    fn default_same_as_new() { let _ = ClientCommandClearSketches::default(); }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandClearSketches::new()).is_empty());
    }
}
