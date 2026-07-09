/// 1:1 translation of com.fumbbl.ffb.IClientPropertyValue (Java interface).
pub trait IClientPropertyValue {
    fn get_value(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl { v: String }
    impl IClientPropertyValue for Impl { fn get_value(&self) -> &str { &self.v } }

    #[test]
    fn get_value_works() {
        assert_eq!(Impl { v: "yes".to_string() }.get_value(), "yes");
    }

    #[test]
    fn empty_value() {
        assert_eq!(Impl { v: String::new() }.get_value(), "");
    }
}
