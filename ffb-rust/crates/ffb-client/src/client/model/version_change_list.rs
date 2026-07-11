/// 1:1 translation of com.fumbbl.ffb.client.model.VersionChangeList (Java class).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct VersionChangeList {
    bugfixes: Vec<String>,
    features: Vec<String>,
    improvements: Vec<String>,
    behavior_changes: Vec<String>,
    removals: Vec<String>,
    rule_changes: Vec<String>,
    description: Option<String>,
    version: String,
}

impl VersionChangeList {
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            ..Default::default()
        }
    }

    pub fn get_description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn add_bugfix(mut self, bugfix: impl Into<String>) -> Self {
        self.bugfixes.push(bugfix.into());
        self
    }

    pub fn add_improvement(mut self, improvement: impl Into<String>) -> Self {
        self.improvements.push(improvement.into());
        self
    }

    pub fn add_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn add_behavior_change(mut self, behavior_change: impl Into<String>) -> Self {
        self.behavior_changes.push(behavior_change.into());
        self
    }

    pub fn add_removal(mut self, removal: impl Into<String>) -> Self {
        self.removals.push(removal.into());
        self
    }

    pub fn add_rule_change(mut self, rule_change: impl Into<String>) -> Self {
        self.rule_changes.push(rule_change.into());
        self
    }

    pub fn get_bugfixes(&self) -> &[String] {
        &self.bugfixes
    }

    pub fn get_features(&self) -> &[String] {
        &self.features
    }

    pub fn get_improvements(&self) -> &[String] {
        &self.improvements
    }

    pub fn get_behavior_changes(&self) -> &[String] {
        &self.behavior_changes
    }

    pub fn get_removals(&self) -> &[String] {
        &self.removals
    }

    pub fn get_rule_changes(&self) -> &[String] {
        &self.rule_changes
    }

    pub fn set_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn get_version(&self) -> &str {
        &self.version
    }

    pub fn has_bugfixes(&self) -> bool {
        !self.bugfixes.is_empty()
    }

    pub fn has_improvements(&self) -> bool {
        !self.improvements.is_empty()
    }

    pub fn has_features(&self) -> bool {
        !self.features.is_empty()
    }

    pub fn has_behavior_changes(&self) -> bool {
        !self.behavior_changes.is_empty()
    }

    pub fn has_description(&self) -> bool {
        self.description.as_deref().is_some_and(|d| !d.is_empty())
    }

    pub fn has_removals(&self) -> bool {
        !self.removals.is_empty()
    }

    pub fn has_rule_changes(&self) -> bool {
        !self.rule_changes.is_empty()
    }

    pub fn has_entries(&self) -> bool {
        self.has_bugfixes()
            || self.has_features()
            || self.has_improvements()
            || self.has_behavior_changes()
            || self.has_description()
            || self.has_removals()
            || self.has_rule_changes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_entries() {
        let v = VersionChangeList::new("3.2.3");
        assert_eq!(v.get_version(), "3.2.3");
        assert!(!v.has_entries());
    }

    #[test]
    fn add_bugfix_tracks_entry() {
        let v = VersionChangeList::new("3.2.3").add_bugfix("fixed something");
        assert!(v.has_bugfixes());
        assert_eq!(v.get_bugfixes(), &["fixed something".to_string()]);
        assert!(v.has_entries());
    }

    #[test]
    fn add_feature_and_improvement() {
        let v = VersionChangeList::new("3.2.0")
            .add_feature("new feature")
            .add_improvement("improved thing");
        assert!(v.has_features());
        assert!(v.has_improvements());
        assert_eq!(v.get_features(), &["new feature".to_string()]);
        assert_eq!(v.get_improvements(), &["improved thing".to_string()]);
    }

    #[test]
    fn add_behavior_change_removal_rule_change() {
        let v = VersionChangeList::new("3.2.1")
            .add_behavior_change("behavior")
            .add_removal("removal")
            .add_rule_change("rule");
        assert!(v.has_behavior_changes());
        assert!(v.has_removals());
        assert!(v.has_rule_changes());
    }

    #[test]
    fn set_description_sets_has_description() {
        let v = VersionChangeList::new("3.0.0").set_description("First version");
        assert!(v.has_description());
        assert_eq!(v.get_description(), Some("First version"));
        assert!(v.has_entries());
    }

    #[test]
    fn empty_description_is_not_a_description() {
        let v = VersionChangeList::new("3.0.0").set_description("");
        assert!(!v.has_description());
    }

    #[test]
    fn equality_compares_all_fields() {
        let a = VersionChangeList::new("1.0").add_bugfix("x");
        let b = VersionChangeList::new("1.0").add_bugfix("x");
        let c = VersionChangeList::new("1.0").add_bugfix("y");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
