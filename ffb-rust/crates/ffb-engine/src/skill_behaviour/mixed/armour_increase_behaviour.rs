use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.mixed.ArmourIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setArmour(min(min(11, pos.getArmour()+2), player.getArmour()+1)))`.
/// BB2020/BB2025: armour cap raised to 11 (vs BB2016's 10).
pub struct ArmourIncreaseBehaviour;

impl ArmourIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for ArmourIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for ArmourIncreaseBehaviour {
    fn name(&self) -> &'static str { "ArmourIncreaseBehaviour" }

    fn apply_modifier(&self, player: &mut Player, position: &RosterPosition) {
        player.armour = (position.armour + 2).min(11).min(player.armour + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(armour: i32) -> RosterPosition {
        RosterPosition { armour, ..Default::default() }
    }

    #[test]
    fn apply_increases_by_one() {
        let b = ArmourIncreaseBehaviour::new();
        let mut player = Player { armour: 8, ..Default::default() };
        b.apply_modifier(&mut player, &pos(8));
        assert_eq!(player.armour, 9);
    }

    #[test]
    fn apply_capped_at_eleven_not_ten() {
        let b = ArmourIncreaseBehaviour::new();
        let mut player = Player { armour: 11, ..Default::default() };
        b.apply_modifier(&mut player, &pos(10));
        assert_eq!(player.armour, 11);
    }

    #[test]
    fn apply_can_reach_eleven() {
        let b = ArmourIncreaseBehaviour::new();
        let mut player = Player { armour: 10, ..Default::default() };
        b.apply_modifier(&mut player, &pos(10));
        assert_eq!(player.armour, 11);
    }

    #[test]
    fn apply_capped_by_position_plus_two() {
        // pos.av=8, cap=10; player.av=10 → min(10, 11) = 10 (no change)
        let b = ArmourIncreaseBehaviour::new();
        let mut player = Player { armour: 10, ..Default::default() };
        b.apply_modifier(&mut player, &pos(8));
        assert_eq!(player.armour, 10);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(ArmourIncreaseBehaviour::new().name(), "ArmourIncreaseBehaviour");
    }
}
