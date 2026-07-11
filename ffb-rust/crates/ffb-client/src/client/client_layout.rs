//! 1:1 translation of `com.fumbbl.ffb.client.ClientLayout`.
//!
//! TRIAGE CORRECTION: previously classified `—` (Swing-skip) by association with the
//! rendering code that consumes it (`PitchDimensionProvider`/`DimensionProvider`), but the
//! enum itself has zero AWT/Swing dependency — it's plain data (a bool + two doubles per
//! variant). Reclassified `○`→`✓`, same pattern as the `ActionKey` correction in Phase ZW.2.

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientLayout {
    LANDSCAPE,
    PORTRAIT,
    SQUARE,
    WIDE,
}

impl ClientLayout {
    /// Java: constructor field `portrait`.
    pub fn is_portrait(self) -> bool {
        matches!(self, ClientLayout::PORTRAIT | ClientLayout::SQUARE)
    }

    /// Java: constructor field `pitchScale`.
    pub fn pitch_scale(self) -> f64 {
        match self {
            ClientLayout::WIDE => 57.0 / 30.0,
            _ => 1.0,
        }
    }

    /// Java: constructor field `dugoutScale`.
    pub fn dugout_scale(self) -> f64 {
        match self {
            ClientLayout::WIDE => 1.25,
            _ => 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscape_is_not_portrait() {
        assert!(!ClientLayout::LANDSCAPE.is_portrait());
    }

    #[test]
    fn portrait_and_square_are_portrait() {
        assert!(ClientLayout::PORTRAIT.is_portrait());
        assert!(ClientLayout::SQUARE.is_portrait());
    }

    #[test]
    fn wide_is_not_portrait() {
        assert!(!ClientLayout::WIDE.is_portrait());
    }

    #[test]
    fn default_pitch_and_dugout_scale_is_one() {
        assert_eq!(ClientLayout::LANDSCAPE.pitch_scale(), 1.0);
        assert_eq!(ClientLayout::LANDSCAPE.dugout_scale(), 1.0);
    }

    #[test]
    fn wide_scales_match_java_constants() {
        assert!((ClientLayout::WIDE.pitch_scale() - 57.0 / 30.0).abs() < f64::EPSILON);
        assert_eq!(ClientLayout::WIDE.dugout_scale(), 1.25);
    }
}
