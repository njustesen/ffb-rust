/// 1:1 translation of com.fumbbl.ffb.model.IKickOffResult (Java interface).
pub trait IKickOffResult {
    fn get_name(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl;
    impl IKickOffResult for Impl { fn get_name(&self) -> &str { "Blitz!" } }

    #[test]
    fn get_name_works() {
        assert_eq!(Impl.get_name(), "Blitz!");
    }

    #[test]
    fn name_not_empty() {
        assert!(!Impl.get_name().is_empty());
    }
}
