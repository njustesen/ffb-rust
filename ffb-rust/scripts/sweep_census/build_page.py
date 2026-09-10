#!/usr/bin/env python
"""Assemble the event-census artifact page from the aggregated sweep data."""
import json
import glob
from collections import Counter, defaultdict

gates = [json.load(open(f)) for f in glob.glob('out/*.json')]
cells = json.load(open('cells.json'))
sk = json.load(open('skill_class.json'))
emit = json.load(open('emit.json'))
fielded = json.load(open('fielded.json'))

T = Counter(); A = Counter(); K = Counter(); SI = Counter()
KED = defaultdict(Counter); WX = Counter(); PR = Counter()
RRr = Counter(); RRt = Counter()
for d in gates:
    T.update(d['types']); A.update(d['actions']); K.update(d['kickoff'])
    SI.update(d['serious_injury']); WX.update(d.get('weather', {})); PR.update(d['prayers'])
    KED[d['edition']].update(d['kickoff'])
    for k, v in d['rolls'].items():
        RRt[k] += v['total']; RRr[k] += v['rerolled']
    RRt['blockRoll'] += d['types'].get('blockRoll', 0); RRr['blockRoll'] += d['block_rerolled']

# pass-2 special action windows
tot2 = Counter(); emp2 = Counter(); win2 = defaultdict(Counter)
for f in glob.glob('out2/*.json'):
    d = json.load(open(f))
    tot2.update(d['total']); emp2.update(d['empty'])
    for k, v in d['windows'].items():
        win2[k].update(v)
BASIC = {"Move", "Block", "Blitz", "BlitzMove", "BlitzSelect", "Foul", "FoulMove", "Pass",
         "PassMove", "HandOver", "HandOverMove", "StandUp"}

n = lambda v: '{:,}'.format(v)


def bars(rows, keyfn=lambda r: r[1]):
    m = max([keyfn(r) for r in rows] + [1])
    return m


def esc(s):
    return str(s).replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')


P = []
o = P.append

