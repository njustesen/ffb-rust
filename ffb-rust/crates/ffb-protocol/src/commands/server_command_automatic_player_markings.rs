use std::collections::HashMap;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ServerCommandAutomaticPlayerMarkings`.
/// Sends automatic player markings (player_id → marking colour) to the client.
#[derive(Debug, Clone, Default)]
pub struct ServerCommandAutomaticPlayerMarkings {
    /// Java: `markings` — map of player_id → marking colour string.
    pub markings: HashMap<String, String>,
    /// Java: `index` — which markings set index this applies to.
    pub index: i32,
}

impl ServerCommandAutomaticPlayerMarkings {
    pub fn new(markings: HashMap<String, String>, index: i32) -> Self {
        Self { markings, index }
    }
    pub fn get_markings(&self) -> &HashMap<String, String> { &self.markings }
    pub fn get_index(&self) -> i32 { self.index }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_stored() {
        let mut map = HashMap::new();
        map.insert("p1".into(), "red".into());
        let cmd = ServerCommandAutomaticPlayerMarkings::new(map.clone(), 2);
        assert_eq!(cmd.get_index(), 2);
        assert_eq!(cmd.get_markings().get("p1").map(|s| s.as_str()), Some("red"));
    }

    #[test]
    fn default_empty() {
        let cmd = ServerCommandAutomaticPlayerMarkings::default();
        assert!(cmd.markings.is_empty());
        assert_eq!(cmd.index, 0);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ServerCommandAutomaticPlayerMarkings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ServerCommandAutomaticPlayerMarkings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ServerCommandAutomaticPlayerMarkings::default());
        assert!(s.contains("ServerCommandAutomaticPlayerMarkings"));
    }
}
