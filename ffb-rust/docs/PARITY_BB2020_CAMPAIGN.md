# PARITY_BB2020_CAMPAIGN.md — drive all 29 bb2020 mirror matchups to 100/100

GOAL: every bb2020 roster 🟢 100/100 (mirror, `--edition bb2020 --tier 3 --seeds 1-100`),
matching what `docs/PARITY_BB2016_CAMPAIGN.md` achieved for bb2016 and the bb2025 matrix before it.

Ground rules (see `docs/PARITY_PROCESS.md`, non-negotiable — identical to the bb2016 campaign):
- **Java is the truth.** Never edit `ffb-java/ffb-common` or `ffb-java/ffb-server` engine code.
  Co-editable: Rust `crates/*`, `random_agent.rs`, harness `ParityRunner.java` (needs a jar rebuild).
- Every Rust change is a **1:1 port** of the corresponding Java class/method. No hacks, no
  parity-only special-cases. Read the Java, port the Java.
- Every fix lands with a colocated `#[cfg(test)]` regression test.
- Verify advance **and** no regression before committing: the roster's own count must drop, and
  `lineman` bb2016 + bb2020 + bb2025 and `cargo test -p ffb-engine` must stay clean. REVERT if
  regressed.
- **Commit AND push after each team goes green with no regressions.**

## Run commands

From `ffb-rust/ffb-rust`:

```bash
cargo build --release -p ffb-parity
./target/release/ffb-parity --home R --away R --edition bb2020 --tier 3 --seeds 1-100 --no-abort
./target/release/ffb-parity --home R --away R --edition bb2020 --tier 3 --seeds N-N     # one seed
```

**NEVER run two parity runs of the SAME matchup concurrently** (even different editions): both write
`parity/<home>_vs_<away>/seed_N_{java,rust}.jsonl` and clobber each other, producing bogus mass
failures. Different matchups are safe in parallel.

Tracing: `FFB_TRACE=1` (RUST_STEP/JSTEP state strings), `FFB_DICE_TRACE=1` (per-die),
`FFB_DICE_DEEP=1` (full `com.fumbbl` stack on JAVA_DIE), `FFB_ACT_TRACE=1` (`JAVA_ACT_PICK`: the
filtered live action list + index + snapshot size), `FFB_DRIVE_TRACE=1` (Rust step order).

### The three tooling lessons carried over from bb2016 — use them from day one
1. **Find the LIVE file with a backtrace, never by reading.** Stale duplicates cost whole iterations
   in the bb2016 campaign (`injury_result` vs `injury`, three Blood Lust impls, bb2016 step files
   that are dead code because the driver's glob import resolves to the bb2025 shared step). Gate
   `std::backtrace::Backtrace::force_capture()` behind an env var in `FieldModel::set_player_state`,
   or in `GameRng::die` firing on an exact `call_count`, and it names the file in ONE run.
2. **A stall = a step returning `Continue` with `prompt.is_none()`.** Probe both driver dispatchers
   for that pair.
3. **When `ParityRunner` abandons something, Rust must abandon it identically** — unhandled acting
   actions DESELECT, unhandled STEPS inject `EndTurn`. Check `handleStep`/`sendConcreteAction` for a
   `default:` arm before porting engine behaviour.

Also: the harness's `playerStateStr` renders BANNED through its `default:` arm as "Reserve", so a
banned player is indistinguishable from an unplaced one in a state string — print `getBase()`.

## Team drafting (done once, 2026-08-14)

bb2020 had **no** team specs, so `make_team` was falling back to the legacy
first-11-by-(quantity,cost) builder. Drafted properly before any parity work:

- `scripts/draft_bb2020_teams.py` — applies the heuristics `docs/TEAM_DRAFTS_BB2025.md` records
  (budget 1.1M; one of every positional incl. a Big Guy within the shared per-team pool; 12+ players;
  2-3 team re-rolls; apothecary when the roster allows and it fits; Dedicated Fans 1→3; remainder
  treasury; jerseys premium-first so jerseys 1-11 are the starters). Validates every spec: 11-16
  players, no overspend, no negative treasury, per-position caps respected.
- Output frozen into `data/teams/bb2020/team_<race>.json` (29 specs).
- `scripts/gen_java_parity_data.py` extended with `"bb2020": "20"`, emitting
  `roster_<race>_bb2020.xml` + `team_<race>_parity20_{home,away}.xml`.
- `runner.rs::java_team_id` now maps `bb2020 → Parity20` (it previously fell through to the legacy
  `Parity` ids, which is why bb2020 was silently running legacy-builder teams).

Two drafting details worth knowing:
- The backfill "lineman" is the cheapest position **with a quantity cap ≥ 6**, not the cheapest
  outright — the renegade rosters' cheapest entry is a quantity-1 Renegade Goblin, which capped
  backfill at one player and produced a 9-man squad.
- Big-Guy detection matches the position **name**, not the id: the FUMBBL rosters use numeric ids
  (`37733` = Renegade Rat Ogre), so an id-based check silently missed them and the shared Big-Guy
  pool went unenforced.

## Status — 🏁 **30/30 GREEN on the first sweep** (2026-08-14)

**No engine fixes were needed.** Once bb2020 was running rule-legal drafted teams against the
matching `Parity20` XML, every roster passed immediately. Scouted at 1-25 (all 0), then the real
gate at 1-100 `--no-abort`:

| | | | | |
|---|---|---|---|---|
| amazon 0 | chaos 0 | chaos_dwarf 0 | chaos_pact 0 | dark_elf 0 |
| dark_elf_league_fumbbl 0 | dwarf 0 | elf 0 | goblin 0 | halfling 0 |
| high_elf 0 | human 0 | khemri 0 | khemri_fumbbl 0 | lizardman 0 |
| necromantic 0 | nippon 0 | norse 0 | nurgle 0 | ogre 0 |
| orc 0 | renegades 0 | skaven 0 | slann 0 | slann_fumbbl 0 |
| undead 0 | underworld 0 | vampire 0 | wood_elf 0 | **lineman 0** |

29 rosters + the `lineman` fixture = **30 matchups, all 100/100**.

### Why it was already green
BB2020 and BB2025 share Java's `mixed/` classes for most behaviour (`RulesCollection.Rules.BB2020`
and `BB2025` both extend `COMMON`, and the bulk of the step/skill code is registered for both), so
the ~40 engine fixes from the bb2025 campaign and the ~20 from bb2016 had already hardened almost
every path bb2020 exercises. The only thing missing was the **data**: no drafted teams and no
`Parity20` XML, which is why nobody could tell.

### Regression evidence for the same commit
- Existing bb2016/bb2025 XML: **zero** modifications from the generator run (`git status` shows all
  261 changed files as new/untracked bb2020 artifacts) — the generator is idempotent per edition, and
  `java_team_id`'s new arm only fires for `bb2020`.
- Re-ran 1-100 in BOTH other editions for lineman, human, goblin, halfling, vampire, ogre,
  renegades: **0 fails everywhere**.
- `cargo test -p ffb-engine` **7115/0**.

### Caveat worth recording
The Java-side bb2020 XML (29 rosters + 58 team files) lives in the `ffb` harness repo, which sits on
a local branch `t3-phase2-wip` with **no upstream configured** — those generated files are untracked
there. Anyone reproducing this needs to re-run `python scripts/gen_java_parity_data.py` after
checking out `ffb-rust`.
