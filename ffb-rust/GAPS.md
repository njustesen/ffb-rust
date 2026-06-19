# FFB Parity Gaps

Findings discovered during parity runs. Do not fix gaps inline — track them here and open dedicated sessions for each fix. Date each finding with the session it was found in.

---

## Logging & Hashing Gaps

### G-LOG-1: State hash does not cover reroll counts or ball carrier
**Found:** 2026-06-02  
**Severity:** Medium — reroll divergences are invisible

`state_string()` in `crates/ffb-model/src/util/state_hash.rs` includes:
- half, turn counters, active side, score, ball coordinate/in-play, per-player (coordinate, base state)

It does **NOT** include:
- Remaining team rerolls (TurnData.rerolls_remaining)
- Ball carrier player ID
- Per-player skill usage flags (used_skills HashSet)
- Injury details (niggling_injuries, stat_injuries — only base state "Injured" is hashed)
- Turn mode / game phase enum
- Team special rules state (e.g. Master Chef uses remaining, KO rerolls)

**Impact:** A bug that burns rerolls incorrectly or mis-tracks ball possession would not be caught until it affects a player coordinate (e.g. wrong reroll → different block outcome → player position changes). These bugs could silently pass the parity check.

**Fix needed:** Expand `state_string()` to include reroll counts + ball carrier ID. Requires matching change to Java's `ParityRunner.stateHash()`.

---

### G-LOG-2: Only turn-boundary decision is logged; all sub-turn decisions invisible
**Found:** 2026-06-02  
**Severity:** High — once player activation is enabled, mid-turn decisions have no audit trail

The JSONL step log is emitted only at `AgentPrompt::ActivatePlayer` boundaries. All other decisions (block die choice, pushback square, follow-up, reroll use, skill activation, kick coordinate) are applied to the engine but never appear in the log.

**Impact:** When a divergence occurs mid-turn (e.g., different block die produces different casualty), the log shows a hash mismatch at the next turn boundary with no way to trace back to the cause. Debugging requires adding temporary Rust/Java print statements.

**Fix needed:**
- Add an `action` log line type that records every decision: prompt type, chosen response (die index, coordinate, boolean), and post-action state hash.
- Alternatively, log all dice rolls (armour, injury, scatter, block) as a `dice` entry.
- This requires matching changes in Java's ParityRunner.

---

### G-LOG-3: `chosen` field now reflects activation decisions (**RESOLVED for T3a**)
**Found:** 2026-06-02 | **Resolved:** 2026-06-12 (session 44)

The `chosen` field now correctly shows actual activation choices, e.g. `"Activate(away_03,Blitz)"`, `"Activate(home_01,Move)"`. This was fixed when Stage B agent activation was implemented.

**Still open:** Mid-turn sub-decisions (block die choice, pushback square, follow-up yes/no) are not logged in the step JSONL — those are part of G-LOG-2.

---

### G-LOG-4: `dice` field is always empty `[]`
**Found:** 2026-06-02  
**Severity:** Low

Java's ParityRunner always writes `"dice": []` and so does Rust. Actual dice roll results from the engine are not captured in the step log. Confirmed still `[]` in the T3a 100/100 run — all dice results appear only in `*_events.jsonl`, not in the step JSONL used for parity comparison.

**Fix needed:** Emit dice results (block dice, armour, injury) into this field or a new `dice_log` field. Requires Java changes too.

---

### G-LOG-5: State string not included in JSONL — diagnosis requires guesswork
**Found:** 2026-06-02  
**Severity:** Medium — divergences are hard to diagnose

When a hash mismatch occurs, the log only shows two hex strings. To understand what differs (wrong player position? different ball location? wrong state?) requires re-running with special debug output.

**Fix needed:** Add optional `state` field to each `step` entry containing the full state string. Controlled by a `--verbose` flag. This is Rust-only; Java could be extended later.

**Status:** `--verbose` flag added to Rust parity runner (see G-LOG-5 implementation below).

---

## Rule/Engine Gaps

### G-RULE-1: Secret Weapon players not removed end-of-drive (**CONFIRMED BLOCKER for Dwarf, Goblin**)
**Found:** 2026-06-02  
**Severity:** High — blocks Dwarf and Goblin from passing T2

**Evidence:** Dwarf and Goblin fail at step 17 (first step of H2) across all seeds. H1 (steps 1–16) matches perfectly. The divergence begins at halftime.

**Affected positions:**
- Dwarf: Death Roller (`human.deathroller` / `dwarf.deathroller`) — has Secret Weapon
- Dwarf: Troll Slayer — does NOT have Secret Weapon (has Frenzy + Dauntless) ← revise if wrong
- Goblin: Fanatic, Bombardier, Looney, Pogo Sticker — all have Secret Weapon

