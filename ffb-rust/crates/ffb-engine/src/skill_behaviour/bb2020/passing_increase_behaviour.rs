use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2020.PassingIncreaseBehaviour`.
///
/// Java:
/// ```java
/// registerModifier(player -> {
///     if (player.getPassing() <= 0) { player.setPassing(6); }
///     else { player.setPassing(max(max(1, pos.getPassing()-2), player.getPassing()-1)); }
/// });
/// ```
/// BB2020: passing stored as minimum roll needed (lower = better). Players with no passing
/// skill (passing <= 0) start at 6+ when gaining the stat.
pub struct PassingIncreaseBehaviour;

impl PassingIncreaseBehaviour {
    pub fn new() -> Self { Self }
}

impl Default for PassingIncreaseBehaviour {
    fn default() -> Self { Self::new() }
}

impl SkillBehaviour for PassingIncreaseBehaviour {
    fn name(&self) -> &'static str { "PassingIncreaseBehaviour" }

    fn apply_modifier(&self, player: &mut Player, position: &RosterPosition) {
        if player.passing <= 0 {
            player.passing = 6;
        } else {
            player.passing = (position.passing - 2).max(1).max(player.passing - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(passing: i32) -> RosterPosition {
        RosterPosition { passing, ..Default::default() }
    }

    #[test]
    fn apply_no_passing_gives_six() {
        let b = PassingIncreaseBehaviour::new();
        let mut player = Player { passing: 0, ..Default::default() };
        b.apply_modifier(&mut player, &pos(0));
        assert_eq!(player.passing, 6);
    }

    #[test]
    fn apply_decreases_by_one() {
        let b = PassingIncreaseBehaviour::new();
        let mut player = Player { passing: 4, ..Default::default() };
        b.apply_modifier(&mut player, &pos(4));
        assert_eq!(player.passing, 3);
    }

    #[test]
    fn apply_capped_at_one() {
        let b = PassingIncreaseBehaviour::new();
        let mut player = Player { passing: 1, ..Default::default() };
        b.apply_modifier(&mut player, &pos(3));
        assert_eq!(player.passing, 1);
    }

    #[test]
    fn apply_capped_by_position_minus_two() {
        // pos.pa=5, floor=3; player.pa=3 → max(3, 2) = 3 (no change)
        let b = PassingIncreaseBehaviour::new();
        let mut player = Player { passing: 3, ..Default::default() };
        b.apply_modifier(&mut player, &pos(5));
        assert_eq!(player.passing, 3);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(PassingIncreaseBehaviour::new().name(), "PassingIncreaseBehaviour");
    }
}
