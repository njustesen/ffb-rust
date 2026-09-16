# Coverage gaps — what the 2026-09-15 sweep never exercised

The sweep is green everywhere parity is measured: **330/330 gates, 100/100 seeds each**,
110 squads × 3 editions × 3 heuristic scales, 33,000 games, 13.98M reports
(`SWEEP_2026-09-15_REPORTS.txt`, `report_census.json`).

Green parity is not coverage. A mechanic that never fires is trivially in parity — both
engines agree about nothing happening. The census records what *did* fire; this document
records what the drafted squads were **carrying** that never fired, and traces each dead
mechanic to a cause.

Rosters are fixed — no draft changes, no star players. Coverage therefore has to come from
the engine and the agent, not from fielding more skills.

**Regenerate:**

```bash
python scripts/sweep_census/gaps.py [census_dir] [report_census.json]
# defaults: scripts/sweep_census/out_s15  docs/report_census.json
# also writes scripts/sweep_census/gaps.json
```

Inputs: `scripts/sweep_census/inventory.json` (110 squads → skills, all `unresolved: []`,
zero stars drafted), the 330 per-gate aggregates in `scripts/sweep_census/out_s15/`, and
`docs/report_census.json`. `gaps.py` replaces the retired `skills.py`, which read a dead
`out/*.json` layout and joined display names ("Always Hungry") against CamelCase enum keys,
so most of its rows silently missed.

## Headline

| | |
|---|---|
| distinct fielded skills | **98** (107 display names, collapsed over edition spellings) |
| exercised | 67 |
| dedicated telemetry exists, never fired | **10** |
| engine names it nowhere | 21 |
| exercised fewer than 1,000 times | 22 |
| `PlayerAction` variants declared | **14 of 57** (27 offered, 16 scored by the agent) |
| `KickoffResult` variants seen | **16 of 16** |
| `GameEvent` variants emitted | 88 of 129 |
| `ReportId`s produced | 87 of 164 |

Edition spellings are collapsed on `[^a-z0-9]`: Bone Head / Bone-Head / bone head,
Claw / Claws, Side Step / Sidestep, Timmm-ber! / Timmmber. A naive name-join drops the
last two.

## The three-way split

The dead mechanics do not share one cause. Traced to source, they separate cleanly, and
this is what the backlog is built on.

**1. Never offered.** `legal_actions/mod.rs` surfaces 27 of the 57 `PlayerAction` variants
(via `PlayerActionChoice`, mapped through its own `PAC::X => PA::Y` table — `HypnoticGaze`
becomes `Gaze`, `HandOff` becomes `HandOver`). **Stab, BreatheFire, ProjectileVomit**
(+`PutridRegurgitation*`), **Chainsaw-as-action, Swoop, DumpOff, HailMaryBomb, StandUp,
RemoveConfusion** are not among them — no agent could ever pick them. Before treating any
of these as a Rust defect, check what Java's ParityRunner offers: if Java does not offer it
either, the cap is the shared harness, not the Rust engine, and the item changes shape.

**2. Offered, never chosen — the agent.** `heuristic_agent.rs` (8,354 lines) scores 16
action kinds; 14 are ever declared. `Gaze` is offered to bb2016 vampires
(`legal_actions/mod.rs:636`) and declared **zero** times. `StandUpBlitz` — the Jump Up path
— is offered *and* scored and declared zero times. `MultipleBlock` is offered but carries
no scorer term; `FoulMove` and `KickEmBlitz` are scored branches the offer set never
reaches. `DumpOff` is declined 3,619 times and executed zero. **This is the dominant lever for
coverage, and it is agent work.**

**3. No emit site.** `GameEvent::BreatheFireRoll` is constructed only in
`ffb-server/src/net/wire.rs` and `ffb-parity/src/coverage_report.rs` — the engine has **no
emit site at all** (`grep -rn "GameEvent::BreatheFireRoll" crates/ffb-engine` → 0 hits).
The other nine dead sites do have live engine emit sites.

## A. Fielded, dedicated telemetry exists, never fired

The real gaps. Each of these squads carries the skill and the sweep proves nothing about it.

| skill | squads | telemetry site | cause |
|---|---|---|---|
| Jump Up | 20 | `jumpUpRoll` (event + report) | agent — `StandUpBlitz` offered, never chosen |
| Projectile Vomit | 16 | `projectileVomitRoll` + `projectileVomit` | not offered |
| Shadowing | 10 | report `tentaclesShadowingRoll` | diagnose (live thread in §H.54) |
| Tentacles | 3 | report `tentaclesShadowingRoll` | diagnose |
| Hypnotic Gaze | 3 | `hypnoticGazeRoll` (event + report) | agent — `Gaze` offered, never chosen |
| Pick-me-up | 3 | `pickMeUpRoll` + `pickMeUp` | diagnose |
| Breathe Fire | 2 | `breatheFireRoll` + `breatheFire` | **no engine emit site**, and not offered |
| Trickster | 2 | report `trapDoor` (shared, not skill-specific) | no skill-specific site exists |
| Fumblerooski | 1 | `fumblerooskie` (event + report) | not offered |
| Pro | 1 | `proRoll` + report `oldPro` | diagnose — never offered as a re-roll source |

