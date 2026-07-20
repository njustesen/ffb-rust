use super::field_model_change_event::FieldModelChangeEvent;

/// 1:1 translation of com.fumbbl.ffb.model.IFieldModelChangeListener.
pub trait IFieldModelChangeListener {
    fn field_model_changed(&mut self, event: &FieldModelChangeEvent);
}
