use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.modifiers.PlayerStatKey.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerStatKey {
    MA,
    ST,
    AG,
    PA,
    AV,
}
