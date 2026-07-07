/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseTeamMatesWisdom`.
/// Sent when a BB2025 player uses Team Mates Wisdom (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseTeamMatesWisdom;

impl ClientCommandUseTeamMatesWisdom {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_construct() { let _ = ClientCommandUseTeamMatesWisdom::new(); }
    #[test]
    fn default_works() { let _ = ClientCommandUseTeamMatesWisdom::default(); }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseTeamMatesWisdom::default()).is_empty());
    }

}
