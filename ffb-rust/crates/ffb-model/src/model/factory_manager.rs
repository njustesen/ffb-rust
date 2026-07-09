use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.FactoryManager.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactoryManager {
    pub factory_types: Vec<String>,
}

impl FactoryManager {
    pub fn register(&mut self, factory_type: String) { self.factory_types.push(factory_type); }
    pub fn len(&self) -> usize { self.factory_types.len() }
    pub fn is_empty(&self) -> bool { self.factory_types.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(FactoryManager::default().is_empty());
    }

    #[test]
    fn register_increases_len() {
        let mut fm = FactoryManager::default();
        fm.register("BlockFactory".to_string());
        assert_eq!(fm.len(), 1);
    }
}
