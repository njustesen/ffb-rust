/// Parity log types and helpers for cross-engine comparison.
///
/// Produces JSONL output (one JSON object per line):
///   game_start → step × N → game_end
///
/// Each step captures:
///   (state_hash_before, sorted action strings, chosen action string, state_hash_after)
///
/// The state hash is FNV-1a 64 over a deterministic ASCII representation so it
/// is easy to reproduce in Java without any extra dependency.
use std::sync::{Arc, Mutex};

use ffb_core::actions::BbAction;
use ffb_core::model::game_state::{DialogState, GameState};
use ffb_core::types::{BlockResult, Half, PlayerAction, PlayerState};
use serde::Serialize;

use crate::simulation::Strategy;

// ── Action string ─────────────────────────────────────────────────────────────

/// Convert a `BbAction` to the canonical cross-engine string.
pub fn action_to_string(action: &BbAction) -> String {
    match action {
        BbAction::EndTurn => "EndTurn".to_string(),
        BbAction::Activate { player_id, action } => {
            format!("Activate:{}:{}", player_id.0, player_action_str(action))
        }
        BbAction::MoveTo(c) => format!("MoveTo:{},{}", c.x, c.y),
        BbAction::BlockTarget(pid) => format!("BlockTarget:{}", pid.0),
        BbAction::ChooseBlockDie(r) => format!("ChooseBlockDie:{}", block_result_str(r)),
        BbAction::ChoosePush(c) => format!("ChoosePush:{},{}", c.x, c.y),
        BbAction::PassTo(c) => format!("PassTo:{},{}", c.x, c.y),
        BbAction::UseReroll(b) => format!("UseReroll:{b}"),
        BbAction::PlaceBall(c) => format!("PlaceBall:{},{}", c.x, c.y),
        BbAction::ChooseFollowup(b) => format!("ChooseFollowup:{b}"),
    }
}

fn player_action_str(a: &PlayerAction) -> &'static str {
    match a {
        PlayerAction::Move => "Move",
        PlayerAction::Block => "Block",
        PlayerAction::Blitz => "Blitz",
        PlayerAction::Pass => "Pass",
        PlayerAction::HandOff => "HandOff",
        PlayerAction::Foul => "Foul",
        PlayerAction::Unused => "Unused",
    }
}

fn block_result_str(r: &BlockResult) -> &'static str {
    match r {
        BlockResult::Skull => "Skull",
        BlockResult::BothDown => "BothDown",
        BlockResult::Pushback => "Pushback",
        BlockResult::PowPushback => "PowPushback",
        BlockResult::Pow => "Pow",
    }
}

// ── Dialog string ─────────────────────────────────────────────────────────────

pub fn dialog_to_string(state: &GameState) -> String {
    match &state.dialog {
        DialogState::None => "None".to_string(),
        DialogState::SelectPlayer { .. } => "SelectPlayer".to_string(),
        DialogState::SelectMoveTarget { .. } => "SelectMoveTarget".to_string(),
        DialogState::SelectBlockTarget { .. } => "SelectBlockTarget".to_string(),
        DialogState::SelectBlockDice { .. } => "SelectBlockDice".to_string(),
        DialogState::SelectBlockReroll { .. } => "SelectBlockReroll".to_string(),
        DialogState::SelectReroll { .. } => "SelectReroll".to_string(),
        DialogState::SelectPush { .. } => "SelectPush".to_string(),
        DialogState::SelectFollowup { .. } => "SelectFollowup".to_string(),
        DialogState::SelectInjury => "SelectInjury".to_string(),
        DialogState::SelectApothecary { .. } => "SelectApothecary".to_string(),
        DialogState::SelectKickTarget => "SelectKickTarget".to_string(),
        DialogState::SelectKickoffReturn => "SelectKickoffReturn".to_string(),
        DialogState::SelectHighKickPlayer => "SelectHighKickPlayer".to_string(),
    }
}

// ── State hash ────────────────────────────────────────────────────────────────

