use super::model_change::ModelChange;

/// 1:1 translation of com.fumbbl.ffb.model.change.ModelChangeProcessor.
pub trait ModelChangeProcessor {
    fn process(&mut self, change: &ModelChange);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::{ModelChangeId, ModelChangeDataType};

    struct NoOp;
    impl ModelChangeProcessor for NoOp {
        fn process(&mut self, _: &ModelChange) {}
    }

    #[test]
    fn noop_compiles() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        NoOp.process(&mc);
    }

    #[test]
    fn noop_does_not_panic() {
        let mc = ModelChange::new(ModelChangeId::GameSetActingTeam, ModelChangeDataType::String, serde_json::Value::Null);
        NoOp.process(&mc);
    }
}
