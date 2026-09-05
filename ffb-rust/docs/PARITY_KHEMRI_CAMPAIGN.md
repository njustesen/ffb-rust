# Khemri — heuristic-agent parity campaign

**Goal**: khemri v khemri, HeuristicAgent both sides, per-step state-hash parity 100/100, seeds
1-100, tier 3, editions bb2016/bb2020/bb2025 × scales 1.0/0/1e6 (nine gates), plus random controls
and the standing regression set. Procedure: `.claude/commands/amz-iter.md` with `MATCHUP=khemri`.
Started 2026-09-05, after goblin (`fc58b0e1c`), halfling (`e6571447c`), high_elf (`77d99aacb`) and
human (`32d858746`).

## Surface

The parity team is 12 players in every edition — 4 Tomb Guardian, 2 Blitz-Ra/Anointed Blitzer,
2 Thro-Ra/Anointed Thrower, 4 Skeleton — and the standout is that **every single player has
Regeneration**. There are **no negatraits and no Throw Team-Mate** on this roster, so none of the
Select-sequence machinery that dominated goblin/halfling/human is in play. What is in play:

- **Regeneration ×12** — fires on every casualty, both teams, all game. The apothecary/Regeneration
  ordering (`canUseApo` → Regeneration) is exercised far harder than on any prior race.
- **Decay** (Tomb Guardian, all editions) — **two** casualty rolls, take the worse. Pairs with
  Regeneration on the same player, so the injury chain is Decay → casualty ×2 → apo? → Regeneration.
- **Thick Skull** — Skeleton in all editions; bb2020/bb2025 also put it on Thrower/Blitzer. Turns a
  KO'd result into Stunned on an armour break, i.e. it changes the *injury* branch taken.
- **Brawler — bb2025 ONLY** (Tomb Guardian). Re-rolls a single Both Down block die. This is an
  edition-gated block-dice re-roll, exactly the shape that the human ITER1 Dodge/Tackle fault took
  (a re-roll source that must or must not survive a filter), and it exists on **no other closed
  race in this sweep**. First place to look if bb2025 diverges and the other two editions do not.
- Block / Pass / Sure Hands — all well covered by closed races.

Prediction: reds, if any, cluster in the **injury/casualty resolution chain** (where the state hash
sees the result but not the intermediate rolls) rather than in action selection.

## Baseline (2026-09-05, measured on `32d858746`, nine gates, seeds 1-100 tier 3)

| edition | @1.0 | @0 | @1e6 |
|---|---|---|---|
| bb2016 | **100** | **100** | **100** |
| bb2020 | **100** | **100** | **100** |
| bb2025 | **100** | **100** | **100** |

**All nine gates green at baseline — zero reds. The first race in the sweep to open fully closed,
and it required no engine change at all.**

Random controls (`FFB_PARITY_ROOT=parity_random`, `--agent random`, khemri v khemri, seeds 1-100):
bb2016 **100/100**, bb2020 **100/100**, bb2025 **100/100**. DoD item 2 fully met (unlike high_elf).

## Why it opened green — and why that is not vacuous

A 9/9 baseline is exactly the shape a *vacuous* green takes, so it was checked rather than assumed:

- **The games are full length.** bb2025 @1.0 `rust_total=38.3s` against human's `40.3s` on the same
  binary and seed count. 88,588 GameEvents over the 100 bb2025 seeds, 12,103 Move / 978 BlitzMove /
  675 Block / 304 Foul / 138 PassMove / 59 HandOverMove declarations, 6 touchdowns.
- **The race's headline skill really fires.** `regenerationRoll` appears **108 / 96 / 100** times
  across the bb2016 / bb2020 / bb2025 runs — Regeneration is exercised *and* evented, on top of
  2,536 injury events in bb2025.
- **The substantive reason for the green** is that khemri carries **no negatraits and no Throw
  Team-Mate**. Nearly every fix in goblin, halfling and human was in the Select-sequence activation
  block or the TTM landing chain; none of that machinery is on this roster. What khemri *does*
  stress — the injury/casualty/Regeneration chain — was already hardened by earlier campaigns, and
  `injury.rs` still carries the comments from the Khemri Tomb Guardian Decay bug fixed then.

