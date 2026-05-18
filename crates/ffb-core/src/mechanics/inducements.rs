use crate::model::game_state::GameState;
use crate::rng::GameRng;
use crate::types::{FieldCoordinate, InducementState, PlayerId, PlayerState};

// ── Wizard ────────────────────────────────────────────────────────────────────

/// Wizard lightning bolt: knocks target down (Prone), no armor roll required.
/// Does nothing if the target is not on the pitch.
pub fn wizard_lightning_bolt(state: &mut GameState, target: &PlayerId) {
    if state.field.player_state(target).map(|s| s.is_on_pitch()).unwrap_or(false) {
        state.field.set_player_state(target, PlayerState::Prone);
    }
}

/// Wizard fireball: scatter d6 squares from center, knock down each player hit.
/// Returns list of players knocked down.
pub fn wizard_fireball(
    state: &mut GameState,
    center: FieldCoordinate,
    rng: &mut GameRng,
) -> Vec<PlayerId> {
    let radius = rng.roll_d6();

    // Collect all players within Chebyshev distance `radius` of center.
    let hit_players: Vec<PlayerId> = state
        .field
        .on_pitch_players()
        .filter(|(_, coord, _)| {
            let dx = (coord.x as i16 - center.x as i16).unsigned_abs();
            let dy = (coord.y as i16 - center.y as i16).unsigned_abs();
            dx.max(dy) as u8 <= radius
        })
        .filter(|(_, _, st)| st.is_on_pitch())
        .map(|(id, _, _)| id.clone())
        .collect();

    for id in &hit_players {
        state.field.set_player_state(id, PlayerState::Prone);
    }

    hit_players
}

// ── Bribe ─────────────────────────────────────────────────────────────────────

/// Attempt to use a bribe to avoid ejection.
/// Returns true if a bribe was available and consumed.
pub fn use_bribe(inducements: &mut InducementState) -> bool {
    if inducements.bribes_remaining > 0 {
        inducements.bribes_remaining -= 1;
        true
    } else {
        false
    }
}

// ── Bloodweiser Babes ─────────────────────────────────────────────────────────

/// Bloodweiser Babes: give a KO'd player an extra recovery roll.
/// Rolls d6; on 4+ the player recovers (set to Stunned, then can stand).
/// Returns true if recovery succeeded.
pub fn babes_ko_recovery(
    state: &mut GameState,
    player_id: &PlayerId,
    rng: &mut GameRng,
) -> bool {
    let current = match state.field.player_state(player_id) {
        Some(s) => s,
        None => return false,
    };
    if current != PlayerState::Ko {
        return false;
    }
    let roll = rng.roll_d6();
    if roll >= 4 {
        state.field.set_player_state(player_id, PlayerState::Stunned);
        true
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::game_state::GameState;
    use crate::model::player::{Player, PlayerStats};
    use crate::model::team::Team;
    use crate::rng::GameRng;
    use crate::skills::SkillSet;
    use crate::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_state_with_players(positions: &[(PlayerId, TeamId, FieldCoordinate, PlayerState)]) -> GameState {
        let home = Team::new("h".into(), "Home".into(), "Human".into(), 2, true);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 2, false);
        let mut state = GameState::new(home, away);
        for (pid, team, coord, st) in positions {
            state.field.place_player(pid.clone(), *team, *coord, *st);
        }
        state
    }

    fn pid(s: &str) -> PlayerId {
        PlayerId(s.into())
    }

    #[test]
    fn lightning_bolt_knocks_down_standing_player() {
        let p = pid("p1");
        let mut state = make_state_with_players(&[(p.clone(), TeamId::Away, FieldCoordinate::new(10, 8), PlayerState::Standing)]);
        wizard_lightning_bolt(&mut state, &p);
        assert_eq!(state.field.player_state(&p), Some(PlayerState::Prone));
    }

    #[test]
    fn lightning_bolt_does_nothing_to_off_pitch_player() {
        let p = pid("p2");
        let home = Team::new("h".into(), "Home".into(), "Human".into(), 2, true);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 2, false);
        let mut state = GameState::new(home, away);
        // player not on pitch — should not panic
        wizard_lightning_bolt(&mut state, &p);
    }

    #[test]
    fn fireball_hits_players_in_radius() {
        // Place 3 players: one at center, one at distance 1, one at distance 5.
        let center = FieldCoordinate::new(10, 8);
        let near = FieldCoordinate::new(11, 8);    // dist=1
        let far = FieldCoordinate::new(15, 8);     // dist=5
        let p1 = pid("p1");
        let p2 = pid("p2");
        let p3 = pid("p3");
        let mut state = make_state_with_players(&[
            (p1.clone(), TeamId::Home, center, PlayerState::Standing),
            (p2.clone(), TeamId::Away, near, PlayerState::Standing),
            (p3.clone(), TeamId::Away, far, PlayerState::Standing),
        ]);
        // radius = d6 = 2 → hits center (dist 0) and near (dist 1), not far (dist 5)
        let mut rng = GameRng::new_test([2]);
        let hit = wizard_fireball(&mut state, center, &mut rng);
        assert!(hit.contains(&p1), "center player should be hit");
        assert!(hit.contains(&p2), "near player should be hit");
        assert!(!hit.contains(&p3), "far player should not be hit");
        assert_eq!(state.field.player_state(&p1), Some(PlayerState::Prone));
        assert_eq!(state.field.player_state(&p2), Some(PlayerState::Prone));
        assert_eq!(state.field.player_state(&p3), Some(PlayerState::Standing));
    }

    #[test]
    fn bribe_consumed_when_available() {
        let mut ind = InducementState { wizard_used: false, bribes_remaining: 2 };
        assert!(use_bribe(&mut ind));
        assert_eq!(ind.bribes_remaining, 1);
    }

    #[test]
    fn bribe_returns_false_when_none_left() {
        let mut ind = InducementState { wizard_used: false, bribes_remaining: 0 };
        assert!(!use_bribe(&mut ind));
    }

    #[test]
    fn babes_recovery_succeeds_on_4plus() {
        let p = pid("p1");
        let mut state = make_state_with_players(&[(p.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Ko)]);
        let mut rng = GameRng::new_test([4]);
        assert!(babes_ko_recovery(&mut state, &p, &mut rng));
        assert_eq!(state.field.player_state(&p), Some(PlayerState::Stunned));
    }

    #[test]
    fn babes_recovery_fails_on_low_roll() {
        let p = pid("p1");
        let mut state = make_state_with_players(&[(p.clone(), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Ko)]);
        let mut rng = GameRng::new_test([3]);
        assert!(!babes_ko_recovery(&mut state, &p, &mut rng));
        assert_eq!(state.field.player_state(&p), Some(PlayerState::Ko));
    }
}
