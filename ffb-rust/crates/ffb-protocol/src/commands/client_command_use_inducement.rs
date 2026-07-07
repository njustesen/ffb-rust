/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUseInducement`.
/// Sent when a coach activates an inducement during the game.
/// Note: InducementType stored as name string; Card stored as name string (full serialisation not yet ported).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseInducement {
    /// Java: `fInducementType` — stored as name string.
    pub inducement_type_name: Option<String>,
    /// Java: `fCard` — stored as card name string.
    pub card_name: Option<String>,
    /// Java: `fPlayerIds`
    pub player_ids: Vec<String>,
}

impl ClientCommandUseInducement {
    pub fn new() -> Self { Self::default() }
    pub fn add_player_id(&mut self, id: impl Into<String>) { self.player_ids.push(id.into()); }
    pub fn get_inducement_type_name(&self) -> Option<&str> { self.inducement_type_name.as_deref() }
    pub fn get_card_name(&self) -> Option<&str> { self.card_name.as_deref() }
    pub fn get_player_ids(&self) -> &[String] { &self.player_ids }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn player_ids_stored() {
        let mut cmd = ClientCommandUseInducement::new();
        cmd.add_player_id("p1");
        assert_eq!(cmd.get_player_ids(), &["p1"]);
    }
    #[test]
    fn default_all_none() {
        let cmd = ClientCommandUseInducement::new();
        assert!(cmd.inducement_type_name.is_none());
        assert!(cmd.player_ids.is_empty());
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseInducement::default()).is_empty());
    }

}