/// 16-char hex FNV-1a 64-bit hash of the canonical game state.
///
/// Canonical format (newline-free ASCII):
///   h<half>t<turn_home><turn_away>a<active>s<score_home>,<score_away>
///   b<ball_x>,<ball_y>,<ball_in_play>
///   p<id>:<x>,<y>,<state>|...   (players sorted by ID; -1,-1 = off-pitch)
pub fn state_hash(state: &GameState) -> String {
    format!("{:016x}", fnv1a_64(build_canonical(state).as_bytes()))
}

pub fn canonical_string(state: &GameState) -> String {
    build_canonical(state)
}

fn build_canonical(state: &GameState) -> String {
    let half = match state.half { Half::First => 1u8, Half::Second => 2u8 };
    let active = if state.home_is_active { "home" } else { "away" };
    let (bx, by) = state.field.ball.coord
        .map(|c| (c.x as i16, c.y as i16))
        .unwrap_or((-1, -1));

    // Use positional indices (h00..h10, a00..a10) instead of player IDs so hashes
    // match the Java engine regardless of ID naming conventions.
    // Home players are sorted by jersey number; away players likewise.
    let mut home_players: Vec<_> = state.home.players().iter().collect();
    home_players.sort_by_key(|p| p.jersey_number);
    let mut away_players: Vec<_> = state.away.players().iter().collect();
    away_players.sort_by_key(|p| p.jersey_number);

    let mut players: Vec<String> = home_players.iter().enumerate()
        .map(|(idx, p)| {
            let (x, y) = state.field.player_coord(&p.id)
                .map(|c| (c.x as i16, c.y as i16))
                .unwrap_or((-1, -1));
            let ps_str = player_state_str(state.field.player_state(&p.id));
            format!("h{:02}:{},{},{}", idx, x, y, ps_str)
        })
        .chain(away_players.iter().enumerate().map(|(idx, p)| {
            let (x, y) = state.field.player_coord(&p.id)
                .map(|c| (c.x as i16, c.y as i16))
                .unwrap_or((-1, -1));
            let ps_str = player_state_str(state.field.player_state(&p.id));
            format!("a{:02}:{},{},{}", idx, x, y, ps_str)
        }))
        .collect();
    players.sort();

    format!(
        "h{}t{}{}a{}s{},{} b{},{},{} p{}",
        half,
        state.turn_data_home.turn_number,
        state.turn_data_away.turn_number,
        active,
        state.home.score,
        state.away.score,
        bx, by, state.field.ball.in_play,
        players.join("|")
    )
}

fn player_state_str(ps: Option<PlayerState>) -> &'static str {
    match ps {
        Some(PlayerState::Standing) => "Standing",
        Some(PlayerState::Moving) => "Moving",
        Some(PlayerState::Prone) => "Prone",
        Some(PlayerState::Stunned) => "Stunned",
        Some(PlayerState::Ko) => "Ko",
        Some(PlayerState::Injured) => "Injured",
        Some(PlayerState::Rooted) => "Rooted",
        Some(PlayerState::Reserve) | None => "Reserve",
    }
}

fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

// ── Log structs ───────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct ParityGameStart {
    pub i: usize,
    #[serde(rename = "type")]
    pub step_type: &'static str,
    pub home: String,
    pub away: String,
    pub seed: u64,
    pub state_hash: String,
}

#[derive(Serialize)]
pub struct ParityStep {
    pub i: usize,
    #[serde(rename = "type")]
    pub step_type: &'static str,
    pub turn: u8,
    pub half: u8,
    pub active: String,
    pub dialog: String,
    pub state_hash: String,
    pub actions: Vec<String>,
    pub chosen: String,
    pub dice: Vec<String>,
    /// Hash of the state after this action was applied.
    /// Filled by `finalize_steps` using the next step's pre-hash.
    pub post_hash: String,
}

