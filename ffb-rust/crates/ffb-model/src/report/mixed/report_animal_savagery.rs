use crate::report::i_report::IReport;
use crate::report::report_id::ReportId;

/// 1:1 translation of `ReportAnimalSavagery.java`.
#[derive(Debug, Clone)]
pub struct ReportAnimalSavagery {
    pub attacker_id: Option<String>,
    pub defender_id: Option<String>,
}

impl ReportAnimalSavagery {
    pub fn new(attacker_id: Option<String>, defender_id: Option<String>) -> Self {
        Self { attacker_id, defender_id }
    }

    pub fn get_attacker_id(&self) -> Option<&str> { self.attacker_id.as_deref() }
    pub fn get_defender_id(&self) -> Option<&str> { self.defender_id.as_deref() }
}

impl IReport for ReportAnimalSavagery {
    fn get_id(&self) -> ReportId { ReportId::ANIMAL_SAVAGERY }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make() -> ReportAnimalSavagery {
        ReportAnimalSavagery::new(Some("a1".into()), Some("d1".into()))
    }

    #[test]
    fn get_id() { assert_eq!(make().get_id(), ReportId::ANIMAL_SAVAGERY); }

    #[test]
    fn get_name() { assert_eq!(make().get_name(), "animalSavagery"); }

    #[test]
    fn get_attacker_id() { assert_eq!(make().get_attacker_id(), Some("a1")); }
}
