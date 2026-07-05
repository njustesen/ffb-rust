use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.mixed.MovementIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setMovement(min(min(9, pos.getMovement()+2), player.getMovement()+1)))`.
/// BB2020/BB2025: movement cap is 9 (vs BB2016's 10).
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
        player.movement = (position.movement + 2).min(9).min(player.movement + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(movement: i32) -> RosterPosition {
        RosterPosition { movement, ..Default::default() }
    }

    #[test]
    fn apply_increases_by_one() {
        let b = MovementIncreaseBehaviour::new();
        let mut player = Player { movement: 6, ..Default::default() };
        b.apply_modifier(&mut player, &pos(6));
        assert_eq!(player.movement, 7);
    }

    #[test]
    fn apply_capped_at_nine_not_ten() {
        let b = MovementIncreaseBehaviour::new();
        let mut player = Player { movement: 9, ..Default::default() };
        b.apply_modifier(&mut player, &pos(9));
        assert_eq!(player.movement, 9);
    }

    #[test]
    fn apply_capped_by_position_plus_two() {
        // pos.ma=6, cap=8; player.ma=8 → min(8, 9) = 8 (no change)
        let b = MovementIncreaseBehaviour::new();
        let mut player = Player { movement: 8, ..Default::default() };
        b.apply_modifier(&mut player, &pos(6));
        assert_eq!(player.movement, 8);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(MovementIncreaseBehaviour::new().name(), "MovementIncreaseBehaviour");
    }
}
