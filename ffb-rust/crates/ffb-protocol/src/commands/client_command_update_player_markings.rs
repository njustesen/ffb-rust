/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandUpdatePlayerMarkings`.
/// Sent to update automatic player marking settings.
/// Note: SortMode stored as name string (ffb_model::marking::sort_mode::SortMode available if needed).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUpdatePlayerMarkings {
    /// Java: `auto`
    pub auto: bool,
    /// Java: `sortMode` — stored as name string.
    pub sort_mode_name: Option<String>,
}

impl ClientCommandUpdatePlayerMarkings {
    pub fn new() -> Self { Self::default() }
    pub fn is_auto(&self) -> bool { self.auto }
    pub fn get_sort_mode_name(&self) -> Option<&str> { self.sort_mode_name.as_deref() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_flag() {
        let mut cmd = ClientCommandUpdatePlayerMarkings::new();
        cmd.auto = true;
        assert!(cmd.is_auto());
    }
    #[test]
    fn default_false() {
        assert!(!ClientCommandUpdatePlayerMarkings::new().auto);
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUpdatePlayerMarkings::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUpdatePlayerMarkings::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUpdatePlayerMarkings::default());
        assert!(s.contains("ClientCommandUpdatePlayerMarkings"));
    }
}
