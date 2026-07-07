use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2016.AgilityIncreaseBehaviour`.
///
/// Java: `registerModifier(player -> player.setAgility(min(min(10, pos.getAgility()+2), player.getAgility()+1)))`.
/// BB2016: agility stored as direct value (higher = better), so increase = +1.
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
        player.agility = (position.agility + 2).min(10).min(player.agility + 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(agility: i32) -> RosterPosition {
        RosterPosition { agility, ..Default::default() }
    }

    #[test]
    fn apply_increases_by_one() {
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 3, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.agility, 4);
    }

    #[test]
    fn apply_capped_at_ten() {
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 10, ..Default::default() };
        b.apply_modifier(&mut player, &pos(9));
        assert_eq!(player.agility, 10);
    }

    #[test]
    fn apply_capped_by_position_plus_two() {
        // pos.ag=4, cap=6; player.ag=6 → min(6, 7) = 6 (no change)
        let b = AgilityIncreaseBehaviour::new();
        let mut player = Player { agility: 6, ..Default::default() };
        b.apply_modifier(&mut player, &pos(4));
        assert_eq!(player.agility, 6);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(AgilityIncreaseBehaviour::new().name(), "AgilityIncreaseBehaviour");
    }
    #[test]
    fn default_creates_instance_same_as_new() {
        let _a = AgilityIncreaseBehaviour::new();
        let _b = AgilityIncreaseBehaviour::default();
    }
}
