/// Factory for BB2025 deferred command IDs — 1:1 translation of Java DeferredCommandIdFactory (bb2025).
pub struct DeferredCommandIdFactory {
    ids: Vec<String>,
}

impl DeferredCommandIdFactory {
    pub fn new() -> Self {
        Self { ids: Vec::new() }
    }

    pub fn for_name(&self, name: &str) -> Option<&str> {
        self.ids.iter().find(|id| id.as_str() == name).map(|s| s.as_str())
    }

    pub fn all(&self) -> &[String] {
        &self.ids
    }

    pub fn register(&mut self, id: impl Into<String>) {
        self.ids.push(id.into());
    }
}

impl Default for DeferredCommandIdFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_factory() {
        let factory = DeferredCommandIdFactory::new();
        assert!(factory.all().is_empty());
    }

    #[test]
    fn test_register_and_lookup() {
        let mut factory = DeferredCommandIdFactory::new();
        factory.register("PASS_BLOCK");
        assert_eq!(factory.for_name("PASS_BLOCK"), Some("PASS_BLOCK"));
        assert!(factory.for_name("UNKNOWN").is_none());
    }
}
