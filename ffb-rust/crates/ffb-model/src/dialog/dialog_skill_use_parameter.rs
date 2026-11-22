use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSkillUseParameter.
/// Note: SkillUse/CommonProperty serialized as String (stubs not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSkillUseParameter {
    pub player_id: Option<String>,
    pub default_value_key: Option<String>,
    pub skill: Option<SkillId>,
    pub modifying_skill: Option<SkillId>,
    pub minimum_roll: i32,
    pub show_never_use: bool,
    /// SkillUse serialized by name.
    pub skill_use: Option<String>,
    /// CommonProperty serialized by key.
    pub menu_property: Option<String>,
}

impl DialogSkillUseParameter {
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_default_value_key(&self) -> Option<&str> { self.default_value_key.as_deref() }
    pub fn get_skill(&self) -> Option<SkillId> { self.skill }
    pub fn get_modifying_skill(&self) -> Option<SkillId> { self.modifying_skill }
    pub fn get_minimum_roll(&self) -> i32 { self.minimum_roll }
    pub fn is_show_never_use(&self) -> bool { self.show_never_use }
    pub fn get_skill_use(&self) -> Option<&str> { self.skill_use.as_deref() }
    pub fn get_menu_property(&self) -> Option<&str> { self.menu_property.as_deref() }
}

impl IDialogParameter for DialogSkillUseParameter {
    fn get_id(&self) -> DialogId { DialogId::SKILL_USE }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSkillUseParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::model::common_property::CommonProperty;
        use crate::model::skill_use::SkillUse;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SKILL_USE, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        self.skill = SkillFactory::from_json_value_in(rules, obj.get("skill"))?;
        self.minimum_roll = jj::get_int(obj, "minimumRoll")?;
        self.modifying_skill = SkillFactory::from_json_value_in(rules, obj.get("modifyingSkill"))?;
        if jj::is_defined_in(obj, "menuProperty") {
            self.menu_property = CommonProperty::for_key_java(jj::get_string(obj, "menuProperty")?.as_deref())
                .map(|p| p.get_key().to_string());
        }
        self.default_value_key = jj::get_string(obj, "defaultValueKey")?;
        if jj::is_defined_in(obj, "showNeverUse") {
            self.show_never_use = jj::get_boolean(obj, "showNeverUse")?;
        }
        // Newer-server members (e.g. `messageArray`) are not read: they vanish.
        self.skill_use = jj::get_enum_name(obj, "skillUse")?.and_then(SkillUse::for_name_java).map(|u| u.get_name().to_string());
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::SKILL_USE.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        o.insert("skill".into(), SkillFactory::to_json_value_in(rules, self.skill)?);
        o.insert("minimumRoll".into(), Value::from(self.minimum_roll));
        o.insert("modifyingSkill".into(), SkillFactory::to_json_value_in(rules, self.modifying_skill)?);
        if let Some(key) = &self.menu_property {
            o.insert("menuProperty".into(), Value::from(key.as_str()));
        }
        o.insert("defaultValueKey".into(), jj::string_or_null(self.default_value_key.as_deref()));
        o.insert("showNeverUse".into(), Value::Bool(self.show_never_use));
        o.insert("skillUse".into(), jj::string_or_null(self.skill_use.as_deref()));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Rules;
    use serde_json::json;

    fn round_trip(rules: Rules, input: serde_json::Value) -> Result<serde_json::Value, String> {
        let mut p = DialogSkillUseParameter::default();
        p.init_from(rules, &input)?;
        p.to_json_value(rules)
    }

    #[test]
    fn json_round_trip_matches_java() {
        let input = json!({"dialogId": "skillUse", "playerId": "p1", "skill": "sidestep", "minimumRoll": 0,
            "modifyingSkill": null, "menuProperty": "setting.reRollBallAndChain", "defaultValueKey": "dodgeBlockAlways",
            "showNeverUse": false, "skillUse": "REROLLPUNTDIRECTION", "messageArray": []});
        let out = round_trip(Rules::Bb2025, input).unwrap();
        // messageArray is not read by this Java class; the skill/skillUse names are canonicalised.
        assert_eq!(out, json!({"dialogId": "skillUse", "playerId": "p1", "skill": "Sidestep", "minimumRoll": 0,
            "modifyingSkill": null, "menuProperty": "setting.reRollBallAndChain", "defaultValueKey": "dodgeBlockAlways",
            "showNeverUse": false, "skillUse": "reRollPuntDirection"}));
    }

    #[test]
    fn defaults_unknown_names_and_shared_skill_use_name() {
        // showNeverUse is optional here (only read when defined); an unknown skill or skillUse is null.
        let out = round_trip(Rules::Bb2016, json!({"dialogId": "skillUse", "skill": "Sidestep", "skillUse": "gainFrenzy"})).unwrap();
        assert_eq!(out["skill"], json!(null)); // BB2016 spells it "Side Step"
        assert_eq!(out["showNeverUse"], json!(false));
        assert_eq!(out["skillUse"], json!("gainFrenzy"));
        assert!(out.get("menuProperty").is_none());
        // CommonProperty.forKey of a key Java does not have -> null -> not written.
        let out = round_trip(Rules::Bb2025, json!({"dialogId": "skillUse", "menuProperty": "setting.dodgeBlock"})).unwrap();
        assert!(out.get("menuProperty").is_none());
        assert!(round_trip(Rules::Bb2025, json!({"dialogId": "skillUse", "showNeverUse": null})).is_err());
    }
}
