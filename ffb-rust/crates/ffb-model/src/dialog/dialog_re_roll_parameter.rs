use serde::{Deserialize, Serialize};
use crate::enums::ReRollSource;
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogReRollParameter.
/// Note: ReRolledAction serialized as String; CommonProperty serialized as String key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogReRollParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    /// ReRolledAction serialized by name.
    pub re_rolled_action: Option<String>,
    pub minimum_roll: i32,
    pub team_re_roll_option: bool,
    pub pro_re_roll_option: bool,
    pub fumble: bool,
    pub single_use_re_roll_source: Option<ReRollSource>,
    pub re_roll_skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    /// Java `List<String> messages`: null unless set (JSON `messageArray` absent -> null); a
    /// JSON-null element is a null String.
    pub messages: Option<Vec<Option<String>>>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogReRollParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_re_rolled_action(&self) -> Option<&str> { self.re_rolled_action.as_deref() }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_team_re_roll_option(&self) -> bool { self.team_re_roll_option }
    pub fn is_pro_re_roll_option(&self) -> bool { self.pro_re_roll_option }
    pub fn is_fumble(&self) -> bool { self.fumble }
    pub fn get_messages(&self) -> Option<&[Option<String>]> { self.messages.as_deref() }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl IDialogParameter for DialogReRollParameter {
    fn get_id(&self) -> DialogId { DialogId::RE_ROLL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogReRollParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::re_roll_source_factory::ReRollSourceFactory;
        use crate::factory::skill_factory::SkillFactory;
        use crate::model::common_property::CommonProperty;
        use crate::model::re_rolled_actions::ReRolledActions;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::RE_ROLL, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.re_rolled_action = jj::get_enum_name(obj, "reRolledAction")?
            .and_then(ReRolledActions::for_name_shared)
            .map(|a| a.get_name().to_string());
        self.minimum_roll = jj::get_int(obj, "minimumRoll")?;
        self.team_re_roll_option = jj::get_boolean(obj, "teamReRollOption")?;
        self.pro_re_roll_option = jj::get_boolean(obj, "proReRollOption")?;
        self.single_use_re_roll_source =
            jj::get_enum_name(obj, "reRollSourceSingleUse")?.and_then(ReRollSourceFactory::for_name_shared).cloned();
        self.fumble = jj::get_boolean(obj, "fumble")?;
        self.re_roll_skill = SkillFactory::from_json_value_in(rules, obj.get("skill"))?;
        self.modifying_skill = SkillFactory::from_json_value_in(rules, obj.get("modifyingSkill"))?;
        if jj::is_defined_in(obj, "menuProperty") {
            self.menu_property = CommonProperty::for_key_java(jj::get_string(obj, "menuProperty")?.as_deref())
                .map(|p| p.get_key().to_string());
        }
        self.default_value_key = jj::get_string(obj, "defaultValueKey")?;
        if jj::is_defined_in(obj, "messageArray") {
            // `Arrays.asList(null)` throws for a JSON-null array.
            self.messages = Some(
                jj::get_string_array(obj, "messageArray")?.ok_or("messageArray: NullPointerException (Arrays.asList(null))")?,
            );
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::RE_ROLL.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("reRolledAction".into(), jj::string_or_null(self.re_rolled_action.as_deref()));
        o.insert("minimumRoll".into(), Value::from(self.minimum_roll));
        o.insert("teamReRollOption".into(), Value::Bool(self.team_re_roll_option));
        o.insert("proReRollOption".into(), Value::Bool(self.pro_re_roll_option));
        o.insert(
            "reRollSourceSingleUse".into(),
            jj::string_or_null(self.single_use_re_roll_source.as_ref().map(|s| s.name.as_str())),
        );
        o.insert("fumble".into(), Value::Bool(self.fumble));
        o.insert("skill".into(), SkillFactory::to_json_value_in(rules, self.re_roll_skill)?);
        o.insert("modifyingSkill".into(), SkillFactory::to_json_value_in(rules, self.modifying_skill)?);
        o.insert("defaultValueKey".into(), jj::string_or_null(self.default_value_key.as_deref()));
        if let Some(key) = &self.menu_property {
            o.insert("menuProperty".into(), Value::from(key.as_str()));
        }
        if let Some(messages) = &self.messages {
            o.insert("messageArray".into(), jj::string_array_or_null(Some(messages)));
        }
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use serde_json::json;

    #[test]
    fn serde_round_trip() {
        let p = DialogReRollParameter {
            player_id: Some("p42".into()),
            minimum_roll: 4,
            team_re_roll_option: true,
            re_roll_skill: Some(SkillId::Pro),
            messages: Some(vec![Some("Roll for it!".into())]),
            ..Default::default()
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogReRollParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_player_id(), Some("p42"));
        assert_eq!(back.get_minimum_roll(), 4);
        assert!(back.is_team_re_roll_option());
        assert_eq!(back.re_roll_skill, Some(SkillId::Pro));
    }

    fn round_trip(rules: Rules, input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogReRollParameter::default();
        p.init_from(rules, &input)?;
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"dialogId": "reRoll", "playerId": "p1", "reRolledAction": "dodge", "minimumRoll": 3,
            "teamReRollOption": true, "proReRollOption": false, "reRollSourceSingleUse": "pro", "fumble": false,
            "skill": "dodge", "modifyingSkill": null, "defaultValueKey": null});
        let out = round_trip(Rules::Bb2016, input).unwrap();
        // forName is lower-case keyed; getName() is written back. menuProperty/messageArray absent -> omitted.
        assert_eq!(out, json!({"dialogId": "reRoll", "playerId": "p1", "reRolledAction": "Dodge", "minimumRoll": 3,
            "teamReRollOption": true, "proReRollOption": false, "reRollSourceSingleUse": "Pro", "fumble": false,
            "skill": "Dodge", "modifyingSkill": null, "defaultValueKey": null}));
    }

    #[test]
    fn skill_class_actions_unknown_names_and_menu_property() {
        let input = json!({"dialogId": "reRoll", "reRolledAction": "com.fumbbl.ffb.skill.common.Pass",
            "teamReRollOption": false, "proReRollOption": false, "fumble": true, "skill": "No Such Skill",
            "menuProperty": "not.a.key", "messageArray": ["a", null]});
        let out = round_trip(Rules::Bb2025, input).unwrap();
        assert_eq!(out["reRolledAction"], json!("com.fumbbl.ffb.skill.common.Pass"));
        assert_eq!(out["skill"], json!(null));
        assert!(out.get("menuProperty").is_none()); // unknown key -> null -> guarded addTo omits it
        assert_eq!(out["messageArray"], json!(["a", null]));
        let ok = json!({"dialogId": "reRoll", "teamReRollOption": false, "proReRollOption": false, "fumble": false,
            "menuProperty": "SETTING.reRollBallAndChain"});
        assert_eq!(round_trip(Rules::Bb2025, ok).unwrap()["menuProperty"], json!("setting.reRollBallAndChain"));
    }

    #[test]
    fn missing_primitive_boolean_and_null_message_array_throw() {
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "reRoll", "proReRollOption": false, "fumble": false})).is_err());
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "reRoll", "teamReRollOption": false,
            "proReRollOption": false, "fumble": false, "messageArray": null})).is_err());
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "skillUse"})).is_err());
    }
}
