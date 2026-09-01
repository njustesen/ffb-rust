# Parity campaign — HeuristicAgent on chaos_dwarf vs chaos_dwarf (all three rulesets)

Started 2026-09-01, immediately after chaos closed (nine gates 100/100). Same goal shape, same
three-loop procedure (`.claude/commands/chaos-iter.md` applies with MATCHUP=chaos_dwarf).

## Goal

Nine `PARITY: 100/100` — chaos_dwarf v chaos_dwarf, tier 3, seeds 1-100, heuristic
`--heur-classes all`, bb2016/bb2020/bb2025 × scales 0/1.0/1e6, plus coverage recorded.

New skill surface vs chaos: Stab+Shadowing (Sneaky Stabba), Breathe Fire + Brawler + Disturbing
Presence (Flamesmith), Iron Hard Skin (Blocker), Sprint + Sure Feet (+ bb2025 Unsteady) (Bull
Centaur), and the bb2016 Wild Animal Minotaur again.

## Status

Baselines @1.0 (chaos units carried over): bb2016 **100** / bb2020 99 / bb2025 98.

## ITER1 — Iron Hard Skin discards foul-assist armour modifiers (bb2020 seed 12, bb2025 seed 51)

Java `ArmorModifierFactory.getFoulAssist`: a defender with an unused
`ignoresArmourModifiersFromSkills` skill (Iron Hard Skin) replaces the WHOLE foul-assist set —
positive, NEGATIVE, and the Foul static bonus — with the skill's own marker (whose
appliesToContext is `false`). Rust applied assists unconditionally: two defensive assists turned
armour 10−2=8 < AV10 (held) where Java's 10 ≥ 10 broke and STUNNED the Blocker. Road: the foul
victim's identity was chased through THREE positional-label misreads (state labels are indices,
not ids — the documented trap) and one wrong theory (bb2020 lacking negative assist modifiers —
it has them); a temporary env-gated JFOUL probe in the stock Java engine (reverted after) printed
`assists=-2 mods=Iron Hard Skin/ broken=true` and named the mechanism. Fix in
`injury_type_foul.rs::armour_roll` (assists + FOUL bonus both gated); test
`iron_hard_skin_ignores_foul_assist_modifiers`. Probes kept (FFB_TRACE-gated): RFOUL, RDEFASSIST,
RFOULCAND, RACT ×2, RBOUNCE.

## ITER2 — the declined Hail Mary Pass re-roll must clear the source (bb2025 seed 99)

Rust bb2025 `StepHailMaryPass.handle_command` had NO `UseReRoll` arm: a NoReRoll fell through with
`re_roll_source` still set from the offer, so the re-entry SPENT the team re-roll (r2→r1) and
rolled a fresh HMP d6 + bounce chain — 3 extra dice and a burnt TRR on a declined dialog (Java:
`setReRollSource(null)`, nothing spent). Fix: the standard decline arm; test
`declined_reroll_spends_nothing_and_keeps_the_roll`.

**Gate after ITER1+2**: chaos_dwarf @1.0 ×3 **100/100**; scales: bb2016 100/100, bb2020 100/100,
bb2025 @1e6 100, @0 97 (seeds 9, 24, 41 — new frontier). chaos ×3 + amazon ×3 + lineman ×3
100/100; random chaos_dwarf ×3 100/100; ffb-engine 7374/0.

## ITER3 — the setup prompt offers only players who can be set up

`StepSetup` listed `team.players` verbatim; only `canBeSetUpNextDrive()` players may be OFFERED
(BANNED/KO'd/casualties excluded), matching Java's client and the harness's `placeReserves`.
Necessary but not sufficient for the @0 family — the real killer was ITER4.

## ITER4 — a turn-8 touchdown ending half 1 is NOT the end of the game (bb2025 seeds 9/24/41 @0)

All three @0 reds ended HALF 1 with a turn-8 TOUCHDOWN. Rust's `!fEndGame` approximation for the
Secret-Weapon argue gate was `(new_half && half > 1) || (touchdown && both turn_nr >= 8)` — the
touchdown disjunct ignored the half, so the HALFTIME argue was skipped as "end of game": Java
argued the away Zzharg (roll 4, failed, banned for half 2) while Rust kept him and FIELDED him,
shifting the whole second-half board one setup slot. Fix: `is_end_of_game` helper —
`half > 1 && (new_half || (touchdown && both >= 8))` — with truth-table test.

**The hunt** (recorded because it burned the tools): the divergence chased through the setup
eligibility (ITER3), the argue thresholds (correct), the ban write (correct), and THREE probe
generations (RSWMARK / RSWARGUE / RSWBAN / RETGATE, all kept FFB_TRACE-gated). The RETGATE probe
ended it in one line: `new_half=true td=true half=1 ... eog=true`. Also: Zzharg Madeye is on the
team spec as star nr 2 — his SECRET WEAPON is what makes chaos_dwarf exercise the whole
drive-end argue machinery; `JAVA_TOOL_OPTIONS=-Dffb.parityDebug=true` switches on ParityRunner's
DEBUG prints (JAVA_ARGUE_DIALOG) without editing anything.

**🏁 GATE: ALL NINE chaos_dwarf gates 100/100** (bb2016/bb2020/bb2025 × scales 0/1.0/1e6).
Standing: chaos ×3 + amazon ×3 + lineman ×3 100/100, random chaos_dwarf ×3 100/100,
ffb-engine 7375/0.
