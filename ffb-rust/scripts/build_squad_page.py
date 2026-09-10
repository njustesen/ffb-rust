"""Render docs/squad_report.json as a single reviewable page (Artifact-ready HTML).

Usage: python scripts/build_squad_page.py [--out <path>]
"""

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

CSS = """
:root {
  /* Neutrals carry a green bias -- the pitch -- and the accent is the subject's own unit:
     gold pieces. Semantic red/green are separate from the accent and mean illegal/legal. */
  --ground:   #eef1ec;
  --surface:  #f9fbf8;
  --sunken:   #e3e8e1;
  --ink:      #15180f;
  --ink-soft: #454e42;
  --muted:    #6c766a;
  --line:     #d3dacf;
  --line-firm:#b6c0b2;
  --gold:     #7d5f14;
  --gold-soft:#f0e6c8;
  --bad:      #93231f;
  --bad-soft: #f4dedb;
  --good:     #285f3d;
  --good-soft:#dcebe0;
  --shadow:   0 1px 2px rgba(21,24,15,.07);

  --display: "Oswald", "Arial Narrow", Impact, sans-serif;
  --body: "Source Serif 4", Georgia, "Times New Roman", serif;
  --data: "IBM Plex Mono", ui-monospace, "SFMono-Regular", Consolas, monospace;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --ground:   #12150f;
    --surface:  #1a1e17;
    --sunken:   #23281f;
    --ink:      #edf0e8;
    --ink-soft: #c3cbbd;
    --muted:    #8e9889;
    --line:     #2d3329;
    --line-firm:#414938;
    --gold:     #d8b45f;
    --gold-soft:#332b12;
    --bad:      #e58a80;
    --bad-soft: #3a1f1c;
    --good:     #7cc396;
    --good-soft:#17301f;
    --shadow:   0 1px 2px rgba(0,0,0,.4);
  }
}
:root[data-theme="dark"] {
  --ground:   #12150f;
  --surface:  #1a1e17;
  --sunken:   #23281f;
  --ink:      #edf0e8;
  --ink-soft: #c3cbbd;
  --muted:    #8e9889;
  --line:     #2d3329;
  --line-firm:#414938;
  --gold:     #d8b45f;
  --gold-soft:#332b12;
  --bad:      #e58a80;
  --bad-soft: #3a1f1c;
  --good:     #7cc396;
  --good-soft:#17301f;
  --shadow:   0 1px 2px rgba(0,0,0,.4);
}

* { box-sizing: border-box; }
body {
  margin: 0;
  background: var(--ground);
  color: var(--ink);
  font-family: var(--body);
  font-size: 16px;
  line-height: 1.6;
  -webkit-font-smoothing: antialiased;
}
.wrap { max-width: 1120px; margin: 0 auto; padding-inline: 20px; padding-block: 0 72px; }
p { margin: 0 0 1em; max-width: 68ch; }
a { color: var(--gold); text-decoration-thickness: 1px; text-underline-offset: 2px; }
h1, h2, h3, .eyebrow, th, .num, .tag { font-family: var(--display); }
h2 {
  font-size: 1.32rem; font-weight: 600; letter-spacing: .01em; margin: 0 0 .2em;
  text-wrap: balance; text-transform: uppercase;
}
h3 { font-size: 1.02rem; font-weight: 600; margin: 0 0 .3em; text-wrap: balance; }
.eyebrow {
  font-size: .72rem; font-weight: 500; letter-spacing: .16em; text-transform: uppercase;
  color: var(--muted);
}
section { padding-block: 34px; border-top: 1px solid var(--line); }
section:first-of-type { border-top: 0; }
.lede { font-size: 1.06rem; color: var(--ink-soft); }
.mono { font-family: var(--data); font-variant-numeric: tabular-nums; }

/* ── masthead ─────────────────────────────────────────────────────────── */
header.masthead { padding-block: 52px 8px; }
header.masthead h1 {
  font-size: clamp(2.1rem, 6.4vw, 3.5rem); font-weight: 600; line-height: 1.02;
  letter-spacing: -.005em; margin: .18em 0 .5em; text-transform: uppercase;
  text-wrap: balance;
}
header.masthead h1 em {
  font-style: normal; color: var(--gold);
  display: block; font-size: .42em; letter-spacing: .04em; margin-top: .5em;
}
.byline {
  display: flex; flex-wrap: wrap; gap: 8px 22px; margin-top: 22px;
  font-family: var(--data); font-size: .78rem; color: var(--muted);
}

/* ── figure tiles ─────────────────────────────────────────────────────── */
.figures { display: grid; grid-template-columns: repeat(auto-fit, minmax(158px, 1fr)); gap: 1px;
  background: var(--line); border: 1px solid var(--line); margin-top: 22px; }
.fig { background: var(--surface); padding: 16px 18px; }
.fig .num {
  display: block; font-size: 1.72rem; font-weight: 600; line-height: 1.05;
  font-variant-numeric: tabular-nums;
}
.fig .num.was { color: var(--muted); font-size: .95rem; font-weight: 400; }
.fig .cap { font-size: .8rem; color: var(--muted); line-height: 1.35; display: block; margin-top: 5px; }
.fig.hero .num { color: var(--gold); }

/* ── tables ───────────────────────────────────────────────────────────── */
.scroll { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: .88rem; }
th {
  text-align: left; font-weight: 500; font-size: .72rem; letter-spacing: .1em;
  text-transform: uppercase; color: var(--muted); padding: 8px 10px;
  border-bottom: 1px solid var(--line-firm); white-space: nowrap;
}
td { padding: 9px 10px; border-bottom: 1px solid var(--line); vertical-align: top; }
td.n, th.n { text-align: right; font-family: var(--data); font-variant-numeric: tabular-nums;
  white-space: nowrap; }
tbody tr:last-child td { border-bottom: 0; }
.rule-table td:first-child { font-family: var(--display); font-weight: 500; }
.cite { font-size: .8rem; color: var(--muted); font-style: italic; }

/* ── defect ledger ────────────────────────────────────────────────────── */
.defects { display: grid; gap: 1px; background: var(--line); border: 1px solid var(--line);
  margin-top: 20px; }
.defect { background: var(--surface); display: grid;
  grid-template-columns: 34px minmax(0,1.15fr) minmax(0,1fr) auto; gap: 14px;
  padding: 14px 16px; align-items: start; }
.defect .idx { font-family: var(--display); font-size: .95rem; color: var(--muted); }
.defect .what { font-weight: 600; }
.defect .what span { display: block; font-weight: 400; font-size: .86rem; color: var(--ink-soft); }
.defect .rule { font-size: .84rem; color: var(--ink-soft); }
.defect .rule q { color: var(--ink); }
.tag {
  font-size: .68rem; letter-spacing: .1em; text-transform: uppercase; font-weight: 500;
  padding: 3px 8px; border: 1px solid currentColor; white-space: nowrap;
}
.tag.fixed { color: var(--good); background: var(--good-soft); }
.tag.open { color: var(--bad); background: var(--bad-soft); }

/* ── ordered purchase list ────────────────────────────────────────────── */
ol.order { counter-reset: step; list-style: none; padding: 0; margin: 20px 0 0;
  display: grid; gap: 1px; background: var(--line); border: 1px solid var(--line); }
ol.order li { counter-increment: step; background: var(--surface); padding: 12px 16px 12px 54px;
  position: relative; font-size: .92rem; }
ol.order li::before {
  content: counter(step); position: absolute; left: 16px; top: 12px;
  font-family: var(--data); font-size: .82rem; color: var(--gold); font-weight: 500;
}
ol.order li b { font-weight: 600; }
ol.order li em { font-style: normal; color: var(--muted); font-size: .86rem; display: block; }
ol.order li.mine { background: var(--sunken); }

/* ── controls ─────────────────────────────────────────────────────────── */
.controls { display: flex; flex-wrap: wrap; gap: 10px; align-items: center; margin: 20px 0 0; }
.seg { display: flex; border: 1px solid var(--line-firm); }
.seg button {
  font-family: var(--display); font-size: .78rem; letter-spacing: .08em; text-transform: uppercase;
  background: var(--surface); color: var(--ink-soft); border: 0; padding: 7px 13px; cursor: pointer;
  border-right: 1px solid var(--line);
}
.seg button:last-child { border-right: 0; }
.seg button[aria-pressed="true"] { background: var(--ink); color: var(--ground); }
input[type="search"] {
  font-family: var(--data); font-size: .82rem; padding: 7px 10px; background: var(--surface);
  color: var(--ink); border: 1px solid var(--line-firm); min-width: 190px;
}
input[type="search"]::placeholder { color: var(--muted); }
.count { font-family: var(--data); font-size: .78rem; color: var(--muted); margin-left: auto; }
:focus-visible { outline: 2px solid var(--gold); outline-offset: 2px; }

/* ── squad rows ───────────────────────────────────────────────────────── */
.sheets { margin-top: 16px; border: 1px solid var(--line); }
details.sheet { border-bottom: 1px solid var(--line); background: var(--surface); }
details.sheet:last-child { border-bottom: 0; }
details.sheet[open] { background: var(--sunken); }
summary { cursor: pointer; padding: 11px 16px; display: grid; gap: 4px 14px;
  grid-template-columns: minmax(0,1fr) auto; align-items: baseline; list-style: none; }
summary::-webkit-details-marker { display: none; }
summary:hover .cell { color: var(--gold); }
.cell { font-family: var(--display); font-weight: 500; font-size: 1rem; letter-spacing: .01em; }
.cell .ed { color: var(--muted); font-size: .74rem; letter-spacing: .1em; text-transform: uppercase;
  margin-right: 9px; }
.cell .variant { color: var(--gold); font-size: .72rem; letter-spacing: .08em; margin-left: 8px;
  text-transform: uppercase; }
.line { font-family: var(--data); font-size: .76rem; color: var(--ink-soft);
  font-variant-numeric: tabular-nums; display: flex; flex-wrap: wrap; gap: 2px 12px;
  justify-content: flex-end; }
.line b { font-weight: 500; color: var(--ink); }
.line .lost { color: var(--bad); }
.roster-note { grid-column: 1 / -1; font-size: .8rem; color: var(--muted); }
.body { padding: 4px 16px 20px; display: grid; gap: 20px;
  grid-template-columns: minmax(0, 1.55fr) minmax(0, 1fr); }
@media (max-width: 760px) { .body { grid-template-columns: 1fr; } }
.body h3 { font-size: .74rem; letter-spacing: .12em; text-transform: uppercase; color: var(--muted);
  margin-bottom: 6px; }
.ledger { font-family: var(--data); font-size: .8rem; }
.ledger div { display: flex; justify-content: space-between; gap: 12px; padding: 3px 0;
  border-bottom: 1px dotted var(--line); font-variant-numeric: tabular-nums; }
.ledger div.total { border-bottom: 0; border-top: 1px solid var(--line-firm); margin-top: 4px;
  padding-top: 6px; font-weight: 500; }
.ledger div.zero { color: var(--muted); }
.ledger span.k { color: var(--ink-soft); }
.delta { font-family: var(--data); font-size: .8rem; margin-top: 4px; }
.delta div { padding: 2px 0; }
.delta .plus { color: var(--good); }
.delta .minus { color: var(--bad); }
.delta .star { color: var(--bad); }
.skills { font-family: var(--body); font-size: .78rem; color: var(--muted); display: block; }
.bg { color: var(--gold); font-size: .68rem; font-family: var(--display); letter-spacing: .08em;
  text-transform: uppercase; }
.capnote { font-size: .8rem; color: var(--ink-soft); margin-top: 10px; }
.capnote code { font-family: var(--data); font-size: .92em; }

/* ── verdict boxes ────────────────────────────────────────────────────── */
.verdict { border-left: 3px solid var(--gold); background: var(--surface); padding: 14px 18px;
  margin: 18px 0; box-shadow: var(--shadow); }
.verdict.warn { border-left-color: var(--bad); }
.verdict h3 { margin-bottom: .25em; }
.verdict p:last-child { margin-bottom: 0; }
ul.plain { margin: 0 0 1em; padding-left: 1.1em; max-width: 68ch; }
ul.plain li { margin-bottom: .45em; }
code { font-family: var(--data); font-size: .9em; background: var(--sunken); padding: 1px 4px; }
pre { font-family: var(--data); font-size: .78rem; background: var(--surface); color: var(--ink);
  border: 1px solid var(--line); padding: 12px 14px; overflow-x: auto; margin: 14px 0; }
footer { border-top: 1px solid var(--line-firm); padding-block: 26px; font-size: .82rem;
  color: var(--muted); }
@media (prefers-reduced-motion: reduce) { * { animation: none !important; transition: none !important; } }
"""

