use crate::model::special_effect::SpecialEffect;
use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportWizardUse.java`.
/// `fWizardSpell` (Java `SpecialEffect`) → `wizard_spell: SpecialEffect`.
#[derive(Debug, Clone)]
pub struct ReportWizardUse {
    pub team_id: String,
    pub wizard_spell: SpecialEffect,
}

impl ReportWizardUse {
    pub fn new(team_id: String, wizard_spell: SpecialEffect) -> Self {
        Self { team_id, wizard_spell }
    }

    pub fn get_team_id(&self) -> &str { &self.team_id }
    pub fn get_wizard_spell(&self) -> SpecialEffect { self.wizard_spell }
}

impl IReport for ReportWizardUse {
    fn get_id(&self) -> ReportId { ReportId::WIZARD_USE }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportWizardUse {
        ReportWizardUse::new("team1".into(), SpecialEffect::FIREBALL)
    }

    #[test]
    fn get_id() {
        assert_eq!(make().get_id(), ReportId::WIZARD_USE);
    }

    #[test]
    fn get_name() {
        assert_eq!(make().get_name(), "wizardUse");
    }

    #[test]
    fn fields() {
        let r = make();
        assert_eq!(r.get_team_id(), "team1");
        assert_eq!(r.get_wizard_spell(), SpecialEffect::FIREBALL);
    }

    #[test]
    fn lightning_spell() {
        let r = ReportWizardUse::new("team2".into(), SpecialEffect::LIGHTNING);
        assert_eq!(r.get_wizard_spell(), SpecialEffect::LIGHTNING);
        assert_eq!(r.get_team_id(), "team2");
    }

    #[test]
    fn different_team_id() {
        let r = ReportWizardUse::new("team3".into(), SpecialEffect::FIREBALL);
        assert_eq!(r.get_team_id(), "team3");
    }
}
