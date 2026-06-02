# FFB Parity Plan

A living document tracking parity test coverage between the Java and Rust Blood Bowl engines. Update status cells inline as tiers pass or fail. Both engines must produce **bit-for-bit identical event logs and state hashes** for every seed.

---

## Test Tier Overview

| ID  | Description | Rosters | Edition | Notable mechanics tested |
|-----|-------------|---------|---------|--------------------------|
| T1a | Lineman vs Lineman (baseline) | Fixed, identical | BB2020 | Core game loop, blocks, movement |
| T1b | Human vs Orc (fixed races) | Fixed, named | BB2020 | Position variety, starting skills |
| T2  | Random race pair | Fixed, random races | Random (BB2016/BB2020/BB2025) | All 29 races, edition rule variations |
| T3  | Random custom TV-matched | Generated, same TV | Random | Custom skills, position diversity |
| T3i | Random custom TV-imbalanced | Generated, unequal TV | Random | Inducements, petty cash, journeymen, wizard |

For each tier, test at scale: **1 → 10 → 100 → 1000 seeds** before advancing to the next tier.

---

## Test Matrix

Mark each cell: `✓` (all pass), `~` (partial, note failure count), `✗` (all fail), `—` (not yet attempted).

| Tier | Description | 1 seed | 10 seeds | 100 seeds | 1000 seeds | Notes |
|------|-------------|:------:|:--------:|:---------:|:----------:|-------|
| T1a  | Lineman vs Lineman, BB2025 | ✓ | ✓ | ✓ | — | 100/100 ✓ |
| T1b  | Human vs Orc, BB2025 | ✓ | ✓ | ✓ | — | 100/100 ✓ |
| T2   | All 29 races self-vs-self, BB2025 | ✓ | ✓ | — | — | 29/29 races 10/10 ✓ (G-RULE-6 fixed: SW penalty rolls + argue RNG sync) |
| T3   | Random custom TV-matched, random edition | — | — | — | — | Needs prereq A + B + C |
| T3i  | Random custom TV-imbalanced, random edition | — | — | — | — | Needs prereq A + B + C + D |

---

## Prerequisites

Each prerequisite must be completed (and T1a re-verified) before the tiers that depend on it are attempted.

### A — Complete Random Agent (both engines) *(Rust: partial ✓ — inducement no-ops added; Java: pending)*

The `ParityAgent` (Rust: `crates/ffb-parity/src/runner.rs`) and Java's `ParityRunner` must handle **every `AgentPrompt` variant** with identical RNG consumption. Both use `Xoshiro256StarStar(seed ^ 0xDEADBEEFCAFE0001)` as `decisionRng`. A missing or extra `rng.next_u64()` call in one engine corrupts the entire downstream decision sequence.

**Implementation checklist:**

- [ ] Audit `RandomStrategy.java` — enumerate which dialogs call `decisionRng.nextLong()` and how many times
- [ ] Verify Rust and Java are already in sync for T1a prompts (ActivatePlayer, BlockChoice, Pushback, FollowUp, ReRollOffer, SkillUse)
- [ ] Add missing handlers to Rust `ParityAgent`:
  - [ ] `ActivatePlayer` — N RNG calls: random player + random action from that player's legal list
  - [ ] `BlockChoice { dice }` — 1 call: `rng.next_u64() % dice.len()` as die index
  - [ ] `BlockChoiceProperties` — 1 call: random reroll yes/no
  - [ ] `ReRollOffer` — 1 call: random yes/no
  - [ ] `FollowUp` — 1 call: random yes/no
  - [ ] `Pushback { squares }` — 1 call: `rng.next_u64() % squares.len()` for square index
  - [ ] `HitAndRun { squares }` — 1 call: use or decline; if use, 1 call for square index
  - [ ] `TricksterMove { squares }` — 1 call: square index
  - [ ] `SkillUse` / `PilingOn` — 1 call each: random yes/no
  - [ ] `DefenderAction { actions }` — 1 call: action index
  - [ ] `Interception` — 1 call: random yes/no
  - [ ] `ApothecaryChoice` / `UseApothecary` — 1 call: random heal yes/no
  - [ ] `SelectPosition { available_positions }` — 1 call: index
  - [ ] `SelectSkill { available }` — 2 calls: category index + skill index
  - [ ] `SelectWeather { options }` — 1 call: option index
  - [ ] `ArgueTheCall` — 1 call: random yes/no → `UseReRoll`
  - [ ] `BriberyAndCorruption` — 1 call: random yes/no → `UseBribe`
  - [ ] `WizardSpell` — 2 calls: random `x` in `[0,25]`, random `y` in `[1,14]`
  - [ ] `BuyInducements { available, budget }` — 0 calls: **always buy nothing** (deterministic, no RNG needed; T3i can add random selection later)
  - [ ] `BuyPrayersAndInducements` — 0 calls: same as above
  - [ ] `PettyCash` — 0 calls: always Confirm (accept petty cash, no decision)
  - [ ] `Journeymen` — 0 calls: always Confirm (always accept journeymen)
  - [ ] `UseInducement` — 0 calls: always Confirm (use the inducement when offered)
  - [ ] `ConcedeGame` — 0 calls: always decline (UseReRoll { use_reroll: false })
  - [ ] `SwarmingPlayers { eligible_players }` — audit Java; likely 1 call: random player index
  - [ ] `SetupError`, `ConfirmEndAction`, `InformationOkay`, `StartGame`, `GameStatistics` — 0 calls: Confirm