Because **no code changed this campaign**, the standing closed-roster regression set is satisfied by
construction: there is no delta that could regress it, and HEAD is the same `32d858746` on which the
human nine gates, the human random controls and `ffb-engine` 7418/0 + `ffb-model` 2802/0 were all
independently re-measured.

## Coverage, honestly bucketed

Harvested ×3 → `docs/EVENT_COVERAGE_khemri_bb2016.md`, `_bb2020.md`, `_bb2025.md`.

**Correction (same day).** The first version of this section said the empty "Skill uses / re-rolls
seen" block was BACKLOG §E6 (skill uses not evented). **That attribution was wrong**, and the
challenge that produced the re-check was right. `GameEvent::SkillUse` *is* emitted, from five sites
— block-result Dodge, Dump Off, Horns, Juggernaut, Wrestle — and dark_elf bb2025 logs **343** of
them. The section was empty because `harvest_coverage.sh` filtered the events through
`"skill[A-Za-z_]*":"[^"]+"`, which requires a **quoted** value, while the event serialises
`"skill_id":127` as a bare number. It therefore matched nothing **in all 26 coverage docs ever
harvested**, for every closed race, no matter what actually fired. Fixed by
`scripts/skill_use_report.py` (resolves ids through the `SkillId` enum's declaration order),
validated on a known-positive (dark_elf seed 1 → `2 Dodge used=true, 1 DumpOff used=false`;
127=Dodge, 5=DumpOff) and a known-zero (khemri) before being trusted. khemri was then re-harvested
×3 on the fixed script — the three runs above are the fixed ones. `GameEvent::ReRoll` having no emit
site at all *is* real, and that half of §E6 stands.

**The khemri conclusion is unchanged**: it genuinely logs zero `skillUse` events, because it carries
none of the five emitting skills. But it now rests on a verified measurement rather than a
mis-cited one. Per-skill buckets:

| skill | bucket | evidence |
|---|---|---|
| **Regeneration** (all 12) | exercised **+ evented** | 108/96/100 `regenerationRoll` events |
| **Decay** (Tomb Guardian) | exercised, **un-evented** | live in `injury.rs:562-577`; the second casualty roll is correctly BB2016-gated (`mixed/Decay` registers only `cancelsAllowsRaisingLineman` in BB2020/25). Parity + 2,536 injury events are the proof. |
| **Thick Skull** | exercised, un-evented | on 4-12 players per edition; changes the injury branch, covered by the injury events + parity |
| Block / Pass / Sure Hands | exercised, un-evented | well covered by closed races |
| **Brawler** (bb2025 Tomb Guardian) | **agent never creates it — UNTESTED** | see below |

### Brawler is unverified, and this is a real gap

`re_roll_source` becomes `"Brawler"` **only** via `Action::UseBrawler`
(`bb2025/block/step_block_roll.rs:83`). Nothing in `crates/ffb-engine/src/agent/` ever emits that
action, and `ParityRunner.java` contains **zero** occurrences of `BRAWLER` — so neither the
heuristic nor the random agent can reach the implicit-re-roll path at
`step_block_roll.rs:213`, in any of the 900 khemri games run here.

This is consistent on *both* sides, which is why parity is genuine rather than papered over: the two
engines agree because neither one ever takes the branch. But it means **bb2025 Brawler carries no
parity evidence from this campaign**. It is the same shape as the Throw-Team-Mate finding
(`parity_tier_ttm.md`), where a mechanic sat dead across thousands of games until the harness was
made to declare it — and switching that one on exposed ten engine bugs. Brawler exists on no other
closed race in this sweep, so nothing else will incidentally cover it.

**Recorded as a follow-up, not silently counted as covered.** Closing it means teaching both agents
to answer the Brawler re-roll offer — a harness + agent change on both sides, i.e. its own iteration.

**🏁 khemri CLOSED at baseline.** Nine gates + three random controls, no engine change required.
Frontier empty; one carry-over (Brawler) logged above.
