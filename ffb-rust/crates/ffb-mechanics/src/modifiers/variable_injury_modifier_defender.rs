use ffb_model::enums::SkillId;
use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.VariableInjuryModifierDefender.
/// appliesToContext: context.isDefenderMode() && UtilCards.hasSkill(defender, registeredTo)
pub struct VariableInjuryModifierDefender {
    pub name: String,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
}

impl VariableInjuryModifierDefender {
    pub fn new(name: impl Into<String>, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), niggling_injury_modifier, registered_to: None }
    }
}

impl InjuryModifier for VariableInjuryModifierDefender {
    fn get_modifier(&self, _attacker: Option<&Player>, defender: &Player) -> i32 {
        defender.get_skill_int_value(self.registered_to.as_deref().unwrap_or(""))
    }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool {
        if !context.is_defender_mode() {
            return false;
        }
        let defender = context.get_defender();
        match &self.registered_to {
            Some(class_name) => SkillId::from_class_name(class_name)
                .map(|id| defender.has_skill(id))
                .unwrap_or(false),
            None => true,
        }
    }
    fn registered_to(&self) -> Option<&str> { self.registered_to.as_deref() }
    fn set_registered_to(&mut self, skill_id: Option<String>) { self.registered_to = skill_id; }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::enums::Rules;
    use ffb_model::model::{Game, Team};
    use ffb_model::model::skill_def::SkillWithValue;

    fn empty_team() -> Team {
        Team {
            id: "t".into(), name: "T".into(), race: "human".into(),
            roster_id: "human".into(), coach: "Coach".into(),
            rerolls: 0, apothecaries: 0, bribes: 0, master_chefs: 0,
            prayers_to_nuffle: 0, bloodweiser_kegs: 0, riotous_rookies: 0,
            cheerleaders: 0, assistant_coaches: 0, fan_factor: 0, dedicated_fans: 0,
            team_value: 0, treasury: 0, special_rules: vec![], players: vec![],
            vampire_lord: false,
            necromancer: false,
        }
    }

    fn minimal_game() -> Game { Game::new(empty_team(), empty_team(), Rules::Bb2025) }
    fn bare_player() -> Player { Player::default() }

    fn ctx<'a>(game: &'a Game, defender: &'a Player) -> InjuryModifierContext<'a> {
        let mut c = InjuryModifierContext::new(game, None, defender, false, false, false, false);
        c.set_defender_mode();
        c
    }

    fn ctx_attacker_mode<'a>(game: &'a Game, defender: &'a Player) -> InjuryModifierContext<'a> {
        InjuryModifierContext::new(game, None, defender, false, false, false, false)
    }

    #[test]
    fn applies_false_when_attacker_mode() {
        let mut m = VariableInjuryModifierDefender::new("test", false);
        m.set_registered_to(Some("Dodge".into()));
        let game = minimal_game();
        let mut defender = bare_player();
        defender.starting_skills.push(SkillWithValue::new(SkillId::Dodge));
        assert!(!m.applies_to_context(&ctx_attacker_mode(&game, &defender)));
    }

    #[test]
    fn applies_false_when_defender_lacks_skill() {
        let mut m = VariableInjuryModifierDefender::new("test", false);
        m.set_registered_to(Some("Dodge".into()));
        let game = minimal_game();
        let defender = bare_player();
        assert!(!m.applies_to_context(&ctx(&game, &defender)));
    }

    #[test]
    fn applies_true_when_defender_has_registered_skill() {
        let mut m = VariableInjuryModifierDefender::new("test", false);
        m.set_registered_to(Some("Dodge".into()));
        let game = minimal_game();
        let mut defender = bare_player();
        defender.starting_skills.push(SkillWithValue::new(SkillId::Dodge));
        assert!(m.applies_to_context(&ctx(&game, &defender)));
    }
}
