use crate::model::game_state::GameState;
use crate::model::player::{Player, PlayerStats};
use crate::skills::SkillSet;
use crate::types::{FieldCoordinate, PlayerId, PlayerState, SpecialRule, TeamId};

// ── Masters of Undeath ────────────────────────────────────────────────────────

/// After a casualty is inflicted on a player belonging to `injured_team`,
/// check whether the opposing team has the Masters of Undeath special rule.
/// If so and there are fewer than 11 Undead players on the pitch, place a new
/// Zombie on the nearest empty square in the opponent's half.
///
/// Returns the `PlayerId` of the raised Zombie if one was placed.
pub fn check_masters_of_undeath(
    state: &mut GameState,
    injured_team: TeamId,
) -> Option<PlayerId> {
    let raising_team = injured_team.opponent();
    if !state.team(raising_team).has_special_rule(SpecialRule::MastersOfUndeath) {
        return None;
    }

    // Count current players on pitch for the raising team
    let on_pitch = state.field
        .team_players_on_pitch(raising_team)
        .count();
    if on_pitch >= 11 {
        return None;
    }

    // Find an empty square in the raising team's half to place the zombie
    let half_y_range = if raising_team == TeamId::Home {
        0u8..=7u8   // home team defends left half (y 0..=7 in our layout)
    } else {
        9u8..=16u8
    };

    let target_square = (0u8..26).flat_map(|x| {
        half_y_range.clone().map(move |y| FieldCoordinate::new(x, y))
    })
    .find(|&c| state.field.player_at(c).is_none());

    let target = target_square?;

    // Create a fresh Zombie and add it to the raising team's roster
    let team = state.team_mut(raising_team);
    let zombie_idx = team.players().len();
    let zombie_id = PlayerId(format!("{}_zombie_{zombie_idx}", raising_team.as_str()));
    let zombie = Player::new(
        zombie_id.clone(),
        format!("Zombie {zombie_idx}"),
        "zombie".into(),
        raising_team,
        (zombie_idx as u8).wrapping_add(1),
        PlayerStats::new(4, 3, 2, 8, None),
        SkillSet::empty(),
    );
    team.add_player(zombie);

    state.field.place_player(zombie_id.clone(), raising_team, target, PlayerState::Standing);
    Some(zombie_id)
}

// ── Swarming ──────────────────────────────────────────────────────────────────

/// Returns how many extra players a Swarming team may field beyond the normal
/// 11-player limit. Equal to the number of their own players currently off-pitch.
pub fn swarming_extra_count(state: &GameState, team: TeamId) -> usize {
    if !state.team(team).has_special_rule(SpecialRule::Swarming) {
        return 0;
    }
    let total = state.team(team).players().len();
    let on_pitch = state.field.team_players_on_pitch(team).count();
    total.saturating_sub(on_pitch)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::player::PlayerStats;
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, SpecialRule, TeamId};

    fn make_necromantic_vs_human() -> GameState {
        let mut home = Team::new("h".into(), "Necromantic".into(), "Necromantic".into(), 3, true);
        home.special_rules.push(SpecialRule::MastersOfUndeath);
        for i in 0..3 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")),
                format!("HPlayer{i}"),
                "zombie".into(),
                TeamId::Home,
                i as u8 + 1,
                PlayerStats::new(4, 3, 2, 8, None),
                SkillSet::empty(),
            ));
        }
        let mut away = Team::new("a".into(), "Humans".into(), "Human".into(), 3, true);
        for i in 0..3 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")),
                format!("APlayer{i}"),
                "lineman".into(),
                TeamId::Away,
                i as u8 + 1,
                PlayerStats::new(6, 3, 4, 8, None),
                SkillSet::empty(),
            ));
        }
        let mut state = GameState::new(home, away);
        // Place home players on the pitch
        for i in 0..3 {
            state.field.place_player(
                PlayerId(format!("h{i}")),
                TeamId::Home,
                FieldCoordinate::new(i as u8 + 2, 4),
                PlayerState::Standing,
            );
        }
        // Place away players on the pitch
        for i in 0..3 {
            state.field.place_player(
                PlayerId(format!("a{i}")),
                TeamId::Away,
                FieldCoordinate::new(i as u8 + 2, 12),
                PlayerState::Standing,
            );
        }
        state
    }

    #[test]
    fn masters_of_undeath_raises_zombie_on_casualty() {
        let mut state = make_necromantic_vs_human();
        let on_pitch_before = state.field.team_players_on_pitch(TeamId::Home).count();
        // Simulate an away player suffering a casualty → home team should raise a zombie
        let raised = check_masters_of_undeath(&mut state, TeamId::Away);
        assert!(raised.is_some(), "a zombie should be raised");
        let on_pitch_after = state.field.team_players_on_pitch(TeamId::Home).count();
        assert_eq!(on_pitch_after, on_pitch_before + 1, "zombie should now be on pitch");
    }

    #[test]
    fn masters_of_undeath_no_raise_when_full() {
        let mut state = make_necromantic_vs_human();
        // Add 8 more home players to reach 11 (using y=5 to avoid y=4 where 3 exist)
        for i in 3..11 {
            let pid = PlayerId(format!("h{i}"));
            state.home.add_player(Player::new(
                pid.clone(),
                format!("HPlayer{i}"),
                "zombie".into(),
                TeamId::Home,
                i as u8 + 1,
                PlayerStats::new(4, 3, 2, 8, None),
                SkillSet::empty(),
            ));
            state.field.place_player(pid, TeamId::Home, FieldCoordinate::new((i as u8 - 3) * 3, 5), PlayerState::Standing);
        }
        let raised = check_masters_of_undeath(&mut state, TeamId::Away);
        assert!(raised.is_none(), "no zombie when pitch is full");
    }

    #[test]
    fn masters_of_undeath_no_raise_if_no_special_rule() {
        let mut state = make_necromantic_vs_human();
        // Remove the special rule
        state.home.special_rules.clear();
        let raised = check_masters_of_undeath(&mut state, TeamId::Away);
        assert!(raised.is_none(), "should not raise without special rule");
    }

    #[test]
    fn swarming_extra_count_zero_without_rule() {
        let state = make_necromantic_vs_human();
        assert_eq!(swarming_extra_count(&state, TeamId::Home), 0);
    }

    #[test]
    fn swarming_extra_count_correct() {
        let mut state = make_necromantic_vs_human();
        state.home.special_rules.push(SpecialRule::Swarming);
        // 3 players in roster, 3 on pitch → 0 off-pitch → 0 extra
        assert_eq!(swarming_extra_count(&state, TeamId::Home), 0);
        // Add a player to the bench (not on pitch)
        state.home.add_player(Player::new(
            PlayerId("h_bench".into()),
            "Bench".into(),
            "zombie".into(),
            TeamId::Home,
            12,
            PlayerStats::new(4, 3, 2, 8, None),
            SkillSet::empty(),
        ));
        assert_eq!(swarming_extra_count(&state, TeamId::Home), 1);
    }
}
