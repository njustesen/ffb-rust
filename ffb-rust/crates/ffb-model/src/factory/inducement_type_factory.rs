use crate::inducement::inducement_collection::{InducementCollection, InducementTypeEntry};

/// 1:1 translation of `com.fumbbl.ffb.factory.InducementTypeFactory`.
///
/// Java selects its `InducementCollection` subclass instance via
/// `Scanner<>(InducementCollection.class).getSubclassInstances(options)` reflection at
/// `initialize(Game)`; Rust has no runtime reflection, so the `InducementCollection` is passed
/// in directly by the caller instead.
pub struct InducementTypeFactory<'a> {
    types: &'a InducementCollection,
}

impl<'a> InducementTypeFactory<'a> {
    pub fn new(types: &'a InducementCollection) -> Self {
        Self { types }
    }

    /// Java: `forName(String)` — case-insensitive lookup by name.
    pub fn for_name(&self, name: &str) -> Option<&'a InducementTypeEntry> {
        self.types
            .get_types()
            .iter()
            .find(|entry| entry.id.eq_ignore_ascii_case(name))
    }

    /// Java: `allTypes()` — all types sorted by name.
    pub fn all_types(&self) -> Vec<&'a InducementTypeEntry> {
        let mut entries: Vec<&InducementTypeEntry> = self.types.get_types().iter().collect();
        entries.sort_by(|a, b| a.id.cmp(&b.id));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_name_finds_extra_team_training() {
        let types = InducementCollection::new();
        let factory = InducementTypeFactory::new(&types);
        let found = factory.for_name("extraTeamTraining").unwrap();
        assert_eq!(found.id, "extraTeamTraining");
    }

    #[test]
    fn for_name_is_case_insensitive() {
        let types = InducementCollection::new();
        let factory = InducementTypeFactory::new(&types);
        assert!(factory.for_name("EXTRATEAMTRAINING").is_some());
    }

    #[test]
    fn for_name_returns_none_for_unknown() {
        let types = InducementCollection::new();
        let factory = InducementTypeFactory::new(&types);
        assert!(factory.for_name("unknown").is_none());
    }

    #[test]
    fn all_types_length_matches() {
        let types = InducementCollection::new();
        let factory = InducementTypeFactory::new(&types);
        assert_eq!(factory.all_types().len(), types.get_types().len());
    }

    #[test]
    fn all_types_are_sorted_by_id() {
        let types = InducementCollection::new();
        let factory = InducementTypeFactory::new(&types);
        let all = factory.all_types();
        let mut ids: Vec<&str> = all.iter().map(|t| t.id.as_str()).collect();
        let mut sorted_ids = ids.clone();
        sorted_ids.sort();
        assert_eq!(ids, sorted_ids);
        ids.clear();
    }
}
