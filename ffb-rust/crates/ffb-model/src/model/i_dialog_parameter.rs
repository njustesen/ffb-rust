/// 1:1 translation of com.fumbbl.ffb.IDialogParameter (Java interface).
pub trait IDialogParameter {
    fn get_dialog_id(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Impl;
    impl IDialogParameter for Impl { fn get_dialog_id(&self) -> &str { "testDialog" } }

    #[test]
    fn get_dialog_id_works() {
        assert_eq!(Impl.get_dialog_id(), "testDialog");
    }

    #[test]
    fn dialog_id_not_empty() {
        assert!(!Impl.get_dialog_id().is_empty());
    }
}
