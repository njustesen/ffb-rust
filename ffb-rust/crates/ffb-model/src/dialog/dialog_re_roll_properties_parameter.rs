use serde::{Deserialize, Serialize};
use crate::enums::ReRollProperty;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::has_re_roll_properties::HasReRollProperties;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollPropertiesParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub minimum_roll: i32,
    pub fumble: bool,
    pub re_roll_skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    /// Java `final List<String> messages` (never null; a JSON-null element is a null String).
    pub messages: Vec<Option<String>>,
    /// Java `final List<ReRollProperty>`; a decoded unknown name is a null entry (`None`).
    pub re_roll_properties: Vec<Option<ReRollProperty>>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogReRollPropertiesParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_fumble(&self) -> bool { self.fumble }
    pub fn get_messages(&self) -> &[Option<String>] { &self.messages }
    pub fn get_re_roll_properties(&self) -> &[Option<ReRollProperty>] { &self.re_roll_properties }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl HasReRollProperties for DialogReRollPropertiesParameter {
    fn has_property(&self, property: ReRollProperty) -> bool {
        self.re_roll_properties.contains(&Some(property))
    }
}

impl IDialogParameter for DialogReRollPropertiesParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL_PROPERTIES }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogReRollPropertiesParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::model::common_property::CommonProperty;
        use crate::model::re_rolled_actions::ReRolledActions;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::RE_ROLL_PROPERTIES, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.re_rolled_action = jj::get_enum_name(obj, "reRolledAction")?
            .and_then(ReRolledActions::for_name_shared)
            .map(|a| a.get_name().to_string());
        self.minimum_roll = jj::get_int(obj, "minimumRoll")?;
        self.fumble = jj::get_boolean(obj, "fumble")?;
        self.re_roll_skill = SkillFactory::from_json_value_in(rules, obj.get("skill"))?;
        self.modifying_skill = SkillFactory::from_json_value_in(rules, obj.get("modifyingSkill"))?;
        // Unconditional here (unlike DialogReRollParameter): forKey(null) is null.
        self.menu_property =
            CommonProperty::for_key_java(jj::get_string(obj, "menuProperty")?.as_deref()).map(|p| p.get_key().to_string());
        self.default_value_key = jj::get_string(obj, "defaultValueKey")?;
        // `messages.addAll(Arrays.asList(null))` throws for an absent or null array.
        self.messages
            .extend(jj::get_string_array(obj, "messageArray")?.ok_or("messageArray: NullPointerException (Arrays.asList(null))")?);
        self.re_roll_properties.extend(ReRollProperty::list_from_json(rules, obj)?);
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::RE_ROLL_PROPERTIES.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("reRolledAction".into(), jj::string_or_null(self.re_rolled_action.as_deref()));
        o.insert("minimumRoll".into(), Value::from(self.minimum_roll));
        o.insert("reRollProperties".into(), ReRollProperty::list_to_json(&self.re_roll_properties)?);
        o.insert("fumble".into(), Value::Bool(self.fumble));
        o.insert("skill".into(), SkillFactory::to_json_value_in(rules, self.re_roll_skill)?);
        o.insert("modifyingSkill".into(), SkillFactory::to_json_value_in(rules, self.modifying_skill)?);
        o.insert("defaultValueKey".into(), jj::string_or_null(self.default_value_key.as_deref()));
        if let Some(key) = &self.menu_property {
            o.insert("menuProperty".into(), Value::from(key.as_str()));
        }
        o.insert("messageArray".into(), jj::string_array_or_null(Some(&self.messages)));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use serde_json::json;

    #[test]
    fn has_property_true_when_present() {
        let p = DialogReRollPropertiesParameter {
            re_roll_properties: vec![Some(ReRollProperty::Trr)],
            ..Default::default()
        };
        assert!(p.has_property(ReRollProperty::Trr));
        assert!(!p.has_property(ReRollProperty::Loner));
    }

    #[test]
    fn has_property_false_when_absent() {
        let p = DialogReRollPropertiesParameter::default();
        assert!(!p.has_property(ReRollProperty::Trr));
    }

    fn round_trip(rules: Rules, input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogReRollPropertiesParameter::default();
        p.init_from(rules, &input)?;
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"dialogId": "reRollProperties", "playerId": "p1", "reRolledAction": "Rush",
            "minimumRoll": 2, "fumble": false, "skill": "swift as the breeze", "modifyingSkill": null,
            "defaultValueKey": null, "messageArray": ["m"], "reRollProperties": ["trr", "LONER"]});
        let out = round_trip(Rules::Bb2025, input).unwrap();
        assert_eq!(out, json!({"dialogId": "reRollProperties", "playerId": "p1", "reRolledAction": "Rush",
            "minimumRoll": 2, "fumble": false, "skill": "Swift As The Breeze", "modifyingSkill": null,
            "defaultValueKey": null, "messageArray": ["m"], "reRollProperties": ["TRR", "LONER"]}));
    }

    #[test]
    fn java_throws_reproduced() {
        let base = json!({"dialogId": "reRollProperties", "fumble": false, "messageArray": [], "reRollProperties": []});
        assert!(round_trip(Rules::Bb2025, base.clone()).is_ok());
        // The ReRollProperty factory exists only in BB2025: factory::forName on null.
        assert!(round_trip(Rules::Bb2020, base.clone()).is_err());
        // Unknown property: decodes to a null entry, toJsonValue NPEs in getName.
        let mut unknown = base.clone();
        unknown["reRollProperties"] = json!(["NOPE"]);
        let mut p = DialogReRollPropertiesParameter::default();
        assert!(p.init_from(Rules::Bb2025, &unknown).is_ok());
        assert!(p.to_json_value(Rules::Bb2025).is_err());
        // Missing messageArray / primitive fumble.
        let mut no_messages = base.clone();
        no_messages.as_object_mut().unwrap().remove("messageArray");
        assert!(round_trip(Rules::Bb2025, no_messages).is_err());
        let mut no_fumble = base;
        no_fumble.as_object_mut().unwrap().remove("fumble");
        assert!(round_trip(Rules::Bb2025, no_fumble).is_err());
    }
}
