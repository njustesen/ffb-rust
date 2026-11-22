use serde::{Deserialize, Serialize};
use crate::enums::SkillId;
use super::dialog_id::DialogId;
use super::i_dialog_parameter::IDialogParameter;

/// 1:1 translation of com.fumbbl.ffb.dialog.DialogSelectSkillParameter.
/// Note: SkillChoiceMode serialized as String (stub not yet translated).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogSelectSkillParameter {
    pub skills: Vec<SkillId>,
    pub player_id: Option<String>,
    pub skill_choice_mode: Option<String>,
    /// Java-only (BACKLOG §O): positions in the Java `skills` list that hold a null `Skill` (a
    /// name the ruleset's SkillFactory does not know); `skills` holds the others in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub null_skill_slots: Vec<usize>,
}

impl DialogSelectSkillParameter {
    pub fn get_skills(&self) -> &[SkillId] { &self.skills }
    pub fn get_player_id(&self) -> Option<&str> { self.player_id.as_deref() }
    pub fn get_skill_choice_mode(&self) -> Option<&str> { self.skill_choice_mode.as_deref() }
    pub fn add_skill(&mut self, skill: SkillId) { self.skills.push(skill); }
}

impl IDialogParameter for DialogSelectSkillParameter {
    fn get_id(&self) -> DialogId { DialogId::SELECT_SKILL }
    fn transform(&self) -> Box<dyn IDialogParameter> { Box::new(self.clone()) }
}

// JSON serialization (BACKLOG §O).
impl DialogSelectSkillParameter {
    /// Java `initFrom(IFactorySource, JsonValue)`.
    pub fn init_from(&mut self, rules: crate::enums::Rules, json: &serde_json::Value) -> Result<(), String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::model::skill_choice_mode::SkillChoiceMode;
        use crate::util::java_json as jj;
        let obj = jj::to_json_object(json)?;
        super::util_dialog_parameter::validate_dialog_id_json(DialogId::SELECT_SKILL, obj)?;
        self.player_id = jj::get_string(obj, "playerId")?;
        // JsonArrayOption (absent/null -> empty); each element UtilJson.toEnumWithName(skill
        // factory): JSON null or an unknown name -> a null list entry.
        for element in jj::get_json_array_option(obj, "skillArray")? {
            match SkillFactory::from_json_value_in(rules, Some(&element))? {
                Some(skill) => self.skills.push(skill),
                None => self.null_skill_slots.push(self.skills.len() + self.null_skill_slots.len()),
            }
        }
        // SkillChoiceMode.valueOf(JsonStringOption): absent/null or not a constant name -> throws.
        let mode = SkillChoiceMode::value_of(jj::get_string(obj, "skillChoiceMode")?.as_deref())?;
        self.skill_choice_mode = Some(mode.name().to_string());
        Ok(())
    }

    /// Java `toJsonValue()`.
    pub fn to_json_value(&self, rules: crate::enums::Rules) -> Result<serde_json::Value, String> {
        use crate::factory::skill_factory::SkillFactory;
        use crate::util::java_json as jj;
        use serde_json::Value;
        let mut o = serde_json::Map::new();
        o.insert("dialogId".into(), Value::from(DialogId::SELECT_SKILL.get_name()));
        o.insert("playerId".into(), jj::string_or_null(self.player_id.as_deref()));
        let mut skills = self.skills.iter();
        let mut array = Vec::with_capacity(self.skills.len() + self.null_skill_slots.len());
        for slot in 0..self.skills.len() + self.null_skill_slots.len() {
            if self.null_skill_slots.contains(&slot) {
                array.push(Value::Null);
            } else {
                array.push(SkillFactory::to_json_value_in(rules, skills.next().copied())?);
            }
        }
        o.insert("skillArray".into(), Value::Array(array));
        // skillChoiceMode.name(): null -> NPE.
        let mode = self.skill_choice_mode.as_deref().ok_or("skillChoiceMode.name() on null")?;
        o.insert("skillChoiceMode".into(), Value::from(mode));
        Ok(Value::Object(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_skill_and_serde() {
        let mut p = DialogSelectSkillParameter {
            player_id: Some("player1".into()),
            ..Default::default()
        };
        p.add_skill(SkillId::Dodge);
        p.add_skill(SkillId::Block);
        let json = serde_json::to_string(&p).unwrap();
        let back: DialogSelectSkillParameter = serde_json::from_str(&json).unwrap();
        assert_eq!(back.get_skills(), &[SkillId::Dodge, SkillId::Block]);
        assert_eq!(back.get_player_id(), Some("player1"));
    }

    #[test]
    fn json_round_trip_keeps_unknown_skills_as_null_slots() {
        use crate::enums::Rules;
        use serde_json::json;
        let input = json!({"dialogId": "selectSkill", "playerId": "p1",
            "skillArray": ["Block", "Dwarfen Grit", "sure hands", null], "skillChoiceMode": "INTENSIVE_TRAINING"});
        let mut p = DialogSelectSkillParameter::default();
        p.init_from(Rules::Bb2025, &input).unwrap();
        assert_eq!(p.get_skills(), &[SkillId::Block, SkillId::SureHands]);
        assert_eq!(p.to_json_value(Rules::Bb2025).unwrap(), json!({"dialogId": "selectSkill", "playerId": "p1",
            "skillArray": ["Block", null, "Sure Hands", null], "skillChoiceMode": "INTENSIVE_TRAINING"}));
        // SkillChoiceMode.valueOf is exact: the display name throws.
        let bad = json!({"dialogId": "selectSkill", "skillChoiceMode": "intensiveTraining"});
        assert!(DialogSelectSkillParameter::default().init_from(Rules::Bb2025, &bad).is_err());
    }
}
