use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.IModelChangeObserver (Java interface).
pub trait IModelChangeObserver {
    fn on_model_change(&mut self, change: &ModelChange);
}
