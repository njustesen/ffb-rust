use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogInterceptionParameter.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogInterceptionParameter {
    pub thrower_id: Option<String>,
    pub interception_skill: Option<SkillId>,
    pub skill_mnemonic: i32,
}

impl DialogInterceptionParameter {
    pub fn get_thrower_id(&self) -> Option<&str> { self.thrower_id.as_deref() }
    pub fn get_interception_skill(&self) -> Option<SkillId> { self.interception_skill }
    pub fn get_skill_mnemonic(&self) -> i32 { self.skill_mnemonic }
}

impl IDialogParameter for DialogInterceptionParameter {
    fn get_id(&self) -> DialogId { DialogId::INTERCEPTION }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogInterceptionParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::INTERCEPTION, obj)?;
        self.thrower_id = jj::get_string(obj, "throwerId")?;
        self.interception_skill = SkillFactory::from_json_value_in(rules, obj.get("skill"))?;
        if jj::is_defined_in(obj, "skillMnemonic") {
            self.skill_mnemonic = jj::get_int(obj, "skillMnemonic")?;
        }
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::INTERCEPTION.get_name()));
        o.insert("throwerId".into(), jj::string_or_null(self.thrower_id.as_deref()));
        o.insert("skill".into(), SkillFactory::to_json_value_in(rules, self.interception_skill)?);
        o.insert("skillMnemonic".into(), Value::from(self.skill_mnemonic));
        Ok(Value::Object(o))
    }
}