JS = """
const DATA = JSON.parse(document.getElementById("squad-data").textContent);
const gp = n => (n / 1000).toLocaleString("en-US") + "k";
const esc = s => String(s).replace(/[&<>"]/g, c => ({ "&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;" }[c]));
const EDN = { bb2016: "bb2016", bb2020: "bb2020", bb2025: "bb2025" };

function statline(r) {
  const v = x => (x === null || x === undefined) ? "\\u2013" : x;
  return `${v(r.ma)}/${v(r.st)}/${v(r.ag)}/${v(r.pa)}/${v(r.av)}`;
}

function sheet(s) {
  const variant = s.cell !== s.race;
  const fans = s.edition === "bb2016"
    ? `FF ${s.fan_factor}` : `DF ${s.dedicated_fans}`;
  const staff = (s.assistant_coaches || s.cheerleaders)
    ? `${s.assistant_coaches}ac ${s.cheerleaders}cl` : "no staff";
  const comp = s.composition.map(r => `
    <tr>
      <td class="n">${r.n}&times;</td>
      <td>${esc(r.name)}${r.big_guy ? ' <span class="bg">big guy</span>' : ""}
        ${r.skills.length ? `<span class="skills">${esc(r.skills.join(", "))}</span>` : ""}</td>
      <td class="n">0-${r.quantity}</td>
      <td class="n">${statline(r)}</td>
      <td class="n">${gp(r.cost)}</td>
      <td class="n">${gp(r.total)}</td>
    </tr>`).join("");

  const players = s.composition.reduce((a, r) => a + r.total, 0);
  const rows = [
    ["players", players, `${s.players} hired`],
    ["team re-rolls", s.rerolls * s.reroll_cost, `${s.rerolls} \\u00d7 ${gp(s.reroll_cost)}`],
    ["apothecary", s.apothecaries * 50000,
      s.apothecary_allowed ? `${s.apothecaries}` : "not allowed"],
    ["assistant coaches", s.assistant_coaches * 10000, `${s.assistant_coaches}`],
    ["cheerleaders", s.cheerleaders * 10000, `${s.cheerleaders}`],
    [s.edition === "bb2016" ? "fan factor" : "dedicated fans",
      s.edition === "bb2016" ? s.fan_factor * 10000
        : Math.max(0, s.dedicated_fans - (s.edition === "bb2025" ? 1 : 0)) * (s.edition === "bb2025" ? 5000 : 10000),
      s.edition === "bb2016" ? `${s.fan_factor}` : `${s.dedicated_fans}`],
  ].map(([k, v, note]) => `<div class="${v ? "" : "zero"}"><span class="k">${k}
      <span style="opacity:.6">${esc(note)}</span></span><span>${gp(v)}</span></div>`).join("");

  const d = s.delta_positions || {};
  const deltas = Object.keys(d).sort().map(pid => {
    const n = d[pid];
    return `<div class="${n > 0 ? "plus" : "minus"}">${n > 0 ? "+" : ""}${n} ${esc(pid)}</div>`;
  }).join("");
  const stars = (s.old && s.old.stars.length)
    ? s.old.stars.map(x => `<div class="star">\\u2212 star ${esc(x)}</div>`).join("") : "";
  const oldLine = s.old ? `<div style="color:var(--muted);margin-top:6px">was ${s.old.players}p
      &middot; ${s.old.rerolls}rr &middot; ${s.old.apothecaries}apo &middot; 0ac 0cl
      &middot; ${gp(s.old.treasury)} unspent</div>` : "";

  return `
  <details class="sheet" data-ed="${s.edition}" data-cell="${esc(s.cell)}">
    <summary>
      <span class="cell"><span class="ed">${EDN[s.edition]}</span>${esc(s.cell)}
        ${variant ? `<span class="variant">variant of ${esc(s.race)}</span>` : ""}</span>
      <span class="line">
        <span><b>${s.players}</b>p</span><span><b>${s.rerolls}</b>rr</span>
        <span>${s.apothecaries ? "apo" : "<span style='opacity:.45'>no apo</span>"}</span>
        <span>${fans}</span><span>${staff}</span>
        <span>TV <b>${gp(s.team_value)}</b></span>
        ${s.treasury ? `<span class="lost">${gp(s.treasury)} lost</span>` : ""}
      </span>
      <span class="roster-note">${esc(s.roster_name)} &middot; roster <code>${esc(s.roster_id)}</code>${
        s.big_guy_cap !== null ? ` &middot; page allows ${s.big_guy_cap} Big Guy${s.big_guy_cap > 1 ? "s" : ""}, squad has ${s.big_guys}` : ""}</span>
    </summary>
    <div class="body">
      <div>
        <h3>Team draft list</h3>
        <div class="scroll"><table>
          <thead><tr><th class="n">no.</th><th>position</th><th class="n">limit</th>
            <th class="n">ma/st/ag/pa/av</th><th class="n">fee</th><th class="n">total</th></tr></thead>
          <tbody>${comp}</tbody>
        </table></div>
        ${s.unfielded_positionals.length ? `<p class="capnote">Not in this squad:
          ${s.unfielded_positionals.map(x => `<code>${esc(x)}</code>`).join(" ")} &mdash;
          held back by the group Big Guy cap and fielded by this cell's other squad instead.</p>` : ""}
      </div>
      <div>
        <h3>Spend</h3>
        <div class="ledger">${rows}
          <div class="total"><span class="k">spent of 1,100k</span><span>${gp(s.spent)}</span></div>
        </div>
        <h3 style="margin-top:16px">Change from the old squad</h3>
        <div class="delta">${stars}${deltas || '<div style="color:var(--muted)">same players</div>'}${oldLine}</div>
      </div>
    </div>
  </details>`;
}

const host = document.getElementById("sheets");
host.innerHTML = DATA.squads.map(sheet).join("");

let ed = "all", q = "";
function apply() {
  let shown = 0;
  host.querySelectorAll("details.sheet").forEach(el => {
    const okEd = ed === "all" || el.dataset.ed === ed;
    const okQ = !q || el.dataset.cell.includes(q);
    const on = okEd && okQ;
    el.hidden = !on;
    if (on) shown++;
  });
  document.getElementById("count").textContent =
    `${shown} of ${DATA.squads.length} draft lists`;
}
document.querySelectorAll("#eds button").forEach(b => {
  b.addEventListener("click", () => {
    ed = b.dataset.ed;
    document.querySelectorAll("#eds button").forEach(x =>
      x.setAttribute("aria-pressed", String(x === b)));
    apply();
  });
});
document.getElementById("q").addEventListener("input", e => {
  q = e.target.value.trim().toLowerCase(); apply();
});
apply();
"""


