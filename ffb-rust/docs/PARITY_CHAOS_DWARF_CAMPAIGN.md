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
