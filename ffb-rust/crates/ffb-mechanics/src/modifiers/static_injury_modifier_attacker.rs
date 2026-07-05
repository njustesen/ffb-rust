use ffb_model::enums::SkillId;
use ffb_model::model::Player;
use crate::modifiers::injury_modifier::InjuryModifier;
use crate::modifiers::injury_modifier_context::InjuryModifierContext;

/// 1:1 translation of com.fumbbl.ffb.modifiers.StaticInjuryModifierAttacker.
/// appliesToContext: UtilCards.hasSkill(attacker, registeredTo) → attacker has_skill matching registered_to.
pub struct StaticInjuryModifierAttacker {
    pub name: String,
    pub modifier: i32,
    pub niggling_injury_modifier: bool,
    pub registered_to: Option<String>,
}

impl StaticInjuryModifierAttacker {
    pub fn new(name: impl Into<String>, modifier: i32, niggling_injury_modifier: bool) -> Self {
        Self { name: name.into(), modifier, niggling_injury_modifier, registered_to: None }
    }
}

impl InjuryModifier for StaticInjuryModifierAttacker {
    fn get_modifier(&self, _attacker: Option<&Player>, _defender: &Player) -> i32 { self.modifier }
    fn get_name(&self) -> &str { &self.name }
    fn is_niggling_injury_modifier(&self) -> bool { self.niggling_injury_modifier }
    fn applies_to_context(&self, context: &InjuryModifierContext<'_>) -> bool {
        let attacker = match context.get_attacker() {
            Some(a) => a,
            None => return false,
        };
        match &self.registered_to {
            Some(class_name) => SkillId::from_class_name(class_name)
                .map(|id| attacker.has_skill(id))
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
        }
    }

    fn minimal_game() -> Game {
        Game::new(empty_team(), empty_team(), Rules::Bb2025)
    }

    fn bare_player() -> Player { Player::default() }

    fn ctx_with_attacker<'a>(game: &'a Game, attacker: &'a Player, defender: &'a Player) -> InjuryModifierContext<'a> {
        InjuryModifierContext::new(game, Some(attacker), defender, false, false, false, false)
    }

    fn ctx_no_attacker<'a>(game: &'a Game, defender: &'a Player) -> InjuryModifierContext<'a> {
        InjuryModifierContext::new(game, None, defender, false, false, false, false)
    }

    #[test]
    fn applies_false_when_no_attacker() {
        let mut m = StaticInjuryModifierAttacker::new("test", 1, false);
        m.set_registered_to(Some("Block".into()));
        let game = minimal_game();
        let defender = bare_player();
        let ctx = ctx_no_attacker(&game, &defender);
        assert!(!m.applies_to_context(&ctx));
    }

    #[test]
    fn applies_false_when_attacker_lacks_skill() {
        let mut m = StaticInjuryModifierAttacker::new("test", 1, false);
        m.set_registered_to(Some("Block".into()));
        let game = minimal_game();
        let attacker = bare_player();
        let defender = bare_player();
        let ctx = ctx_with_attacker(&game, &attacker, &defender);
        assert!(!m.applies_to_context(&ctx));
    }

    #[test]
    fn applies_true_when_attacker_has_registered_skill() {
        let mut m = StaticInjuryModifierAttacker::new("test", 1, false);
        m.set_registered_to(Some("Block".into()));
        let game = minimal_game();
        let mut attacker = bare_player();
        attacker.starting_skills.push(SkillWithValue::new(SkillId::Block));
        let defender = bare_player();
        let ctx = ctx_with_attacker(&game, &attacker, &defender);
        assert!(m.applies_to_context(&ctx));
    }

    #[test]
    fn applies_true_when_no_registered_to() {
        let m = StaticInjuryModifierAttacker::new("test", 1, false);
        let game = minimal_game();
        let attacker = bare_player();
        let defender = bare_player();
        let ctx = ctx_with_attacker(&game, &attacker, &defender);
        assert!(m.applies_to_context(&ctx));
    }
}
