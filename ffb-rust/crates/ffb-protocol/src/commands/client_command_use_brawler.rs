/// 1:1 translation of ClientCommandUseBrawler (Java field: targetId).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseBrawler {
    pub target_id: Option<String>,
}

impl ClientCommandUseBrawler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_target_id(id: impl Into<String>) -> Self {
        Self { target_id: Some(id.into()) }
    }

    pub fn get_target_id(&self) -> Option<&str> {
        self.target_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_target_id() {
        let cmd = ClientCommandUseBrawler::new();
        assert!(cmd.get_target_id().is_none());
    }

    #[test]
    fn with_target_id_stores_value() {
        let cmd = ClientCommandUseBrawler::with_target_id("t-1");
        assert_eq!(cmd.get_target_id(), Some("t-1"));
    }
}
