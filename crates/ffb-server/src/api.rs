use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use ffb_core::{
    actions::{enumerate_actions, BbAction},
    model::{
        game_state::{DialogState, GameState},
        player::{Player, PlayerStats},
        team::Team,
    },
    rng::GameRng,
    types::{BlockResult, FieldCoordinate, Half, PlayerId, PlayerAction, PlayerState, TeamId},
};
use ffb_mcts::{MctsConfig, MctsSearch, RolloutDepth};
use ffb_sim::{
    setup::{default_kickoff_ball_placement, place_players_for_kickoff},
    simulation::NullStrategy,
};

// ── Shared State ──────────────────────────────────────────────────────────────

pub struct Session {
    pub state: GameState,
    pub rng: GameRng,
    pub mcts_budget: u32,
    pub human_team: TeamId,
}

pub type AppState = HashMap<String, Session>;
pub type SharedState = Arc<Mutex<AppState>>;

// ── JSON types ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct NewGameRequest {
    pub seed: Option<u64>,
    pub mcts_budget: Option<u32>,
    pub human_team: Option<String>,
}

#[derive(Serialize)]
pub struct NewGameResponse {
    pub game_id: String,
}

#[derive(Deserialize)]
pub struct ActionRequest {
    #[serde(rename = "type")]
    pub action_type: String,
    pub player_id: Option<String>,
    pub action: Option<String>,
    pub x: Option<u8>,
    pub y: Option<u8>,
    pub result: Option<String>,
    pub use_it: Option<bool>,
}

#[derive(Serialize)]
pub struct PlayerJson {
    pub id: String,
    pub team: String,
    pub x: u8,
    pub y: u8,
    pub state: String,
}

#[derive(Serialize)]
pub struct BallJson {
    pub x: Option<u8>,
    pub y: Option<u8>,
    pub in_play: bool,
}

#[derive(Serialize)]
pub struct BoardStateJson {
    pub game_id: String,
    pub turn_home: u8,
    pub turn_away: u8,
    pub half: u8,
    pub home_score: u8,
    pub away_score: u8,
    pub active_team: String,
    pub finished: bool,
    pub players: Vec<PlayerJson>,
    pub ball: BallJson,
    pub legal_actions: Vec<serde_json::Value>,
    pub dialog: String,
}

// ── Error helper ──────────────────────────────────────────────────────────────

pub struct AppError(String);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.0).into_response()
    }
}

impl From<String> for AppError {
    fn from(s: String) -> Self { AppError(s) }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self { AppError(s.to_string()) }
}

// ── Index handler (serves HTML) ───────────────────────────────────────────────