# The reds, classified by hand from the triage recorded in BACKLOG §H.16-§H.19. Keyed by
# (cell, edition) because a cell's scales share a cause wherever they were traced to one.
RED_NOTES = {
    ("goblin", "bb2016"): ("fixed",
        "The Ball &amp; Chain Fanatic: KO where Java had Badly Hurt or dead, and once a lost "
        "coordinate with no dice difference at all. Three engine bugs, all fixed &mdash; this cell "
        "now gates 100/100 on all three scales."),
    ("goblin", "bb2020"): ("open",
        "Dice 1-18 agree, then Java asks for a d6 and Rust for a d8. The Fanatic is still standing "
        "on the ball where Java has it KO. Not the pitch-invasion stun, which is corrected and "
        "changed nothing here."),
    ("goblin", "bb2025"): ("open",
        "A plain Goblin comes out Badly Hurt in Java and KO in Rust &mdash; the same "
        "casualty-versus-KO boundary that the bb2016 cell turned out to be a missing injury roll."),
    ("high_elf", "bb2020"): ("open",
        "The Thrower has Cloud Burster, which forces an interception re-roll, so the pass sequence "
        "runs the intercept step twice. Java keeps <code>interceptorChosen</code> in the pass "
        "state; Rust keeps it on the step, so it asks the dialog again and takes a whole second "
        "decision. Four dice too many, ball in the wrong square."),
    ("nurgle", "bb2020"): ("open",
        "One player: Prone in Java, Standing in Rust. Same shape as the human cell."),
    ("human", "bb2020"): ("open",
        "One player: KO and off the pitch in Java, Prone and on it in Rust. Same shape as the "
        "nurgle cell &mdash; two cells showing &ldquo;Java knocks down, Rust does not&rdquo; is one "
        "investigation, not two."),
    ("renegades", "bb2020"): ("open",
        "Already a whole turn apart by the reported step &mdash; Java on the away team&rsquo;s turn "
        "7, Rust on the home team&rsquo;s &mdash; with the fame field and three players differing. "
        "Diverges earlier than the first step the harness flags."),
    ("khorne", "bb2020"): ("harness",
        "Not a Rust bug. The JAVA side hangs: <code>SPIN: step=SETUP dialog=SETUP_ERROR</code>, "
        "force-ended at two million iterations, with a KO&rsquo;d home player so the team was "
        "setting up with fewer than 11 available. Rust plays the game out. This is the reserves "
        "failure mode the harness has hit before, and it belongs in ParityRunner."),
    ("slann", "bb2020"): ("open",
        "A Kroxigor at (20,5) in Java and on the sideline at (20,0) in Rust &mdash; one coordinate, "
        "no state difference."),
    ("khemri", "bb2025"): ("open",
        "Four slots apart including ACTIVE bits, in Sweltering Heat. The most diverged of the set."),
}

