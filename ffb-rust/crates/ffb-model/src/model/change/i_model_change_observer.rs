use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.IModelChangeObserver (Java interface).
pub trait IModelChangeObserver {
    fn on_model_change(&mut self, change: &ModelChange);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    struct NoOp;
    impl IModelChangeObserver for NoOp {
        fn on_model_change(&mut self, _: &ModelChange) {}
    }

    #[test]
    fn noop_compiles() {
        let mut n = NoOp;
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        n.on_model_change(&mc);
    }

    #[test]
    fn noop_does_not_panic() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        NoOp.on_model_change(&mc);
    }
}
