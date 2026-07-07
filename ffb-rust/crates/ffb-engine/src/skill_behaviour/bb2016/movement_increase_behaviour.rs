use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2016.MovementIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setMovement(min(min(10, pos.getMovement()+2), player.getMovement()+1)))`.
pub struct MovementIncreaseBehaviour;

impl MovementIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for MovementIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for MovementIncreaseBehaviour {
    fn name(&self) -> &'static str { "MovementIncreaseBehaviour" }

    fn apply_modifier(&self, player: &mut Player, position: &RosterPosition) {
        player.movement = (position.movement + 2).min(10).min(player.movement + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_model::model::roster_position::RosterPosition;

    fn pos(movement: i32) -> RosterPosition {
        RosterPosition { movement, strength: 3, agility: 3, passing: 0, armour: 8, ..Default::default() }
    }

    #[test]
    fn apply_increases_by_one() {
        let b = MovementIncreaseBehaviour::new();
        let p_pos = pos(6);
        let mut player = Player { movement: 6, ..Default::default() };
        b.apply_modifier(&mut player, &p_pos);
        assert_eq!(player.movement, 7);
    }

    #[test]
    fn apply_capped_at_ten() {
        let b = MovementIncreaseBehaviour::new();
        let p_pos = pos(9);
        let mut player = Player { movement: 10, ..Default::default() };
        b.apply_modifier(&mut player, &p_pos);
        assert_eq!(player.movement, 10);
    }

    #[test]
    fn apply_capped_by_position_plus_two() {
        // pos.ma=6, cap=8; player.ma=8 → min(8, 9) = 8 (no change)
        let b = MovementIncreaseBehaviour::new();
        let p_pos = pos(6);
        let mut player = Player { movement: 8, ..Default::default() };
        b.apply_modifier(&mut player, &p_pos);
        assert_eq!(player.movement, 8);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(MovementIncreaseBehaviour::new().name(), "MovementIncreaseBehaviour");
    }
#[test]    fn name_is_not_empty() {        assert!(!MovementIncreaseBehaviour::new().name().is_empty());    }    #[test]    fn execute_step_hook_false_with_bb2020() {        use ffb_model::enums::Rules;        use crate::step::framework::test_team;        let b = MovementIncreaseBehaviour::new();        let mut game = ffb_model::model::game::Game::new(            test_team("home", 0), test_team("away", 0), Rules::Bb2020,        );        assert!(!b.execute_step_hook(&mut game));    }
}
