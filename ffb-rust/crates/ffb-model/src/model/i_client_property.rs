/// 1:1 translation of com.fumbbl.ffb.IClientProperty (Java interface).
pub trait IClientProperty {
    fn get_key(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl { key: String }
    impl IClientProperty for Impl { fn get_key(&self) -> &str { &self.key } }

    #[test]
    fn get_key_works() {
        assert_eq!(Impl { key: "k".to_string() }.get_key(), "k");
    }

    #[test]
    fn empty_key() {
        assert_eq!(Impl { key: String::new() }.get_key(), "");
    }
}
