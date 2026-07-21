use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.SketchState.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SketchState {
    Idle,
    Drawing,
    Done,
}

impl Default for SketchState {
    fn default() -> Self { Self::Idle }
}

impl SketchState {
    pub fn is_done(self) -> bool { self == Self::Done }
}
