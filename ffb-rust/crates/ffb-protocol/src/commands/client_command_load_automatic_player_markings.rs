/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandLoadAutomaticPlayerMarkings.
///
/// Java: `game` field is the full Game object — omitted; Game serialisation not yet ported.
/// Only `index` and `coach` are stored for now.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandLoadAutomaticPlayerMarkings {
    /// Java: `index`
    pub index: i32,
    /// Java: `coach`
    pub coach: Option<String>,
    // Java `game` field omitted — Game serialisation requires full model port (not yet done)
}

impl ClientCommandLoadAutomaticPlayerMarkings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getIndex()`
    pub fn get_index(&self) -> i32 {
        self.index
    }

    /// Java: `getCoach()`
    pub fn get_coach(&self) -> Option<&str> {
        self.coach.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_index_is_zero() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::new();
        assert_eq!(cmd.get_index(), 0);
    }

    #[test]
    fn stores_index_and_coach() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings {
            index: 3,
            coach: Some("CoachB".to_string()),
        };
        assert_eq!(cmd.get_index(), 3);
        assert_eq!(cmd.get_coach(), Some("CoachB"));
    }

    #[test]
    fn coach_none_by_default() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::default();
        assert!(cmd.get_coach().is_none());
    }

    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandLoadAutomaticPlayerMarkings::default();
        assert!(!format!("{cmd:?}").is_empty());
    }
}
