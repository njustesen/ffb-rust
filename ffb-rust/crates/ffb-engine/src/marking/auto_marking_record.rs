/// 1:1 translation of com.fumbbl.ffb.server.marking.AutoMarkingRecord.
use ffb_model::factory::skill_factory::SkillFactory;
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

    /// Java: `AutoMarkingRecord.toJsonValue()`.
    ///
    /// Keys are the `IJsonOption` constants used in the Java source:
    /// `SKILL_ARRAY` = `"skillArray"`, `INJURY_ATTRIBUTES` = `"injuryAttributes"`,
    /// `APPLY_TO` = `"applyTo"`, `GAINED_ONLY` = `"gainedOnly"`, `MARKING` = `"marking"`,
    /// `APPLY_REPEATEDLY` = `"applyRepeatedly"`.
    ///
    /// Java serializes each skill via `UtilJson.toJsonValue(skill)` = `skill.getName()` (the
    /// human-readable display name). This crate's `SkillId` exposes only the Java class simple
    /// name (`class_name()`), not a separate `getName()` display-name table, so the class name
    /// is emitted here; it round-trips through [`Self::from_json`], which resolves names via
    /// `SkillFactory::for_name` (whose normalisation accepts either form). Java's `applyTo`
    /// is always non-null in this crate (`ApplyTo` has no null state), so it is always written.
    pub fn to_json_value(&self) -> serde_json::Value {
        let mut obj = serde_json::Map::new();
        let skill_array: Vec<serde_json::Value> = self
            .skills
            .iter()
            .map(|s| serde_json::Value::String(s.class_name().to_string()))
            .collect();
        obj.insert("skillArray".to_string(), serde_json::Value::Array(skill_array));
        let injuries: Vec<serde_json::Value> = self
            .injuries
            .iter()
            .map(|i| serde_json::Value::String(i.get_name().to_string()))
            .collect();
        obj.insert("injuryAttributes".to_string(), serde_json::Value::Array(injuries));
        obj.insert("applyTo".to_string(), serde_json::Value::String(self.apply_to.name().to_string()));
        obj.insert("gainedOnly".to_string(), serde_json::Value::Bool(self.gained_only));
        obj.insert("marking".to_string(), serde_json::Value::String(self.marking.clone()));
        obj.insert("applyRepeatedly".to_string(), serde_json::Value::Bool(self.apply_repeatedly));
        serde_json::Value::Object(obj)
    }

    /// Java: `AutoMarkingRecord.initFrom(IFactorySource source, JsonValue jsonValue)`.
    ///
    /// Field order and semantics mirror the Java source: skills, then (optional) `applyTo`,
    /// `gainedOnly`, `marking`, `applyRepeatedly`, then injuries. Skill names are resolved via
    /// `SkillFactory::for_name` (Java: `UtilJson.toEnumWithName(skillFactory, ...)` →
    /// `skillFactory.forName(name)`); a name the factory cannot resolve is skipped (Java would
    /// add a `null` skill — this crate has no null `SkillId`, so it drops it rather than
    /// fabricating one). `applyTo` is only overwritten when present (Java:
    /// `IJsonOption.APPLY_TO.isDefinedIn(jsonObject)`), otherwise the `ApplyTo::Both` default
    /// from `new()` stands.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let factory = SkillFactory::new();
        let mut record = AutoMarkingRecord::new();

        if let Some(arr) = json.get("skillArray").and_then(|v| v.as_array()) {
            for value in arr {
                if let Some(name) = value.as_str() {
                    if let Some(skill) = factory.for_name(name) {
                        record.skills.push(skill);
                    }
                }
            }
        }

        if let Some(name) = json.get("applyTo").and_then(|v| v.as_str()) {
            if let Some(apply_to) = ApplyTo::value_of(name) {
                record.apply_to = apply_to;
            }
        }

        record.gained_only = json.get("gainedOnly").and_then(|v| v.as_bool()).unwrap_or(false);
        record.marking = json.get("marking").and_then(|v| v.as_str()).unwrap_or("").to_string();
        record.apply_repeatedly = json.get("applyRepeatedly").and_then(|v| v.as_bool()).unwrap_or(false);

        if let Some(arr) = json.get("injuryAttributes").and_then(|v| v.as_array()) {
            for value in arr {
                if let Some(name) = value.as_str() {
                    if let Some(injury) = InjuryAttribute::for_name(name) {
                        record.injuries.push(injury);
                    }
                }
            }
        }

        record
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

    #[test]
    fn to_json_value_uses_java_option_keys() {
        let record = Builder::new()
            .with_skill(SkillId::Block)
            .with_injury(InjuryAttribute::NI)
            .with_marking("B")
            .with_gained_only(true)
            .with_apply_repeatedly(true)
            .with_apply_to(ApplyTo::Opponent)
            .build();
        let json = record.to_json_value();
        assert_eq!(json["skillArray"], serde_json::json!(["Block"]));
        assert_eq!(json["injuryAttributes"], serde_json::json!(["NI"]));
        assert_eq!(json["applyTo"], "OPPONENT");
        assert_eq!(json["gainedOnly"], true);
        assert_eq!(json["marking"], "B");
        assert_eq!(json["applyRepeatedly"], true);
    }

    #[test]
    fn from_json_reads_all_fields() {
        let json = serde_json::json!({
            "skillArray": ["Block", "Tackle"],
            "injuryAttributes": ["MA", "AV"],
            "applyTo": "OWN",
            "gainedOnly": true,
            "marking": "BT",
            "applyRepeatedly": true,
        });
        let record = AutoMarkingRecord::from_json(&json);
        assert_eq!(record.skills, vec![SkillId::Block, SkillId::Tackle]);
        assert_eq!(record.injuries, vec![InjuryAttribute::MA, InjuryAttribute::AV]);
        assert_eq!(record.apply_to(), ApplyTo::Own);
        assert!(record.is_gained_only());
        assert_eq!(record.marking(), "BT");
        assert!(record.is_apply_repeatedly());
    }

    #[test]
    fn from_json_defaults_apply_to_both_when_absent() {
        let json = serde_json::json!({ "skillArray": ["Block"], "marking": "B" });
        let record = AutoMarkingRecord::from_json(&json);
        assert_eq!(record.apply_to(), ApplyTo::Both);
        assert!(!record.is_gained_only());
    }

    #[test]
    fn from_json_skips_unresolvable_skill_names() {
        let json = serde_json::json!({ "skillArray": ["Block", "NoSuchSkillXyz"] });
        let record = AutoMarkingRecord::from_json(&json);
        assert_eq!(record.skills, vec![SkillId::Block]);
    }

    #[test]
    fn json_round_trip_preserves_record() {
        let original = Builder::new()
            .with_skill(SkillId::Block)
            .with_skill(SkillId::Tackle)
            .with_injury(InjuryAttribute::NI)
            .with_marking("BT")
            .with_gained_only(true)
            .with_apply_to(ApplyTo::Own)
            .build();
        let restored = AutoMarkingRecord::from_json(&original.to_json_value());
        assert_eq!(restored, original);
    }
}