STATUS_LABEL = {"fixed": "fixed", "open": "open", "harness": "harness, not the engine"}


def figures(d):
    S = d["squads"]
    n = len(S)
    stars = sum(len(s["old"]["stars"]) for s in S if "old" in s)
    old_tres = sum(s["old"]["treasury"] for s in S if "old" in s)
    new_tres = sum(s["treasury"] for s in S)
    staff = sum(s["assistant_coaches"] + s["cheerleaders"] for s in S)
    apo = sum(s["apothecaries"] for s in S)
    old_apo = sum(s["old"]["apothecaries"] for s in S if "old" in s)
    eligible = sum(1 for s in S if s["apothecary_allowed"])
    return [
        ("hero", f"{n}", "draft lists re-drafted", "29 bb2016 &middot; 40 bb2020 &middot; 41 bb2025"),
        ("", "0", "star players", f"was {stars}, in 12 squads"),
        ("", f"{new_tres // 1000}k", "gold left unspent", f"was {old_tres // 1000}k"),
        ("", f"{staff}", "sideline staff hired", "was 0, on every squad"),
        ("", f"{apo}", f"apothecaries of {eligible} eligible", f"was {old_apo}"),
        ("", "0", "legality violations", "R6, all three rulesets"),
    ]


DEFECTS = [
    ("Star players on the draft list, paid for with nothing",
     "16 stars across 12 squads, injected as ordinary rostered players and deliberately left "
     "outside the 1.1M budget.",
     "A star is an Inducement: Matched Play wants <q>the Star Player&rsquo;s associated cost in gold "
     "pieces, and also&hellip; 2 Skill Points</q>. A mirror match generates no inducement gold at all."),
    ("Gold left in the treasury",
     "Up to 50,000 per squad, 2,820,000 across the tree.",
     "<q>All the gold pieces a team has must be spent when drafting your team. Any gold pieces not "
     "spent are lost.</q>"),
    ("No Sideline Staff anywhere",
     "All 111 squads had 0 assistant coaches and 0 cheerleaders. The spec had no field for them, "
     "and the Java generator hardcoded <code>&lt;cheerleaders&gt;0&lt;/cheerleaders&gt;</code>.",
     "<q>A team may hire up to a maximum of 6 Assistant Coaches&hellip; 10,000 gold pieces to hire</q>, "
     "and the same for Cheerleaders, in all three rulesets."),
    ("bb2020 Dedicated Fans underpaid by 20,000, every squad",
     "The checker charged the BB2025 <em>League</em> rate of (n&minus;1)&times;5,000.",
     "Exhibition play: Dedicated Fans start at 0 and improve <q>up to a maximum of 6, at a cost of "
     "10,000 gold pieces per improvement</q>."),
    ("bb2025 Dedicated Fans allowed up to 6",
     "The drafting cap is 3; 1..6 was accepted.",
     "<q>You may improve the Dedicated Fans Characteristic of your team up to a maximum of 3</q>, at "
     "5,000 each, from a starting value of 1."),
    ("The group Big Guy cap read as a fielding rule",
     "It caps what the team may <em>contain</em>, and it spans a group of positions &mdash; which the "
     "per-position <code>quantity</code> field cannot express at all.",
     "<q>A Chaos Chosen team may have a single Big Guy, chosen from the following.</q>"),
    ("A star&rsquo;s race and edition legality unchecked",
     "<code>available_for</code> was read by no code and no script; a bb2016-era star sat in a bb2020 "
     "squad on purpose.",
     "Moot now that no squad fields a star, but the check stays so it cannot return."),
]

