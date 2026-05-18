/// Static evaluation function for the current game state.
use ffb_core::model::game_state::GameState;
use ffb_core::types::TeamId;

/// Sigmoid activation: maps any real value to (0, 1).
#[inline]
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Returns a heuristic evaluation of `state` from the perspective of `team`.
/// Result is in (0.0, 1.0) — higher means better for `team`.
///
/// Formula:
///   sigmoid(3 * score_delta + 1 * ball_signal + 0.15 * standing_delta + urgency_term)
///
/// - `score_delta`    = team score - opponent score
/// - `ball_signal`    = 1.0 if team has ball carrier in opponent half,
///   0.5 if team has ball carrier in own half,
///   0.0 otherwise (normalised to [-1, 1] by subtracting 0.5)
/// - `standing_delta` = (team standing - opp standing) / 22.0
/// - `urgency_term`   = 0.3 * score_delta * turns_remaining / max_turns
pub fn static_eval(state: &GameState, team: TeamId) -> f64 {
    let opp = team.opponent();

    // ── Score delta ───────────────────────────────────────────────────────────
    let score_delta = state.result.score_delta(team) as f64;

    // ── Ball signal ───────────────────────────────────────────────────────────
    // Determine whether `team` is on offense (attacking toward higher x if home, lower x if away)
    let team_is_offense = match team {
        TeamId::Home => state.home_is_offense,
        TeamId::Away => !state.home_is_offense,
    };
    // The opponent end zone threshold
    let opp_half_x_threshold: u8 = 13; // x ≥ 13 is "away side"; x < 13 is "home side"

    let ball_signal = compute_ball_signal(state, team, opp_half_x_threshold, team_is_offense);

    // ── Standing delta ────────────────────────────────────────────────────────
    let team_standing = count_standing(state, team) as f64;
    let opp_standing = count_standing(state, opp) as f64;
    let standing_delta = (team_standing - opp_standing) / 22.0;

    // ── Urgency term ──────────────────────────────────────────────────────────
    let max_turns = state.options.max_turns_per_half as f64 * 2.0;
    let turns_played = state.result.turns_played as f64;
    let turns_remaining = (max_turns - turns_played).max(0.0);
    let urgency_term = 0.3 * score_delta * (turns_remaining / max_turns.max(1.0));

    let x = 3.0 * score_delta + 1.0 * ball_signal + 0.15 * standing_delta + urgency_term;
    sigmoid(x)
}

fn compute_ball_signal(
    state: &GameState,
    team: TeamId,
    opp_half_threshold: u8,
    team_is_offense: bool,
) -> f64 {
    // Find ball position and carrier
    let ball_coord = match state.field.ball.coord {
        Some(c) if state.field.ball.in_play => c,
        _ => return 0.0,
    };

    // Check if a team player is carrying the ball (ball at same square as a player)
    let carrier = state.field.player_at(ball_coord);
    let carrier_team = carrier.and_then(|pid| state.field.player_team(pid));

    if carrier_team != Some(team) {
        return 0.0;
    }

    // Team has possession; check which half the carrier is in
    // Home attacks toward higher x; away attacks toward lower x
    let in_opp_half = if team_is_offense {
        match team {
            TeamId::Home => ball_coord.x >= opp_half_threshold,
            TeamId::Away => ball_coord.x < opp_half_threshold,
        }
    } else {
        match team {
            TeamId::Home => ball_coord.x < opp_half_threshold,
            TeamId::Away => ball_coord.x >= opp_half_threshold,
        }
    };

    // Normalise: subtract 0.5 to centre around 0
    // 1.0 → opp half → signal = 0.5 above centre
    // 0.5 → own half → signal = 0 at centre
    if in_opp_half { 0.5 } else { 0.0 }
}

fn count_standing(state: &GameState, team: TeamId) -> u32 {
    state.field
        .team_players_on_pitch(team)
        .filter(|(_, _, s)| s.is_active())
        .count() as u32
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::model::game_state::GameState;
    use ffb_core::model::player::{Player, PlayerStats};
    use ffb_core::model::team::Team;
    use ffb_core::skills::SkillSet;
    use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_state() -> GameState {
        let home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        let away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        GameState::new(home, away)
    }

    #[test]
    fn eval_range_zero_to_one() {
        let state = make_state();
        let v = static_eval(&state, TeamId::Home);
        assert!(v > 0.0 && v < 1.0, "eval={v} must be in (0,1)");
    }

    #[test]
    fn eval_symmetric_at_start() {
        let state = make_state();
        let h = static_eval(&state, TeamId::Home);
        let a = static_eval(&state, TeamId::Away);
        // Symmetric start — both should be ~0.5
        assert!((h - 0.5).abs() < 0.1);
        assert!((a - 0.5).abs() < 0.1);
    }

    #[test]
    fn eval_higher_score_better() {
        let mut state = make_state();
        state.result.score_home = 2;
        state.result.score_away = 0;
        let h = static_eval(&state, TeamId::Home);
        let a = static_eval(&state, TeamId::Away);
        assert!(h > 0.5, "home should have eval > 0.5 when leading");
        assert!(a < 0.5, "away should have eval < 0.5 when trailing");
    }

    #[test]
    fn eval_values_inverse_of_each_other() {
        let state = make_state();
        let h = static_eval(&state, TeamId::Home);
        let a = static_eval(&state, TeamId::Away);
        // With no score difference and symmetric board they should roughly sum to 1.0
        assert!((h + a - 1.0).abs() < 0.05);
    }
}
