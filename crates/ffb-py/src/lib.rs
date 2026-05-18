// PyO3 macro expansion triggers false-positive useless_conversion warnings on PyResult return types.
#![allow(clippy::useless_conversion)]
/// PyO3 Python bindings for RL training.
///
/// Exposes a gym-compatible `FfbEnv` class with `reset`, `step`, `action_mask`,
/// and `legal_actions` methods.
use pyo3::prelude::*;
use pyo3::types::PyList;
use numpy::{IntoPyArray, PyArray1};

use ffb_core::actions::{enumerate_actions, BbAction};
use ffb_core::model::game_state::GameState;
use ffb_core::model::player::{Player, PlayerStats};
use ffb_core::model::team::Team;
use ffb_core::rng::GameRng;
use ffb_core::skills::SkillSet;
use ffb_core::types::{FieldCoordinate, PlayerId, TeamId, TurnMode};
use ffb_mcts::{MctsConfig, MctsSearch};
use ffb_sim::evaluation::static_eval;
use ffb_sim::setup::{default_kickoff_ball_placement, place_players_for_kickoff};

// ── Action space ──────────────────────────────────────────────────────────────

/// Total action space size.
/// Simple flat encoding: EndTurn(0), Activate(1..22), MoveTo(23..465), BlockTarget(466..476),
/// ChooseBlockDie(477..481), ChoosePush(482..923), UseReroll(924..925), PlaceBall/PassTo(926..1367),
/// ChooseFollowup(1368..1369)
const ACTION_SPACE: usize = 1370;

fn action_to_idx(action: &BbAction) -> usize {
    match action {
        BbAction::EndTurn => 0,
        BbAction::Activate { player_id, action: _ } => {
            let n: usize = player_id.0.trim_start_matches(|c: char| !c.is_numeric())
                .parse().unwrap_or(0);
            1 + (n % 22)
        }
        BbAction::MoveTo(c) => 23 + coord_idx(*c),
        BbAction::BlockTarget(pid) => {
            let n: usize = pid.0.trim_start_matches(|c: char| !c.is_numeric())
                .parse().unwrap_or(0);
            466 + (n % 11)
        }
        BbAction::ChooseBlockDie(r) => {
            use ffb_core::types::BlockResult;
            let i = match r {
                BlockResult::Skull => 0,
                BlockResult::BothDown => 1,
                BlockResult::Pushback => 2,
                BlockResult::PowPushback => 3,
                BlockResult::Pow => 4,
            };
            477 + i
        }
        BbAction::ChoosePush(c) => 482 + coord_idx(*c),
        BbAction::UseReroll(b) => if *b { 924 } else { 925 },
        BbAction::PassTo(c) => 926 + coord_idx(*c),
        BbAction::PlaceBall(c) => 926 + coord_idx(*c),
        BbAction::ChooseFollowup(b) => if *b { 1368 } else { 1369 },
    }
}

fn coord_idx(c: FieldCoordinate) -> usize {
    (c.y as usize) * 26 + (c.x as usize)
}

// ── Gym environment ───────────────────────────────────────────────────────────

#[pyclass]
pub struct FfbEnv {
    state: GameState,
    rng: GameRng,
    mcts_budget: u32,
    human_team: TeamId, // the team the Python agent controls
}

#[pymethods]
impl FfbEnv {
    #[new]
    #[pyo3(signature = (seed=42, mcts_budget=100))]
    pub fn new(seed: u64, mcts_budget: u32) -> Self {
        let state = make_default_game();
        Self {
            state,
            rng: GameRng::new_live(seed),
            mcts_budget,
            human_team: TeamId::Home,
        }
    }

