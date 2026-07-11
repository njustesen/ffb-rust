use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;
use ffb_model::enums::NetCommandId;
use ffb_model::model::inducement_set::InducementSet;

/// 1:1 translation of com.fumbbl.ffb.net.commands.ClientCommandBuyInducements.

#[derive(Debug, Clone, Default)]
pub struct ClientCommandBuyInducements {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fTeamId`
    pub team_id: Option<String>,
    /// Java: `fAvailableGold`
    pub available_gold: i32,
    /// Java: `fInducementSet` — `InducementSet` already derives Serialize/Deserialize.
    pub inducement_set: Option<InducementSet>,
    /// Java: `fStarPlayerPositionIds`
    pub star_player_position_ids: Vec<String>,
    /// Java: `fMercenaryPositionIds`
    pub mercenary_position_ids: Vec<String>,
    /// Java: `staffPositionIds`
    pub staff_position_ids: Vec<String>,
    /// Java: `fMercenarySkills` — DEFERRED: Skill is complex; stored as skill id/name strings.
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

    /// Java: `getInducementSet()`
    pub fn get_inducement_set(&self) -> Option<&InducementSet> {
        self.inducement_set.as_ref()
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

    /// Java: `getMercenarySkills()` — DEFERRED: returns skill id/name strings.
    pub fn get_mercenary_skill_ids(&self) -> &[String] {
        &self.mercenary_skill_ids
    }

    /// Java: `ClientCommandBuyInducements.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(team_id) = &self.team_id {
            map.insert("teamId".to_string(), serde_json::json!(team_id));
        }
        if let Some(inducement_set) = &self.inducement_set {
            map.insert(
                "inducementSet".to_string(),
                serde_json::to_value(inducement_set).unwrap_or(serde_json::Value::Null),
            );
        }
        map.insert(
            "starPlayerPositionIds".to_string(),
            serde_json::json!(self.star_player_position_ids),
        );
        map.insert("availableGold".to_string(), serde_json::json!(self.available_gold));
        map.insert(
            "mercenaryPositionIds".to_string(),
            serde_json::json!(self.mercenary_position_ids),
        );
        map.insert("mercenarySkills".to_string(), serde_json::json!(self.mercenary_skill_ids));
        map.insert("staffPositionIds".to_string(), serde_json::json!(self.staff_position_ids));
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandBuyInducements.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        let team_id = json.get("teamId").and_then(|v| v.as_str()).map(|s| s.to_string());
        // Java: `fInducementSet = new InducementSet();` unconditionally, then
        // `fInducementSet.initFrom(source, inducementSetObject)` only `if (inducementSetObject != null)`.
        let mut inducement_set = InducementSet::new();
        if let Some(v) = json.get("inducementSet").filter(|v| !v.is_null()) {
            if let Ok(parsed) = serde_json::from_value::<InducementSet>(v.clone()) {
                inducement_set = parsed;
            }
        }
        let inducement_set = Some(inducement_set);
        let star_player_position_ids = json
            .get("starPlayerPositionIds")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        // Java: `if (IJsonOption.STAFF_POSITION_IDS.isDefinedIn(jsonObject)) { ... }`.
        let staff_position_ids = json
            .get("staffPositionIds")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let available_gold = json.get("availableGold").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let mercenary_position_ids: Vec<String> = json
            .get("mercenaryPositionIds")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let mercenary_skill_ids: Vec<String> = json
            .get("mercenarySkills")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        // Java: only populates mercenary positions/skills when both arrays are non-empty (`StringTool.isProvided`).
        let (mercenary_position_ids, mercenary_skill_ids) =
            if !mercenary_position_ids.is_empty() && !mercenary_skill_ids.is_empty() {
                (mercenary_position_ids, mercenary_skill_ids)
            } else {
                (Vec::new(), Vec::new())
            };
        Self {
            entropy: base.entropy,
            team_id,
            available_gold,
            inducement_set,
            star_player_position_ids,
            mercenary_position_ids,
            staff_position_ids,
            mercenary_skill_ids,
        }
    }
}

impl NetCommand for ClientCommandBuyInducements {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientBuyInducements
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

    #[test]
    fn star_player_ids_stored() {
        let cmd = ClientCommandBuyInducements {
            star_player_position_ids: vec!["pos1".to_string()],
            ..Default::default()
        };
        assert_eq!(cmd.get_star_player_position_ids().len(), 1);
    }


    #[test]
    fn debug_format_nonempty() {
        let cmd = ClientCommandBuyInducements::default();
        assert!(!format!("{cmd:?}").is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandBuyInducements::default().clone();
    }

    #[test]
    fn get_id_is_client_buy_inducements() {
        assert_eq!(ClientCommandBuyInducements::new().get_id(), NetCommandId::ClientBuyInducements);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_team_id() {
        let cmd = ClientCommandBuyInducements {
            team_id: Some("team_home".to_string()),
            ..Default::default()
        };
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientBuyInducements");
        assert_eq!(json["teamId"], "team_home");
    }

    #[test]
    fn round_trip_with_populated_data() {
        let mut inducement_set = InducementSet::new();
        inducement_set.add_available_card("Chop Block");
        let cmd = ClientCommandBuyInducements {
            entropy: Some(7),
            team_id: Some("team_home".to_string()),
            available_gold: 150000,
            inducement_set: Some(inducement_set),
            star_player_position_ids: vec!["pos1".to_string()],
            mercenary_position_ids: vec!["merc1".to_string()],
            staff_position_ids: vec!["staff1".to_string()],
            mercenary_skill_ids: vec!["BLOCK".to_string()],
        };
        let json = cmd.to_json_value();
        let restored = ClientCommandBuyInducements::from_json(&json);
        assert_eq!(restored.entropy, Some(7));
        assert_eq!(restored.get_team_id(), Some("team_home"));
        assert_eq!(restored.get_available_gold(), 150000);
        assert!(restored.get_inducement_set().unwrap().is_available("Chop Block"));
        assert_eq!(restored.get_star_player_position_ids(), &["pos1".to_string()]);
        assert_eq!(restored.get_mercenary_position_ids(), &["merc1".to_string()]);
        assert_eq!(restored.get_staff_position_ids(), &["staff1".to_string()]);
        assert_eq!(restored.get_mercenary_skill_ids(), &["BLOCK".to_string()]);
    }

    #[test]
    fn round_trip_with_default_empty_data() {
        let cmd = ClientCommandBuyInducements::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandBuyInducements::from_json(&json);
        assert_eq!(restored.entropy, None);
        assert!(restored.get_team_id().is_none());
        assert_eq!(restored.get_available_gold(), 0);
        assert!(restored.get_inducement_set().unwrap().get_all_cards().is_empty());
        assert!(restored.get_star_player_position_ids().is_empty());
        assert!(restored.get_mercenary_position_ids().is_empty());
        assert!(restored.get_staff_position_ids().is_empty());
        assert!(restored.get_mercenary_skill_ids().is_empty());
    }
}
