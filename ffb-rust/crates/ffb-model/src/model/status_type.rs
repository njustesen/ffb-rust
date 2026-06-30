use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.StatusType.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusType {
    WAITING,
    REF,
}
