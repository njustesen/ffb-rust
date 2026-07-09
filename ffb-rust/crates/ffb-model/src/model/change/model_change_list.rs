use serde::{Deserialize, Serialize};
use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChangeList.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelChangeList {
    pub changes: Vec<ModelChange>,
}

impl ModelChangeList {
    pub fn add(&mut self, change: ModelChange) { self.changes.push(change); }
    pub fn len(&self) -> usize { self.changes.len() }
    pub fn is_empty(&self) -> bool { self.changes.is_empty() }
    pub fn iter(&self) -> impl Iterator<Item = &ModelChange> { self.changes.iter() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    #[test]
    fn empty_by_default() {
        assert!(ModelChangeList::default().is_empty());
    }

    #[test]
    fn add_increases_len() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null));
        assert_eq!(list.len(), 1);
    }
}
