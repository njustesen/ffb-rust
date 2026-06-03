# FFB Parity Plan

A living document tracking parity test coverage between the Java and Rust Blood Bowl engines. Update status cells inline as tiers pass or fail. Both engines must produce **bit-for-bit identical event logs and state hashes** for every seed.

---

## Test Tier Overview

| ID  | Description | Rosters | Edition | Notable mechanics tested |
|-----|-------------|---------|---------|--------------------------|
| T1a | Lineman vs Lineman (baseline) | Fixed, identical | BB2025 | Core game loop, kickoff, halftime |
| T1b | Human vs Orc (fixed races) | Fixed, named | BB2025 | Position variety, starting skills |
| T2  | All 29 races, self-vs-self | Fixed, canonical | BB2025 | All races, SW ejection, argue, Stunty Leeg SW penalty |
| T3  | Random custom TV-matched | Generated, same TV | Random | Extra skills, position diversity, random editions |
| T3i | Random custom TV-imbalanced | Generated, unequal TV | Random | Inducements, petty cash, journeymen, wizard |

For each tier, test at scale: **1 → 10 → 100 → 1000 seeds** before advancing to the next tier.

---

## Current Status

**T1a:** ✓ 100/100 | **T1b:** ✓ 100/100 | **T2:** ✓ 29/29 races × 10/10 seeds

**Next milestone:** T2 × 100 seeds per race, then T3.

**Blocker for T3 being meaningful:** player activation (G-RULE-3). Without real moves and blocks, T3 and T2 test the same mechanics — only kickoff/halftime/ejection logic fires. T3 adds value only after activation is implemented.

---

## Test Matrix

Mark each cell: `✓` (all pass), `~` (partial, note failure count), `✗` (all fail), `—` (not yet attempted).

| Tier | Description | 1 seed | 10 seeds | 100 seeds | 1000 seeds | Notes |
|------|-------------|:------:|:--------:|:---------:|:----------:|-------|
| T1a  | Lineman vs Lineman, BB2025 | ✓ | ✓ | ✓ | — | 100/100 ✓ |
| T1b  | Human vs Orc, BB2025 | ✓ | ✓ | ✓ | — | 100/100 ✓ |
| T2   | All 29 races self-vs-self, BB2025 | ✓ | ✓ | — | — | 29/29 races 10/10 ✓ |
| T3   | Random custom TV-matched, random edition | — | — | — | — | Blocked on G-RULE-3 (activation) + prereq C |
| T3i  | Random custom TV-imbalanced, random edition | — | — | — | — | Blocked on G-RULE-3 + prereqs C, D |

---

## Prerequisites

### A — Complete Random Agent *(no-activation games: ✓ sufficient for T2; activation: pending)*

For the current no-activation parity games (T1a/T1b/T2), both agents are in sync — every dialog encountered is handled identically. The checklist below is for when real player activation is added (needed for T3 to be meaningful).

**Dialogs already handled (no-activation games):**
- `CoinChoice`, `ReceiveChoice`, `KickBall`, `TeamSetup`, `Touchback` — ✓
- `ArgueTheCall` — ✓ (Java: first eligible player; Rust: same)
- `PlayerChoice (DECLARE_DIVING_CATCH)` — ✓ (both decline deterministically)
- `ConfirmSetup`, `EndTurn` — ✓

**Dialogs needed for activation (T3):**
- [ ] `ActivatePlayer` — pick a random player + random legal action
- [ ] `BlockChoice { dice }` — 1 decision RNG call: index into dice list
- [ ] `BlockChoiceProperties` — 1 call: reroll yes/no
- [ ] `ReRollOffer` — 1 call: yes/no
- [ ] `FollowUp` — 1 call: yes/no
- [ ] `Pushback { squares }` — 1 call: square index
- [ ] `HitAndRun`, `TricksterMove` — 1 call each
- [ ] `SkillUse`, `PilingOn` — 1 call each
- [ ] `DefenderAction`, `Interception`, `ApothecaryChoice`, `UseApothecary` — 1 call each
- [ ] `SelectPosition`, `SelectSkill`, `SelectWeather` — 1–2 calls each
- [ ] `BriberyAndCorruption` — 1 call: yes/no
- [ ] `WizardSpell` — 2 calls: x, y
- [ ] `BuyInducements` — 0 calls: always buy nothing (deterministic)
- [ ] `PettyCash`, `Journeymen`, `UseInducement`, `ConcedeGame` — 0 calls each
- [ ] `SwarmingPlayers` — audit Java; likely 1 call

**Used by:** T3, T3i (required before activation is added)

---

### B — Roster Parameterization *(✓ done)*

`--home RACE --away RACE --seeds N-M --no-abort` all work. All 29 races loadable in both engines.

**Used by:** T1b, T2 ✓

---

### C — Custom Roster Support (team_spec.json) *(pending)*

Both engines must accept a shared JSON team spec so generated rosters (random extra skills, TV budgets) can be compared identically.

**Shared format — `parity/specs/home_seedN.json`:**
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

