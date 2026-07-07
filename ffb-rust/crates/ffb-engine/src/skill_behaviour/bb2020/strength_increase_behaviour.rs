use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2020.StrengthIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setStrength(min(min(8, pos.getStrength()+2), player.getStrength()+1)))`.
/// BB2020: strength cap is 8 (vs BB2016's 10).
pub struct StrengthIncreaseBehaviour;

impl StrengthIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for StrengthIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for StrengthIncreaseBehaviour {
    fn name(&self) -> &'static str { "StrengthIncreaseBehaviour" }

    fn apply_modifier(&self, player: &mut Player, position: &RosterPosition) {
        player.strength = (position.strength + 2).min(8).min(player.strength + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(strength: i32) -> RosterPosition {
        RosterPosition { strength, ..Default::default() }
    }

    #[test]
    fn apply_increases_by_one() {
        let b = StrengthIncreaseBehaviour::new();
        let mut player = Player { strength: 3, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.strength, 4);
    }

    #[test]
    fn apply_capped_at_eight_not_ten() {
        let b = StrengthIncreaseBehaviour::new();
        let mut player = Player { strength: 8, ..Default::default() };
        b.apply_modifier(&mut player, &pos(9));
        assert_eq!(player.strength, 8);
    }

    #[test]
    fn apply_capped_by_position_plus_two() {
        // pos.st=3, cap=5; player.st=5 → min(5, 6) = 5 (no change)
        let b = StrengthIncreaseBehaviour::new();
        let mut player = Player { strength: 5, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.strength, 5);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(StrengthIncreaseBehaviour::new().name(), "StrengthIncreaseBehaviour");
    }
#[test]    fn name_is_not_empty() {        assert!(!StrengthIncreaseBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = StrengthIncreaseBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
