/// Canonical strategy for cross-engine parity testing.
///
/// Always picks the first action in a deterministically sorted list so that
/// two independent engine implementations (Rust + Java) produce identical
/// game transcripts given the same seed.
///
/// Sort key (primary → secondary → tertiary):
///   1. action_type_ordinal (0=EndTurn … 9=ChooseFollowup)
///   2. player_id string (lexicographic)
///   3. target_coord flat index (y * 26 + x)
use ffb_core::actions::BbAction;
use ffb_core::model::game_state::GameState;
use ffb_core::types::{FieldCoordinate, PITCH_WIDTH};

use crate::simulation::Strategy;

// ── Ordinal mapping ──────────────────────────────────────────────────────────

fn action_ordinal(action: &BbAction) -> u8 {
    match action {
        BbAction::EndTurn => 0,
        BbAction::Activate { .. } => 1,
        BbAction::MoveTo(_) => 2,
        BbAction::BlockTarget(_) => 3,
        BbAction::ChooseBlockDie(_) => 4,
        BbAction::ChoosePush(_) => 5,
        BbAction::PassTo(_) => 6,
        BbAction::UseReroll(_) => 7,
        BbAction::PlaceBall(_) => 8,
        BbAction::ChooseFollowup(_) => 9,
    }
}

fn action_player_key(action: &BbAction) -> String {
    match action {
        BbAction::Activate { player_id, .. } => player_id.0.clone(),
        BbAction::BlockTarget(pid) => pid.0.clone(),
        _ => String::new(),
    }
}

fn coord_flat(coord: FieldCoordinate) -> u32 {
    coord.y as u32 * PITCH_WIDTH as u32 + coord.x as u32
}

fn action_coord_key(action: &BbAction) -> u32 {
    match action {
        BbAction::MoveTo(c) | BbAction::ChoosePush(c) | BbAction::PassTo(c) | BbAction::PlaceBall(c) => {
            coord_flat(*c)
        }
        _ => 0,
    }
}

// ── CanonicalStrategy ─────────────────────────────────────────────────────────

pub struct CanonicalStrategy;

impl Strategy for CanonicalStrategy {
    fn choose_action(&self, _state: &GameState, legal_actions: &[BbAction]) -> BbAction {
        assert!(!legal_actions.is_empty(), "CanonicalStrategy: no legal actions");

        let mut sorted: Vec<&BbAction> = legal_actions.iter().collect();
        sorted.sort_by(|a, b| {
            action_ordinal(a)
                .cmp(&action_ordinal(b))
                .then_with(|| action_player_key(a).cmp(&action_player_key(b)))
                .then_with(|| action_coord_key(a).cmp(&action_coord_key(b)))
        });

        sorted[0].clone()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ffb_core::actions::enumerate_actions;
    use ffb_core::model::game_state::GameState;
    use ffb_core::model::player::{Player, PlayerStats};
    use ffb_core::model::team::Team;
    use ffb_core::skills::SkillSet;
    use ffb_core::types::{FieldCoordinate, PlayerId, PlayerState, TeamId};

    fn make_default_game() -> GameState {
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        for i in 0..11u8 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")),
                format!("HP{i}"),
                "lineman".into(),
                TeamId::Home,
                i + 1,
                PlayerStats::new(6, 3, 4, 9, Some(5)),
                SkillSet::empty(),
            ));
        }

        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        for i in 0..11u8 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")),
                format!("AP{i}"),
                "lineman".into(),
                TeamId::Away,
                i + 1,
                PlayerStats::new(5, 4, 3, 9, None),
                SkillSet::empty(),
            ));
        }

        let mut state = GameState::new(home, away);
        state.home_is_active = true;
        state
            .field
            .place_player(PlayerId("h0".into()), TeamId::Home, FieldCoordinate::new(5, 5), PlayerState::Standing);
        state
    }

    #[test]
    fn canonical_strategy_is_deterministic() {
        let state = make_default_game();
        let legal = enumerate_actions(&state, TeamId::Home);
        let a1 = CanonicalStrategy.choose_action(&state, &legal);
        let a2 = CanonicalStrategy.choose_action(&state, &legal);
        assert_eq!(a1, a2);
    }

    #[test]
    fn canonical_strategy_prefers_lower_ordinal() {
        let state = make_default_game();
        let legal = enumerate_actions(&state, TeamId::Home);
        let chosen = CanonicalStrategy.choose_action(&state, &legal);
        // EndTurn (ordinal 0) should be chosen when Activate (ordinal 1) is also present
        assert_eq!(chosen, BbAction::EndTurn, "EndTurn has lowest ordinal and must be chosen");
    }
}
