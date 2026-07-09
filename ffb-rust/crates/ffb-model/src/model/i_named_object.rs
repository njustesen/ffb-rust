/// 1:1 translation of com.fumbbl.ffb.INamedObject (Java interface).
pub trait INamedObject {
    fn get_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl;
    impl INamedObject for Impl { fn get_name(&self) -> &str { "TestObject" } }

    #[test]
    fn get_name_returns_name() {
        assert_eq!(Impl.get_name(), "TestObject");
    }

    #[test]
    fn name_not_empty() {
        assert!(!Impl.get_name().is_empty());
    }
}