o('''<title>Full-Matrix Event Census</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter+Tight:wght@400;500;600;680&family=IBM+Plex+Mono:wght@400;500;600&display=swap">
<style>
  :root{
    --bg:#ecefe9; --panel:#ffffff; --panel2:#f6f8f3; --ink:#15211a; --muted:#5a655e;
    --border:#dbe0d8; --accent:#1f7a4d; --accent-ink:#125334; --accent-soft:#d8ecdf;
    --amber:#8f6410; --amber-soft:#f0e4c8; --clay:#9c3320; --clay-soft:#f3ddd7;
    --track:#e6e9e2; --shadow:rgba(20,40,28,.06);
  }
  @media (prefers-color-scheme: dark){
    :root:not([data-theme="light"]){
      --bg:#0e130f; --panel:#161d17; --panel2:#1b231c; --ink:#e7ede7; --muted:#95a29a;
      --border:#28311f; --accent:#4cbd7c; --accent-ink:#a6e3c0; --accent-soft:#173021;
      --amber:#d5a556; --amber-soft:#2c2413; --clay:#e08b74; --clay-soft:#33190f;
      --track:#222a22; --shadow:rgba(0,0,0,.3);
    }
  }
  :root[data-theme="dark"]{
      --bg:#0e130f; --panel:#161d17; --panel2:#1b231c; --ink:#e7ede7; --muted:#95a29a;
      --border:#28311f; --accent:#4cbd7c; --accent-ink:#a6e3c0; --accent-soft:#173021;
      --amber:#d5a556; --amber-soft:#2c2413; --clay:#e08b74; --clay-soft:#33190f;
      --track:#222a22; --shadow:rgba(0,0,0,.3);
  }
  *{box-sizing:border-box}
  html{-webkit-text-size-adjust:100%}
  body{margin:0; background:var(--bg); color:var(--ink);
    font-family:"Inter Tight",system-ui,-apple-system,"Segoe UI",Roboto,Helvetica,Arial,sans-serif;
    line-height:1.5; letter-spacing:-.005em}
  .mono{font-family:"IBM Plex Mono",ui-monospace,Menlo,Consolas,monospace}
  .wrap{max-width:1120px; margin:0 auto; padding:clamp(20px,4vw,52px) clamp(16px,4vw,40px) 80px}
  header.mast{border-bottom:2px solid var(--ink); padding-bottom:22px; margin-bottom:30px}
  .eyebrow{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:12px; letter-spacing:.18em;
    text-transform:uppercase; color:var(--accent-ink); display:flex; align-items:center; gap:10px; margin-bottom:14px}
  .eyebrow::before{content:""; width:26px; height:2px; background:var(--accent)}
  h1{font-size:clamp(28px,5vw,44px); line-height:1.04; margin:0 0 10px; font-weight:680;
    text-wrap:balance; letter-spacing:-.02em}
  .sub{color:var(--muted); font-size:16px; max-width:68ch; margin:0}
  code{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:.86em; background:var(--panel2);
    padding:1px 6px; border-radius:5px; border:1px solid var(--border); color:var(--ink)}
  .stats{display:grid; grid-template-columns:repeat(auto-fit,minmax(140px,1fr)); gap:1px; background:var(--border);
    border:1px solid var(--border); border-radius:12px; overflow:hidden; margin:26px 0 8px; box-shadow:0 1px 0 var(--shadow)}
  .stat{background:var(--panel); padding:16px 18px; display:flex; flex-direction:column; gap:3px}
  .stat .k{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:11px; letter-spacing:.12em;
    text-transform:uppercase; color:var(--muted)}
  .stat .v{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:25px; font-weight:600;
    font-variant-numeric:tabular-nums; letter-spacing:-.01em}
  .stat.hero .v{color:var(--accent)}
  .stat.warn .v{color:var(--clay)}
  .stat .v small{font-size:13px; color:var(--muted); font-weight:500}
  .note{display:flex; gap:10px; align-items:flex-start; background:var(--accent-soft);
    border:1px solid color-mix(in srgb,var(--accent) 30%, transparent); color:var(--accent-ink);
    border-radius:10px; padding:12px 15px; font-size:13.5px; margin:18px 0 2px}
  .note .dot{width:9px;height:9px;border-radius:50%;background:var(--accent);margin-top:5px;flex:none}
  .note b{font-weight:650}
  section{margin-top:46px}
  .sec-head{display:flex; align-items:baseline; gap:14px; margin-bottom:6px; flex-wrap:wrap}
  h2{font-size:19px; margin:0; font-weight:640; letter-spacing:-.01em}
  .sec-head .idx{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:12px; color:var(--accent); font-weight:600}
  .sec-desc{color:var(--muted); font-size:13.5px; margin:2px 0 16px; max-width:74ch}
  .card{background:var(--panel); border:1px solid var(--border); border-radius:12px; overflow:hidden;
    box-shadow:0 1px 2px var(--shadow)}
  .scroll{overflow-x:auto}
  table{width:100%; border-collapse:collapse; font-size:14px}
  thead th{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:11px; letter-spacing:.1em;
    text-transform:uppercase; color:var(--muted); text-align:left; padding:11px 14px;
    border-bottom:1px solid var(--border); background:var(--panel2); font-weight:600; white-space:nowrap}
  th.num,td.num{text-align:right; font-variant-numeric:tabular-nums}
  tbody td{padding:8px 14px; border-bottom:1px solid var(--border); vertical-align:middle}
  tbody tr:last-child td{border-bottom:none}
  tbody tr:hover{background:var(--panel2)}
  .name{font-weight:560}
  td.count{font-family:"IBM Plex Mono",ui-monospace,monospace; font-variant-numeric:tabular-nums;
    font-weight:600; font-size:14px; white-space:nowrap}
  .desc-cell{color:var(--muted); font-size:13px}
  .barcell{width:26%; min-width:110px}
  .bar{position:relative; height:8px; background:var(--track); border-radius:5px; overflow:hidden}
  .bar > span{position:absolute; inset:0 auto 0 0; width:var(--w,0%); background:var(--accent); border-radius:5px}
  .bar.amber > span{background:var(--amber)}
  .bar.clay > span{background:var(--clay)}
  .chip{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:10.5px; letter-spacing:.06em;
    text-transform:uppercase; padding:2px 8px; border-radius:999px; font-weight:600; white-space:nowrap; display:inline-block}
  .chip.core{background:var(--accent);color:#fff}
  .chip.cov{background:var(--accent-soft);color:var(--accent-ink);border:1px solid color-mix(in srgb,var(--accent) 30%,transparent)}
  .chip.rare{background:var(--amber-soft);color:var(--amber);border:1px solid color-mix(in srgb,var(--amber) 34%,transparent)}
  .chip.bad{background:var(--clay-soft);color:var(--clay);border:1px solid color-mix(in srgb,var(--clay) 34%,transparent)}
  :root[data-theme="dark"] .chip.core{color:#0e130f}
  @media (prefers-color-scheme:dark){:root:not([data-theme="light"]) .chip.core{color:#0e130f}}
  .grid2{display:grid; grid-template-columns:repeat(auto-fit,minmax(330px,1fr)); gap:18px}
  /* findings */
  .find{background:var(--panel); border:1px solid var(--border); border-left:3px solid var(--muted);
    border-radius:10px; padding:16px 18px; box-shadow:0 1px 2px var(--shadow)}
  .find.sev1{border-left-color:var(--clay)}
  .find.sev2{border-left-color:var(--amber)}
  .find.sev3{border-left-color:var(--accent)}
  .find h3{margin:0 0 6px; font-size:16px; font-weight:640; letter-spacing:-.01em; text-wrap:balance}
  .find p{margin:0 0 8px; font-size:13.5px; color:var(--muted)}
  .find p:last-child{margin-bottom:0}
  .find .ev{font-family:"IBM Plex Mono",ui-monospace,monospace; font-size:12px; color:var(--ink);
    background:var(--panel2); border:1px solid var(--border); border-radius:8px; padding:8px 11px; display:block; margin:0 0 9px;
    overflow-x:auto; white-space:pre}
  .findlist{display:flex; flex-direction:column; gap:14px}
  .fhead{display:flex; justify-content:space-between; align-items:baseline; gap:12px; margin-bottom:8px; flex-wrap:wrap}
  footer{margin-top:56px; padding-top:20px; border-top:1px solid var(--border); color:var(--muted);
    font-size:12.5px; font-family:"IBM Plex Mono",ui-monospace,monospace; line-height:1.7}
  footer b{color:var(--ink);font-weight:600}
  .toggle{position:fixed; top:14px; right:14px; z-index:10; font-family:"IBM Plex Mono",ui-monospace,monospace;
    font-size:12px; background:var(--panel); color:var(--muted); border:1px solid var(--border);
    border-radius:8px; padding:7px 11px; cursor:pointer}
  .toggle:hover{color:var(--ink); border-color:var(--accent)}
  .toggle:focus-visible{outline:2px solid var(--accent); outline-offset:2px}
  @media (prefers-reduced-motion:reduce){*{transition:none!important}}
</style>

<button class="toggle" id="tog" aria-label="Toggle colour theme">&#9686; theme</button>
<div class="wrap">
  <header class="mast">
    <div class="eyebrow">Sweep 2026-09-10 &middot; event census</div>
    <h1>What the 333-gate matrix actually exercised</h1>
    <p class="sub">Every <code>GameEvent</code> the Rust engine emitted across the full-matrix sweep &mdash;
      111 cells &times; 3 sampling scales, 33,300 heuristic-agent games, all of them step-for-step
      identical to stock Java. Parity is green everywhere; this page asks the different question of
      <em>which rules were on the pitch when it went green</em>.</p>
  </header>''')

