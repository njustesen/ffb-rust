# Parity campaign — HeuristicAgent on dark_elf_league_fumbbl (all three rulesets)

Started 2026-09-02, after dwarf closed (nine gates 100/100, pushed `d0290a9fa`). Same three-loop
procedure (`.claude/commands/amz-iter.md`, MATCHUP=dark_elf_league_fumbbl). Race order is now
strictly alphabetical per the user; rosters are only tested on rulesets where they exist (this
one exists in all three).

## Goal

Nine `PARITY: 100/100` — dark_elf_league_fumbbl v itself, tier 3, seeds 1-100, heuristic
`--heur-classes all`, bb2016/bb2020/bb2025 × scales 0/1.0/1e6, plus coverage recorded.

## Surface

A FUMBBL numeric-id dark elf variant (ids 37734-37738; the make_team alias from the random-agent
era maps the CLI name to the numeric roster). NO stars in any edition. Witch Elves
(Dodge/Frenzy/Jump Up), Assassins (Stab/Shadowing), Runners (Dump-Off), Blitzers (Block), plain
linemen. Everything here was exercised by dark_elf/amazon already — expectation: mostly green
from shared fixes; watch the Dump-Off pin and Stab under the heuristic.

## Baseline (pre-fix probe, seeds 1-20 @1.0)

bb2016 **20/20**, bb2020 **20/20**, bb2025 **20/20** — fully carried by the shared fixes
(dark_elf/dwarf campaigns). Straight to the nine-gate battery + randoms; no engine change since
the dwarf push, so the standing/family regressions from `d0290a9fa` remain valid unless a fix
lands here.

## ITER1 — bb2016 @0 seed 3: the feature-cache stamp is blind to a chain push (FIXED)

Baseline batteries: bb2016 @1.0 100/100, @0 **99/100** (seed 3), @1e6 100/100; bb2020 and
bb2025 100/100 at all three scales; randoms ×3 100/100. One red in the whole matrix.

**Symptom.** Seed 3, hash idx 60 (t5, away): both engines declare away_03 BLOCK on the same
defender, the block dice, the pushback chain (Home1 (16,5)→(16,6), chained Home2
(16,6)→(16,7)) and every die match — but Java FOLLOWS UP and Rust STAYS. Candidate lists at
k=61 differ by 84/15 rows, all downstream of attacker position (Java-free (15,4) vs
Rust-free (16,5)).

**Chase.** FFB_CANDSUM matched through k=60. RFOLLOW probe: identical formula inputs except
`tz_against(target=16,5)`: Rust 1, Java 0 (w 0.15 vs 0.5 → argmax stay vs follow at scale 0).
Java's HeuristicDriver recomputes tzAgainst LIVE at the FOLLOWUP dialog (defender FALLING →
no TZ). Probing inside `Features::build` showed the raster used at the Rust follow-up was
built on the **pre-push board** (home_01 still at 16,5, home_02 standing at 16,6) — yet
`f.stamp == positions_stamp(live)`.

**Root cause.** `positions_stamp` summed weakly-mixed per-player terms
(`h = fnv(id) ^ (x<<8 ^ y); h = h*31 + state; acc += h`). This exact chain push (two players
each +1 in y) produced term deltas that cancelled in the sum: pre-push and post-push boards
had EQUAL stamps (verified arithmetically), so the follow-up prompt reused the pre-push
tackle-zone raster cached at the pushback prompt.

**Fix** (`heuristic_agent.rs`): run each per-player term through a splitmix64 finalizer
(`mix64`) before the order-independent sum — full avalanche makes cancellation ~2⁻⁶⁴. Pure
cache-soundness fix: a sound cache is bit-identical to a fresh build, so no Java twin and no
scoring change. Test `chain_push_changes_the_positions_stamp` reproduces the exact colliding
transition. Seed 3 green after fix.

**Lesson.** A cache keyed by a weak additive hash fails EXACTLY on the correlated small
mutations the game generates (chain pushes move two adjacent players one square each). If a
fresh-build probe and a direct read of the same board disagree, suspect the cache key before
the build.

## Nine gates — GREEN (2026-09-02)

After the mix64 stamp fix: bb2016 / bb2020 / bb2025 × scales 1.0 / 0 / 1e6 all **100/100**;
random controls ×3 editions 100/100 (isolated FFB_PARITY_ROOT); `cargo test -p ffb-engine`
7390/0. Standing + family regressions (amazon, lineman, chaos, chaos_dwarf, chaos_pact,
dark_elf, dwarf ×3 @1.0) re-run after the engine change — results below.
Regressions: amazon, lineman, chaos, chaos_dwarf, chaos_pact, dark_elf, dwarf — each ×3
editions @1.0 — **all 100/100** (21/21 gates). The campaign needed exactly ONE engine fix.
