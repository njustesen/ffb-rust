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

    /// Java `getChanges()`.
    pub fn get_changes(&self) -> &[ModelChange] { &self.changes }

    /// Java `clear()`.
    pub fn clear(&mut self) { self.changes.clear(); }

    /// Java `ModelChangeList.initFrom`: clear, then one `ModelChange.initFrom` per element of
    /// `modelChangeArray` (absent/null -> JsonArrayOption's empty array).
    pub fn init_from(&mut self, json: &serde_json::Value) -> Result<(), String> {
        self.clear();
        let obj = json.as_object().ok_or_else(|| "modelChangeList: not an object".to_string())?;
        match obj.get("modelChangeArray") {
            None | Some(serde_json::Value::Null) => {}
            Some(serde_json::Value::Array(arr)) => {
                for mc in arr {
                    self.add(ModelChange::init_from(mc)?);
                }
            }
            Some(other) => return Err(format!("modelChangeArray: not an array: {other}")),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    #[test]
    fn add_increases_len() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::GameSetHomePlaying, ModelChangeDataType::String, serde_json::Value::Null));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn init_from_clears_and_reads_in_order() {
        let mut list = ModelChangeList::default();
        list.add(ModelChange::new(ModelChangeId::GameSetHomePlaying, ModelChangeDataType::Boolean, serde_json::Value::Null));
        list.init_from(&serde_json::json!({"modelChangeArray": [
            {"modelChangeId": "gameSetHalf", "modelChangeValue": 1},
            {"modelChangeId": "turnDataSetTurnNr", "modelChangeKey": "home", "modelChangeValue": 3}]})).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.get_changes()[0].change_id, ModelChangeId::GameSetHalf);
        assert_eq!(list.get_changes()[1].get_key(), Some("home"));
        list.init_from(&serde_json::json!({})).unwrap();
        assert!(list.is_empty());
    }
}