o('''  <div class="stats">
    <div class="stat hero"><span class="k">Gates</span><span class="v">333<small> / 333 parity</small></span></div>
    <div class="stat"><span class="k">Games</span><span class="v">%s</span></div>
    <div class="stat"><span class="k">Events</span><span class="v">%.2fM</span></div>
    <div class="stat"><span class="k">Event types seen</span><span class="v">77<small> / 128</small></span></div>
    <div class="stat warn"><span class="k">Gates short on coverage</span><span class="v">26</span></div>
    <div class="stat warn"><span class="k">Block re-rolls</span><span class="v">0<small> / 660,522</small></span></div>
  </div>
  <div class="note"><span class="dot"></span><div><b>How to read a zero.</b> A zero on this page means
    <em>no event was emitted</em>. That is three different things: the rule never triggered, the rule
    triggered but the engine emits nothing that names it, or the mechanic is unreachable. Every finding
    below says which of the three it is, and how that was established.</div></div>''' % (n(33300), 27858969/1e6))

# ── findings ────────────────────────────────────────────────────────────────
FINDINGS = [
 (1, 'A block die is never re-rolled &mdash; in 660,522 blocks',
  '<span class="ev">blockRoll: 660,522 &nbsp; rerolled=true: 0 &nbsp;&nbsp;|&nbsp;&nbsp; lonerRoll: 0 &nbsp; proRoll: 0 &nbsp; teamCaptainRoll: 0 &nbsp; leader: 0</span>'
  '<p>Dodge, rush, pickup, catch and pass rolls all get re-rolled at 6&ndash;16%. Blocks never do, not once, in any '
  'edition or roster. <code>StepBlockRoll</code> emits <code>AgentPrompt::BlockChoice</code> carrying only the dice, '
  'and its <code>ask_for_reroll_if_available(game, "BLOCK", ...)</code> branch sits behind <code>do_roll == false</code>, '
  'which a first pass cannot reach &mdash; so no agent can ever ask. The heuristic agent has no <code>"BLOCK"</code> case in its '
  're-roll policy either, and would accept at weight 0.45 if offered.</p>'
  '<p><b>What it takes down with it:</b> team re-rolls on a Skull / Both Down, Loner (fielded in 73 of 111 cells, '
  'zero rolls), Pro and Old Pro (3 cells, zero rolls), Brawler (7 cells), Hatred (3 cells) &mdash; the last two are reachable '
  'only through the same <code>re_roll_source</code> that nothing sets. Both harnesses agree, so parity stays green.</p>'),
 (1, 'The whole bb2016 column emits no movement at all',
  '<span class="ev">bb2016  playerMoved 0        goForItRoll 0        (8,700 games, 1,159,804 Move actions)\n'
  'bb2020  playerMoved 7,588,679  goForItRoll 443,582\nbb2025  playerMoved 7,559,379  goForItRoll 436,003</span>'
  '<p>bb2016 players clearly do move &mdash; 77,331 dodges, 6,506 touchdowns, 20,514 pick-ups. But '
  '<code>GameEvent::PlayerMoved</code> and <code>GoForItRoll</code> are emitted only from '
  '<code>step/bb2025/move_/*</code>, and <code>driver.rs:425</code> routes bb2016 to its own '
  '<code>step/bb2016/move_/*</code> family, which emits neither. One third of the matrix has no movement or rush '
  'telemetry, and the rush mechanic there is asserted by nothing but the state hash.</p>'),
 (1, 'Two checklist items carry stale BLOCKED notes that hide this',
  '<span class="ev">touchdowns  25,856 (checklist note: "BLOCKED &hellip; a carrier cannot cross the pitch")\n'
  'GFI rolls   879,585 total, but 0 in all 87 bb2016 gates (same note)</span>'
  '<p><code>t3_checklist.rs</code> still marks <code>touchdowns</code> and <code>GFI rolls</code> as blocked on the '
  '"one square per activation" decision. That is no longer true anywhere: the matrix scores 25,856 touchdowns and '
  'rolls 879,585 rushes. Because both items are flagged <code>blocked</code>, they cannot fail a gate &mdash; which is '
  'exactly why the bb2016 movement hole above has been invisible. The GFI zero is a per-edition instrumentation gap '
  'wearing a note about agent behaviour.</p>'),
 (1, 'The apothecary is offered 62,279 times and never once used',
  '<span class="ev">apothecaryRoll 62,279  &mdash;  with a roll value: 0  &mdash;  with a new state: 0\napothecaryChoice: never emitted (3 sites in step_apothecary.rs)</span>'
  '<p>Every one of the 62,279 apothecary events carries <code>roll:null, new_state:null, new_serious_injury:null</code>. '
  'Across 49,530 casualties and 6,239 deaths, no casualty is ever healed and no apothecary decision is ever recorded. '
  'This is the user-visible shape of the gap: the offer happens, the choice never does.</p>'),
 (2, 'Six declared actions never produce the roll that defines them',
  '<span class="ev">BalefulHex      535 declarations &rarr; balefulHexRoll      0\n'
  'AutoGazeZoat    540 declarations &rarr; hypnoticGazeRoll    0\n'
  'LookIntoMyEyes   13 declarations &rarr; lookIntoMyEyesRoll  0\n'
  'ProjectileVomit  (16 cells field it) &rarr; projectileVomitRoll 0\n'
  'BreatheFire      ( 2 cells field it) &rarr; breatheFireRoll     0\n'
  'JumpUp           (21 cells field it) &rarr; jumpUpRoll          0</span>'
  '<p>Traced one Baleful Hex declaration through a log: the action is selected and the caster simply walks '
  '&mdash; <code>playerMoved</code>, nothing else, no hex. 64% of Baleful Hex activations and 80% of Zoat gaze '
  'activations contain nothing but movement. <code>BalefulHexRoll</code>, <code>HypnoticGazeRoll</code>, '
  '<code>LookIntoMyEyesRoll</code> and <code>BreatheFireRoll</code> have <em>no construction site anywhere in '
  '<code>ffb-engine</code></em>; <code>ProjectileVomitRoll</code> and <code>JumpUpRoll</code> do have one and never reach it.</p>'),
 (2, '61 of the 117 fielded skills are invisible to the event stream',
  '<span class="ev">Mighty Blow 78 cells   Thick Skull 69   Block 61   Pass 52   Stunty 44   Frenzy 43\n'
  'Sure Hands 29   Claw 14   Stab 12   Diving Tackle 9   Guard 4   Tentacles 4   &hellip;</span>'
  '<p>These skills are implemented and referenced in production code, and the state hash covers their '
  '<em>effects</em> &mdash; but the engine emits nothing that names them, so no coverage assertion can be made either '
  'way. Diving Tackle is the exact case in point: fielded in 9 cells, and the sweep can prove neither a use nor a '
  'decline. <code>GameEvent::SkillUse</code> has only six live skills behind it (Dodge, Horns, Wrestle, Juggernaut, '
  'Tackle, Dump Off) and <code>GameEvent::ReRoll</code> has no producer at all.</p>'),
 (2, '&ldquo;Declined&rdquo; is almost entirely uninstrumented &mdash; and Dump Off is 100% declined',
  '<span class="ev">skillUse used=true    Dodge 15,496  Horns 16,021  Wrestle 1,736  Juggernaut 630  Tackle 205  DumpOff 0\n'
  'skillUse used=false   DumpOff 3,074  Dodge 205  Wrestle 453  Juggernaut 182  Tackle 0</span>'
  '<p>Only four skills ever record a decline. Dump Off is offered 3,074 times and taken zero times &mdash; a '
  'reactive pass that has never executed in the matrix. Tackle records 205 uses and no declines, which for an '
  'always-on skill is expected; for the rest, absence of a decline is absence of instrumentation, not evidence.</p>'),
 (2, 'Prayers to Nuffle fire in bb2020 and never in bb2025',
  '<span class="ev">bb2020  prayerRoll 4,305   all 16 catalog entries rolled\nbb2025  prayerRoll     0   0 of 16   (11,100 games)\nprayerAmount: never emitted &mdash; the TV computation behind the decision is unobservable</span>'
  '<p>Same mirror matchups, same equal team values, both editions carrying a 16-entry prayer table and a live '
  '<code>StepPrayers</code>. bb2020 grants prayers; bb2025 never does. One of the two is wrong about the '
  'underdog rule, and <code>PrayerAmount</code> &mdash; the event that would show the TV gap each engine computed &mdash; '
  'has no producer, so the input cannot be inspected from the logs.</p>'),
 (2, 'The entire pre-game economy never runs',
  '<span class="ev">buyInducement 0   inducement 0   pettyCash 0   cardsAndInducementsBought 0   playCard 0\n'
  'cardEffectRoll 0   wizardUse 0   masterChefRoll 0   bribesRoll 0   coinThrow 0   receiveChoice 0   gameOptions 0</span>'
  '<p>No inducement is ever bought, no card ever played, no wizard ever cast, no chef ever steals a re-roll. '
  '"Get the Ref" hands out a free bribe 2,460 times and <code>bribesRoll</code> stays at zero. Brilliant Coaching '
  'and its bb2016 twin grant 16,423 extra re-rolls that, per finding 1, can never be spent on a block. Post-game '
  'is healthier: 66,500 winnings rolls and 66,500 MVP rolls, exactly two per game.</p>'),
 (3, '20 gates miss Hand-Over and 6 miss Pass &mdash; agent behaviour, not engine disagreement',
  '<span class="ev">missing required items across the 26 short gates:  action HandOver &times;20   action Pass &times;6   pass rolls &times;5</span>'
  '<p>Recomputing the harness checklist from the raw logs reproduces the sweep\'s 26 short gates exactly, item for '
  'item. Every miss is one of three ball-handling actions the heuristic simply never chose with that roster at that '
  'sampling scale &mdash; concentrated at <code>@1e6</code> (near-deterministic) and <code>@0</code> (near-uniform), the two '
  'extremes. Khemri and its FUMBBL twin never pass at <code>@0</code> in either bb2020 or bb2025.</p>'),
 (3, '83 of 200 skills and 32 of 58 actions are never drafted or never declared',
  '<span class="ev">never fielded by any of the 111 squads: Kick, Leader, Team Captain, Pass Block, Piling On, Pile Driver,\n'
  'Sneaky Git, Kick-Off Return, Bounding Leap, Halfling Luck, Brutal Block, Whirling Dervish, &hellip; (83 total)\n'
  'never declared once: Stab, Chainsaw, BreatheFire, ProjectileVomit, Gaze, DumpOff, Swoop, MaximumCarnage,\n'
  'PutridRegurgitation{Move,Blitz,Block}, KickEm{Block,Blitz}, TheFlashingBlade, ViciousVines, Chomp, &hellip; (32 total)</span>'
  '<p>This is a drafting-coverage gap rather than an engine gap, and it is the cheapest to close: the squads in '
  '<code>data/teams/</code> simply never take these skills. Kick and Leader are ordinary picks on real teams, and Kick '
  'in particular gates a core kick-off rule that no gate touches.</p>'),
 (3, 'Everything the census does prove',
  '<span class="ev">every drafted player number acted at least once, in every one of the 111 cells\n'
  'all 11 kick-off results covered in each edition\'s own table   all 5 weather results in all 3 editions\n'
  'all 20 serious injuries   6,239 deaths (12.6% of casualties, against a 12.5% d16 expectation)</span>'
  '<p>Positional coverage is complete: no drafted player sits out. The kick-off table, the weather table, the '
  'injury table and the block-result table are fully covered in every edition, and the tails match their '
  'expected frequencies. 45 fielded skills are provably exercised, including the whole big-guy negatrait family '
  '(435,074 Bone Head / Really Stupid / Take Root rolls) and 14,542 team-mate throws.</p>'),
]