**Rust changes (`crates/ffb-parity/src/`):**
- [ ] Add `--home-spec PATH` / `--away-spec PATH` CLI flags
- [ ] `TeamSpec::from_json(path) -> Team` using existing `ffb-model` types (load extra skills, rerolls, etc.)

**Java changes (`ParityRunner.java`):**
- [ ] Add `--home-spec PATH` / `--away-spec PATH` argument parsing
- [ ] Parse team spec JSON via Jackson; build equivalent in-memory team

**Note:** `scripts/roster_gen.py` already generates valid `team_spec.json` files. The gap is loading them in both engines.

**Used by:** T3, T3i

---

### D — Inducement Purchasing (greedy, deterministic) *(pending)*

For T3i (TV-imbalanced), the weaker team receives petty cash. Both agents must buy identically.

Strategy: **greedy buy-all** — iterate `available` in definition order, buy one of each as long as budget allows, repeat until exhausted. Zero RNG calls.

- [ ] Implement in Rust `ParityAgent` `BuyInducements` handler
- [ ] Implement identically in Java `ParityRunner`
- [ ] Test with hand-crafted TV-imbalanced spec

**Used by:** T3i

---

### E — Python Orchestration Scripts *(✓ done)*

All scripts exist and are functional:
- `scripts/parity_run.py` — runs Java + Rust, compares, writes results
- `scripts/parity_compare.py` — diffs two JSONL logs
- `scripts/parity_report.py` — generates results table
- `scripts/roster_gen.py` — generates deterministic team specs

```bash
python scripts/roster_gen.py --seed 42 --home-race human --away-race orc
python scripts/parity_run.py --tier T2 --seeds 1-10
```

**Used by:** T2 ✓, T3, T3i

---

### F — Roster Generator *(✓ done)*

`scripts/roster_gen.py` generates valid, deterministic team specs from a seed. Tested functional.

**Used by:** T3, T3i

---

## How to Run

### T1a — lineman baseline (100 seeds)
```bash
./target/release/ffb-parity.exe --seeds 1-100
```

### T1b — fixed race pair
```bash
./target/release/ffb-parity.exe --home human --away orc --seeds 1-100
```

### T2 — all races (10 seeds each)
```bash
python scripts/parity_run.py --tier T2 --seeds 1-10
# or one race at a time:
./target/release/ffb-parity.exe --home goblin --away goblin --seeds 1-10
```

### T3 — custom rosters (after prereqs C + activation)
```bash
python scripts/roster_gen.py --seed $S --edition random
python scripts/parity_run.py --tier T3 --seeds 1-100 --custom-roster
```

### T3i — inducement testing (after all prereqs)
```bash
python scripts/parity_run.py --tier T3i --seeds 1-100 --custom-roster --tv-imbalance 200000
```

---

## Results Log

| Date | Tier | Seeds | Passed | First Failure | Notes |
|------|------|-------|--------|---------------|-------|
| 2026-06-02 | T1a | 1–100 | 100/100 | — | Lineman vs Lineman, BB2025 |
| 2026-06-02 | T1b | 1–10 | 10/10 | — | Human vs Orc, BB2025 |
| 2026-06-02 | T1b | 1–100 | 100/100 | — | Human vs Orc, BB2025 |
| 2026-06-02 | T2 | 1–10 | 27/29 races | dwarf, goblin | SW ejection (G-RULE-6); slann/FUMBBL races also fixed |
| 2026-06-03 | T2 | 1–10 | 29/29 races | — | G-RULE-6 fixed: SW penalty rolls (bombardier) + argue RNG alignment |
| 2026-06-03 | T1a | 1–100 | 100/100 | — | Re-verified after G-RULE-6 fix |

---

## What's Left Before T3

### Must-have
1. **G-RULE-3: Player activation** — both agents must activate players, make moves, blocks, passes. Without this, T3 tests the same mechanics as T2 (only kickoff logic). This is the largest remaining piece.
2. **Prereq C: Custom roster loading** — both engines accept `team_spec.json` with extra skills and custom TV. Needed to test skill interactions.

### Nice-to-have (T3 quality)
3. **G-LOG-1: Expand state hash** — add reroll counts + ball carrier to hash so reroll-consumption bugs aren't silent.
4. **G-LOG-2: Sub-turn decision logging** — log every block/push/follow-up decision so mid-turn divergences are diagnosable without print statements.

### T3i only
5. **Prereq D: Inducement purchasing** — greedy buy-all strategy, deterministic, zero RNG calls.

---

## Reference

- Rust parity runner: `crates/ffb-parity/src/`
- Java parity runner: `ffb-java/ffb/ffb-ai/src/main/java/com/fumbbl/ffb/ai/parity/ParityRunner.java`
- Roster definitions: `data/rosters/{bb2016,bb2020,bb2025}/`
- Parity JSONL logs: `parity/{home}_vs_{away}/seed_N_{java,rust}.jsonl`
- State hash: `crates/ffb-model/src/util/state_hash.rs`
- `AgentPrompt` variants: `crates/ffb-model/src/prompts/agent_prompt.rs`
