use serde::{Deserialize, Serialize};

/// 1:1 translation of com.fumbbl.ffb.model.RosterSkeleton.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RosterSkeleton {
    pub id: String,
    pub name: String,
}

impl RosterSkeleton {
    pub fn get_id(&self) -> &str { &self.id }
    pub fn get_name(&self) -> &str { &self.name }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_empty() {
        assert!(RosterSkeleton::default().id.is_empty());
    }

    #[test]
    fn get_name_returns_name() {
        let r = RosterSkeleton { id: "1".to_string(), name: "Undead".to_string() };
        assert_eq!(r.get_name(), "Undead");
    }
}
