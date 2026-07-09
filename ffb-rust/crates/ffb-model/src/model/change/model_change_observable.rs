use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChangeObservable (Java interface).
pub trait ModelChangeObservable {
    fn notify_change(&mut self, change: &ModelChange);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    struct NoOp;
    impl ModelChangeObservable for NoOp {
        fn notify_change(&mut self, _: &ModelChange) {}
    }

    #[test]
    fn noop_compiles() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        NoOp.notify_change(&mc);
    }

    #[test]
    fn trait_is_object_safe() {
        let _: Option<Box<dyn ModelChangeObservable>> = None;
    }
}
