use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChangeProcessor.
pub trait ModelChangeProcessor {
    fn process(&mut self, change: &ModelChange);
}
