use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.PushbackMode.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PushbackMode {
    REGULAR,
    SIDE_STEP,
    GRAB,
}

impl PushbackMode {
    pub fn get_name(self) -> &'static str {
        match self {
            PushbackMode::REGULAR => "regular",
            PushbackMode::SIDE_STEP => "sideStep",
            PushbackMode::GRAB => "grab",
        }
    }

    pub fn for_name(name: &str) -> Option<Self> {
        match name {
            "regular" => Some(PushbackMode::REGULAR),
            "sideStep" => Some(PushbackMode::SIDE_STEP),
            "grab" => Some(PushbackMode::GRAB),
            _ => None,
        }
    }
}

impl Default for PushbackMode {
    fn default() -> Self { PushbackMode::REGULAR }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_name_returns_camel_case() {
        assert_eq!(PushbackMode::REGULAR.get_name(), "regular");
        assert_eq!(PushbackMode::SIDE_STEP.get_name(), "sideStep");
        assert_eq!(PushbackMode::GRAB.get_name(), "grab");
    }

    #[test]
    fn for_name_round_trips() {
        assert_eq!(PushbackMode::for_name("sideStep"), Some(PushbackMode::SIDE_STEP));
        assert_eq!(PushbackMode::for_name("invalid"), None);
    }

    #[test]
    fn default_is_regular() {
        assert_eq!(PushbackMode::default(), PushbackMode::REGULAR);
    }

    #[test]
    fn for_name_covers_all_variants() {
        assert_eq!(PushbackMode::for_name("regular"), Some(PushbackMode::REGULAR));
        assert_eq!(PushbackMode::for_name("sideStep"), Some(PushbackMode::SIDE_STEP));
        assert_eq!(PushbackMode::for_name("grab"), Some(PushbackMode::GRAB));
        assert_eq!(PushbackMode::for_name("unknown"), None);
    }

    #[test]
    fn copy_and_clone_produce_equal_values() {
        let original = PushbackMode::SIDE_STEP;
        let copied = original;
        let cloned = original.clone();
        assert_eq!(original, copied);
        assert_eq!(original, cloned);
    }
}
