use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportGameOptions.java`.
/// Note: Java source comments this report as "no longer used, remains for
/// compatibility with older versions".
#[derive(Debug, Clone, Default)]
pub struct ReportGameOptions {
    /// Translated from `fOvertime`.
    pub overtime: bool,
    /// Translated from `fTurntime`.
    pub turntime: i32,
    /// Translated from `fSneakyGitAsFoulGuard`.
    pub sneaky_git_as_foul_guard: bool,
    /// Translated from `fFoulBonusOutsideTacklezone`.
    pub foul_bonus_outside_tacklezone: bool,
    /// Translated from `fRightStuffCancelsTackle`.
    pub right_stuff_cancels_tackle: bool,
    /// Translated from `fPilingOnWithoutModifier`.
    pub piling_on_without_modifier: bool,
}

impl ReportGameOptions {
    pub fn new(
        overtime: bool,
        turntime: i32,
        sneaky_git_as_foul_guard: bool,
        foul_bonus_outside_tacklezone: bool,
        right_stuff_cancels_tackle: bool,
        piling_on_without_modifier: bool,
    ) -> Self {
        Self {
            overtime,
            turntime,
            sneaky_git_as_foul_guard,
            foul_bonus_outside_tacklezone,
            right_stuff_cancels_tackle,
            piling_on_without_modifier,
        }
    }

    pub fn is_overtime(&self) -> bool {
        self.overtime
    }

    pub fn get_turntime(&self) -> i32 {
        self.turntime
    }

    pub fn is_sneaky_git_as_foul_guard(&self) -> bool {
        self.sneaky_git_as_foul_guard
    }

    pub fn is_foul_bonus_outside_tacklezone(&self) -> bool {
        self.foul_bonus_outside_tacklezone
    }

    pub fn is_right_stuff_cancels_tackle(&self) -> bool {
        self.right_stuff_cancels_tackle
    }

    pub fn is_piling_on_without_modifier(&self) -> bool {
        self.piling_on_without_modifier
    }
}

impl IReport for ReportGameOptions {
    fn get_id(&self) -> ReportId {
        ReportId::GAME_OPTIONS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportGameOptions {
        ReportGameOptions::new(true, 60, false, true, false, true)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::GAME_OPTIONS);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "gameOptions");
    }

    #[test]
    fn field_getters() {
        let r = make();
        assert!(r.is_overtime());
        assert_eq!(r.get_turntime(), 60);
        assert!(!r.is_sneaky_git_as_foul_guard());
        assert!(r.is_foul_bonus_outside_tacklezone());
        assert!(!r.is_right_stuff_cancels_tackle());
        assert!(r.is_piling_on_without_modifier());
    }
}