**Root cause (hypothesis):** Java's engine removes Secret Weapon players from the field at the end of each drive (placed in Reserves). Rust's engine does not, or applies the rule at a different point. As a result:
- Java: H2 team setup uses N-K players (K secret weapon players in Reserve)
- Rust: H2 team setup uses N players (all including secret weapon players)
- → Different canonical field positions → hash diverges at H2T1

**Secondary symptom:** For some seeds, Java logs H2T2 as the first step but Rust logs H2T1 — turn numbering diverges. This may be caused by the same secret weapon removal triggering an extra INIT_SELECTING event in Java.

**Note on Troll Slayers:** Troll Slayers have Frenzy and Dauntless, NOT Secret Weapon. If dwarf is failing, the Death Roller (1 per team) is the likely culprit. Verify that exactly 1 player (the Death Roller) is absent from Java's H2 setup vs Rust's.

**Fix needed:**
1. Identify exactly where the Rust engine handles end-of-drive Secret Weapon removal
2. Verify it runs at halftime, not just at the end of a full drive after a TD
3. Also verify it runs after Overtime drives
4. Check goblin positions for exact Secret Weapon list

---

### G-RULE-2: FUMBBL-specific rosters fail Java team loading
**Found:** 2026-06-02  
**Severity:** Medium — blocks 4 races from T2 testing

**Affected races:** dark_elf_league_fumbbl, khemri_fumbbl, renegades, slann_fumbbl

**Symptom:** `java=None` at step 0 — Java produces an empty log (no steps). Java's ParityRunner can't complete the game setup.

**Root cause (hypothesis):** The Rust JSON roster IDs for these races are FUMBBL numeric IDs (`4959`, `55051`, `1050157`, `744258`). The Java XML roster files use the same numeric `id` attributes, and the generated team XML uses `<rosterId>4959</rosterId>`. However, there may be a secondary issue:
- The Java server `roster_dark_elf_league_fumbbl.xml` has `<roster team="1084086" id="4959">` — the `team` attribute references an external FUMBBL team. This may require network access or special server configuration to resolve.
- Java's `RosterCache` might need additional data files or URL resolvers for FUMBBL-linked rosters.
- The Rust positions use numeric IDs (37738, etc.) that exist in the Java XML, so position resolution isn't the issue.

**Not affected:** chaos_pact (`id: chaospact.lrb6`) and nippon (`id: 5681`) both pass — nippon uses a simple numeric id, chaos_pact uses the named convention.

**Fix needed:**
1. Examine why Java fails to produce output for these 4 races (add Java debug logging)
2. Check if `roster_dark_elf_league_fumbbl.xml` etc. require special server configuration
3. If Java can't load these rosters without FUMBBL server connectivity, skip these races in automated T2 runs

---

### G-RULE-3: No player activation — parity is "no-op game" only (**FIXED 2026-06-18**)
**Found:** 2026-06-02 | **Fixed:** sessions 44–49 (2026-06-12 to 2026-06-18)

Real player activation implemented in both agents per AGENT_CONTRACT.md:
- Rust `RandomAgent::act()`: Stage B — 1 decisionRng player pick + 1 actionRng action pick; Move/StandUp/Block/Blitz/Pass/HandOver/Foul all occur organically.
- Java `ParityRunner`: intercepts INIT_SELECTING, picks deterministically from eligible list using `decisionRng`/`actionRng`, sends concrete game commands.

**T3a lineman_vs_lineman 100/100** — all gameplay mechanics verified bit-for-bit identical: dodge, GFI, block dice, armor/injury, foul referee, pass rolls, catch, argue the call.

---

### G-RULE-4: "bone head" / "bone-head" duplicate skill entries in Human Ogre and similar
**Found:** 2026-06-02  
**Severity:** Low — visible in both Java and Rust data, likely handled consistently

Both the Rust JSON (`data/rosters/bb2025/roster_human.json`) and Java XML (`roster_human.xml`) list both `"bone head"` and `"bone-head"` as skills for the Ogre position. Both engines likely apply BoneHead twice (or deduplicate). As long as both handle it identically, this is not a parity risk, but it's a data quality issue.

**Also affects:** Any other positions that use legacy skill name variants.

**Fix needed:** Deduplicate skill entries in roster JSON/XML data files (not an engine fix).

---

## T2 Test Results (as of 2026-06-02)

