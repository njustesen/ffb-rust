/// 1:1 translation of `com.fumbbl.ffb.util.UtilRangeRuler`.
///
/// The Java `createRangeRuler` method needs a live `Game` reference.  The full
/// implementation lives in `ffb-engine`.  This module retains the struct for
/// structural completeness.
pub struct UtilRangeRuler;

impl UtilRangeRuler {
    pub fn new() -> Self { Self }
}

impl Default for UtilRangeRuler {
    fn default() -> Self { Self::new() }
}
