/// Marker trait for card handler keys — 1:1 translation of Java CardHandlerKey interface.
pub trait CardHandlerKey {}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestKey;
    impl CardHandlerKey for TestKey {}

    #[test]
    fn test_trait_can_be_implemented() {
        let _key = TestKey;
    }

    #[test]
    fn test_trait_object() {
        let _key: &dyn CardHandlerKey = &TestKey;
    }
}