#[derive(Serialize)]
pub struct ParityGameEnd {
    pub i: usize,
    #[serde(rename = "type")]
    pub step_type: &'static str,
    pub home_score: u8,
    pub away_score: u8,
    pub state_hash: String,
}

// ── Pending step (before post_hash is known) ──────────────────────────────────

#[derive(Debug)]
pub struct PendingStep {
    pub i: usize,
    pub turn: u8,
    pub half: u8,
    pub active: String,
    pub dialog: String,
    pub state_hash: String,
    pub actions: Vec<String>,
    pub chosen: String,
    pub dice: Vec<String>,
}

/// Fill in `post_hash` values: step[i].post_hash = step[i+1].state_hash.
/// The last step gets `game_end_hash`.
pub fn finalize_steps(pending: Vec<PendingStep>, game_end_hash: &str) -> Vec<ParityStep> {
    let n = pending.len();
    let pre_hashes: Vec<String> = pending.iter().map(|p| p.state_hash.clone()).collect();

    pending.into_iter().enumerate().map(|(idx, p)| {
        let post_hash = if idx + 1 < n {
            pre_hashes[idx + 1].clone()
        } else {
            game_end_hash.to_string()
        };
        ParityStep {
            i: p.i,
            step_type: "step",
            turn: p.turn,
            half: p.half,
            active: p.active,
            dialog: p.dialog,
            state_hash: p.state_hash,
            actions: p.actions,
            chosen: p.chosen,
            dice: p.dice,
            post_hash,
        }
    }).collect()
}

// ── Logging strategy ──────────────────────────────────────────────────────────

/// Wraps any `Strategy` and records each decision point.
/// Uses `Arc<Mutex<Vec<PendingStep>>>` so two instances can share one store (home + away).
pub struct LoggingStrategy<S> {
    inner: S,
    steps: Arc<Mutex<Vec<PendingStep>>>,
}