o('  <section>\n    <div class="sec-head"><span class="idx">01</span><h2>Findings, worst first</h2></div>')
o('    <p class="sec-desc">Severity is about what the gate can no longer promise, not about how many events are '
  'missing. Bar one, every item here sits underneath a green parity verdict.</p>')
o('    <div class="findlist">')
for sev, title, bodyhtml in FINDINGS:
    lab = {1: '<span class="chip bad">blocks a claim</span>', 2: '<span class="chip rare">narrows a claim</span>',
           3: '<span class="chip cov">scope / confirmed</span>'}[sev]
    o('      <div class="find sev%d"><div class="fhead"><h3>%s</h3>%s</div>%s</div>' % (sev, title, lab, bodyhtml))
o('    </div>\n  </section>')

# ── actions ─────────────────────────────────────────────────────────────────
o('  <section>\n    <div class="sec-head"><span class="idx">02</span><h2>Actions declared</h2>'
  '<span class="sec-desc" style="margin:0">26 of 58 <code>PlayerAction</code> variants, 5,447,331 declarations</span></div>')
o('    <div class="card scroll"><table><thead><tr><th>Action</th><th class="num">Declared</th>'
  '<th class="barcell">Share</th><th class="num">Move&#8209;only windows</th><th>What the activation contained</th></tr></thead><tbody>')
