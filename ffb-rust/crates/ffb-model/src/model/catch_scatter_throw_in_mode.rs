use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.CatchScatterThrowInMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CatchScatterThrowInMode {
    CatchAccurateBomb,
    CatchAccurateBombEmptySquare,
    CatchAccuratePass,
    CatchAccuratePassEmptySquare,
    CatchBomb,
    CatchHandOff,
    CatchKickoff,
    CatchMissedPass,
    CatchPunt,
    CatchScatter,
    CatchThrowIn,
    Deflected,
    DeflectedBomb,
    FailedCatch,
    FailedPickUp,
    FailedDeflectionConversion,
    ScatterBall,
    ThreeSquareScatter,
    ThrowIn,
}

impl CatchScatterThrowInMode {
    pub fn get_name(self) -> &'static str {
        match self {
            CatchScatterThrowInMode::CatchAccurateBomb => "catchAccurateBomb",
            CatchScatterThrowInMode::CatchAccurateBombEmptySquare => "catchAccurateBombEmptySquare",
            CatchScatterThrowInMode::CatchAccuratePass => "catchAccuratePass",
            CatchScatterThrowInMode::CatchAccuratePassEmptySquare => "catchAccuratePassEmptySquare",
            CatchScatterThrowInMode::CatchBomb => "catchBomb",
            CatchScatterThrowInMode::CatchHandOff => "catchHandOff",
            CatchScatterThrowInMode::CatchKickoff => "catchKickoff",
            CatchScatterThrowInMode::CatchMissedPass => "catchMissedPass",
            CatchScatterThrowInMode::CatchPunt => "catchPunt",
            CatchScatterThrowInMode::CatchScatter => "catchScatter",
            CatchScatterThrowInMode::CatchThrowIn => "catchThrowIn",
            CatchScatterThrowInMode::Deflected => "deflected",
            CatchScatterThrowInMode::DeflectedBomb => "deflectedBomb",
            CatchScatterThrowInMode::FailedCatch => "failedCatch",
            CatchScatterThrowInMode::FailedPickUp => "failedPickUp",
            CatchScatterThrowInMode::FailedDeflectionConversion => "failedDeflectionConversion",
            CatchScatterThrowInMode::ScatterBall => "scatterBall",
            CatchScatterThrowInMode::ThreeSquareScatter => "threeSquareScatter",
            CatchScatterThrowInMode::ThrowIn => "throwIn",
        }
    }

    pub fn is_bomb(self) -> bool {
        matches!(self,
            CatchScatterThrowInMode::CatchAccurateBomb
            | CatchScatterThrowInMode::CatchAccurateBombEmptySquare
            | CatchScatterThrowInMode::CatchBomb
            | CatchScatterThrowInMode::DeflectedBomb
        )
    }

    pub fn for_name(name: &str) -> Option<Self> {
        Self::all().iter().copied().find(|v| v.get_name() == name)
    }

    pub fn all() -> &'static [Self] {
        &[
            CatchScatterThrowInMode::CatchAccurateBomb,
            CatchScatterThrowInMode::CatchAccurateBombEmptySquare,
            CatchScatterThrowInMode::CatchAccuratePass,
            CatchScatterThrowInMode::CatchAccuratePassEmptySquare,
            CatchScatterThrowInMode::CatchBomb,
            CatchScatterThrowInMode::CatchHandOff,
            CatchScatterThrowInMode::CatchKickoff,
            CatchScatterThrowInMode::CatchMissedPass,
            CatchScatterThrowInMode::CatchPunt,
            CatchScatterThrowInMode::CatchScatter,
            CatchScatterThrowInMode::CatchThrowIn,
            CatchScatterThrowInMode::Deflected,
            CatchScatterThrowInMode::DeflectedBomb,
            CatchScatterThrowInMode::FailedCatch,
            CatchScatterThrowInMode::FailedPickUp,
            CatchScatterThrowInMode::FailedDeflectionConversion,
            CatchScatterThrowInMode::ScatterBall,
            CatchScatterThrowInMode::ThreeSquareScatter,
            CatchScatterThrowInMode::ThrowIn,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_accurate_bomb_is_bomb() {
        assert!(CatchScatterThrowInMode::CatchAccurateBomb.is_bomb());
    }

    #[test]
    fn catch_scatter_is_not_bomb() {
        assert!(!CatchScatterThrowInMode::CatchScatter.is_bomb());
    }

    #[test]
    fn for_name_round_trip() {
        assert_eq!(CatchScatterThrowInMode::for_name("catchHandOff"), Some(CatchScatterThrowInMode::CatchHandOff));
    }

    #[test]
    fn for_name_unknown_returns_none() {
        assert_eq!(CatchScatterThrowInMode::for_name("unknown"), None);
    }
}