Cells:

- **Jump Up** — amazon bb2020/25, dark_elf ×3, dark_elf_league_fumbbl ×3, gnome bb2020/25,
  khorne bb2025, norse ×3, slann ×3, slann_fumbbl ×3
- **Projectile Vomit** — black_orc bb2020/25, chaos_chaostroll bb2020, chaos_pact bb2020,
  chaos_troll bb2025, goblin bb2020/25, orc bb2020/25, renegades_37730 bb2020,
  renegades_37733 bb2025, renegades bb2025, snotling bb2020/25, underworld bb2025,
  underworld_underworldtroll bb2020
- **Shadowing** — chaos_dwarf bb2020/25, dark_elf ×3, dark_elf_league_fumbbl ×3,
  lizardman bb2020/25
- **Tentacles** — nurgle ×3 · **Hypnotic Gaze** — vampire ×3 · **Pick-me-up** — norse
  bb2020/25, nurgle bb2025 · **Breathe Fire** — chaos_dwarf bb2020/25 · **Trickster** —
  gnome bb2020/25 · **Fumblerooski** — elf bb2025 · **Pro** — imperial_nobility bb2025

## B. Fielded, the engine names it nowhere

Expected-dark: a passive skill raises no event in either engine and appears only inside
`rollModifiers` / `armorModifiers` / `injuryModifiers` / `casualtyModifiers` on a report
(BACKLOG §H.50). These 21 appear in none of the 65 observed modifier names either. Stated
once so the silence is on the record, not rediscovered each sweep.

Thick Skull (68 squads), Block (61), Frenzy (42), Sprint (12), Diving Tackle (9), Brawler
(7), Unsteady (7), No Hands (6), Timmm-ber! (6), On the Ball (4), No Ball (4), My Ball (3),
Hatred (3), Insignificant (3), Grab (2), Iron Hard Skin (2), Guard (2), Plague Ridden (2),
Nurgle's Rot (1), Running Pass (1), Give and Go (1).

**Two worth a second look.** Java names dodge and assist modifiers, so **Diving Tackle**
(a −2 dodge modifier) and **Guard** (which cancels assists — visible today only as a
*smaller* assist count, never by name) plausibly should appear. Flagged for diagnosis, not
asserted.

## C. Fielded and exercised, but thinly

Under 1,000 occurrences in 33,000 games; the single-digit rows are effectively anecdotes.

| skill | squads | count | evidence |
|---|---|---|---|
| Cloud Burster | 2 | 3 | report `cloudBurster` |
| Safe Throw | 1 | 4 | event `safeThrowRoll` |
| Safe Pass | 4 | 9 | SkillUse |
| Side Step | 16 | 11 | SkillUse (declined) |
| Hail Mary Pass | 1 | 22 | action `HailMaryPass` |
| Diving Catch | 7 | 60 | rollModifier |
| Swoop | 2 | 62 | event `swoopPlayer` |
| Strip Ball | 5 | 106 | SkillUse |
| Safe Pair of Hands | 4 | 117 | SkillUse (declined) |
| Break Tackle | 4 | 143 | rollModifier `Break Tackle ST 3-` / `ST 5+` |
| Punt | 1 | 183 | action `Punt` |
| Dirty Player | 7 | 333 | armour + injury modifier |
| Stab | 11 | 444 | playerEvent "gains Stab" (the *prayer*, not the skill action) |
| Juggernaut | 10 | 565 | SkillUse |
| Leap / Very Long Legs / Pogo / Pogo Stick | 13/7/2/2 | 664 | event `jumpRoll` (shared) |
| Ball and Chain | 3 | 820 | event `escapeRoll` |
| Chainsaw | 3 | 828 | armour modifier (the *action* is never declared) |
| Sure Feet | 3 | 847 | ReRoll source |
| Hit and Run | 3 | 993 | SkillUse |

Note Stab: the 444 hits are the `STILETTO` prayer granting the skill, not the Stab action,
which is never offered. Same shape for Chainsaw — the roll fires 144 times through a block,
the Chainsaw action never.

## D. Fielded by a squad, never exercised in that squad's own gates

The mechanic is alive elsewhere, so this is thin sampling rather than a dead path.

| skill | cells |
|---|---|
| Side Step | elf bb2016, ogre bb2016, gnome bb2020, necromantic bb2020, ogre bb2020, snotling bb2020 |
| Diving Catch | slann bb2016/20/25, slann_fumbbl bb2020/25, elf bb2025 |
| Tackle | chaos_dwarf bb2016, dwarf bb2016/20/25 |
| Safe Pass | amazon bb2020, high_elf bb2020 |
| Pass | khemri_fumbbl bb2020/25 |
| Sidestep | snotling bb2025 |

## E. Actions — 14 of 57 declared

Declared: Move 4.27M, Block 456K, BlitzMove 208K, ThrowTeamMate 101K, Blitz 73K, Foul 65K,
PassMove 22K, HandOverMove 21K, ThrowBomb 8.8K, Pass 6.2K, HandOver 5.6K, KickTeamMate
4.6K, Punt 183, HailMaryPass 22.

