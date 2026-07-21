use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.BreatheFireResult.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BreatheFireResult {
    FAILURE,
    NO_EFFECT,
    PRONE,
    KNOCK_DOWN,
}

impl BreatheFireResult {
    pub fn get_message(self) -> &'static str {
        match self {
            BreatheFireResult::FAILURE => "You would be knocked down causing a turnover",
            BreatheFireResult::NO_EFFECT => "The current result would have no effect",
            BreatheFireResult::PRONE => "Opponent would be place prone without armour roll",
            BreatheFireResult::KNOCK_DOWN => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_round_trip() {
        for v in [BreatheFireResult::FAILURE, BreatheFireResult::NO_EFFECT, BreatheFireResult::PRONE, BreatheFireResult::KNOCK_DOWN] {
            let s = serde_json::to_string(&v).unwrap();
            let back: BreatheFireResult = serde_json::from_str(&s).unwrap();
            assert_eq!(v, back);
        }
    }

}