| Race | 10-seed result | Notes |
|------|---------------|-------|
| amazon | 10/10 ✓ | |
| chaos | 10/10 ✓ | |
| chaos_dwarf | 10/10 ✓ | |
| chaos_pact | 5/5 ✓ | FUMBBL roster, passes |
| dark_elf | 10/10 ✓ | |
| dark_elf_league_fumbbl | 0/5 ✗ | G-RULE-2 (Java load failure) |
| dwarf | 1/10 ✗ | G-RULE-1 (Secret Weapon) |
| elf | 10/10 ✓ | |
| goblin | 0/10 ✗ | G-RULE-1 (Secret Weapon) |
| halfling | 10/10 ✓ | |
| high_elf | 10/10 ✓ | |
| khemri | 10/10 ✓ | |
| khemri_fumbbl | 0/5 ✗ | G-RULE-2 (Java load failure) |
| lizardman | 10/10 ✓ | |
| necromantic | 10/10 ✓ | |
| nippon | 5/5 ✓ | FUMBBL id=5681, passes |
| norse | 10/10 ✓ | |
| nurgle | 10/10 ✓ | |
| ogre | 10/10 ✓ | |
| orc | 100/100 ✓ | T1b baseline |
| renegades | 0/5 ✗ | G-RULE-2 (Java load failure) |
| skaven | 10/10 ✓ | |
| slann | 10/10 ✓ | |
| slann_fumbbl | 0/5 ✗ | G-RULE-2 (Java load failure) |
| undead | 10/10 ✓ | |
| underworld | 10/10 ✓ | |
| vampire | 10/10 ✓ | |
| wood_elf | 10/10 ✓ | |

**Summary (isolated runs, 10 seeds each):** 27/29 races pass; dwarf + goblin fail (G-RULE-1). Slann passes 97/100 but has 3 genuine failures (seeds 2, 18, 74) related to skill-specific Java behavior (G-RULE-5).

**Verbose diagnosis of G-RULE-1 (Dwarf seed 1, --verbose):**
Rust step 17 state string: `h2t10ahomes0,0 b8,0,true pa00..Standing|...|h00:12,7,Standing|...|h10:3,6,Standing`
All 11 home players are Standing. Java has a different hash, consistent with 1 home player (Death Roller) being Reserve after Secret Weapon ejection at end of H1.

---

---

### G-RULE-5: Slann-specific kickoff event divergence (3 seeds/100 fail)
**Found:** 2026-06-02  
**Severity:** Low — only 3% of slann games diverge  

Seeds 2, 18, 74 fail for slann. Two patterns:
- **Seeds 2, 18 (step 16, H2T1)**: Java produces a state that doesn't match "all-11-standing" with any ball position, suggesting some slann player is in a non-Standing state after H1. Likely cause: a kickoff event (QuickSnap or similar) triggers Java to use its static non-seeded `Random` for skill-specific processing of slann positions (Leap, Very Long Legs). This non-determinism causes game RNG divergence.
- **Seed 74 (step 0, H1T1)**: Divergence from the very first step. Likely cause: Java's canonical team setup applies a Big Guy placement restriction for the Kroxigor (can't be placed on Line of Scrimmage), while Rust places all players starting from los[0]. Different initial player positions → different hash immediately.

**Fix needed:**
1. For seeds 2/18: audit which kickoff events trigger Leap-capable player interactions in Java. Match Rust's handling or add the same dice consumption.
2. For seed 74: check if Rust's `place_team_canonical` needs Big Guy restrictions (can't set up on LOS without 3+ other players on LOS).

---

### G-RULE-6: Secret Weapon ejection (**FIXED 2026-06-03**)
**Found:** 2026-06-02  
**Severity:** High — blocks dwarf and goblin from T2

From reading Java's `StepEndTurn.java`:
1. At end of each regular turn: `markPlayedAndSecretWeapons()` marks SW players as "used SW" if their state has `canBeSetUpNextDrive() = true` AND `!= RESERVE`.
   - `canBeSetUpNextDrive()` returns true for: STANDING, MOVING, PRONE, STUNNED, RESERVE, FALLING, BLOCKED.
   - Returns false for: KNOCKED_OUT, BANNED, BADLY_HURT, SERIOUS_INJURY, RIP.
2. At halftime: `removeUsedSecretWeapons()` → for each player with `hasUsedSecretWeapon=true` → if NOT in `REMOVED_FROM_PLAY` (={BANNED, BADLY_HURT, SERIOUS_INJURY, RIP}) → eject (BANNED state).
3. For LRB6 SecretWeapon (penalty=0): automatic ejection, no dice roll.

**What we know from exhaustive analysis:**
1. Java lineman and Java dwarf consume DIFFERENT numbers of game RNG dice between H1 end and H2 scatter
2. Rust lineman and Rust dwarf consume the SAME number (verified with call counter = 11 for both before H2 scatter)
3. Java dwarf H2 scatter uses positions 11+K for some K, while Java lineman uses position 11
4. Exhaustive Python hash search for Java dwarf seed 3 state finds NO match for any combination of:
   - Standard vs compact vs preserve-slot canonical placement
   - All 6 possible deathroller states (Standing/Prone/Injured/Ko/Reserve/Banned)
   - All 26×15 ball positions