    /// Reset environment: create new game, return observation.
    #[pyo3(signature = (seed=None))]
    #[allow(clippy::useless_conversion)]
    pub fn reset<'py>(&mut self, py: Python<'py>, seed: Option<u64>) -> PyResult<Bound<'py, PyArray1<f32>>> {
        if let Some(s) = seed {
            self.rng = GameRng::new_live(s);
        }
        self.state = make_default_game();
        let obs = self.observation();
        Ok(obs.into_pyarray_bound(py))
    }

    /// Step: apply action_idx, advance until human's turn or game over.
    /// Returns (obs, reward, done, info_dict).
    #[allow(clippy::useless_conversion)]
    pub fn step<'py>(&mut self, py: Python<'py>, action_idx: u32) -> PyResult<PyObject> {
        // Map action_idx back to BbAction via legal actions
        let legal = enumerate_actions(&self.state, self.human_team);
        let action = legal.into_iter()
            .find(|a| action_to_idx(a) == action_idx as usize)
            .unwrap_or(BbAction::EndTurn);

        let prev_score_h = self.state.result.score_home;
        let prev_score_a = self.state.result.score_away;

        // Apply human action
        apply_action_simple(&mut self.state, action, &mut self.rng);

        // Advance AI turns until human's turn or game over
        let mut safety = 0;
        while !self.state.result.finished && self.state.active_team_id() != self.human_team {
            let legal_ai = enumerate_actions(&self.state, self.state.active_team_id());
            if legal_ai.is_empty() { break; }
            let ai_action = if self.mcts_budget > 0 {
                let cfg = MctsConfig {
                    budget: self.mcts_budget,
                    team: self.state.active_team_id(),
                    ..Default::default()
                };
                MctsSearch::search(&self.state, &cfg, &mut self.rng)
            } else {
                legal_ai[0].clone()
            };
            apply_action_simple(&mut self.state, ai_action, &mut self.rng);
            safety += 1;
            if safety > 1000 { break; }
        }

        let done = self.state.result.finished;
        let new_score_h = self.state.result.score_home;
        let new_score_a = self.state.result.score_away;

        // Reward: +1 for TD scored, -1 for TD conceded, 1/-1 at game end
        let reward: f32 = if done {
            let delta = self.state.result.score_delta(self.human_team);
            if delta > 0 { 1.0 } else if delta < 0 { -1.0 } else { 0.0 }
        } else {
            let td_scored = match self.human_team {
                TeamId::Home => new_score_h.saturating_sub(prev_score_h) as f32,
                TeamId::Away => new_score_a.saturating_sub(prev_score_a) as f32,
            };
            let td_conceded = match self.human_team {
                TeamId::Home => new_score_a.saturating_sub(prev_score_a) as f32,
                TeamId::Away => new_score_h.saturating_sub(prev_score_h) as f32,
            };
            td_scored - td_conceded
        };

        let obs = self.observation().into_pyarray_bound(py);
        let result = (obs, reward, done, py.None());
        Ok(result.into_py(py))
    }

    /// Return boolean mask of legal actions in the full action space.
    pub fn action_mask<'py>(&self, py: Python<'py>) -> Bound<'py, PyArray1<bool>> {
        let mut mask = vec![false; ACTION_SPACE];
        for action in enumerate_actions(&self.state, self.human_team) {
            let idx = action_to_idx(&action);
            if idx < ACTION_SPACE {
                mask[idx] = true;
            }
        }
        mask.into_pyarray_bound(py)
    }

    /// Return list of legal action indices.
    pub fn legal_actions<'py>(&self, py: Python<'py>) -> Bound<'py, PyList> {
        let indices: Vec<u32> = enumerate_actions(&self.state, self.human_team)
            .iter()
            .map(|a| action_to_idx(a) as u32)
            .collect();
        PyList::new_bound(py, &indices)
    }

    /// Return flat float32 observation vector of length 32 (non-spatial).
    fn observation(&self) -> Vec<f32> {
        extract_features_flat(&self.state, self.human_team)
    }
}

// ── Feature extraction (T-37) ─────────────────────────────────────────────────

/// Non-spatial features: 32 floats.
/// [turn/max_turn, half, score_delta, rerolls_home, rerolls_away,
///  standing_home_frac, standing_away_frac, ball_x/26, ball_y/17,
///  ... padding to 32]
fn extract_features_flat(state: &GameState, team: TeamId) -> Vec<f32> {
    let max_turns = state.options.max_turns_per_half as f32 * 2.0;
    let turn_frac = state.result.turns_played as f32 / max_turns.max(1.0);
    let half_enc = match state.half {
        ffb_core::types::Half::First => 0.0f32,
        ffb_core::types::Half::Second => 1.0f32,
    };
    let score_delta = state.result.score_delta(team) as f32 / 4.0;
    let rr_home = state.home.rerolls_remaining as f32 / state.home.rerolls_total.max(1) as f32;
    let rr_away = state.away.rerolls_remaining as f32 / state.away.rerolls_total.max(1) as f32;

    let standing_home = count_active(state, TeamId::Home) as f32 / 11.0;
    let standing_away = count_active(state, TeamId::Away) as f32 / 11.0;

    let (ball_x, ball_y) = state.field.ball.coord
        .map(|c| (c.x as f32 / 26.0, c.y as f32 / 17.0))
        .unwrap_or((0.5, 0.5));

    let eval = static_eval(state, team) as f32;

    let mut feats = vec![
        turn_frac, half_enc, score_delta, rr_home, rr_away,
        standing_home, standing_away, ball_x, ball_y, eval,
    ];
    // Pad to 32
    feats.resize(32, 0.0);
    feats
}

