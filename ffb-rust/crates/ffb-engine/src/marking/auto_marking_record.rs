/// 1:1 translation of com.fumbbl.ffb.server.marking.AutoMarkingRecord.
use ffb_model::model::injury_attribute::InjuryAttribute;
use ffb_model::model::skill_def::SkillId;
use crate::marking::apply_to::ApplyTo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoMarkingRecord {
    pub skills: Vec<SkillId>,
    pub injuries: Vec<InjuryAttribute>,
    pub gained_only: bool,
    pub apply_repeatedly: bool,
    pub apply_to: ApplyTo,
    pub marking: String,
}

impl AutoMarkingRecord {
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            injuries: Vec::new(),
            gained_only: false,
            apply_repeatedly: false,
            apply_to: ApplyTo::Both,
            marking: String::new(),
        }
    }

    pub fn skills(&self) -> &[SkillId] { &self.skills }
    pub fn injuries(&self) -> &[InjuryAttribute] { &self.injuries }
    pub fn marking(&self) -> &str { &self.marking }
    pub fn is_gained_only(&self) -> bool { self.gained_only }
    pub fn is_apply_repeatedly(&self) -> bool { self.apply_repeatedly }
    pub fn apply_to(&self) -> ApplyTo { self.apply_to }

    /// Java: isInjuryOnly() — true when there are no skill requirements.
    pub fn is_injury_only(&self) -> bool {
        self.skills.is_empty()
    }

    /// Java: isSubSetOf(AutoMarkingRecord other).
    /// Returns true when `other` contains all skills and injuries of self.
    pub fn is_subset_of(&self, other: &AutoMarkingRecord) -> bool {
        self.skills.iter().all(|s| other.skills.contains(s))
            && self.injuries.iter().all(|inj| other.injuries.contains(inj))
    }
}

impl Default for AutoMarkingRecord {
    fn default() -> Self { Self::new() }
}

/// Java: AutoMarkingRecord.Builder (inner class).
pub struct Builder {
    record: AutoMarkingRecord,
}

impl Builder {
    pub fn new() -> Self {
        Self { record: AutoMarkingRecord::new() }
    }

    pub fn with_skill(mut self, skill_id: SkillId) -> Self {
        self.record.skills.push(skill_id);
        self
    }

    pub fn with_injury(mut self, injury: InjuryAttribute) -> Self {
        self.record.injuries.push(injury);
        self
    }

    pub fn with_gained_only(mut self, gained_only: bool) -> Self {
        self.record.gained_only = gained_only;
        self
    }

    pub fn with_apply_to(mut self, apply_to: ApplyTo) -> Self {
        self.record.apply_to = apply_to;
        self
    }

    pub fn with_marking(mut self, marking: impl Into<String>) -> Self {
        self.record.marking = marking.into();
        self
    }

    pub fn with_apply_repeatedly(mut self, apply_repeatedly: bool) -> Self {
        self.record.apply_repeatedly = apply_repeatedly;
        self
    }

    pub fn build(mut self) -> AutoMarkingRecord {
        let record = self.record;
        self.record = AutoMarkingRecord::new();
        record
    }
}

impl Default for Builder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::skill_def::SkillId;

    #[test]
    fn new_has_defaults() {
        let r = AutoMarkingRecord::new();
        assert!(r.skills.is_empty());
        assert!(r.injuries.is_empty());
        assert!(!r.gained_only);
        assert!(!r.apply_repeatedly);
        assert_eq!(r.apply_to, ApplyTo::Both);
        assert!(r.marking.is_empty());
    }

    #[test]
    fn is_injury_only_when_no_skills() {
        let r = AutoMarkingRecord::new();
        assert!(r.is_injury_only());
    }

    #[test]
    fn is_injury_only_false_when_has_skills() {
        let mut r = AutoMarkingRecord::new();
        r.skills.push(SkillId::Block);
        assert!(!r.is_injury_only());
    }

    #[test]
    fn is_subset_of_empty_record() {
        let r = AutoMarkingRecord::new();
        let other = AutoMarkingRecord::new();
        assert!(r.is_subset_of(&other));
    }

    #[test]
    fn is_subset_of_superset() {
        let mut r = AutoMarkingRecord::new();
        r.skills.push(SkillId::Block);

        let mut other = AutoMarkingRecord::new();
        other.skills.push(SkillId::Block);
        other.skills.push(SkillId::Tackle);

        assert!(r.is_subset_of(&other));
        assert!(!other.is_subset_of(&r));
    }

    #[test]
    fn builder_creates_record_with_skill_and_marking() {
        let record = Builder::new()
            .with_skill(SkillId::Block)
            .with_marking("B")
            .with_gained_only(true)
            .build();

        assert_eq!(record.skills, vec![SkillId::Block]);
        assert_eq!(record.marking(), "B");
        assert!(record.is_gained_only());
    }

    #[test]
    fn builder_resets_after_build() {
        let mut builder = Builder::new();
        builder = builder.with_skill(SkillId::Block).with_marking("B");
        let _first = builder.build();
        // After build, a new record was started — builder is consumed so no further test needed
        // (in Java, builder is mutable and reused; here each build() consumes the builder)
    }

    #[test]
    fn builder_with_injury() {
        let record = Builder::new()
            .with_injury(InjuryAttribute::NI)
            .with_marking("NI")
            .build();
        assert_eq!(record.injuries, vec![InjuryAttribute::NI]);
        assert!(record.is_injury_only()); // no skills → injury-only
    }

    #[test]
    fn is_subset_of_injury_check() {
        let mut r = AutoMarkingRecord::new();
        r.injuries.push(InjuryAttribute::MA);

        let mut other = AutoMarkingRecord::new();
        other.injuries.push(InjuryAttribute::MA);
        other.injuries.push(InjuryAttribute::AV);

        assert!(r.is_subset_of(&other));
    }
}
