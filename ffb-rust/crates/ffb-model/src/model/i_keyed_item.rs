/// 1:1 translation of com.fumbbl.ffb.model.IKeyedItem (Java interface).
pub trait IKeyedItem {
    fn get_key(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl { key: String }
    impl IKeyedItem for Impl { fn get_key(&self) -> &str { &self.key } }

    #[test]
    fn get_key_returns_key() {
        assert_eq!(Impl { key: "abc".into() }.get_key(), "abc");
    }

    #[test]
    fn empty_key_ok() {
        assert_eq!(Impl { key: String::new() }.get_key(), "");
    }
}
