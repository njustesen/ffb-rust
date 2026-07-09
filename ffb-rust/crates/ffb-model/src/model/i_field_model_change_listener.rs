use super::field_model_change_event::FieldModelChangeEvent;

/// 1:1 translation of com.fumbbl.ffb.model.IFieldModelChangeListener.
pub trait IFieldModelChangeListener {
    fn field_model_changed(&mut self, event: &FieldModelChangeEvent);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::field_model_change_event::FieldModelChangeEvent;

    struct NoOp;
    impl IFieldModelChangeListener for NoOp {
        fn field_model_changed(&mut self, _: &FieldModelChangeEvent) {}
    }

    #[test]
    fn noop_impl_compiles() {
        let mut n = NoOp;
        n.field_model_changed(&FieldModelChangeEvent::default());
    }

    #[test]
    fn noop_does_not_panic() {
        NoOp.field_model_changed(&FieldModelChangeEvent::default());
    }
}