5. Java's state string format for BANNED players is unknown (Java doesn't expose state strings in the JSONL)

**What needs to happen next:**
1. Modify Java's `stateHash()` to also write the state STRING to the JSONL file (not just the hash)
2. OR add `System.out.println(stateString)` to the Java ParityRunner at H2T1 boundaries
3. This would reveal: (a) exact canonical positions Java uses, (b) exact deathroller state in Java's hash, (c) exact ball position
4. Once Java's state string is known, implement exact Rust match

**Blocking factor**: The `311ef3e2` hash for Java dwarf seed 3 H2T1 cannot be reproduced by any canonical placement formula I know, suggesting Java uses a placement algorithm or state encoding that differs from what Rust implements.

---

---

### G-RULE-7: Foul referee + pass out-of-range — T3 seeds 75/79/80/81 (**FIXED 2026-06-18**)
**Found:** 2026-06-18  
**Severity:** Medium — blocked seeds 75, 79, 80, 81 from T3 parity

**Root causes (three separate bugs):**

**1. Pass out-of-range (seed 81) — `engine.rs` `Step::DoPass`**  
When the pass target is `deltaX >= 14 || deltaY >= 14`, `passing_distance_bb2025()` returns `None`. Java's `StepInitPassing.executeStep()` never advances `NEXT_STEP` when `findPassingDistance()` returns `null` — the parity runner sends `CLIENT_END_TURN`, consuming 0 dice, ball stays at thrower, turnover. Rust was using `.unwrap_or(PassingDistance::LongBomb)`, rolling 2 dice and treating it as a long-bomb attempt. Fix: early-return from `DoPass` with `game.turnover = true` when distance is `None`.

**2. Coach ban persists for drive (seeds 79/80) — `turn_data.rs` `reset_for_turn()`**  
Java's `TurnData.startTurn()` resets `blitzUsed`, `foulUsed`, `passUsed`, etc. per turn, but does NOT reset `coachBanned` — it persists for the full drive. Seeds 79 and 80 had a prior foul where the argue roll returned 1 (coach banned), and a subsequent foul in the same drive should also skip the argue die. Rust was incorrectly resetting `coach_banned = false` each turn. Fix: remove `coach_banned = false` from `reset_for_turn()`.

**3. Wrong argue-skip condition (seed 1 regression) — `engine.rs` `apply_foul_injury()`**  
A stale `auto_eject = armor_doubles && broke` condition caused Rust to skip the argue die whenever armor doubles broke armor. Java's actual condition (from `StepBribes.askForArgueTheCall()`) is `!isCoachBanned() && !wasCased`, where `wasCased` checks if the fouler is already a casualty (never true in normal play). The argue die should always be offered when the referee spots the foul and the coach is not already banned. Fix: replace `auto_eject` logic with a simple `!game.turn_data().coach_banned` guard.

**Status:** All three fixed. T3 now 100/100.

---

### G-RULE-8: HandOff dice consumption — Rust 4 dice vs Java 3 (T3b seed 6)
**Found:** 2026-06-19  
**Severity:** Medium — blocks amazon_vs_amazon seeds 6+ from passing T3b

At step 151 of seed 6, away_04 performs a HandOff. Both engines have `rng_calls=54` at step start. After the HandOff:
- Java: `rng_calls=57` (consumed 3 dice)
- Rust: `rng_calls=58` (consumed 4 dice)

The 1-die offset causes step 153's home_03 Blitz to read different block dice: Rust gets BothDown → turnover; Java gets a non-BothDown result → game continues.

**Likely root cause:** Rust's HandOff handler consumes 1 extra die somewhere. Candidates:
- Catch skill auto-reroll check consuming an extra die (Amazon Catchers have Catch skill)
- An extra dodge roll for the HandOff movement
- A tackle-zone penalty calculation triggering an extra die

**Fix needed:** Add `FFB_DICE_TRACE=1` for seed 6, compare Java vs Rust dice stacks at the HandOff step to identify which extra die Rust is rolling.

---

## Priority Order for Fixes

1. **G-RULE-8** — HandOff 1-extra-die (blocks amazon T3b seeds 6+; next session priority)
2. **Prereq C** — Custom roster loading (`team_spec.json` in both engines; blocks T3 proper)
3. **G-LOG-1** — Expand state hash (catch silent reroll/ball-carrier bugs)
4. **G-LOG-2/4** — Log sub-turn decisions + dice (mid-turn divergences currently undiagnosable)
5. **G-RULE-2** — FUMBBL race loading (investigate Java side; blocks 4 races from T2 suite)
6. **G-RULE-1** — Secret Weapon end-of-drive (Dwarf + Goblin T2, lower priority now that T3a is done)
7. **G-LOG-5** — `--verbose` state string logging (already implemented, see runner.rs)
