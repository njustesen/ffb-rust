use std::fmt::Write as FmtWrite;
use crate::runner::GameSnap;

pub fn generate_html(seed: u64, home: &str, away: &str, edition: &str, snaps: &[GameSnap]) -> String {
    let json = serde_json::to_string(snaps).unwrap_or_default();
    let json_safe = json.replace("</script>", "<\\/script>");
    let total = snaps.len();
    let max_idx = total.saturating_sub(1);

    let mut h = String::with_capacity(256 * 1024);

    h.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"UTF-8\">\n");
    write!(h, "<title>BB Replay: {} vs {} seed {} ({})</title>\n", home, away, seed, edition).ok();
    h.push_str("<style>\n");
    h.push_str(CSS);
    h.push_str("</style>\n</head>\n<body>\n");

    write!(h, "<div class=\"header\"><h2>⚔ Blood Bowl Replay — {} vs {} &nbsp;·&nbsp; Seed {} &nbsp;·&nbsp; {}</h2></div>\n",
        home, away, seed, edition.to_uppercase()).ok();

    h.push_str("<div class=\"layout\">\n<div class=\"main\">\n");
    h.push_str("<div id=\"info\">loading...</div>\n");
    write!(h, "<svg id=\"board\" xmlns=\"http://www.w3.org/2000/svg\" width=\"624\" height=\"500\" style=\"display:block;max-width:100%\"></svg>\n").ok();
    h.push_str("<div class=\"controls\">\n");
    h.push_str("  <div class=\"step-line\"><span id=\"step-counter\">1/1</span> &nbsp; <span id=\"step-label\"></span></div>\n");
    write!(h, "  <input type=\"range\" id=\"scrubber\" min=\"0\" max=\"{}\" value=\"0\" oninput=\"goTo(+this.value)\">\n", max_idx).ok();
    h.push_str("  <div class=\"buttons\">\n");
    h.push_str("    <button onclick=\"goFirst()\" title=\"First step\">|◀</button>\n");
    h.push_str("    <button onclick=\"prevTurn()\" title=\"Previous turn\">◀ Turn</button>\n");
    h.push_str("    <button onclick=\"goPrev()\" title=\"Previous step [←]\">◀</button>\n");
    h.push_str("    <button id=\"play-btn\" onclick=\"togglePlay()\">▶ Play</button>\n");
    h.push_str("    <button onclick=\"goNext()\" title=\"Next step [→]\">▶</button>\n");
    h.push_str("    <button onclick=\"nextTurn()\" title=\"Next turn\">Turn ▶</button>\n");
    h.push_str("    <button onclick=\"goLast()\" title=\"Last step\">▶|</button>\n");
    h.push_str("    <label>Speed: <select id=\"speed\" onchange=\"updateSpeed()\">\n");
    h.push_str("      <option value=\"1000\">0.5×</option>\n");
    h.push_str("      <option value=\"500\" selected>1×</option>\n");
    h.push_str("      <option value=\"250\">2×</option>\n");
    h.push_str("      <option value=\"125\">4×</option>\n");
    h.push_str("    </select></label>\n");
    h.push_str("  </div>\n");
    h.push_str("</div>\n"); // controls

    h.push_str("<div id=\"events\"></div>\n"); // events panel
    h.push_str("<div class=\"legend\">\n");
    h.push_str("<span class=\"leg-item\"><span class=\"leg-dot\" style=\"background:#3b82f6\"></span>Home</span>\n");
    h.push_str("<span class=\"leg-item\"><span class=\"leg-dot\" style=\"background:#ef4444\"></span>Away</span>\n");
    h.push_str("<span class=\"leg-item\"><span class=\"leg-dot\" style=\"background:#6b7280\"></span>Stunned</span>\n");
    h.push_str("<span class=\"leg-item\"><span class=\"leg-dot\" style=\"border:3px solid #fbbf24;background:transparent\"></span>Acting</span>\n");
    h.push_str("<span class=\"leg-item\">Circle size = ST &nbsp; X = Prone &nbsp; Faded = Used/KO/Inj &nbsp; Click player for details</span>\n");
    h.push_str("</div>\n"); // legend

    // Player detail popup (hidden until player clicked)
    h.push_str("<div id=\"player-detail\" style=\"display:none\"></div>\n");

    h.push_str("</div>\n"); // main
    h.push_str("<div class=\"sidebar\">\n");
    h.push_str("<div class=\"sidebar-title\">Action Log</div>\n");
    h.push_str("<div id=\"log\"></div>\n");
    h.push_str("</div>\n"); // sidebar
    h.push_str("</div>\n"); // layout

    // Script
    h.push_str("<script>\n");
    h.push_str("const STEPS = ");
    h.push_str(&json_safe);
    h.push_str(";\n");
    write!(h, "const TOTAL = {};\n", total).ok();
    h.push_str(JS);
    h.push_str("\n// Init\nbuildLog();\nrender(0);\n");
    h.push_str("</script>\n</body>\n</html>\n");

    h
}