mx = max(A.values())
for a, c in A.most_common():
    e = emp2.get(a, 0)
    pct = '%.0f%%' % (100.0 * e / c) if a not in BASIC else '&mdash;'
    inner = ', '.join('%s&nbsp;%s' % (k, n(v)) for k, v in win2[a].most_common(4) if k != 'playerMoved') if a not in BASIC else ''
    chip = ''
    if a not in BASIC and e and 100.0 * e / c > 60:
        chip = ' <span class="chip rare">degenerates</span>'
    o('      <tr><td class="name mono">%s%s</td><td class="count num">%s</td>'
      '<td class="barcell"><div class="bar"><span style="--w:%.1f%%"></span></div></td>'
      '<td class="count num">%s</td><td class="desc-cell mono" style="font-size:11.5px">%s</td></tr>'
      % (a, chip, n(c), 100.0 * c / mx, pct, inner or '&mdash;'))
o('    </tbody></table></div>')

# ── skills three-way ───────────────────────────────────────────────────────
o('  <section>\n    <div class="sec-head"><span class="idx">03</span><h2>Every skill the 111 squads field</h2></div>')
o('    <p class="sec-desc">117 distinct <code>SkillId</code>s are fielded. Split by what the event stream can '
  'prove about each: 45 provably exercised, 11 with a dedicated event that never fired, 61 with no event that '
  'names them at all.</p>')