impl<S: Strategy> LoggingStrategy<S> {
    /// Create a new strategy with its own step store.
    pub fn new(inner: S) -> Self {
        Self { inner, steps: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Create a strategy sharing an existing step store (for logging both teams).
    pub fn new_shared(inner: S, steps: Arc<Mutex<Vec<PendingStep>>>) -> Self {
        Self { inner, steps }
    }

    /// Get a clone of the Arc for sharing with a second strategy.
    pub fn steps_handle(&self) -> Arc<Mutex<Vec<PendingStep>>> {
        Arc::clone(&self.steps)
    }

    /// Consume and return the recorded pending steps.
    /// Panics if there are other Arc owners; use `steps_handle()` to extract shared steps.
    pub fn into_pending(self) -> Vec<PendingStep> {
        Arc::try_unwrap(self.steps)
            .expect("other Arc owners still alive; drop them or extract via steps_handle()")
            .into_inner()
            .unwrap()
    }
}

impl<S: Strategy> Strategy for LoggingStrategy<S> {
    fn choose_action(&self, state: &GameState, legal_actions: &[BbAction]) -> BbAction {
        let canonical = build_canonical(state);
        let pre_hash = format!("{:016x}", fnv1a_64(canonical.as_bytes()));
        let half = match state.half { Half::First => 1u8, Half::Second => 2u8 };
        let active = if state.home_is_active { "home" } else { "away" };
        let dialog = dialog_to_string(state);
        let turn = if state.home_is_active {
            state.turn_data_home.turn_number
        } else {
            state.turn_data_away.turn_number
        };

        let mut action_strings: Vec<String> = legal_actions.iter().map(action_to_string).collect();
        action_strings.sort(); // alphabetic for stable log ordering

        let chosen = self.inner.choose_action(state, legal_actions);
        let chosen_str = action_to_string(&chosen);

        let mut steps = self.steps.lock().unwrap();
        let i = steps.len() + 1; // 0 reserved for game_start
        steps.push(PendingStep {
            i,
            turn,
            half,
            active: active.to_string(),
            dialog,
            state_hash: pre_hash,
            actions: action_strings,
            chosen: chosen_str,
            dice: Vec::new(), // TODO: capture dice rolls when LoggingRng is implemented
        });

        chosen
    }
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

    fn make_small_state() -> GameState {
        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        home.add_player(Player::new(
            PlayerId("h0".into()), "HP0".into(), "lineman".into(), TeamId::Home, 1,
            PlayerStats::new(6, 3, 4, 8, None), SkillSet::empty(),
        ));
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        away.add_player(Player::new(
            PlayerId("a0".into()), "AP0".into(), "lineman".into(), TeamId::Away, 1,
            PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
        ));
        let mut state = GameState::new(home, away);
        state.field.place_player(PlayerId("h0".into()), TeamId::Home,
            FieldCoordinate::new(5, 5), PlayerState::Standing);
        state.field.place_player(PlayerId("a0".into()), TeamId::Away,
            FieldCoordinate::new(15, 8), PlayerState::Standing);
        state
    }

    #[test]
    fn state_hash_is_deterministic() {
        let s = make_small_state();
        assert_eq!(state_hash(&s), state_hash(&s));
    }

    #[test]
    fn state_hash_changes_on_move() {
        let mut s = make_small_state();
        let h1 = state_hash(&s);
        s.field.move_player(&PlayerId("h0".into()), FieldCoordinate::new(6, 5));
        assert_ne!(h1, state_hash(&s));
    }

    #[test]
    fn state_hash_changes_on_score() {
        let mut s = make_small_state();
        let h1 = state_hash(&s);
        s.home.score += 1;
        assert_ne!(h1, state_hash(&s));
    }

    #[test]
    fn action_strings_are_non_empty() {
        let cases = vec![
            BbAction::EndTurn,
            BbAction::Activate { player_id: PlayerId("h1".into()), action: PlayerAction::Move },
            BbAction::MoveTo(FieldCoordinate::new(7, 3)),
            BbAction::BlockTarget(PlayerId("a2".into())),
            BbAction::ChooseBlockDie(BlockResult::Pow),
            BbAction::ChoosePush(FieldCoordinate::new(10, 5)),
            BbAction::UseReroll(true),
            BbAction::PlaceBall(FieldCoordinate::new(13, 8)),
            BbAction::ChooseFollowup(false),
        ];
        for a in &cases {
            assert!(!action_to_string(a).is_empty());
        }
    }

    #[test]
    fn logging_strategy_records_steps() {
        use crate::canonical_strategy::CanonicalStrategy;
        use crate::simulation::SimulationLoop;
        use ffb_core::rng::GameRng;
        use ffb_core::types::TurnMode;

        let mut home = Team::new("h".into(), "Home".into(), "Human".into(), 3, true);
        for i in 0..11u8 {
            home.add_player(Player::new(
                PlayerId(format!("h{i}")), format!("HP{i}"), "lineman".into(),
                TeamId::Home, i + 1, PlayerStats::new(6, 3, 4, 9, None), SkillSet::empty(),
            ));
        }
        let mut away = Team::new("a".into(), "Away".into(), "Orc".into(), 3, false);
        for i in 0..11u8 {
            away.add_player(Player::new(
                PlayerId(format!("a{i}")), format!("AP{i}"), "lineman".into(),
                TeamId::Away, i + 1, PlayerStats::new(5, 4, 3, 9, None), SkillSet::empty(),
            ));
        }
        let mut state = GameState::new(home, away);
        state.turn_mode = TurnMode::Regular;
        state.home_is_active = true;
        state.field.place_player(PlayerId("h0".into()), TeamId::Home,
            FieldCoordinate::new(5, 5), PlayerState::Standing);

        let logger = LoggingStrategy::new(CanonicalStrategy);
        let mut rng = GameRng::new_live(0);
        let _ = SimulationLoop::run(state, &logger, &CanonicalStrategy, &mut rng);

        let pending = logger.into_pending();
        assert!(!pending.is_empty(), "at least one decision should be recorded");
        assert_eq!(pending[0].i, 1, "first step index is 1");
    }
}
