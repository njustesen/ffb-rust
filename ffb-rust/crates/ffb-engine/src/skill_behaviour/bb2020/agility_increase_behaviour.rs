use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2020.AgilityIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setAgility(max(max(1, pos.getAgility()-2), player.getAgility()-1)))`.
/// BB2020: agility stored as minimum roll needed (lower = better), so "increase" = decrement.
pub struct AgilityIncreaseBehaviour;

impl AgilityIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for AgilityIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for AgilityIncreaseBehaviour {
    fn name(&self) -> &'static str { "AgilityIncreaseBehaviour" }

    fn apply_modifier(&self, player: &mut Player, position: &RosterPosition) {
        player.agility = (position.agility - 2).max(1).max(player.agility - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(agility: i32) -> RosterPosition {
        RosterPosition { agility, ..Default::default() }
    }

    #[test]
    fn apply_decreases_by_one() {
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 3, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.agility, 2);
    }

    #[test]
    fn apply_capped_at_one() {
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 1, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.agility, 1);
    }

    #[test]
    fn apply_capped_by_position_minus_two() {
        // pos.ag=5, floor=3; player.ag=3 → max(3, 2) = 3 (no change)
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 3, ..Default::default() };
        b.apply_modifier(&mut player, &pos(5));
        assert_eq!(player.agility, 3);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(AgilityIncreaseBehaviour::new().name(), "AgilityIncreaseBehaviour");
    }
}