pub async fn index_handler() -> Html<String> {
    Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
<title>Blood Bowl</title>
<style>
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ background: #1a1a2e; color: #eee; font-family: monospace; font-size: 13px; }}
#header {{ background: #16213e; padding: 8px 12px; display: flex; align-items: center; gap: 16px; border-bottom: 2px solid #0f3460; }}
#header h1 {{ font-size: 18px; color: #e94560; }}
#scoreboard {{ display: flex; gap: 12px; align-items: center; }}
.score-item {{ background: #0f3460; padding: 4px 10px; border-radius: 4px; }}
#active-team-label {{ padding: 4px 10px; border-radius: 4px; font-weight: bold; }}
#status-msg {{ color: #aaa; font-size: 12px; }}
#main {{ display: flex; gap: 8px; padding: 8px; }}
#board-wrap {{ flex: 0 0 auto; }}
table#board {{ border-collapse: collapse; }}
table#board td {{
  width: 24px; height: 24px;
  border: 1px solid #333;
  position: relative;
  cursor: default;
  background: #2d5a27;
  text-align: center;
  vertical-align: middle;
}}
/* End zones */
td.endzone-home {{ background: #1a3a5c; }}
td.endzone-away {{ background: #3a1a1a; }}
/* Center line */
td.center {{ border-left: 2px solid #666; }}
/* Highlights */
td.highlight-move {{ background: #1a5c1a !important; cursor: pointer; outline: 2px solid #4f4; }}
td.highlight-block {{ background: #5c1a1a !important; cursor: pointer; outline: 2px solid #f44; }}
td.highlight-selected {{ outline: 3px solid #ff0 !important; }}
/* Player circle */
.player-circle {{
  width: 18px; height: 18px;
  border-radius: 50%;
  display: inline-flex; align-items: center; justify-content: center;
  font-size: 9px; font-weight: bold; color: #fff;
  cursor: pointer; user-select: none; position: relative;
  border: 2px solid rgba(255,255,255,0.5);
}}
.player-circle.home {{ background: #1565c0; border-color: #90caf9; }}
.player-circle.away {{ background: #b71c1c; border-color: #ef9a9a; }}
.player-circle.prone {{ border-radius: 50% / 25%; height: 12px; }}
.player-circle.ko {{ background: #555 !important; border-color: #888 !important; }}
.player-circle.ko::after {{ content: 'X'; font-size: 10px; }}
.player-circle.stunned {{ background: #666 !important; border-color: #999 !important; }}
.player-circle.injured {{ background: #333 !important; border-color: #555 !important; opacity: 0.5; }}
.ball-marker {{
  width: 8px; height: 8px;
  background: #f5c518;
  border-radius: 50%;
  border: 1px solid #c8a000;
  position: absolute;
  bottom: 2px; right: 2px;
  pointer-events: none;
}}
#sidebar {{ flex: 1; display: flex; flex-direction: column; gap: 8px; min-width: 200px; }}
#actions-panel {{ background: #16213e; padding: 8px; border-radius: 6px; border: 1px solid #0f3460; }}
#actions-panel h3 {{ color: #e94560; margin-bottom: 6px; }}
#action-buttons {{ display: flex; flex-direction: column; gap: 4px; }}
.action-btn {{
  background: #0f3460; color: #eee; border: 1px solid #1a4a7a;
  padding: 5px 10px; cursor: pointer; border-radius: 3px; text-align: left; font-size: 12px;
}}
.action-btn:hover {{ background: #1a4a7a; }}
.action-btn.end-turn {{ background: #4a0; border-color: #6c0; }}
.action-btn.end-turn:hover {{ background: #5b0; }}
#log {{ background: #16213e; padding: 8px; border-radius: 6px; border: 1px solid #0f3460; flex: 1; overflow-y: auto; max-height: 300px; }}
#log h3 {{ color: #e94560; margin-bottom: 6px; }}
#log-entries {{ display: flex; flex-direction: column; gap: 2px; font-size: 11px; color: #aaa; }}
#new-game-panel {{ background: #16213e; padding: 8px; border-radius: 6px; border: 1px solid #0f3460; }}
#new-game-panel h3 {{ color: #e94560; margin-bottom: 6px; }}
#new-game-panel label {{ display: block; margin-bottom: 3px; font-size: 11px; color: #aaa; }}
#new-game-panel input, #new-game-panel select {{ background: #0f3460; color: #eee; border: 1px solid #1a4a7a; padding: 3px 6px; border-radius: 3px; width: 100%; margin-bottom: 6px; font-size: 12px; }}
#start-btn {{ background: #e94560; color: #fff; border: none; padding: 6px 12px; cursor: pointer; border-radius: 3px; width: 100%; font-size: 13px; }}
#start-btn:hover {{ background: #c73350; }}
</style>
</head>
<body>
<div id="header">
  <h1>Blood Bowl</h1>
  <div id="scoreboard">
    <div class="score-item">Home: <span id="score-home">0</span></div>
    <div class="score-item">Away: <span id="score-away">0</span></div>
    <div class="score-item">Half: <span id="half-num">1</span></div>
    <div class="score-item">Turn H: <span id="turn-home">0</span> / A: <span id="turn-away">0</span></div>
  </div>
  <div id="active-team-label">-</div>
  <div id="status-msg">Start a new game to play.</div>
</div>
<div id="main">
  <div id="board-wrap">
    <table id="board"></table>
  </div>
  <div id="sidebar">
    <div id="new-game-panel">
      <h3>New Game</h3>
      <label>Seed <input id="seed-input" type="number" value="42"/></label>
      <label>MCTS Budget <input id="budget-input" type="number" value="50"/></label>
      <label>Play as
        <select id="human-team-select">
          <option value="home">Home (Human)</option>
          <option value="away">Away (Orc)</option>
        </select>
      </label>
      <button id="start-btn" onclick="startGame()">Start Game</button>
    </div>
    <div id="actions-panel">
      <h3>Actions</h3>
      <div id="action-buttons"><em style="color:#666">No game active</em></div>
    </div>
    <div id="log">
      <h3>Log</h3>
      <div id="log-entries"></div>
    </div>
  </div>
</div>
<script>
const COLS = 26, ROWS = 17;
let gameId = null;
let ws = null;
let currentState = null;
let selectedCell = null;  // {{ x, y }} of selected player square
let pendingActivation = null; // {{ player_id, action_type }}

// ── Board init ────────────────────────────────────────────────────────────────
function buildBoard() {{
  const tbl = document.getElementById('board');
  tbl.innerHTML = '';
  for (let y = 0; y < ROWS; y++) {{
    const tr = document.createElement('tr');
    for (let x = 0; x < COLS; x++) {{
      const td = document.createElement('td');
      td.id = `c${{x}}_${{y}}`;
      if (x <= 1) td.classList.add('endzone-home');
      else if (x >= 24) td.classList.add('endzone-away');
      if (x === 13) td.classList.add('center');
      td.onclick = () => cellClick(x, y);
      tr.appendChild(td);
    }}
    tbl.appendChild(tr);
  }}
}}

// ── Game creation ─────────────────────────────────────────────────────────────
async function startGame() {{
  const seed = parseInt(document.getElementById('seed-input').value) || 42;
  const budget = parseInt(document.getElementById('budget-input').value) || 50;
  const human_team = document.getElementById('human-team-select').value;
  document.getElementById('start-btn').disabled = true;
  document.getElementById('status-msg').textContent = 'Creating game...';

  try {{
    const resp = await fetch('/game/new', {{
      method: 'POST',
      headers: {{ 'Content-Type': 'application/json' }},
      body: JSON.stringify({{ seed, mcts_budget: budget, human_team }})
    }});
    if (!resp.ok) throw new Error(await resp.text());
    const data = await resp.json();
    gameId = data.game_id;
    logMsg('Game started: ' + gameId.slice(0, 8) + '...');
    connectWs();
    await fetchAndRender();
  }} catch(e) {{
    document.getElementById('status-msg').textContent = 'Error: ' + e.message;
  }}
  document.getElementById('start-btn').disabled = false;
}}

// ── WebSocket ─────────────────────────────────────────────────────────────────
function connectWs() {{
  if (ws) ws.close();
  const proto = location.protocol === 'https:' ? 'wss' : 'ws';
  ws = new WebSocket(`${{proto}}://${{location.host}}/game/${{gameId}}/ws`);
  ws.onmessage = (e) => {{
    try {{
      const state = JSON.parse(e.data);
      applyState(state);
      logMsg('AI moved');
    }} catch(err) {{
      console.error('WS parse error', err);
    }}
  }};
  ws.onerror = () => logMsg('WS error');
  ws.onclose = () => logMsg('WS closed');
}}

// ── State fetch ───────────────────────────────────────────────────────────────
async function fetchAndRender() {{
  if (!gameId) return;
  const resp = await fetch(`/game/${{gameId}}/state`);
  if (!resp.ok) {{ logMsg('State fetch failed'); return; }}
  const state = await resp.json();
  applyState(state);
}}

// ── State render ──────────────────────────────────────────────────────────────
function applyState(state) {{
  currentState = state;
  selectedCell = null;
  pendingActivation = null;

  // Header
  document.getElementById('score-home').textContent = state.home_score;
  document.getElementById('score-away').textContent = state.away_score;
  document.getElementById('half-num').textContent = state.half;
  document.getElementById('turn-home').textContent = state.turn_home;
  document.getElementById('turn-away').textContent = state.turn_away;
  const atLabel = document.getElementById('active-team-label');
  atLabel.textContent = state.active_team.toUpperCase() + ' turn';
  atLabel.style.background = state.active_team === 'home' ? '#1565c0' : '#b71c1c';

  const isHumanTurn = state.active_team === humanTeam();

  if (state.finished) {{
    document.getElementById('status-msg').textContent =
      `Game over! Home ${{state.home_score}} - ${{state.away_score}} Away`;
  }} else {{
    document.getElementById('status-msg').textContent =
      isHumanTurn ? 'Your turn' : 'AI thinking...';
  }}

  // Clear board
  for (let y = 0; y < ROWS; y++)
    for (let x = 0; x < COLS; x++) {{
      const td = document.getElementById(`c${{x}}_${{y}}`);
      td.innerHTML = '';
      td.classList.remove('highlight-move','highlight-block','highlight-selected');
    }}

  // Ball
  if (state.ball && state.ball.in_play && state.ball.x != null) {{
    const td = document.getElementById(`c${{state.ball.x}}_${{state.ball.y}}`);
    if (td) {{
      const ball = document.createElement('div');
      ball.className = 'ball-marker';
      td.appendChild(ball);
    }}
  }}

  // Players
  for (const p of state.players) {{
    const td = document.getElementById(`c${{p.x}}_${{p.y}}`);
    if (!td) continue;
    const div = document.createElement('div');
    div.className = `player-circle ${{p.team}}`;
    if (p.state === 'prone') div.classList.add('prone');
    if (p.state === 'ko') div.classList.add('ko');
    if (p.state === 'stunned') div.classList.add('stunned');
    if (p.state === 'injured') div.classList.add('injured');
    // Short label
    const label = p.id.replace('home_','h').replace('away_','a').replace('h','h').replace('a','a');
    div.textContent = p.id.length > 3 ? p.id.slice(-2) : p.id;
    div.title = `${{p.id}} (${{p.team}}) - ${{p.state}}`;
    td.appendChild(div);
  }}

  // Action buttons
  renderActionButtons(state);
}}

function humanTeam() {{
  return document.getElementById('human-team-select').value;
}}

// ── Action UI ─────────────────────────────────────────────────────────────────
function renderActionButtons(state) {{
  const container = document.getElementById('action-buttons');
  container.innerHTML = '';

  if (!state || state.finished) {{
    container.innerHTML = '<em style="color:#666">Game over</em>';
    return;
  }}

  const isHuman = state.active_team === humanTeam();
  if (!isHuman) {{
    container.innerHTML = '<em style="color:#888">AI is thinking...</em>';
    return;
  }}

  // Show top-level legal actions as buttons
  const legal = state.legal_actions;
  if (!legal || legal.length === 0) {{
    container.innerHTML = '<em style="color:#666">No actions available</em>';
    return;
  }}

  // Group: EndTurn, Activate (by player), direct dialog actions
  const endTurns = legal.filter(a => a.type === 'end_turn');
  const activates = legal.filter(a => a.type === 'activate');
  const others = legal.filter(a => a.type !== 'end_turn' && a.type !== 'activate');

  // Dialog actions first
  for (const a of others) {{
    const btn = makeActionBtn(a, labelFor(a));
    container.appendChild(btn);
  }}

  // Activation buttons
  const playerActions = {{}};
  for (const a of activates) {{
    if (!playerActions[a.player_id]) playerActions[a.player_id] = [];
    playerActions[a.player_id].push(a);
  }}
  for (const [pid, acts] of Object.entries(playerActions)) {{
    for (const a of acts) {{
      const btn = makeActionBtn(a, `${{pid}} - ${{a.action}}`);
      container.appendChild(btn);
    }}
  }}

  // End turn
  for (const a of endTurns) {{
    const btn = makeActionBtn(a, 'End Turn');
    btn.classList.add('end-turn');
    container.appendChild(btn);
  }}
}}

function labelFor(a) {{
  switch (a.type) {{
    case 'end_turn': return 'End Turn';
    case 'activate': return `Activate ${{a.player_id}} (${{a.action}})`;
    case 'move_to': return `Move to (${{a.x}},${{a.y}})`;
    case 'block_target': return `Block ${{a.player_id}}`;
    case 'choose_block_die': return `Block die: ${{a.result}}`;
    case 'choose_push': return `Push to (${{a.x}},${{a.y}})`;
    case 'use_reroll': return a.use_it ? 'Use Reroll' : 'Decline Reroll';
    case 'place_ball': return `Place Ball (${{a.x}},${{a.y}})`;
    default: return JSON.stringify(a);
  }}
}}

function makeActionBtn(action, label) {{
  const btn = document.createElement('button');
  btn.className = 'action-btn';
  btn.textContent = label;
  btn.onclick = () => sendAction(action);
  return btn;
}}

// ── Cell click ────────────────────────────────────────────────────────────────
function cellClick(x, y) {{
  if (!currentState) return;
  if (currentState.active_team !== humanTeam()) return;

  const legal = currentState.legal_actions;

  // Check if clicking a move target
  const moveAction = legal.find(a => a.type === 'move_to' && a.x === x && a.y === y);
  if (moveAction) {{ sendAction(moveAction); return; }}

  // Check if clicking a push target
  const pushAction = legal.find(a => a.type === 'choose_push' && a.x === x && a.y === y);
  if (pushAction) {{ sendAction(pushAction); return; }}

  // Check if there's a player at this cell
  const player = currentState.players.find(p => p.x === x && p.y === y);
  if (!player) {{ clearHighlights(); return; }}

  // Check if block target
  const blockAction = legal.find(a => a.type === 'block_target' && a.player_id === player.id);
  if (blockAction) {{ sendAction(blockAction); return; }}

  // Check activate actions for this player
  const activateActions = legal.filter(a => a.type === 'activate' && a.player_id === player.id);
  if (activateActions.length === 0) {{ clearHighlights(); return; }}

  // If one option, pick first (Move by default), then show move targets
  // If player is clicked, auto-activate with Move if available, else first option
  const moveActivate = activateActions.find(a => a.action === 'move') || activateActions[0];
  sendAction(moveActivate);
}}

function clearHighlights() {{
  for (let y = 0; y < ROWS; y++)
    for (let x = 0; x < COLS; x++) {{
      const td = document.getElementById(`c${{x}}_${{y}}`);
      td.classList.remove('highlight-move','highlight-block','highlight-selected');
    }}
  selectedCell = null;
}}

function highlightLegal(state) {{
  const legal = state.legal_actions;
  for (const a of legal) {{
    if (a.type === 'move_to') {{
      const td = document.getElementById(`c${{a.x}}_${{a.y}}`);
      if (td) td.classList.add('highlight-move');
    }}
    if (a.type === 'choose_push') {{
      const td = document.getElementById(`c${{a.x}}_${{a.y}}`);
      if (td) td.classList.add('highlight-move');
    }}
    if (a.type === 'block_target') {{
      const p = state.players.find(p2 => p2.id === a.player_id);
      if (p) {{
        const td = document.getElementById(`c${{p.x}}_${{p.y}}`);
        if (td) td.classList.add('highlight-block');
      }}
    }}
  }}
}}

// ── Send action ───────────────────────────────────────────────────────────────
async function sendAction(action) {{
  if (!gameId) return;
  logMsg('Action: ' + labelFor(action));
  try {{
    const resp = await fetch(`/game/${{gameId}}/action`, {{
      method: 'POST',
      headers: {{ 'Content-Type': 'application/json' }},
      body: JSON.stringify(action)
    }});
    if (!resp.ok) {{ logMsg('Action error: ' + await resp.text()); return; }}
    const state = await resp.json();
    applyState(state);
    highlightLegal(state);
  }} catch(e) {{
    logMsg('Error: ' + e.message);
  }}
}}

// ── Log ───────────────────────────────────────────────────────────────────────
function logMsg(msg) {{
  const el = document.getElementById('log-entries');
  const div = document.createElement('div');
  div.textContent = new Date().toLocaleTimeString() + ' ' + msg;
  el.insertBefore(div, el.firstChild);
  if (el.children.length > 50) el.removeChild(el.lastChild);
}}

// ── Init ──────────────────────────────────────────────────────────────────────
buildBoard();
</script>
</body>
</html>"#.to_string())
}

// ── New game ──────────────────────────────────────────────────────────────────

pub async fn new_game(
    State(shared): State<SharedState>,
    Json(req): Json<NewGameRequest>,
) -> Result<Json<NewGameResponse>, AppError> {
    let seed = req.seed.unwrap_or(42);
    let mcts_budget = req.mcts_budget.unwrap_or(50);
    let human_team = match req.human_team.as_deref() {
        Some("away") => TeamId::Away,
        _ => TeamId::Home,
    };

    // Build 11v11 teams: Home = Human, Away = Orc
    let mut home = Team::new("home".into(), "Reavers".into(), "Human".into(), 3, true);
    for i in 0..11u8 {
        home.add_player(Player::new(
            PlayerId(format!("h{i}")),
            format!("Human{i}"),
            "lineman".into(),
            TeamId::Home,
            i + 1,
            PlayerStats::new(6, 3, 4, 8, None),
            ffb_core::SkillSet::empty(),
        ));
    }

    let mut away = Team::new("away".into(), "Raiders".into(), "Orc".into(), 3, false);
    for i in 0..11u8 {
        away.add_player(Player::new(
            PlayerId(format!("a{i}")),
            format!("Orc{i}"),
            "lineman".into(),
            TeamId::Away,
            i + 1,
            PlayerStats::new(5, 4, 3, 9, None),
            ffb_core::SkillSet::empty(),
        ));
    }

    let mut state = GameState::new(home, away);
    let mut rng = GameRng::new_live(seed);

    // Place players and start game
    place_players_for_kickoff(&mut state);
    default_kickoff_ball_placement(&mut state);
    state.turn_mode = ffb_core::types::TurnMode::Regular;
    // Receiving team (non-kicking) acts first
    state.home_is_active = !state.home_is_offense;
    ffb_core::steps::turn_step::begin_turn(&mut state);

    // If AI goes first, let it move
    let human_team_copy = human_team;
    run_ai_turns(&mut state, &mut rng, mcts_budget, human_team_copy);

    let game_id = Uuid::new_v4().to_string();
    {
        let mut map = shared.lock().unwrap();
        map.insert(
            game_id.clone(),
            Session {
                state,
                rng,
                mcts_budget,
                human_team,
            },
        );
    }

    Ok(Json(NewGameResponse { game_id }))
}

// ── Get state ─────────────────────────────────────────────────────────────────

pub async fn get_state(
    State(shared): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<BoardStateJson>, AppError> {
    let map = shared.lock().unwrap();
    let session = map.get(&id).ok_or("game not found")?;
    Ok(Json(build_board_state(&id, &session.state, session.human_team)))
}

// ── Post action ───────────────────────────────────────────────────────────────

pub async fn post_action(
    State(shared): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<ActionRequest>,
) -> Result<Json<BoardStateJson>, AppError> {
    let mut map = shared.lock().unwrap();
    let session = map.get_mut(&id).ok_or("game not found")?;

    let action = parse_action(&req, &session.state)?;

    // Apply human action
    apply_action_sim(&mut session.state, action, &mut session.rng);

    // Let AI respond until it's human's turn again or game over
    run_ai_turns(&mut session.state, &mut session.rng, session.mcts_budget, session.human_team);

    Ok(Json(build_board_state(&id, &session.state, session.human_team)))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Run AI turns until it's the human's turn or the game is finished.
pub fn run_ai_turns(state: &mut GameState, rng: &mut GameRng, budget: u32, human_team: TeamId) {
    const MAX_AI_ACTIONS: u32 = 500;
    let mut count = 0u32;

    while !state.result.finished && state.active_team_id() != human_team && count < MAX_AI_ACTIONS {
        let ai_team = state.active_team_id();
        let legal = enumerate_actions(state, ai_team);
        if legal.is_empty() {
            break;
        }

        let action = if budget <= 1 || legal.len() == 1 {
            // Fast path: NullStrategy
            legal
                .iter()
                .find(|a| **a == BbAction::EndTurn)
                .cloned()
                .unwrap_or_else(|| legal[0].clone())
        } else {
            let cfg = MctsConfig {
                budget,
                rollout_depth: RolloutDepth::None,
                team: ai_team,
                rollout_strategy: Box::new(NullStrategy),
                ..Default::default()
            };
            MctsSearch::search(state, &cfg, rng)
        };

        apply_action_sim(state, action, rng);
        count += 1;
    }
}

/// Apply a single BbAction to the game state, mirroring the simulation loop logic.
fn apply_action_sim(state: &mut GameState, action: BbAction, rng: &mut GameRng) {
    use ffb_core::pathfinding::find_paths;
    use ffb_core::steps::{
        apply_block_dice_choice, apply_push_choice, begin_activation, begin_block, begin_move,
        end_activation, end_turn,
        TurnStepResult,
    };

    let result = match action {
        BbAction::EndTurn => {
            end_activation(state);
            let r = end_turn(state);
            Some(r)
        }
        BbAction::Activate { player_id, action: player_action } => {
            let r = begin_activation(state, &player_id, rng);
            if r == TurnStepResult::TurnOver {
                let r2 = end_turn(state);
                Some(r2)
            } else {
                if let Some(ap) = state.acting_player.as_mut() {
                    ap.current_action = Some(player_action);
                    match player_action {
                        PlayerAction::Blitz => ap.has_blitzed = true,
                        PlayerAction::Pass => ap.has_passed = true,
                        PlayerAction::HandOff => ap.has_handed_off = true,
                        PlayerAction::Foul => ap.has_fouled = true,
                        _ => {}
                    }
                }
                None
            }
        }
        BbAction::MoveTo(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let player_id = ap.player_id.clone();
                let team = ap.team;
                let movement_remaining = {
                    let p = state.team(team).player_by_id(&player_id).expect("player");
                    p.effective_ma().saturating_sub(state.acting_player.as_ref().unwrap().movement_used)
                };
                let path = {
                    let player = state.team(team).player_by_id(&player_id).expect("player");
                    let paths = find_paths(&state.field, player, &player_id, team, movement_remaining);
                    paths.get(&coord).map(|e| e.path.to_vec())
                };
                if let Some(p) = path {
                    begin_move(state, &player_id, &p, rng);
                }
                end_activation(state);
            }
            None
        }
        BbAction::BlockTarget(defender_id) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                begin_block(state, &attacker_id, &defender_id, rng);
            }
            None
        }
        BbAction::ChooseBlockDie(result) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&attacker_id);
                if let Some(att_coord) = att_coord {
                    let defender_id = att_coord.neighbors().find_map(|n| {
                        let pid = state.field.player_at(n)?;
                        let t = state.field.player_team(pid)?;
                        if t != team { Some(pid.clone()) } else { None }
                    });
                    if let Some(def_id) = defender_id {
                        apply_block_dice_choice(state, &attacker_id, &def_id, result, rng);
                        if state.dialog == DialogState::None {
                            end_activation(state);
                        }
                    }
                }
            }
            None
        }
        BbAction::ChoosePush(coord) => {
            if let Some(ap) = state.acting_player.as_ref() {
                let attacker_id = ap.player_id.clone();
                let team = ap.team;
                let att_coord = state.field.player_coord(&attacker_id);
                if let Some(att_coord) = att_coord {
                    let defender_id = att_coord.neighbors().find_map(|n| {
                        let pid = state.field.player_at(n)?;
                        let t = state.field.player_team(pid)?;
                        if t != team { Some(pid.clone()) } else { None }
                    });
                    if let Some(def_id) = defender_id {
                        apply_push_choice(state, &attacker_id, &def_id, coord, rng);
                    }
                }
                if state.dialog == DialogState::None {
                    end_activation(state);
                }
            }
            None
        }
        BbAction::UseReroll(use_it) => {
            state.dialog = DialogState::None;
            if use_it {
                if let Some(ap) = state.acting_player.as_ref() {
                    let pid = ap.player_id.clone();
                    ffb_core::steps::turn_step::use_team_reroll(state, &pid, rng);
                }
            }
            None
        }
        BbAction::PlaceBall(coord) => {
            state.field.ball.coord = Some(coord);
            state.field.ball.in_play = true;
            state.dialog = DialogState::None;
            None
        }
        BbAction::PassTo(_) => {
            end_activation(state);
            None
        }
        BbAction::ChooseFollowup(_) => {
            // Delegate to simulation loop by running a single step
            None
        }
    };

    if let Some(ts) = result {
        handle_turn_result(state, ts, rng);
    }
}

fn handle_turn_result(
    state: &mut GameState,
    result: ffb_core::steps::TurnStepResult,
    _rng: &mut GameRng,
) {
    use ffb_core::steps::TurnStepResult;
    use ffb_core::types::TurnMode;

    match result {
        TurnStepResult::Ok | TurnStepResult::TurnOver => {
            ffb_core::steps::turn_step::begin_turn(state);
        }
        TurnStepResult::HalfEnd => {
            place_players_for_kickoff(state);
            default_kickoff_ball_placement(state);
            state.turn_mode = TurnMode::Regular;
            state.home_is_active = !state.home_is_offense;
            ffb_core::steps::turn_step::begin_turn(state);
        }
        TurnStepResult::GameEnd => {
            state.result.finished = true;
        }
    }
}

/// Parse an ActionRequest JSON into a BbAction.
fn parse_action(req: &ActionRequest, _state: &GameState) -> Result<BbAction, AppError> {
    match req.action_type.as_str() {
        "end_turn" => Ok(BbAction::EndTurn),
        "activate" => {
            let pid = req
                .player_id
                .as_deref()
                .ok_or("activate requires player_id")?;
            let act = match req.action.as_deref() {
                Some("move") => PlayerAction::Move,
                Some("block") => PlayerAction::Block,
                Some("blitz") => PlayerAction::Blitz,
                Some("pass") => PlayerAction::Pass,
                Some("hand_off") => PlayerAction::HandOff,
                Some("foul") => PlayerAction::Foul,
                _ => PlayerAction::Move,
            };
            Ok(BbAction::Activate {
                player_id: PlayerId(pid.to_string()),
                action: act,
            })
        }
        "move_to" => {
            let x = req.x.ok_or("move_to requires x")?;
            let y = req.y.ok_or("move_to requires y")?;
            Ok(BbAction::MoveTo(FieldCoordinate::new(x, y)))
        }
        "block_target" => {
            let pid = req
                .player_id
                .as_deref()
                .ok_or("block_target requires player_id")?;
            Ok(BbAction::BlockTarget(PlayerId(pid.to_string())))
        }
        "choose_block_die" => {
            let r = req.result.as_deref().ok_or("choose_block_die requires result")?;
            let br = parse_block_result(r)?;
            Ok(BbAction::ChooseBlockDie(br))
        }
        "choose_push" => {
            let x = req.x.ok_or("choose_push requires x")?;
            let y = req.y.ok_or("choose_push requires y")?;
            Ok(BbAction::ChoosePush(FieldCoordinate::new(x, y)))
        }
        "use_reroll" => {
            let use_it = req.use_it.unwrap_or(true);
            Ok(BbAction::UseReroll(use_it))
        }
        "place_ball" => {
            let x = req.x.ok_or("place_ball requires x")?;
            let y = req.y.ok_or("place_ball requires y")?;
            Ok(BbAction::PlaceBall(FieldCoordinate::new(x, y)))
        }
        "pass_to" => {
            let x = req.x.ok_or("pass_to requires x")?;
            let y = req.y.ok_or("pass_to requires y")?;
            Ok(BbAction::PassTo(FieldCoordinate::new(x, y)))
        }
        other => Err(AppError(format!("unknown action type: {other}"))),
    }
}

fn parse_block_result(s: &str) -> Result<BlockResult, AppError> {
    match s {
        "skull" => Ok(BlockResult::Skull),
        "both_down" => Ok(BlockResult::BothDown),
        "pushback" => Ok(BlockResult::Pushback),
        "pow_pushback" => Ok(BlockResult::PowPushback),
        "pow" => Ok(BlockResult::Pow),
        other => Err(AppError(format!("unknown block result: {other}"))),
    }
}

/// Serialize a BbAction to a JSON value for the legal_actions list.
fn action_to_json(action: &BbAction) -> serde_json::Value {
    use serde_json::json;
    match action {
        BbAction::EndTurn => json!({ "type": "end_turn" }),
        BbAction::Activate { player_id, action } => {
            let act_str = match action {
                PlayerAction::Move => "move",
                PlayerAction::Block => "block",
                PlayerAction::Blitz => "blitz",
                PlayerAction::Pass => "pass",
                PlayerAction::HandOff => "hand_off",
                PlayerAction::Foul => "foul",
                PlayerAction::Unused => "unused",
            };
            json!({ "type": "activate", "player_id": player_id.0, "action": act_str })
        }
        BbAction::MoveTo(coord) => json!({ "type": "move_to", "x": coord.x, "y": coord.y }),
        BbAction::BlockTarget(pid) => json!({ "type": "block_target", "player_id": pid.0 }),
        BbAction::ChooseBlockDie(r) => {
            let s = match r {
                BlockResult::Skull => "skull",
                BlockResult::BothDown => "both_down",
                BlockResult::Pushback => "pushback",
                BlockResult::PowPushback => "pow_pushback",
                BlockResult::Pow => "pow",
            };
            json!({ "type": "choose_block_die", "result": s })
        }
        BbAction::ChoosePush(coord) => json!({ "type": "choose_push", "x": coord.x, "y": coord.y }),
        BbAction::PassTo(coord) => json!({ "type": "pass_to", "x": coord.x, "y": coord.y }),
        BbAction::UseReroll(use_it) => json!({ "type": "use_reroll", "use_it": use_it }),
        BbAction::PlaceBall(coord) => json!({ "type": "place_ball", "x": coord.x, "y": coord.y }),
        BbAction::ChooseFollowup(follow_up) => json!({ "type": "choose_followup", "follow_up": follow_up }),
    }
}

fn dialog_name(dialog: &DialogState) -> &'static str {
    match dialog {
        DialogState::None => "none",
        DialogState::SelectPlayer { .. } => "select_player",
        DialogState::SelectMoveTarget { .. } => "select_move_target",
        DialogState::SelectBlockTarget { .. } => "select_block_target",
        DialogState::SelectBlockDice { .. } => "select_block_dice",
        DialogState::SelectReroll { .. } => "select_reroll",
        DialogState::SelectBlockReroll { .. } => "select_block_reroll",
        DialogState::SelectPush { .. } => "select_push",
        DialogState::SelectInjury => "select_injury",
        DialogState::SelectApothecary { .. } => "select_apothecary",
        DialogState::SelectKickTarget => "select_kick_target",
        DialogState::SelectKickoffReturn => "select_kickoff_return",
        DialogState::SelectHighKickPlayer => "select_high_kick_player",
        DialogState::SelectFollowup { .. } => "select_followup",
    }
}

pub fn build_board_state(game_id: &str, state: &GameState, _human_team: TeamId) -> BoardStateJson {
    let active_team_id = state.active_team_id();

    // Enumerate legal actions for active team
    let legal_actions: Vec<serde_json::Value> = enumerate_actions(state, active_team_id)
        .iter()
        .map(action_to_json)
        .collect();

    // Build player list from all on-pitch players
    let mut players: Vec<PlayerJson> = Vec::new();
    for (pid, coord, pstate) in state.field.on_pitch_players() {
        let team = state.field.player_team(pid).unwrap_or(TeamId::Home);
        let state_str = match pstate {
            PlayerState::Standing => "standing",
            PlayerState::Moving => "moving",
            PlayerState::Prone => "prone",
            PlayerState::Stunned => "stunned",
            PlayerState::Ko => "ko",
            PlayerState::Injured => "injured",
            PlayerState::Reserve => "reserve",
            PlayerState::Rooted => "rooted",
        };
        players.push(PlayerJson {
            id: pid.0.clone(),
            team: if team == TeamId::Home { "home".into() } else { "away".into() },
            x: coord.x,
            y: coord.y,
            state: state_str.into(),
        });
    }

    let ball = BallJson {
        x: state.field.ball.coord.map(|c| c.x),
        y: state.field.ball.coord.map(|c| c.y),
        in_play: state.field.ball.in_play,
    };

    BoardStateJson {
        game_id: game_id.to_string(),
        turn_home: state.turn_data_home.turn_number,
        turn_away: state.turn_data_away.turn_number,
        half: match state.half {
            Half::First => 1,
            Half::Second => 2,
        },
        home_score: state.result.score_home,
        away_score: state.result.score_away,
        active_team: if active_team_id == TeamId::Home {
            "home".into()
        } else {
            "away".into()
        },
        finished: state.result.finished,
        players,
        ball,
        legal_actions,
        dialog: dialog_name(&state.dialog).to_string(),
    }
}
