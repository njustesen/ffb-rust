/// 1:1 translation of com.fumbbl.ffb.mechanics.StatsDrawingModifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatsDrawingModifier {
    improvement: bool,
    impairment: bool,
    absolute_modifier: i32,
}

impl StatsDrawingModifier {
    fn new_inner(improvement: bool, impairment: bool, absolute_modifier: i32) -> Self {
        StatsDrawingModifier { improvement, impairment, absolute_modifier }
    }

    pub fn positive_improves(modifier: i32) -> Self {
        if modifier == 0 { Self::neutral() }
        else if modifier > 0 { Self::improvement(modifier) }
        else { Self::impairment(modifier) }
    }

    pub fn positive_impairs(modifier: i32) -> Self {
        if modifier == 0 { Self::neutral() }
        else if modifier < 0 { Self::improvement(modifier) }
        else { Self::impairment(modifier) }
    }

    fn neutral() -> Self { Self::new_inner(false, false, 0) }
    fn improvement(modifier: i32) -> Self { Self::new_inner(true, false, modifier.unsigned_abs() as i32) }
    fn impairment(modifier: i32) -> Self { Self::new_inner(false, true, modifier.unsigned_abs() as i32) }

    pub fn is_improvement(&self) -> bool { self.improvement }
    pub fn is_impairment(&self) -> bool { self.impairment }
    pub fn get_absolute_modifier(&self) -> i32 { self.absolute_modifier }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_improves_positive_is_improvement() {
        let m = StatsDrawingModifier::positive_improves(2);
        assert!(m.is_improvement());
        assert!(!m.is_impairment());
        assert_eq!(m.get_absolute_modifier(), 2);
    }

    #[test]
    fn positive_improves_negative_is_impairment() {
        let m = StatsDrawingModifier::positive_improves(-1);
        assert!(!m.is_improvement());
        assert!(m.is_impairment());
        assert_eq!(m.get_absolute_modifier(), 1);
    }

    #[test]
    fn positive_improves_zero_is_neutral() {
        let m = StatsDrawingModifier::positive_improves(0);
        assert!(!m.is_improvement());
        assert!(!m.is_impairment());
        assert_eq!(m.get_absolute_modifier(), 0);
    }

    #[test]
    fn positive_impairs_positive_is_impairment() {
        let m = StatsDrawingModifier::positive_impairs(3);
        assert!(!m.is_improvement());
        assert!(m.is_impairment());
        assert_eq!(m.get_absolute_modifier(), 3);
    }

    #[test]
    fn positive_impairs_negative_is_improvement() {
        let m = StatsDrawingModifier::positive_impairs(-2);
        assert!(m.is_improvement());
        assert!(!m.is_impairment());
        assert_eq!(m.get_absolute_modifier(), 2);
    }
}