o('    <div class="grid2">')
o('      <div class="card scroll"><table><thead><tr><th>Fielded, event never fired</th><th class="num">Cells</th><th>Event</th></tr></thead><tbody>')
for name, c, lab, cnt in sorted(sk['dead'], key=lambda r: -r[1]):
    o('        <tr><td class="name">%s</td><td class="count num">%d</td><td class="desc-cell mono" style="font-size:11.5px">%s = 0</td></tr>' % (name, c, lab.replace('event ', '')))
o('      </tbody></table></div>')
o('      <div class="card scroll"><table><thead><tr><th>Provably exercised</th><th class="num">Cells</th><th class="num">Evidence count</th></tr></thead><tbody>')
for name, c, lab, cnt in sorted(sk['live'], key=lambda r: -r[3]):
    o('        <tr><td class="name">%s</td><td class="count num">%d</td><td class="count num">%s</td></tr>' % (name, c, n(cnt)))
o('      </tbody></table></div>')
o('    </div>')
o('    <div class="card scroll" style="margin-top:18px"><table><thead><tr><th>Fielded, but the engine emits nothing that names it &mdash; 61 skills</th><th class="num">Cells</th></tr></thead><tbody>')
row = ', '.join('%s <span class="mono" style="color:var(--muted)">%d</span>' % (nm, c) for nm, c, _, _ in sorted(sk['silent'], key=lambda r: -r[1]))
o('      <tr><td colspan="2" style="line-height:2">%s</td></tr>' % row)
o('    </tbody></table></div>')
o('  </section>')

# ── never-emitted events ───────────────────────────────────────────────────
never = emit['never']
withsite = [v for v in never if emit['emit'].get(v, 0) > 0]
nosite = [v for v in never if emit['emit'].get(v, 0) == 0]
o('  <section>\n    <div class="sec-head"><span class="idx">04</span><h2>The 51 event types that never fired</h2></div>')
o('    <p class="sec-desc">Of 128 <code>GameEvent</code> variants, 77 appeared. The other 51 split cleanly: '
  '%d have a production emit site in <code>ffb-engine</code> that the sweep never reached, and %d have no producer '
  'anywhere &mdash; they are declared and consumed by the coverage/wire layers only.</p>' % (len(withsite), len(nosite)))