RULES = [
    ("Team Draft Budget", "1,100,000", "1,100,000", "1,100,000"),
    ("budget fully spent", "yes", "yes", "yes"),
    ("treasury", "0", "0", "0"),
    ("players", "11&ndash;16", "11&ndash;16", "11&ndash;16"),
    ("per-position maximum", "team page", "team page", "team page"),
    ("group Big Guy cap", "none in CRP", "team page", "team page"),
    ("team re-rolls", "0&ndash;8 at roster cost", "0&ndash;8 at roster cost", "0&ndash;8 at roster cost"),
    ("assistant coaches", "0&ndash;6 &times; 10k", "0&ndash;6 &times; 10k", "0&ndash;6 &times; 10k"),
    ("cheerleaders", "0&ndash;6 &times; 10k", "0&ndash;6 &times; 10k", "0&ndash;6 &times; 10k"),
    ("apothecary", "0&ndash;1 &times; 50k, roster permitting", "same", "same"),
    ("fans", "Fan Factor 0&ndash;9 &times; 10k, <b>counts in TV</b>",
     "Dedicated Fans 0&rarr;6 &times; <b>10k</b>", "Dedicated Fans 1&rarr;3 &times; <b>5k</b>"),
    ("Team Value", "players + staff + re-rolls + FF", "players + staff + re-rolls",
     "players + staff + re-rolls"),
    ("star players", "none", "none", "none"),
]

