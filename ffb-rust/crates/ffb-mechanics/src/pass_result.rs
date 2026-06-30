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