o('    <div class="grid2">')
o('      <div class="card scroll"><table><thead><tr><th>Has an emit site, never reached</th><th>Site</th></tr></thead><tbody>')
for v in withsite:
    site = emit['where'].get(v, ['-'])[0].replace('crates/ffb-engine/src/', '')
    o('        <tr><td class="name mono">%s</td><td class="desc-cell mono" style="font-size:11px">%s</td></tr>' % (v, site))
o('      </tbody></table></div>')
o('      <div class="card scroll"><table><thead><tr><th>No producer in the engine at all</th></tr></thead><tbody>')
for v in nosite:
    o('        <tr><td class="name mono">%s</td></tr>' % v)
o('      </tbody></table></div>')
o('    </div>\n  </section>')

# ── tables: kickoff / weather / injury / rerolls ───────────────────────────
o('  <section>\n    <div class="sec-head"><span class="idx">05</span><h2>Tables the matrix covers completely</h2></div>')
o('    <p class="sec-desc">Kick-off, weather, serious injury and block result are each fully covered &mdash; every '
  'entry of every edition\'s own table appeared, with tails at their expected frequencies.</p>')
o('    <div class="grid2">')
o('      <div class="card scroll"><table><thead><tr><th>Kick-off result</th><th class="num">bb2016</th><th class="num">bb2020</th><th class="num">bb2025</th></tr></thead><tbody>')
for k, c in K.most_common():
    o('        <tr><td class="name">%s</td>%s</tr>' % (k, ''.join(
        '<td class="count num">%s</td>' % (n(KED[e][k]) if KED[e].get(k) else '<span style="color:var(--muted)">n/a</span>')
        for e in ('bb2016', 'bb2020', 'bb2025'))))
o('      </tbody></table></div>')
o('      <div class="card scroll"><table><thead><tr><th>Serious injury</th><th class="num">Count</th><th class="barcell">Share</th></tr></thead><tbody>')
mxs = max(SI.values())
for k, c in SI.most_common():
    o('        <tr><td class="name">%s</td><td class="count num">%s</td>'
      '<td class="barcell"><div class="bar amber"><span style="--w:%.1f%%"></span></div></td></tr>' % (k, n(c), 100.0*c/mxs))
o('      </tbody></table></div>')
o('    </div>')
o('    <div class="grid2" style="margin-top:18px">')
o('      <div class="card scroll"><table><thead><tr><th>Re-roll rate by roll type</th><th class="num">Rolls</th><th class="num">Re-rolled</th><th class="num">%</th></tr></thead><tbody>')
for k in sorted(RRt, key=lambda k: -RRt[k])[:16]:
    pc = 100.0 * RRr[k] / max(RRt[k], 1)
    cls = ' class="chip bad"' if RRr[k] == 0 and RRt[k] > 10000 else ''
    o('        <tr><td class="name mono">%s</td><td class="count num">%s</td><td class="count num">%s</td>'
      '<td class="count num">%s</td></tr>' % (k, n(RRt[k]), n(RRr[k]),
      ('<span%s>0.00</span>' % cls) if RRr[k] == 0 else '%.2f' % pc))
o('      </tbody></table></div>')
o('      <div class="card scroll"><table><thead><tr><th>Weather</th><th class="num">bb2016</th><th class="num">bb2020</th><th class="num">bb2025</th></tr></thead><tbody>')
WXE = defaultdict(Counter)
for d in gates:
    WXE[d['edition']].update(d.get('weather', {}))
for k, c in WX.most_common():
    o('        <tr><td class="name">%s</td>%s</tr>' % (k, ''.join('<td class="count num">%s</td>' % n(WXE[e][k]) for e in ('bb2016', 'bb2020', 'bb2025'))))
o('      </tbody></table>')
o('      <table><thead><tr><th>Prayers to Nuffle</th><th class="num">Rolls</th></tr></thead><tbody>')
for k, c in PR.most_common():
    o('        <tr><td class="name mono" style="font-size:12px">%s</td><td class="count num">%s</td></tr>' % (k, n(c)))
o('      </tbody></table></div>')
o('    </div>\n  </section>')

# ── per-cell table ─────────────────────────────────────────────────────────
o('  <section>\n    <div class="sec-head"><span class="idx">06</span><h2>All 111 matchups</h2>'
  '<span class="sec-desc" style="margin:0">mirror matchups, 300 games each (3 sampling scales &times; 100 seeds)</span></div>')