// ── CSS ───────────────────────────────────────────────────────────────────────

static CSS: &str = r##"
* { box-sizing: border-box; margin: 0; padding: 0; }
body { background: #0d1117; color: #e6edf3; font-family: 'Courier New', monospace; font-size: 13px; }
.header { padding: 12px 16px; border-bottom: 1px solid #30363d; }
.header h2 { font-size: 16px; font-weight: 600; color: #58a6ff; }
.layout { display: flex; gap: 12px; padding: 12px; align-items: flex-start; }
.main { flex: 0 0 auto; }
.sidebar { flex: 0 0 260px; }
.sidebar-title { font-size: 12px; font-weight: 700; color: #8b949e; text-transform: uppercase;
    letter-spacing: 1px; padding: 6px 0 4px; border-bottom: 1px solid #21262d; margin-bottom: 4px; }
#info { padding: 6px 8px; background: #161b22; border: 1px solid #30363d; border-radius: 6px;
    margin-bottom: 8px; min-height: 28px; font-size: 12px; }
#info .half { color: #79c0ff; font-weight: 700; }
#info .turn { color: #d2a8ff; }
#info .active { color: #ffa657; font-weight: 600; }
#info .score { color: #3fb950; font-weight: 700; font-size: 14px; }
#info .rr { color: #8b949e; }
#info .bribes { color: #e3b341; }
#board { border: 1px solid #30363d; border-radius: 4px; background: #0d1117; display: block; cursor: default; }
.controls { margin-top: 8px; }
.step-line { display: flex; align-items: center; gap: 8px; margin-bottom: 6px; font-size: 12px; }
#step-counter { color: #8b949e; white-space: nowrap; }
#step-label { color: #e6edf3; font-weight: 600; white-space: nowrap; overflow: hidden;
    text-overflow: ellipsis; max-width: 380px; }
#scrubber { width: 624px; max-width: 100%; margin: 4px 0; accent-color: #58a6ff; }
.buttons { display: flex; gap: 6px; align-items: center; flex-wrap: wrap; margin-top: 6px; }
button { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px;
    padding: 4px 10px; cursor: pointer; font-size: 12px; font-family: inherit; }
button:hover { background: #30363d; color: #e6edf3; }
#play-btn { background: #238636; border-color: #2ea043; color: #fff; padding: 4px 14px; }
#play-btn:hover { background: #2ea043; }
#play-btn.paused { background: #b45309; border-color: #d97706; }
select { background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 4px;
    padding: 3px 6px; font-family: inherit; font-size: 12px; }
.legend { display: flex; gap: 12px; flex-wrap: wrap; padding: 8px 0; font-size: 11px; color: #8b949e; }
.leg-item { display: flex; align-items: center; gap: 4px; }
.leg-dot { display: inline-block; width: 12px; height: 12px; border-radius: 50%;
    border: 1px solid rgba(255,255,255,0.2); }
#log { height: 380px; overflow-y: auto; font-size: 11px; line-height: 1.5;
    scrollbar-width: thin; scrollbar-color: #30363d #0d1117; }
#log::-webkit-scrollbar { width: 6px; }
#log::-webkit-scrollbar-track { background: #0d1117; }
#log::-webkit-scrollbar-thumb { background: #30363d; border-radius: 3px; }
.log-entry { padding: 2px 6px; cursor: pointer; border-radius: 3px; white-space: nowrap;
    overflow: hidden; text-overflow: ellipsis; color: #8b949e; }
.log-entry:hover { background: #1c2128; color: #c9d1d9; }
.log-entry.active { background: #1f3c5e; color: #79c0ff; font-weight: 600; }
#events { min-height: 28px; max-height: 80px; overflow-y: auto; display: flex;
    flex-wrap: wrap; gap: 4px; padding: 4px 0; font-size: 11px; }
.ev { padding: 2px 6px; border-radius: 10px; white-space: nowrap; font-size: 10px; }
.ev-ok  { background: #1a3a1a; color: #4ade80; border: 1px solid #166534; }
.ev-fail { background: #3a1a1a; color: #f87171; border: 1px solid #7f1d1d; }
.ev-block { background: #1a1a3a; color: #a5b4fc; border: 1px solid #312e81; }
.ev-inj  { background: #3a2a1a; color: #fbbf24; border: 1px solid #78350f; }
.ev-td   { background: #1a3a2a; color: #34d399; border: 1px solid #065f46; font-weight: 700; }
.ev-rr   { background: #2a1a3a; color: #c084fc; border: 1px solid #4c1d95; }
.ev-misc { background: #1a1f2a; color: #94a3b8; border: 1px solid #334155; }
#player-detail { position: fixed; top: 80px; left: 50%; transform: translateX(-50%);
    background: #1c2128; border: 1px solid #58a6ff; border-radius: 8px;
    padding: 12px 16px; min-width: 220px; max-width: 320px; z-index: 100;
    box-shadow: 0 4px 24px rgba(0,0,0,0.5); font-size: 12px; }
#player-detail .pd-close { float: right; cursor: pointer; color: #8b949e; font-size: 16px; line-height: 1; }
#player-detail .pd-close:hover { color: #e6edf3; }
#player-detail .pd-name { font-size: 14px; font-weight: 700; margin-bottom: 6px; }
#player-detail .pd-home { color: #93c5fd; }
#player-detail .pd-away { color: #fca5a5; }
#player-detail .pd-stats { display: grid; grid-template-columns: repeat(5, 1fr);
    gap: 4px; margin-bottom: 6px; text-align: center; }
#player-detail .pd-stat-lbl { color: #8b949e; font-size: 10px; }
#player-detail .pd-stat-val { color: #e6edf3; font-weight: 600; font-size: 13px; }
#player-detail .pd-skills { color: #c9d1d9; font-size: 11px; }
#player-detail .pd-status { margin-top: 6px; }
"##;

// ── JS ────────────────────────────────────────────────────────────────────────

static JS: &str = r##"
const CELL = 24;
const PW = 26 * CELL;
const PH = 15 * CELL;
const DH = 66;
const DP = 4;
const PY = DH + DP;
const SW = PW;
const SH = DH + DP + PH + DP + DH;

// Base state constants (matching Rust PS_ values)
const PS_STANDING=1, PS_MOVING=2, PS_PRONE=3, PS_STUNNED=4,
      PS_KO=5, PS_BH=6, PS_SI=7, PS_RIP=8, PS_RESERVE=9,
      PS_MISSING=10, PS_BANNED=13;

// Dugout sections for home (negative x) and away (x >= 30)
const SECTIONS = [
  { hx:-1,  ax:30, lbl:'Rsv', bg:'#1a2e4a' },
  { hx:-2,  ax:31, lbl:'KO',  bg:'#3b1f6e' },
  { hx:-3,  ax:32, lbl:'BH',  bg:'#6b3400' },
  { hx:-4,  ax:33, lbl:'SI',  bg:'#6b0000' },
  { hx:-5,  ax:34, lbl:'✝',   bg:'#1a1a1a' },
  { hx:-6,  ax:35, lbl:'⊘',   bg:'#7f1010' },
];
const SEC_W = PW / SECTIONS.length;  // ~104

let cur = 0, playing = false, timer = null;

function pFill(p) {
  if (p.bs === PS_STUNNED) return '#6b7280';
  if (p.bs === PS_PRONE)   return p.h ? '#1e3a8a' : '#7f1d1d';
  return p.h ? '#3b82f6' : '#ef4444';
}
function pOpacity(p) {
  if (p.bs >= PS_KO && p.bs <= PS_RIP) return 0.38;
  if (p.bs === PS_BANNED)              return 0.45;
  if (p.bs === PS_RESERVE || p.bs === PS_MISSING) return 0.7;
  if (!p.act && (p.bs === PS_STANDING || p.bs === PS_MOVING)) return 0.60;
  return 1.0;
}
function pStroke(p) {
  if (p.cur) return ['#fbbf24', 3];
  return [p.h ? '#93c5fd' : '#fca5a5', 1];
}

function drawPlayerAt(p, cx, cy, r, maStr) {
  const fill = pFill(p);
  const op   = pOpacity(p);
  const [sc, sw] = pStroke(p);
  let s = `<circle cx="${cx.toFixed(1)}" cy="${cy.toFixed(1)}" r="${r}" fill="${fill}" opacity="${op}" stroke="${sc}" stroke-width="${sw}"/>`;
  // Prone — single horizontal line (lying flat)
  if (p.bs === PS_PRONE) {
    const d = r * 0.6;
    s += `<line x1="${(cx-d).toFixed(1)}" y1="${cy.toFixed(1)}" x2="${(cx+d).toFixed(1)}" y2="${cy.toFixed(1)}" stroke="rgba(255,255,255,0.9)" stroke-width="2.5" stroke-linecap="round" opacity="${op}"/>`;
  }
  // Stunned — X cross
  if (p.bs === PS_STUNNED) {
    const d = r * 0.52;
    s += `<line x1="${(cx-d).toFixed(1)}" y1="${(cy-d).toFixed(1)}" x2="${(cx+d).toFixed(1)}" y2="${(cy+d).toFixed(1)}" stroke="rgba(255,255,255,0.9)" stroke-width="1.5" opacity="${op}"/>`;
    s += `<line x1="${(cx+d).toFixed(1)}" y1="${(cy-d).toFixed(1)}" x2="${(cx-d).toFixed(1)}" y2="${(cy+d).toFixed(1)}" stroke="rgba(255,255,255,0.9)" stroke-width="1.5" opacity="${op}"/>`;
  }
  // Jersey number
  const fs = r >= 10 ? 9 : 7;
  s += `<text x="${cx.toFixed(1)}" y="${cy.toFixed(1)}" text-anchor="middle" dominant-baseline="middle" font-size="${fs}" fill="white" opacity="${op}" font-weight="bold" font-family="monospace">${p.nr}</text>`;
  // MA display — bold black with white outline, positioned above-left of circle
  if (maStr) {
    const tx = (cx - r - 2).toFixed(1), ty = (cy - r + 4).toFixed(1);
    s += `<text x="${tx}" y="${ty}" font-size="8" font-family="monospace" font-weight="bold" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" paint-order="stroke fill">${maStr}</text>`;
    s += `<text x="${tx}" y="${ty}" font-size="8" font-family="monospace" font-weight="bold" fill="#111">${maStr}</text>`;
  }
  // Tooltip for native browser hover
  const status = p.bs===PS_PRONE?'Prone':p.bs===PS_STUNNED?'Stunned':p.bs===PS_KO?'KO':p.bs===PS_BH?'BH':p.bs===PS_SI?'SI':p.bs===PS_RIP?'Dead':p.bs===PS_BANNED?'Ejected':'';
  const skills = (p.skls||[]).join(', ');
  s += `<title>${p.nm} #${p.nr} | MA${p.ma} ST${p.st} AG${p.ag}+ AV${p.av}+${skills?' | '+skills:''}${status?' | '+status:''}</title>`;
  const safeId = (p.id||'').replace(/'/g,'');
  // Hover to show detail panel, hide on mouseout
  return `<g style="cursor:pointer" onmouseenter="showPlayerById('${safeId}')" onmouseleave="closeDetail()">${s}</g>`;
}

function drawDugout(snap, isHome) {
  const sy = isHome ? 0 : PY + PH + DP;
  let s = '';
  // bg strip
  s += `<rect x="0" y="${sy}" width="${SW}" height="${DH}" fill="${isHome?'#0a1a33':'#330a0a'}"/>`;
  // team label
  s += `<text x="${SW/2}" y="${sy+DH-6}" text-anchor="middle" font-size="9" fill="rgba(150,150,150,0.5)" font-family="monospace">${isHome?'HOME DUGOUT':'AWAY DUGOUT'}</text>`;

  for (let i = 0; i < SECTIONS.length; i++) {
    const sec = SECTIONS[i];
    const xKey = isHome ? sec.hx : sec.ax;
    const sx = i * SEC_W;

    // Section background
    s += `<rect x="${sx+1}" y="${sy+2}" width="${SEC_W-2}" height="${DH-4}" fill="${sec.bg}" opacity="0.45" rx="3"/>`;
    // Section label
    s += `<text x="${sx+SEC_W/2}" y="${sy+12}" text-anchor="middle" font-size="8" fill="rgba(180,180,180,0.75)" font-family="monospace">${sec.lbl}</text>`;

    // Players in this section
    const players = snap.ps.filter(p => p.x === xKey && p.h === isHome);
    const sr = 7;
    for (let j = 0; j < players.length && j < 8; j++) {
      const col = j % 4;
      const row = Math.floor(j / 4);
      const pcx = sx + sr + 4 + col * ((SEC_W - sr*2 - 4) / 3.5);
      const pcy = sy + 20 + row * (sr * 2 + 4);
      if (pcy + sr < sy + DH - 2) s += drawPlayerAt(players[j], pcx, pcy, sr, '');
    }
  }
  return s;
}

function buildSVG(snap) {
  let s = '';
  // Full background
  s += `<rect width="${SW}" height="${SH}" fill="#0d1117"/>`;

  // Dugouts
  s += drawDugout(snap, true);
  s += drawDugout(snap, false);

  // Pitch background
  s += `<rect x="0" y="${PY}" width="${PW}" height="${PH}" fill="#1d5c0c"/>`;

  // Endzones — col 0 (home) and col 25 (away), each exactly 1 square wide
  s += `<rect x="0" y="${PY}" width="${CELL}" height="${PH}" fill="rgba(30,64,175,0.35)"/>`;
  s += `<rect x="${CELL*25}" y="${PY}" width="${CELL}" height="${PH}" fill="rgba(159,18,18,0.35)"/>`;

  // Wide zone shading (rows 0-2 and 12-14), full pitch width including endzones
  for (const [r0, r1] of [[0,3],[12,15]]) {
    s += `<rect x="0" y="${PY+r0*CELL}" width="${PW}" height="${(r1-r0)*CELL}" fill="rgba(255,255,255,0.05)"/>`;
  }

  // LOS dashed lines (between col 12-13 and 13-14)
  s += `<line x1="${13*CELL}" y1="${PY}" x2="${13*CELL}" y2="${PY+PH}" stroke="rgba(255,255,255,0.4)" stroke-width="1" stroke-dasharray="4,3"/>`;

  // Grid lines
  for (let c = 0; c <= 26; c++) {
    s += `<line x1="${c*CELL}" y1="${PY}" x2="${c*CELL}" y2="${PY+PH}" stroke="rgba(255,255,255,0.10)" stroke-width="0.5"/>`;
  }
  for (let r = 0; r <= 15; r++) {
    s += `<line x1="0" y1="${PY+r*CELL}" x2="${PW}" y2="${PY+r*CELL}" stroke="rgba(255,255,255,0.10)" stroke-width="0.5"/>`;
  }

  // Column labels (every 4th, on pitch edge)
  for (const c of [0,4,8,12,13,14,18,22,25]) {
    s += `<text x="${c*CELL+CELL/2}" y="${PY-3}" text-anchor="middle" font-size="7" fill="rgba(255,255,255,0.35)" font-family="monospace">${c}</text>`;
  }

  // On-pitch players (drawn before ball so ball renders on top)
  for (const p of snap.ps) {
    if (p.x >= 0 && p.x < 26 && p.y >= 0 && p.y < 15) {
      const cx = p.x * CELL + CELL/2;
      const cy = PY + p.y * CELL + CELL/2;
      const r = Math.max(7, Math.min(13, 5.5 + p.st * 1.4));
      // MA display for acting player
      const maStr = (snap.act_id === p.id && snap.act_ma_max > 0)
        ? `${snap.act_ma_spent}/${snap.act_ma_max}` : '';
      s += drawPlayerAt(p, cx, cy, r, maStr);
    }
  }

  // Ball drawn AFTER players so it's always visible on top
  if (snap.bp && snap.bx != null && snap.bx >= 0 && snap.bx < 26 && snap.by >= 0 && snap.by < 15) {
    const bcx = snap.bx * CELL + CELL/2;
    const bcy = PY + snap.by * CELL + CELL/2;
    const carrier = snap.ps.find(p => p.id === snap.ball_carrier && p.x >= 0 && p.x < 26);
    if (carrier) {
      // When a player carries the ball, show it as a small dot at the top-right of their circle
      const r = Math.max(7, Math.min(13, 5.5 + carrier.st * 1.4));
      s += `<circle cx="${bcx + r*0.65}" cy="${bcy - r*0.65}" r="4" fill="#f59e0b" stroke="#d97706" stroke-width="1.5"/>`;
    } else {
      // Ball on the ground
      s += `<ellipse cx="${bcx+1}" cy="${bcy+3}" rx="5" ry="2.5" fill="rgba(0,0,0,0.4)"/>`;
      s += `<circle cx="${bcx}" cy="${bcy}" r="5.5" fill="#f59e0b" stroke="#d97706" stroke-width="1.5"/>`;
    }
  }

  return s;
}

function render(i) {
  cur = i;
  const snap = STEPS[i];
  if (!snap) return;

  try {
    document.getElementById('board').innerHTML = buildSVG(snap);
  } catch(e) {
    document.getElementById('board').innerHTML =
      `<text x="10" y="50" fill="#ef4444" font-family="monospace" font-size="12">Render error: ${e.message}</text>`;
  }

  // Info bar
  const halfLbl = snap.hl > 0 ? `H${snap.hl}` : 'Pre-game';
  const turnLbl = snap.t > 0 ? `T${snap.t}` : '–';
  const activeLbl = snap.hp ? '🔵 HOME' : '🔴 AWAY';
  let info = `<span class="half">${halfLbl}</span> · <span class="turn">${turnLbl}</span>` +
    ` · <span class="active">${activeLbl}</span>` +
    ` &nbsp;<span class="score">${snap.hs}–${snap.aw}</span>` +
    ` &nbsp;<span class="rr">Home RR:${snap.hr}  Away RR:${snap.ar}</span>`;
  if (snap.hb + snap.ab > 0) info += ` <span class="bribes">Bribes H:${snap.hb} A:${snap.ab}</span>`;
  document.getElementById('info').innerHTML = info;

  // Events panel
  document.getElementById('events').innerHTML = renderEvents(snap.evs || []);

  // Step counter & label
  document.getElementById('step-counter').textContent = `${i+1} / ${TOTAL}`;
  document.getElementById('step-label').textContent = snap.l;

  // Scrubber
  document.getElementById('scrubber').value = i;

  // Action log highlight
  const entries = document.querySelectorAll('.log-entry');
  entries.forEach((el, idx2) => el.classList.toggle('active', idx2 === i));
  const el = document.getElementById(`log-${i}`);
  if (el) el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
}

// ── Event rendering ───────────────────────────────────────────────────────────

// JSON uses snake_case field names (serde rename_all="camelCase" only renames variant tags).
function renderEvents(evs) {
  if (!evs || !evs.length) return '';
  return evs.map(ev => {
    const t = ev.type || '';
    if (t === 'dodgeRoll' || t === 'goForItRoll' || t === 'jumpRoll' || t === 'standUpRoll') {
      const ok = ev.success;
      const cls = ok ? 'ev-ok' : 'ev-fail';
      const rr = ev.rerolled ? ' ↻' : '';
      return `<span class="ev ${cls}">${fmtPid(ev.player_id)} ${shortType(t)} ${ev.roll}/${ev.target}${ok?'✓':'✗'}${rr}</span>`;
    }
    if (t === 'blockRoll') {
      const dice = ev.dice || [];
      const sel = dice[ev.selected_index] ?? '?';
      const rr = ev.rerolled ? ' ↻' : '';
      return `<span class="ev ev-block">⚔ [${dice.join(',')}]→${sel}${rr}</span>`;
    }
    if (t === 'injury') {
      const ko = ev.was_ko, cas = ev.was_cas;
      const cls = cas ? 'ev-inj' : ko ? 'ev-inj' : 'ev-fail';
      const tag = ev.serious_injury ? `(${ev.serious_injury})` : cas ? '(CAS)' : ko ? '(KO)' : '(BH)';
      const av = ev.armor_roll ? `AV[${ev.armor_roll.join(',')}] ` : '';
      return `<span class="ev ${cls}">☠ ${fmtPid(ev.player_id)} ${av}${tag} — off pitch</span>`;
    }
    if (t === 'apothecaryChoice') {
      // healed=true means the apothecary was used (re-rolls injury dice, picks better result).
      // Even with apothecary, BH still removes the player from the pitch this drive.
      if (ev.healed) {
        return `<span class="ev ev-ok">🏥 ${fmtPid(ev.player_id)} Apothecary used — injury re-rolled (see result above)</span>`;
      } else {
        return `<span class="ev ev-misc">🏥 ${fmtPid(ev.player_id)} Apothecary declined</span>`;
      }
    }
    if (t === 'touchdown') {
      return `<span class="ev ev-td">🏈 TD! ${fmtPid(ev.player_id)}</span>`;
    }
    if (t === 'ballPickedUp') {
      return `<span class="ev ev-ok">↑ Ball ${fmtPid(ev.player_id)}</span>`;
    }
    if (t === 'catchRoll') {
      const cls = ev.success ? 'ev-ok' : 'ev-fail';
      return `<span class="ev ${cls}">${fmtPid(ev.player_id)} Catch ${ev.roll}/${ev.target}${ev.success?'✓':'✗'}</span>`;
    }
    if (t === 'passRoll') {
      const cls = (ev.result === 'complete' || ev.result === 'inaccurate') ? 'ev-ok' : 'ev-fail';
      return `<span class="ev ${cls}">Pass ${ev.roll}→${ev.result}</span>`;
    }
    if (t === 'reRoll') {
      return `<span class="ev ev-rr">↻ RR(${ev.rerolled_action})</span>`;
    }
    if (t === 'playerFellDown') {
      return `<span class="ev ev-fail">↓ ${fmtPid(ev.player_id)} fell</span>`;
    }
    if (t === 'playerAction') {
      return `<span class="ev ev-misc">${fmtPid(ev.player_id)} ${ev.action}</span>`;
    }
    if (t === 'playerMoved') {
      return `<span class="ev ev-misc">→ ${fmtPid(ev.player_id)} (${ev.coord?.x},${ev.coord?.y})</span>`;
    }
    if (t === 'pushback') {
      return `<span class="ev ev-misc">⇒ ${fmtPid(ev.defender_id)} pushed</span>`;
    }
    if (t === 'turnEnd') { return ''; }
    if (t === 'startHalf') {
      return `<span class="ev ev-misc">▶ Half ${ev.half}</span>`;
    }
    if (t === 'kickoffResultEvent') {
      return `<span class="ev ev-misc">⚽ Kickoff: ${ev.result}</span>`;
    }
    return '';
  }).filter(Boolean).join('');
}
function fmtPid(pid) { return pid ? '#' + pid.replace(/^(home|away)_0*/, '') : '?'; }
function shortType(t) {
  return {dodgeRoll:'Dodge', goForItRoll:'GFI', jumpRoll:'Jump', standUpRoll:'StandUp'}[t] || t;
}

// ── Player detail panel ───────────────────────────────────────────────────────

let selectedPlayerId = null;

function showPlayerById(pid) {
  const snap = STEPS[cur];
  if (!snap) return;
  const p = snap.ps.find(pl => pl.id === pid);
  if (p) showPlayerDetail(p);
}

function showPlayerDetail(p) {
  if (!p) return;
  selectedPlayerId = p.id;
  const teamCls = p.h ? 'pd-home' : 'pd-away';
  const teamName = p.h ? 'Home' : 'Away';
  const status = p.bs===PS_PRONE?'Prone':p.bs===PS_STUNNED?'Stunned':p.bs===PS_KO?'KO':
    p.bs===PS_BH?'Badly Hurt':p.bs===PS_SI?'Serious Inj':p.bs===PS_RIP?'Dead':
    p.bs===PS_BANNED?'Ejected':'Standing';
  const skls = (p.skls || []).join(', ') || '–';
  const html = `
    <span class="pd-close" onclick="closeDetail()">✕</span>
    <div class="pd-name ${teamCls}">#${p.nr} ${p.nm} <small>${teamName}</small></div>
    <div class="pd-stats">
      <div><div class="pd-stat-lbl">MA</div><div class="pd-stat-val">${p.ma}</div></div>
      <div><div class="pd-stat-lbl">ST</div><div class="pd-stat-val">${p.st}</div></div>
      <div><div class="pd-stat-lbl">AG</div><div class="pd-stat-val">${p.ag}+</div></div>
      <div><div class="pd-stat-lbl">PA</div><div class="pd-stat-val">${p.pa > 0 ? p.pa+'+' : '–'}</div></div>
      <div><div class="pd-stat-lbl">AV</div><div class="pd-stat-val">${p.av}+</div></div>
    </div>
    <div class="pd-skills"><b>Skills:</b> ${skls}</div>
    <div class="pd-status"><b>Status:</b> ${status}${p.cur?' · Acting':''}</div>
  `;
  const panel = document.getElementById('player-detail');
  panel.innerHTML = html;
  panel.style.display = 'block';
}

function closeDetail() {
  document.getElementById('player-detail').style.display = 'none';
  selectedPlayerId = null;
}

// Pin panel on click (so it stays while you read), close on background click
document.getElementById('board').addEventListener('click', e => {
  if (!e.target.closest('g[onmouseenter]')) closeDetail();
});
document.addEventListener('click', e => {
  const pd = document.getElementById('player-detail');
  if (pd.style.display !== 'none'
      && !pd.contains(e.target)
      && !e.target.closest('#board')) {
    closeDetail();
  }
});

function goTo(i) {
  if (i >= 0 && i < TOTAL) render(i);
}
function goFirst() { goTo(0); }
function goLast()  { goTo(TOTAL - 1); }
function goNext()  { goTo(cur + 1); }
function goPrev()  { goTo(cur - 1); }

function nextTurn() {
  const ct = STEPS[cur].t, chp = STEPS[cur].hp, chl = STEPS[cur].hl;
  for (let i = cur + 1; i < TOTAL; i++) {
    const s = STEPS[i];
    if (s.t !== ct || s.hp !== chp || s.hl !== chl) { goTo(i); return; }
  }
  goTo(TOTAL - 1);
}

function prevTurn() {
  if (cur === 0) return;
  const ct = STEPS[cur].t, chp = STEPS[cur].hp, chl = STEPS[cur].hl;
  // Walk back to find a step in a different turn
  let i = cur - 1;
  while (i > 0 && STEPS[i].t === ct && STEPS[i].hp === chp && STEPS[i].hl === chl) i--;
  // Now find the start of that turn
  const nt = STEPS[i].t, nhp = STEPS[i].hp, nhl = STEPS[i].hl;
  while (i > 0 && STEPS[i-1].t === nt && STEPS[i-1].hp === nhp && STEPS[i-1].hl === nhl) i--;
  goTo(i);
}

function togglePlay() {
  playing = !playing;
  const btn = document.getElementById('play-btn');
  if (playing) {
    btn.textContent = '⏸ Pause';
    btn.classList.add('paused');
    const sp = +document.getElementById('speed').value;
    timer = setInterval(() => {
      if (cur >= TOTAL - 1) { togglePlay(); return; }
      goNext();
    }, sp);
  } else {
    btn.textContent = '▶ Play';
    btn.classList.remove('paused');
    clearInterval(timer);
    timer = null;
  }
}

function updateSpeed() {
  if (playing) { clearInterval(timer); timer = null; playing = false; togglePlay(); }
}

function buildLog() {
  const log = document.getElementById('log');
  log.innerHTML = STEPS.map((s, i) => {
    const hl = s.hl > 0 ? `H${s.hl}` : 'Pre';
    const t  = s.t > 0 ? `T${s.t}` : '';
    const tm = s.hp ? '🔵' : '🔴';
    return `<div class="log-entry" id="log-${i}" onclick="goTo(${i})">${i+1}. ${hl}${t}${tm} ${s.l}</div>`;
  }).join('');
}

document.addEventListener('keydown', e => {
  if (e.key === 'ArrowRight' || e.key === 'ArrowDown') { goNext(); e.preventDefault(); }
  else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') { goPrev(); e.preventDefault(); }
  else if (e.key === ' ') { togglePlay(); e.preventDefault(); }
  else if (e.key === 'Home') { goFirst(); e.preventDefault(); }
  else if (e.key === 'End')  { goLast();  e.preventDefault(); }
  else if (e.key === 'PageDown') { nextTurn(); e.preventDefault(); }
  else if (e.key === 'PageUp')   { prevTurn(); e.preventDefault(); }
});
"##;
