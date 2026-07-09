use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.EnhancementRegistry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnhancementRegistry {
    pub enhancements: Vec<String>,
}

impl EnhancementRegistry {
    pub fn register(&mut self, name: String) { self.enhancements.push(name); }
    pub fn is_registered(&self, name: &str) -> bool { self.enhancements.iter().any(|e| e == name) }
    pub fn len(&self) -> usize { self.enhancements.len() }
    pub fn is_empty(&self) -> bool { self.enhancements.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(EnhancementRegistry::default().is_empty());
    }

    #[test]
    fn register_and_check() {
        let mut r = EnhancementRegistry::default();
        r.register("DrainPipe".to_string());
        assert!(r.is_registered("DrainPipe"));
        assert!(!r.is_registered("Unknown"));
    }
}