o('    <div class="card scroll"><table><thead><tr><th>Matchup</th><th class="num">Events</th><th class="num">Blocks</th>'
  '<th class="num">Injuries</th><th class="num">Cas</th><th class="num">KO</th><th class="num">Dead</th><th class="num">TD</th>'
  '<th class="num">Dodges</th><th class="num">Rushes</th><th class="num">Passes</th><th class="num">Hand&#8209;offs</th>'
  '<th class="num">Fouls</th><th class="num">Ejected</th><th class="num">Kick&#8209;offs</th><th class="num">TTM&nbsp;thrown</th>'
  '<th class="num">Bombs</th><th class="num">Negatrait</th><th class="num">Prayers</th><th class="num">Apo</th></tr></thead><tbody>')
for r in cells:
    z = lambda v: n(v) if v else '<span style="color:var(--clay)">0</span>'
    o('      <tr><td class="name mono" style="font-size:12.5px">%s <span style="color:var(--muted)">%s</span></td>'
      '<td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td>'
      '<td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td>'
      '<td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td>'
      '<td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td>'
      '<td class="count num">%s</td><td class="count num">%s</td><td class="count num">%s</td></tr>'
      % (r['race'], r['edition'], n(r['events']), n(r['blocks']), n(r['injuries']), n(r['cas']), n(r['ko']),
         n(r['dead']), z(r['td']), n(r['dodges']), z(r['gfi']), n(r['passes']), z(r['handovers']), n(r['fouls']),
         n(r['ejected']), n(r['kickoffs']), z(r['ttm']) if r['ttm_decl'] else '&mdash;', z(r['bombs']) if r['bombs'] else '&mdash;',
         z(r['confusion']) if r['confusion'] else '&mdash;', z(r['prayers']) if r['edition'] != 'bb2016' else '&mdash;', n(r['apo'])))
o('    </tbody></table></div>')
o('    <p class="sec-desc" style="margin-top:12px">Red zeros are the ones worth reading: the <b>Rushes</b> column is '
  'zero for every bb2016 row (finding 2), and <b>Prayers</b> is zero for every bb2025 row (finding 8). A dash means '
  'the mechanic is not on that roster at all.</p>')
o('  </section>')

o('''  <section>
    <div class="sec-head"><span class="idx">07</span><h2>Method</h2></div>
    <p class="sec-desc">Counts come from the sweep's own <code>seed_*_rust_events.jsonl</code> logs left behind by
      the 333 <code>parity_sw_*</code> roots &mdash; 2.0&nbsp;GB, 27,858,969 events, re-tallied with the same rules
      <code>coverage_report.rs::tally()</code> uses, so an item here means what it means in the harness checklist.
      The reconstruction reproduces the sweep's 26 short gates exactly, item for item, which is what licenses the
      rest of the numbers. A second pass windowed each special-action declaration to the next
      <code>playerAction</code> / <code>turnEnd</code> to separate "declared" from "executed". Skill inventories come
      from <code>data/teams/</code> and <code>data/rosters/</code> resolved through
      <code>SkillId::from_class_name</code> &mdash; all 117 roster skill strings resolve, so nothing is silently
      dropped. Because every per-activation state hash matched stock Java, the Rust events are a faithful proxy for
      both engines.</p>
    <p class="sec-desc">Two limits worth restating. The state hash is narrow &mdash; no <code>passing</code>, no
      <code>reroll_used</code>, no ACTIVE bit, nothing above player 11 &mdash; so "covered by the hash" is weaker than
      it sounds for the 61 invisible skills. And green is scoped to the drafted squads: 83 skills and 7 official
      teams are outside the matrix entirely.</p>
  </section>

  <footer>
    <b>Source</b> 333 &times; <code>parity_sw_&lt;race&gt;_&lt;edition&gt;_&lt;scale&gt;/</code> &mdash; heuristic agent,
    seeds 1&ndash;100, <code>--heur-classes all</code>, scales 0 / 1.0 / 1e6<br>
    <b>Parity</b> 333/333 gates <code>100/100 games match</code>, 0 failures, 0 panics &middot; tag
    <code>parity-matrix-green-2026-09-10</code><br>
    <b>Census</b> 33,300 games &middot; 27,858,969 events &middot; 77 of 128 event types &middot; 25 variants with no producer in the engine
  </footer>
</div>
<script>
  (function(){
    var b=document.getElementById('tog');
    b.addEventListener('click',function(){
      var r=document.documentElement, cur=r.getAttribute('data-theme');
      if(!cur){cur=matchMedia('(prefers-color-scheme: dark)').matches?'dark':'light';}
      r.setAttribute('data-theme', cur==='dark'?'light':'dark');
    });
  })();
</script>''')

open('census.html', 'w', encoding='utf-8').write('\n'.join(P))
print('wrote census.html', len('\n'.join(P)), 'bytes')
