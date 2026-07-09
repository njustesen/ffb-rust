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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_idle() {
        assert_eq!(SketchState::default(), SketchState::Idle);
    }

    #[test]
    fn done_is_done() {
        assert!(SketchState::Done.is_done());
    }
}
