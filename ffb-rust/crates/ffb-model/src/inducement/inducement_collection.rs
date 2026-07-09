use crate::inducement::usage::Usage;

/// Describes a single inducement type — its id, names, and usages.
/// Simplified from Java `InducementType` for the base Rust translation.
#[derive(Debug, Clone)]
pub struct InducementTypeEntry {
    pub id: String,
    pub usages: Vec<Usage>,
}

impl InducementTypeEntry {
    pub fn new(id: impl Into<String>, usages: Vec<Usage>) -> Self {
        Self { id: id.into(), usages }
    }
}

/// 1:1 translation of `com.fumbbl.ffb.inducement.InducementCollection` (abstract base).
///
/// Holds the common set of inducement types shared by all editions.
#[derive(Debug, Default)]
pub struct InducementCollection {
    types: Vec<InducementTypeEntry>,
}

impl InducementCollection {
    pub fn new() -> Self {
        let types = vec![
            InducementTypeEntry::new("extraTeamTraining", vec![Usage::REROLL]),
            InducementTypeEntry::new("wanderingApothecaries", vec![Usage::APOTHECARY]),
            InducementTypeEntry::new("starPlayers", vec![Usage::STAR]),
            InducementTypeEntry::new("mercenaries", vec![Usage::LONER]),
            InducementTypeEntry::new("wizard", vec![Usage::SPELL]),
        ];
        Self { types }
    }

    /// Java: `getTypes()` — all types (base + sub-types).
    pub fn get_types(&self) -> &[InducementTypeEntry] {
        &self.types
    }

    /// Java: `getKey()` — returns class simple name.
    pub fn get_key(&self) -> &str {
        "InducementCollection"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_collection_has_five_types() {
        let c = InducementCollection::new();
        assert_eq!(c.get_types().len(), 5);
    }

    #[test]
    fn has_extra_team_training() {
        let c = InducementCollection::new();
        assert!(c.get_types().iter().any(|t| t.id == "extraTeamTraining"));
    }

    #[test]
    fn star_players_usage_is_star() {
        let c = InducementCollection::new();
        let sp = c.get_types().iter().find(|t| t.id == "starPlayers").unwrap();
        assert!(sp.usages.contains(&Usage::STAR));
    }
}
