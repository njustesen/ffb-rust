/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandConfirm`.
/// Sent when a coach confirms a dialog choice (no payload).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandConfirm;

impl ClientCommandConfirm {
    pub fn new() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _ = ClientCommandConfirm::new();
    }

    #[test]
    fn default_same_as_new() {
        let _ = ClientCommandConfirm::default();
    }
}
