/// 1:1 translation of com.fumbbl.ffb.mechanics.PassResult.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PassResult {
    FUMBLE,
    SAVED_FUMBLE,
    WILDLY_INACCURATE,
    INACCURATE,
    ACCURATE,
}

impl PassResult {
    pub fn get_name(self) -> &'static str {
        match self {
            PassResult::FUMBLE => "FUMBLE",
            PassResult::SAVED_FUMBLE => "SAVED_FUMBLE",
            PassResult::WILDLY_INACCURATE => "WILDLY_INACCURATE",
            PassResult::INACCURATE => "INACCURATE",
            PassResult::ACCURATE => "ACCURATE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accurate_name() {
        assert_eq!(PassResult::ACCURATE.get_name(), "ACCURATE");
    }

    #[test]
    fn fumble_name() {
        assert_eq!(PassResult::FUMBLE.get_name(), "FUMBLE");
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(PassResult::ACCURATE, PassResult::INACCURATE);
    }

    #[test]
    fn all_variants_have_names() {
        assert_eq!(PassResult::SAVED_FUMBLE.get_name(), "SAVED_FUMBLE");
        assert_eq!(PassResult::WILDLY_INACCURATE.get_name(), "WILDLY_INACCURATE");
        assert_eq!(PassResult::INACCURATE.get_name(), "INACCURATE");
    }

    #[test]
    fn clone_and_copy_preserve_equality() {
        let r = PassResult::ACCURATE;
        let c = r;           // copy
        let cl = r.clone();  // clone
        assert_eq!(r, c);
        assert_eq!(r, cl);
    }
}