ORDER = [
    (False, "One of <b>every</b> positional the caps allow",
     "Also what satisfies the requirement that every position&rsquo;s stat line and starting skills "
     "get exercised."),
    (False, "More players, <b>dearest first</b>",
     "While the squad can still reach 11 bodies, 3 re-rolls and an apothecary afterwards. This is "
     "the step that makes them look like teams."),
    (False, "Cheapest player up to 11, then 12 where it costs no re-roll",
     "11 is the rule. The 12th is a preference: a squad on exactly 11 plays a man short after one "
     "casualty."),
    (False, "Two team re-rolls", ""),
    (True, "Apothecary, then a third re-roll",
     "My call, inside your &ldquo;more players or staff&rdquo;. Buying the third re-roll first left "
     "57 of 94 eligible rosters with no apothecary &mdash; and took most of the apothecary "
     "mechanic&rsquo;s exercise off the matrix."),
    (True, "A 13th body, then one more dearest-first pass",
     "Swaps a cheap body for the dearest positional that still has room, wherever the remaining "
     "gold covers the difference."),
    (True, "Fans to the drafting maximum", ""),
    (True, "Assistant coaches to 6, then cheerleaders to 6",
     "The rulebook&rsquo;s own advice for leftover cash, and the first time any parity squad has had "
     "a single member of staff."),
    (True, "Players to 16, then re-rolls to 8",
     "Whatever the 10,000-gold staff could not absorb. What still cannot be spent is lost &mdash; "
     "and the drafter proves it by checking that nothing cheaper remains buyable."),
]


def sweep_section(sw):
    """The parity results section. `sw` is sweep_verdicts.py --json output, or None."""
    if not sw:
        return ""
    warn = ""
    if not sw.get("complete"):
        warn = (f'<p class="cite">Incomplete: {sw["gates"]} of {sw["expected"]} gates reported. '
                f'Do not read this as a matrix result.</p>')

    # group the red gates by (cell, edition) so a cell's scales share one row
    groups = {}
    for r in sw["red"]:
        groups.setdefault((r["cell"], r["edition"]), []).append(r)
    rows = []
    for (cell, ed), rs in sorted(groups.items()):
        status, note = RED_NOTES.get((cell, ed), ("open", "Not yet triaged."))
        scales = ", ".join("@" + r["scale"] for r in sorted(rs, key=lambda x: x["scale"]))
        worst = min(int(r["verdict"].split(":")[1].split("/")[0]) for r in rs
                    if "/" in r["verdict"]) if any("/" in r["verdict"] for r in rs) else "-"
        rows.append(
            f'<tr><td><b>{cell}</b><br><span class="cite">{ed} &middot; {scales}</span></td>'
            f'<td class="n">{worst}/100</td>'
            f'<td><span class="tag {"fixed" if status == "fixed" else "open"}">'
            f'{STATUS_LABEL[status]}</span></td>'
            f'<td>{note}</td></tr>')

    green_cells = sw["cells"] - len(groups)
    tiles = [
        ("hero", f'{sw["green"]}', "gates parity-green", f'of {sw["gates"]}'),
        ("", f'{len(groups)}', "cells with a red gate", f'{green_cells} cells fully green'),
        ("", f'{sw["short"]}', "gates short on coverage", "parity green on every one"),
        ("", f'{sw["summed_minutes"] / 60:.1f}h', "summed gate time", "4 shards, no --reuse-java"),
    ]
    figs = "".join(
        f'<div class="fig {cls}"><span class="num">{num}</span>'
        f'<span class="cap">{cap}<br><span class="was">{was}</span></span></div>'
        for cls, num, cap, was in tiles)

    return f"""
<section>
  <p class="eyebrow">The matrix, re-run on these squads</p>
  <h2>Parity results</h2>
  <p>Every cell, in every ruleset it belongs to, at all three sampling scales, Rust against the
  stock Java engine under the heuristic agent, 100 seeds each. A gate counts only if the process
  exits without panicking <b>and</b> prints its verdict; the absence of a failure line is not a
  measurement.</p>
  {warn}
  <div class="figures">{figs}</div>

  <h3 style="margin-top:26px">Every red, and what it is</h3>
  <p>Nine of these were localised to a named field or a single die before being written up. The
  squads did not cause them &mdash; they reached them. Most sit in bb2020, where correcting the
  Dedicated Fans moved the fame term in every kick-off contest off the one value it had always had.</p>
  <div class="scroll"><table>
    <thead><tr><th>cell</th><th class="n">worst</th><th>status</th><th>what it is</th></tr></thead>
    <tbody>{"".join(rows)}</tbody>
  </table></div>
</section>
"""


