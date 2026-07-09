use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.KeyedItemRegistry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KeyedItemRegistry {
    pub items: HashMap<String, String>,
}

impl KeyedItemRegistry {
    pub fn register(&mut self, key: String, value: String) { self.items.insert(key, value); }
    pub fn get(&self, key: &str) -> Option<&String> { self.items.get(key) }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn is_empty(&self) -> bool { self.items.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_by_default() {
        assert!(KeyedItemRegistry::default().is_empty());
    }

    #[test]
    fn register_and_get() {
        let mut r = KeyedItemRegistry::default();
        r.register("key1".to_string(), "val1".to_string());
        assert_eq!(r.get("key1").map(|s| s.as_str()), Some("val1"));
    }
}
