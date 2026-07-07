use ffb_model::model::SpecialEffect;
use ffb_model::types::FieldCoordinate;

/// 1:1 translation of `com.fumbbl.ffb.net.commands.ClientCommandWizardSpell`.
/// Sent when a wizard casts a spell at a target square.
#[derive(Debug, Clone, Default)]
pub struct ClientCommandWizardSpell {
    /// Java: `fWizardSpell`
    pub wizard_spell: Option<SpecialEffect>,
    /// Java: `fTargetCoordinate`
    pub target_coordinate: Option<FieldCoordinate>,
}

impl ClientCommandWizardSpell {
    pub fn new() -> Self { Self::default() }
    pub fn with_spell(spell: SpecialEffect, target: FieldCoordinate) -> Self {
        Self { wizard_spell: Some(spell), target_coordinate: Some(target) }
    }
    pub fn get_wizard_spell(&self) -> Option<SpecialEffect> { self.wizard_spell }
    pub fn get_target_coordinate(&self) -> Option<FieldCoordinate> { self.target_coordinate }
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
}