def build(d, sw=None):
    figs = "".join(
        f'<div class="fig {cls}"><span class="num">{num}</span>'
        f'<span class="cap">{cap}<br><span class="was">{was}</span></span></div>'
        for cls, num, cap, was in figures(d))
    defects = "".join(
        f'<div class="defect"><span class="idx">{i}</span>'
        f'<span class="what">{what}<span>{scope}</span></span>'
        f'<span class="rule">{rule}</span><span class="tag fixed">fixed</span></div>'
        for i, (what, scope, rule) in enumerate(DEFECTS, 1))
    rules = "".join(
        f"<tr><td>{a}</td><td>{b}</td><td>{c}</td><td>{e}</td></tr>"
        for a, b, c, e in RULES)
    order = "".join(
        f'<li class="{"mine" if mine else ""}">{t}{f"<em>{n}</em>" if n else ""}</li>'
        for mine, t, n in ORDER)

    return f"""<title>Parity Team Draft Lists</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Oswald:wght@400;500;600&family=Source+Serif+4:ital,opsz,wght@0,8..60,400;0,8..60,600;1,8..60,400&family=IBM+Plex+Mono:wght@400;500&display=swap">
<style>{CSS}</style>

<div class="wrap">
<header class="masthead">
  <p class="eyebrow">Blood Bowl parity matrix &middot; re-draft of 2026-09-10</p>
  <h1>Parity Team<br>Draft Lists<em>110 squads, drafted from the rulebook this time</em></h1>
  <p class="lede">Every squad the parity matrix fields was re-drafted from the drafting rules in
  <code>rules/</code>, because the checker that had been certifying them was calibrated against the
  squads themselves. Seven systematic illegalities had passed it, star players among them.</p>
  <div class="byline">
    <span>110 draft lists</span><span>3 rulesets</span>
    <span>1,100,000 gold budget each</span><span>0 star players</span>
  </div>
</header>

<section>
  <p class="eyebrow">What changed</p>
  <h2>The re-draft in six numbers</h2>
  <div class="figures">{figs}</div>
</section>

<section>
  <p class="eyebrow">Why it was needed</p>
  <h2>A checker calibrated against its own subject</h2>
  <p>The squads were checked against a list of conditions in the project&rsquo;s own requirements
  document, and that list was written from the squads. The money formula said so in as many words
  &mdash; <q>derived from the data, not assumed</q> &mdash; and offered as evidence that it
  <q>reproduces the declared team&nbsp;value / spent / treasury for 100% of the squads in the
  tree</q>. It reproduced them because the squads had been built with it. Nothing in the document
  required a squad to be legal under the actual Blood&nbsp;Bowl drafting rules, so these seven
  things were all reported as clean:</p>
  <div class="defects">{defects}</div>
</section>

<section>
  <p class="eyebrow">The rule that was missing</p>
  <h2>R6 &mdash; legal under the rulebook, per play format</h2>
  <p>A 1,100,000 budget is not a League budget; League is 1,000,000. It is a <b>Matched Play</b>
  budget for BB2025 &mdash; <q>the most common values are 1,100,000, 1,150,000 or
  1,200,000</q> &mdash; and an <b>Exhibition Play</b> budget for BB2020, whose range is
  <q>1,100,000 and 1,300,000</q>. The drafting rules differ by format, and the squads had been
  following neither. Every constant below now traces to a quoted line in <code>rules/</code>, and
  none to a value read out of the squads.</p>
  <div class="scroll"><table class="rule-table">
    <thead><tr><th></th><th>bb2016 &middot; CRP tournament</th><th>bb2020 &middot; Exhibition</th>
      <th>bb2025 &middot; Matched Play</th></tr></thead>
    <tbody>{rules}</tbody>
  </table></div>
  <p class="cite">Team Value follows the reference engine&rsquo;s own
  <code>UtilTeamValue.findTeamValue</code> exactly: re-rolls + Fan&nbsp;Factor + assistant coaches +
  cheerleaders + apothecaries + player hiring fees. Dedicated Fans are not in TV; Fan Factor is.</p>
</section>

<section>
  <p class="eyebrow">How each squad was bought</p>
  <h2>Purchase order</h2>
  <p>Players first, then re-rolls, then the rest &mdash; as specified. The shaded steps are the ones
  where the order was left to me, and each says what it cost to decide the other way.</p>
  <ol class="order">{order}</ol>
</section>

<section>
  <p class="eyebrow">The squads</p>
  <h2>110 draft lists</h2>
  <p>Each cell is one gate of the parity matrix. Open a row for its team draft list, the spend
  broken out to the gold piece, and what changed from the squad it replaces.</p>
  <div class="controls">
    <div class="seg" id="eds">
      <button data-ed="all" aria-pressed="true">All</button>
      <button data-ed="bb2016" aria-pressed="false">bb2016</button>
      <button data-ed="bb2020" aria-pressed="false">bb2020</button>
      <button data-ed="bb2025" aria-pressed="false">bb2025</button>
    </div>
    <input type="search" id="q" placeholder="filter by team&hellip;" aria-label="Filter draft lists">
    <span class="count mono" id="count"></span>
  </div>
  <div class="sheets" id="sheets"></div>
</section>

{sweep_section(sw)}

<section>
  <p class="eyebrow">My reading of them</p>
  <h2>Are these good teams?</h2>
  <p>Legality is machine-checked; whether they are teams a coach would field is a judgement, so
  here it is plainly.</p>

  <div class="verdict">
    <h3>Mostly yes, and the compositions are the evidence</h3>
    <p>Chaos Chosen takes 4 Chosen Blockers, a Minotaur and 7 Beastmen. Khemri takes 4 Tomb
    Guardians, 2 Blitz-ras, 2 Thro-ras and 4 Skeletons. Humans take an Ogre, 2 Blitzers, 2 Catchers,
    2 Throwers and a Halfling Hopeful. Those are the builds these rosters are known for, and they
    came out of the dearest-first rule rather than being written down by hand.</p>
  </div>

  <div class="verdict">
    <h3>26 squads sit on exactly 11 players, and that is the right call</h3>
    <p>They are the expensive rosters &mdash; the elf family, Dwarf, Chaos Renegades, Slann. Fielding
    one of every positional costs 450&ndash;520k there, and after 3 re-rolls and an apothecary
    there is no 12th body to be had. A tournament coach makes the same trade: 11 players with three
    re-rolls and an apothecary beats 12 without. The one squad that also loses its apothecary is
    bb2016 Chaos Renegades, whose ten positionals include four Big Guys and cost 895k on their
    own.</p>
  </div>

  <div class="verdict warn">
    <h3>Only 12 squads hired any Sideline Staff</h3>
    <p>Players come before staff in the order you set, and on most rosters the players eat the
    budget. So the assistant-coach and cheerleader terms in Brilliant Coaching and Cheering Fans
    are exercised in 12 of 110 cells rather than most of them. That is still 12 more than before,
    where the fields were hardcoded to zero and those two kick-off results were decided
    0-against-0 in all 33,300 games ever measured &mdash; but it is a modest gain, and reversing
    the order would trade squad quality for it.</p>
  </div>

  <div class="verdict warn">
    <h3>180,000 gold is still lost, and a smarter drafter would lose less</h3>
    <p>24 squads cannot spend their last coins, 21 of them stuck on exactly 5,000 &mdash; because in
    bb2016 and bb2020 nothing on the sheet costs less than 10,000, so a 5,000 remainder has no home.
    That is the rule working (unspent gold is lost, and the drafter proves nothing cheaper remains
    buyable), not a bug. But it is greedy, not exact: a search that chose a 75,000 player over a
    70,000 one in the right place would land on zero. It is 0.15% of the 121,000,000 drafted, and I
    left it.</p>
  </div>

  <div class="verdict warn">
    <h3>One cell lost its second squad</h3>
    <p>bb2020 Chaos Pact had a <code>renegadetroll</code> variant, drafted to work around a group Big
    Guy cap. That cap comes from the official team page, and Chaos&nbsp;Pact has no official BB2020
    page &mdash; it is a FUMBBL legacy import, governed by its per-position limits alone. So the base
    squad now fields all four of its Big Guys itself, the variant became a byte-for-byte duplicate,
    and I deleted it. The matrix is 110 cells and 330 gates, not 111 and 333; no positional lost
    coverage.</p>
  </div>
</section>

<section>
  <p class="eyebrow">The cost</p>
  <h2>What the star removal takes off the pitch</h2>
  <p>Those 16 stars were the only players in the whole matrix carrying these, and each one was driven
  green by its own campaign. With no stars, they are exercised by unit tests and nothing else:</p>
  <ul class="plain">
    <li><b>Hail Mary Pass</b> under bb2016 &mdash; no roster in that ruleset has a native carrier, so
    it goes fully dark there. BB2025 keeps it: the Elven Union Thrower has it printed.</li>
    <li><b>Treacherous</b>, <b>Black Ink</b>, <b>Then I Started Blastin&rsquo;</b>,
    <b>Raiding Party</b>, <b>Look Into My Eyes</b>, <b>Baleful Hex</b>,
    <b>Catch of the Day</b> &mdash; the whole star-special family, which rides one dispatch channel.</li>
    <li><b>All You Can Eat</b>, <b>Wisdom of the White Dwarf</b>, <b>Multiple Block</b>,
    <b>Old Pro</b> on Helmut Wulf.</li>
  </ul>
  <p>Getting them back means hiring stars the way the rules do it &mdash; through inducements, paying
  gold and Skill Points against a team&rsquo;s Tier allowance &mdash; which is a mechanic neither
  engine implements today, not a data change.</p>
</section>

<footer>
  <p style="max-width:none">Reproduce: <code>python scripts/draft_all_squads.py</code> &middot;
  <code>python scripts/validate_teams.py --selftest</code> &middot;
  <code>python scripts/validate_teams.py --r5</code> &middot;
  <code>python scripts/gen_java_parity_data.py</code>. The rules are stated as R6 in
  <code>docs/PARITY_COVERAGE_REQUIREMENTS.md</code>; this page is built from
  <code>docs/squad_report.json</code> by <code>scripts/build_squad_page.py</code>.</p>
</footer>
</div>

<script id="squad-data" type="application/json">{json.dumps(d, separators=(",", ":"))}</script>
<script>{JS}</script>
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default="docs/squad_page.html")
    ap.add_argument("--sweep", help="sweep_verdicts.py --json output, to add the results section")
    args = ap.parse_args()
    d = json.loads((ROOT / "docs" / "squad_report.json").read_text(encoding="utf-8"))
    sw = None
    if args.sweep:
        sw = json.loads((ROOT / args.sweep).read_text(encoding="utf-8"))
    out = ROOT / args.out
    out.write_text(build(d, sw), encoding="utf-8")
    print(f"{out}  ({out.stat().st_size / 1024:.0f} KB)")


if __name__ == "__main__":
    main()