fn count_active(state: &GameState, team: TeamId) -> usize {
    state.field.team_players_on_pitch(team)
        .filter(|(_, _, s)| s.is_active())
        .count()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_default_game() -> GameState {
    let mut home = Team::new("h".into(), "Reikland Reavers".into(), "Human".into(), 3, true);
    for i in 0..11 {
        home.add_player(Player::new(
            PlayerId(format!("h{i}")),
            format!("HPlayer{i}"),
            "lineman".into(),
            TeamId::Home,
            i as u8 + 1,
            PlayerStats::new(6, 3, 4, 8, None),
            SkillSet::empty(),
        ));
    }
    let mut away = Team::new("a".into(), "Grudgebearers".into(), "Orc".into(), 3, false);
    for i in 0..11 {
        away.add_player(Player::new(
            PlayerId(format!("a{i}")),
            format!("APlayer{i}"),
            "lineman".into(),
            TeamId::Away,
            i as u8 + 1,
            PlayerStats::new(5, 4, 3, 9, None),
            SkillSet::empty(),
        ));
    }
    let mut state = GameState::new(home, away);
    place_players_for_kickoff(&mut state);
    default_kickoff_ball_placement(&mut state);
    state.turn_mode = TurnMode::Regular;
    state.home_is_active = true;
    ffb_core::steps::turn_step::begin_turn(&mut state);
    state
}

fn apply_action_simple(state: &mut GameState, action: BbAction, rng: &mut GameRng) {
    use ffb_core::steps::{
        apply_block_dice_choice, apply_push_choice, begin_activation, begin_block, begin_move,
        end_activation, end_turn, use_team_reroll,
    };
    use ffb_core::pathfinding::find_paths;

    match action {
        BbAction::EndTurn => {
            end_activation(state);
            end_turn(state);
        }
        BbAction::Activate { player_id, action: player_action } => {
            let _ = begin_activation(state, &player_id, rng);
            if let Some(ap) = state.acting_player.as_mut() {
                ap.current_action = Some(player_action);
            }
        }
        BbAction::MoveTo(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let pid = ap.player_id.clone();
                let team = ap.team;
                let ma_rem = {
                    let p = state.team(team).player_by_id(&pid).unwrap();
                    p.effective_ma().saturating_sub(ap.movement_used)
                };
                let path = {
                    let p = state.team(team).player_by_id(&pid).unwrap();
                    find_paths(&state.field, p, &pid, team, ma_rem)
                        .get(&coord).map(|e| e.path.to_vec())
                };
                if let Some(p) = path {
                    begin_move(state, &pid, &p, rng);
                }
                end_activation(state);
            }
        }
        BbAction::BlockTarget(def_id) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let att_id = ap.player_id.clone();
                begin_block(state, &att_id, &def_id, rng);
            }
        }
        BbAction::ChooseBlockDie(result) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let att_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&att_id);
                let def_id = att_coord.and_then(|c| {
                    c.neighbors().find_map(|n| {
                        state.field.player_at(n).and_then(|pid| {
                            if state.field.player_team(pid) != Some(team) {
                                Some(pid.clone())
                            } else { None }
                        })
                    })
                });
                if let Some(def) = def_id {
                    apply_block_dice_choice(state, &att_id, &def, result, rng);
                }
            }
        }
        BbAction::ChoosePush(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let att_id = ap.player_id.clone();
                let team = ap.team;
                let def_id = state.field.player_coord(&att_id).and_then(|c| {
                    c.neighbors().find_map(|n| {
                        state.field.player_at(n).and_then(|pid| {
                            if state.field.player_team(pid) != Some(team) {
                                Some(pid.clone())
                            } else { None }
                        })
                    })
                });
                if let Some(def) = def_id {
                    apply_push_choice(state, &att_id, &def, coord, rng);
                }
            }
        }
        BbAction::UseReroll(use_it) => {
            if use_it {
                if let Some(ap) = state.acting_player.as_ref() {
                    let pid = ap.player_id.clone();
                    use_team_reroll(state, &pid, rng);
                }
            }
            state.dialog = ffb_core::model::game_state::DialogState::None;
        }
        BbAction::PassTo(coord) | BbAction::PlaceBall(coord) => {
            state.field.ball.coord = Some(coord);
        }
        BbAction::ChooseFollowup(_) => {
            // Handled by simulation loop
        }
    }
}

// ── Module registration ───────────────────────────────────────────────────────

#[pymodule]
fn ffb(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FfbEnv>()?;
    m.add("ACTION_SPACE", ACTION_SPACE as u32)?;
    Ok(())
}