- [ ] Add matching handlers to Java `ParityRunner` for any dialogs currently delegating incorrectly to `RandomStrategy` (especially inducement dialogs: `BUY_INDUCEMENTS`, `WIZARD`, `ARGUE_THE_CALL`, `BRIBERY_AND_CORRUPTION`)
- [ ] Re-run T1a (100 seeds) — must still be 100/100 ✓

**Used by:** T1b, T2, T3, T3i

---

### B — Roster Parameterization *(✓ implemented)*

Both engines must accept an arbitrary (race, edition) team pair as input, not just the hardcoded lineman teams.

**Rust (`crates/ffb-parity/src/main.rs`, `runner.rs`):**
- [ ] Add `--home ROSTER_ID` / `--away ROSTER_ID` CLI flags (e.g. `--home human --away orc`)
- [ ] Add `--edition bb2016|bb2020|bb2025` flag (default `bb2020`)
- [ ] `make_team()` in `runner.rs`: load the named roster from `ffb-model`'s embedded JSON and build a `Team` with the roster's canonical positions and skills
- [ ] Short-name aliases matching Java: `human` → human roster, `orc` → orc roster, etc.

**Java** already accepts team IDs as positional arguments. No change needed for fixed rosters; verify the aliases match.

- [ ] Re-run T1a — still 100/100 ✓
- [ ] Run T1b (Human vs Orc, 1 seed) — confirm parity holds

**Used by:** T1b, T2, T3, T3i

---

### C — Custom Roster Support (team_spec.json)

Both engines must be able to accept an externally-generated team definition file so that generated rosters (random skills, TV budgets) can be shared between them.

**Shared format — `team_spec.json`:**
```json
{
  "id": "custom_home_seed42",
  "name": "Custom Home",
  "race": "human",
  "edition": "bb2020",
  "rerolls": 2,
  "apothecary": true,
  "team_value": 1100000,
  "players": [
    { "nr": 1, "name": "P1", "position": "human.lineman", "extra_skills": ["Block", "Dodge"] },
    { "nr": 2, "name": "P2", "position": "human.blitzer" }
  ]
}
```

**Rust changes:**
- [ ] Add `--home-spec PATH` / `--away-spec PATH` CLI flags
- [ ] `TeamSpec::from_json(path) -> Team` using existing `ffb-model` types

**Java changes:**
- [ ] Add `--home-spec PATH` / `--away-spec PATH` to `ParityRunner`
- [ ] Parse `team_spec.json` via Jackson; build equivalent in-memory team

- [ ] Verify T3 with one hand-crafted spec: `cargo run -p ffb-parity -- --home-spec tests/human_basic.json --away-spec tests/orc_basic.json`

**Used by:** T3, T3i

---

### D — Inducement Purchasing (random, matched)

For T3i (TV-imbalanced teams), the weaker team receives petty cash / inducement budget. Both agents must buy the same set of inducements from the same budget.

Strategy: **greedy buy-all** — iterate `available` in definition order, buy one of each as long as budget allows, then repeat until budget is exhausted. This is deterministic (zero RNG calls), so no RNG alignment is needed.

- [ ] Implement greedy buy-all in `BuyInducements` handler (Rust `ParityAgent`)
- [ ] Implement identical greedy buy-all in Java `ParityRunner`'s inducement dialog handler
- [ ] Test with a hand-crafted TV-imbalanced team spec (home TV 1000k, away TV 1200k → away gets petty cash)
- [ ] Re-run T1a — still 100/100 ✓

**Used by:** T3i

---

### E — Python Orchestration Scripts *(✓ implemented)*

Shell scripts that run both engines, compare logs, and report results.

**`scripts/parity_run.py`** — orchestrate a parity run:
```
python scripts/parity_run.py --tier T1b --seeds 1-10
python scripts/parity_run.py --tier T2  --seeds 1-100 --parallel 8
python scripts/parity_run.py --tier T3  --seeds 1-10  --custom-roster
```
- Invokes Java subprocess: `java -cp $PARITY_CP com.fumbbl.ffb.ai.parity.ParityRunner ...`
- Invokes Rust binary: `cargo run --release -p ffb-parity -- ...` (or prebuilt)
- Collects JSONL output in `parity/results/TIER/`
- Calls `parity_compare.py` per seed
- Writes `parity/results/TIER/summary.json`

