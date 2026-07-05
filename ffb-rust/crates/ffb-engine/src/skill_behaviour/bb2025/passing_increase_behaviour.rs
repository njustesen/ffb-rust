use crate::skill_behaviour::SkillBehaviour;
use ffb_model::model::player::Player;
use ffb_model::model::roster_position::RosterPosition;

/// 1:1 translation of `com.fumbbl.ffb.server.skillbehaviour.bb2025.PassingIncreaseBehaviour`.
///
/// Java:
/// ```java
/// registerModifier(player -> {
///     if (player.getPassing() <= 0) { player.setPassing(6); }
///     else { player.setPassing(max(max(1, pos.getPassing()-2), player.getPassing()-1)); }
/// });
/// ```
/// BB2025: identical to BB2020; players with no passing skill start at 6+.
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
    fn negative_passing_also_gives_six() {
        let b = PassingIncreaseBehaviour::new();
        let mut player = Player { passing: -1, ..Default::default() };
        b.apply_modifier(&mut player, &pos(0));
        assert_eq!(player.passing, 6);
    }

    #[test]
    fn name_is_correct() {
        assert_eq!(PassingIncreaseBehaviour::new().name(), "PassingIncreaseBehaviour");
    }
}
