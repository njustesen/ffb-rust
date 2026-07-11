use ffb_model::enums::NetCommandId;
use ffb_model::model::SpecialEffect;
use ffb_model::types::FieldCoordinate;
use crate::commands::client_command::ClientCommand;
use crate::net_command::NetCommand;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandWizardSpell`.
/// Sent when a wizard casts a spell at a target square.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandWizardSpell {
    /// Java: base-class `ClientCommand.fEntropy`.
    pub entropy: Option<u8>,
    /// Java: `fWizardSpell`
    pub wizard_spell: Option<SpecialEffect>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandWizardSpell {
    pub fn new() -> Self { Self::default() }
    pub fn with_spell(spell: SpecialEffect, target: FieldCoordinate) -> Self {
        Self { entropy: None, wizard_spell: Some(spell), target_coordinate: Some(target) }
    }
    pub fn get_wizard_spell(&self) -> Option<SpecialEffect> { self.wizard_spell }
    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }

    /// Java: `ClientCommandWizardSpell.toJsonValue()` (calls `super.toJsonValue()` first).
    pub fn to_json_value(&self) -> serde_json::Value {
        let base = ClientCommand { entropy: self.entropy };
        let mut map = base.base_json_fields(self.get_id());
        if let Some(spell) = self.wizard_spell {
            map.insert("wizardSpell".to_string(), serde_json::json!(spell.get_name()));
        }
        if let Some(coordinate) = self.target_coordinate {
            map.insert("targetCoordinate".to_string(), coordinate.to_json_value());
        }
        serde_json::Value::Object(map)
    }

    /// Java: `ClientCommandWizardSpell.initFrom(source, jsonValue)`.
    pub fn from_json(json: &serde_json::Value) -> Self {
        let base = ClientCommand::base_from_json(json);
        Self {
            entropy: base.entropy,
            wizard_spell: json.get("wizardSpell").and_then(|v| v.as_str()).and_then(SpecialEffect::for_name),
            target_coordinate: json.get("targetCoordinate").and_then(FieldCoordinate::from_json),
        }
    }
}

impl NetCommand for ClientCommandWizardSpell {
    fn get_id(&self) -> NetCommandId {
        NetCommandId::ClientWizardSpell
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_none() {
        let cmd = ClientCommandWizardSpell::new();
        assert!(cmd.wizard_spell.is_none());
    }

    #[test]
    fn with_spell_stores_values() {
        use ffb_model::types::FieldCoordinate;
        let cmd = ClientCommandWizardSpell::with_spell(SpecialEffect::FIREBALL, FieldCoordinate::new(3, 5));
        assert!(cmd.get_wizard_spell().is_some());
        assert_eq!(cmd.get_target_coordinate(), Some(FieldCoordinate::new(3, 5)));
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandWizardSpell::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandWizardSpell::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandWizardSpell::default());
        assert!(s.contains("ClientCommandWizardSpell"));
    }

    #[test]
    fn get_id_is_client_wizard_spell() {
        assert_eq!(ClientCommandWizardSpell::new().get_id(), NetCommandId::ClientWizardSpell);
    }

    #[test]
    fn to_json_value_has_net_command_id_and_wizard_spell() {
        let cmd = ClientCommandWizardSpell::with_spell(SpecialEffect::ZAP, FieldCoordinate::new(1, 1));
        let json = cmd.to_json_value();
        assert_eq!(json["netCommandId"], "clientWizardSpell");
        assert_eq!(json["wizardSpell"], "zap");
    }

    #[test]
    fn round_trip_with_spell_and_entropy() {
        let mut cmd = ClientCommandWizardSpell::with_spell(SpecialEffect::BOMB, FieldCoordinate::new(4, 6));
        cmd.entropy = Some(3);
        let json = cmd.to_json_value();
        let restored = ClientCommandWizardSpell::from_json(&json);
        assert_eq!(restored.entropy, Some(3));
        assert_eq!(restored.wizard_spell, Some(SpecialEffect::BOMB));
        assert_eq!(restored.target_coordinate, Some(FieldCoordinate::new(4, 6)));
    }

    #[test]
    fn round_trip_with_no_spell() {
        let cmd = ClientCommandWizardSpell::new();
        let json = cmd.to_json_value();
        let restored = ClientCommandWizardSpell::from_json(&json);
        assert!(restored.wizard_spell.is_none());
        assert!(restored.target_coordinate.is_none());
    }
}