**`scripts/parity_compare.py`** — compare two JSONL logs:
```
python scripts/parity_compare.py parity/seed_1_java.jsonl parity/seed_1_rust.jsonl
```
- Compares event-by-event on `state_hash`, `chosen`, `turn`, `half`, `active`
- Reports first divergence (turn, half, side, expected vs actual hash)
- Exit 0 on match, exit 1 on divergence

**`scripts/parity_report.py`** — generate results table:
```
python scripts/parity_report.py
```
- Reads all `parity/results/*/summary.json`
- Prints a Markdown table suitable for pasting into the **Test Matrix** above
- Optionally regenerates `progress.html`

- [ ] `scripts/parity_run.py` created and functional
- [ ] `scripts/parity_compare.py` created and functional
- [ ] `scripts/parity_report.py` created and functional

**Used by:** T2, T3, T3i (T1a/T1b can use the existing Rust parity runner directly)

---

### F — Roster Generator *(✓ implemented)*

A script that generates valid, deterministic `team_spec.json` files from a seed.

**`scripts/roster_gen.py`:**
```
python scripts/roster_gen.py --seed 42 --edition random --race random
python scripts/roster_gen.py --seed 42 --edition bb2020 --race human --tv 1100000
python scripts/roster_gen.py --seed 42 --tier T3i   # generates mismatched TV pair
```
- Reads `data/rosters/` JSON to know legal positions, costs, and skill categories
- Edition chosen randomly from `[bb2016, bb2020, bb2025]` when `--edition random`
- Race chosen randomly from 29 when `--race random`
- Fills up to 16 players within TV budget, weighted toward cheaper positions
- Randomly assigns 0–2 extra skills per player from their legal normal/double categories
- Outputs `parity/specs/home_seed{N}.json` and `parity/specs/away_seed{N}.json`

- [ ] `scripts/roster_gen.py` created
- [ ] Generated spec can be loaded by both engines without error
- [ ] Two runs with the same `--seed` produce identical files

**Used by:** T2 (selects race pair), T3, T3i

---

## How to Run

### Existing baseline (T1a)
```bash
cargo run --release -p ffb-parity
# → runs seeds 1–100, lineman vs lineman, BB2020
# → writes parity/seed_N_{java,rust}.jsonl
# → exits 0 if all 100 match
```

### T1b — fixed race (after prereq B)
```bash
cargo run --release -p ffb-parity -- --home human --away orc --seeds 1-10
```

### T2 — random race pair (after prereqs A, B, E, F)
```bash
python scripts/parity_run.py --tier T2 --seeds 1-100 --parallel 8
python scripts/parity_report.py
```

### T3 — custom TV-matched rosters (after prereqs A, B, C, E, F)
```bash
python scripts/roster_gen.py --seed $S --edition random --race random
python scripts/parity_run.py --tier T3 --seeds 1-100 --custom-roster
```

### T3i — inducement testing (after all prereqs)
```bash
python scripts/parity_run.py --tier T3i --seeds 1-100 --custom-roster --tv-imbalance 200000
```

---

## Results Log

Record every run here. Format: `YYYY-MM-DD | TIER | scale | pass/total | first failure (if any)`.

| Date | Tier | Seeds | Passed | First Failure | Notes |
|------|------|-------|--------|---------------|-------|
| 2026-06-02 | T1a | 1–100 | 100/100 | — | Lineman vs Lineman, BB2025 |
| 2026-06-02 | T1b | 1–10 | 10/10 | — | Human vs Orc, BB2025 |
| 2026-06-02 | T1b | 1–100 | 100/100 | — | Human vs Orc, BB2025 |
| 2026-06-02 | T2 | 1–10 | 27/29 races (10/10) | dwarf (G-RULE-6), goblin (G-RULE-6) | All 29 races isolated; slann/slann_fumbbl fixed; human added |
| 2026-06-02 | T2 | 1–100 | slann: 100/100 | — | Java DC fix (DECLARE_DIVING_CATCH now declines deterministically) |
| 2026-06-02 | T2 | 1–100 | goblin: 0/100 | all seeds | Secret Weapon ejection (G-RULE-6) |
| 2026-06-03 | T2 | 1–10 | 29/29 races | — | G-RULE-6 fixed: SW penalty rolls (bombardier) + argue RNG |
| — | T1a | 1–1000 | — | — | Not yet run |
| — | T2 | 1–1 | — | — | |
| — | T3 | 1–1 | — | — | |
| — | T3i | 1–1 | — | — | |

---

## Reference

- Rust parity runner: `crates/ffb-parity/src/`
- Java parity runner: `../ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/parity/ParityRunner.java`
- Java random strategy: `../ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/RandomStrategy.java`
- Roster definitions: `data/rosters/{bb2016,bb2020,bb2025}/`
- Parity JSONL logs: `parity/`
- State hash: `crates/ffb-model/src/util/state_hash.rs`
- `AgentPrompt` variants: `crates/ffb-model/src/prompts/agent_prompt.rs`