Never declared, by cause:

- **Offered and scored, never chosen** — `Gaze`, `StandUpBlitz`.
- **Scored by the agent but never offered** — `FoulMove`, `KickEmBlitz` (dead scorer branches).
- **Offered, not scored** — `MultipleBlock`, `SecureTheBall`.
- **Not offered by `legal_actions`** — `Stab`, `BreatheFire`, `ProjectileVomit`,
  `PutridRegurgitation{Move,Blitz,Block}`, `Chainsaw`, `Swoop`, `DumpOff`, `HailMaryBomb`,
  `StandUp`, `RemoveConfusion`, `KickEmBlock`, `MaximumCarnage`, `TheFlashingBlade`,
  `ViciousVines`, `Incorporeal`, `Chomp`, `Forgo`.
- **Sub-state labels**, never tallied on their own — `BlitzSelect`, `GazeSelect`,
  `GazeMove`, `ThrowTeamMateMove`, `KickTeamMateMove`, `PuntMove`.
- **Star-only, unreachable by design** (see below) — `Treacherous`,
  `WisdomOfTheWhiteDwarf`, `ThrowKeg`, `RaidingParty`, `LookIntoMyEyes`, `BalefulHex`,
  `AllYouCanEat`, `BlackInk`, `CatchOfTheDay`, `ThenIStartedBlastin`, `FuriousOutburst`,
  `AutoGazeZoat`.

## F. Kickoff — no gap

All **16 of 16** `KickoffResult` variants fire. Thinnest are Riot 1,271 and ThrowARock
1,285; richest CheeringFans 14,492. Nothing to do here.

## G. Activations — no gap

`by_player` shows every drafted position activating in every gate, 32,537–53,959
activations per cell across its three scale gates, evenly spread across the 11–16 players
(≈360–470 per player per 100-game gate). No dead position, no starved roster slot.

## H. One-sided streams

A report that is never produced although the mechanic demonstrably runs. These are
report-layer fidelity gaps of the kind `report_compare.rs` already measures — the game is
right, the record of it is incomplete.

| report never produced | but the mechanic ran |
|---|---|
| `throwAtStallingPlayer` | event `throwAtStallingPlayer` = 412 |
| `kickTeamMateRoll` | action `KickTeamMate` = 4,566 (and `kickTeamMateFumble` = 181) |
| `swoopDirectionRoll`, `swoopDistanceRoll` | event `swoopPlayer` = 62 |
| `passBlock` | event `passBlock` = 850 |
| `nervesOfSteel` | rollModifier "Nerves of Steel" = 1,135 |
| `winningsRoll` | event `winningsRoll` = 65,994 |
| `blockReRoll` | Brawler / Pro fielded by 8 squads |
| `oldPro` | Pro fielded by 1 squad |

## I. Closed by design — not backlog material

Rosters are fixed and **no star player is drafted in any of the 110 squads**, so these are
unreachable by construction, not defects:

- ~18 star-player actions: Treacherous, WisdomOfTheWhiteDwarf, ThrowKeg, RaidingParty,
  MaximumCarnage, LookIntoMyEyes, BalefulHex, AllYouCanEat, BlackInk, CatchOfTheDay,
  ThenIStartedBlastin, TheFlashingBlade, ViciousVines, FuriousOutburst, AutoGazeZoat,
  Chomp, Incorporeal, MultipleBlock-as-star.
- The inducement / card / wizard / league slice of the 76 never-produced reports:
  `inducement*`, `cards*`, `playCard`, `cardEffectRoll`, `cardDeactivated`, `wizardUse`,
  `weatherMage*`, `masterChefRoll`, `bribesRoll`, `biasedRef`, `saboteurRoll`, `mascotUsed`,
  `showStarReRoll*`, `pumpUpTheCrowd*`, `doubleHired*`, `pettyCash`, `freePettyCash`,
  `defectingPlayers`, `riotousRookies`, `raidingParty`, `twoForOne`, `penaltyShootout`,
  `fumbblResultUpload`, `gameOptions`, `teamCaptainRoll`, `thrownKeg`, `weepingDaggerRoll`,
  `chompRoll`, `chompRemoved`, `catchOfTheDay`, `allYouCanEat`, `balefulHex`,
  `lookIntoMyEyesRoll`, `thenIStartedBlastin`.

Also structurally dark: **102 of the 199 distinct `SkillId` variants** (200 names, `Claw`
and `Claws` collapsing to one) are fielded by no squad at all.

## Status

This is the **2026-09-15 snapshot**. Backlog §H.56 (2026-09-16) has since closed six of the
report-layer items from it — `throwAtStallingPlayer` and `winningsRoll` now have live emit
sites, and the bb2016 `turnEnd` / `injury` / `blockRoll` payloads plus the `gettingEvenRoll`
keyword now match Java. Re-run `gaps.py` against the next census rather than trusting the
numbers above for those six.

## Where this feeds

Backlog §H.55 carries the ordered work queue arising from this report.
`COVERAGE_REPORT.md` explains the two streams and how the census is captured.
