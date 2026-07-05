/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBuyInducements.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandBuyInducements {
    /// Java: `fTeamId`
    pub team_id: Option<String>,
    /// Java: `fAvailableGold`
    pub available_gold: i32,
    /// Java: `fInducementSet` — DEFERRED: InducementSet is complex; stored as raw JSON string for now.
    pub inducement_set_json: Option<String>,
    /// Java: `fStarPlayerPositionIds`
    pub star_player_position_ids: Vec<String>,
    /// Java: `fMercenaryPositionIds`
    pub mercenary_position_ids: Vec<String>,
    /// Java: `staffPositionIds`
    pub staff_position_ids: Vec<String>,
    /// Java: `fMercenarySkills` — DEFERRED: Skill is complex; stored as skill id strings.
    pub mercenary_skill_ids: Vec<String>,
}

impl ClientCommandBuyInducements {
    pub fn new() -> Self {
        Self::default()
    }

    /// Java: `getTeamId()`
    pub fn get_team_id(&self) -> Option<&str> {
        self.team_id.as_deref()
    }

    /// Java: `getAvailableGold()`
    pub fn get_available_gold(&self) -> i32 {
        self.available_gold
    }

    /// Java: `getInducementSet()` — DEFERRED: returns raw JSON placeholder.
    pub fn get_inducement_set_json(&self) -> Option<&str> {
        self.inducement_set_json.as_deref()
    }

    /// Java: `getStarPlayerPositionIds()`
    pub fn get_star_player_position_ids(&self) -> &[String] {
        &self.star_player_position_ids
    }

    /// Java: `getMercenaryPositionIds()`
    pub fn get_mercenary_position_ids(&self) -> &[String] {
        &self.mercenary_position_ids
    }

    /// Java: `getStaffPositionIds()`
    pub fn get_staff_position_ids(&self) -> &[String] {
        &self.staff_position_ids
    }

    /// Java: `getMercenarySkills()` — DEFERRED: returns skill id strings.
    pub fn get_mercenary_skill_ids(&self) -> &[String] {
        &self.mercenary_skill_ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_available_gold_is_zero() {
        let cmd = ClientCommandBuyInducements::new();
        assert_eq!(cmd.get_available_gold(), 0);
    }

    #[test]
    fn stores_team_id_and_gold() {
        let cmd = ClientCommandBuyInducements {
            team_id: Some("team_home".to_string()),
            available_gold: 150000,
            ..Default::default()
        };
        assert_eq!(cmd.get_team_id(), Some("team_home"));
        assert_eq!(cmd.get_available_gold(), 150000);
    }
}
