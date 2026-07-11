/// 1:1 translation of com.fumbbl.ffb.marking.SortMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortMode {
    #[default]
    Default,
    None,
}

impl SortMode {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "DEFAULT" => Some(SortMode::Default),
            "NONE" => Some(SortMode::None),
            _ => None,
        }
    }

    /// Java: `SortMode.name()` (enum constant name).
    pub fn name(self) -> &'static str {
        match self {
            SortMode::Default => "DEFAULT",
            SortMode::None => "NONE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_default() {
        assert_eq!(SortMode::default(), SortMode::Default);
    }

    #[test]
    fn from_name_default() {
        assert_eq!(SortMode::from_name("DEFAULT"), Some(SortMode::Default));
    }

    #[test]
    fn from_name_none() {
        assert_eq!(SortMode::from_name("NONE"), Some(SortMode::None));
    }

    #[test]
    fn from_name_unknown_returns_none() {
        assert!(SortMode::from_name("INVALID").is_none());
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(SortMode::Default, SortMode::None);
    }

    #[test]
    fn from_name_round_trip_none_variant() {
        let mode = SortMode::from_name("NONE").unwrap();
        assert_eq!(mode, SortMode::None);
    }
}
