/// 1:1 translation of com.fumbbl.ffb.IIconProperty (Java interface).
pub trait IIconProperty {
    fn get_icon_path(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl;
    impl IIconProperty for Impl { fn get_icon_path(&self) -> &str { "/icons/test.png" } }

    #[test]
    fn get_icon_path_works() {
        assert_eq!(Impl.get_icon_path(), "/icons/test.png");
    }

    #[test]
    fn icon_path_not_empty() {
        assert!(!Impl.get_icon_path().is_empty());
    }
}
