# 30_skills.md — skill-behaviour hook system (Phase E; STUB)

Skill-less linemen need **zero** of this (Phase D ports 0 behaviours). This file is a stub for
Phase E (full BB2025 positionals/skills); per-behaviour entries are authored then, in the same
shape as `20_steps/` entries.

## Java mechanism (to mirror 1:1 in Phase E)
- Skills attach behaviour via `skillbehaviour/` classes. A `SkillBehaviour` registers
  `StepModifier`s for specific steps: `registerModifier(new StepModifier<StepX, StepX.StepState>(){…})`
  with `handleCommandHook` / `handleExecuteStepHook` (e.g. `bb2025/SneakyGitBehaviour` modifies
  `StepReferee` and `StepEjectPlayer`).
- A step exposes hook points; at runtime, when a player with the skill is involved, its
  modifier's hook runs inside the step's `handleCommand`/`executeStep` (see
  `AbstractStep.handleSkillCommand` and the `StepModifier` registry).
- Sequences also `insertHooks(HookPoint.X, …)` to inject whole hook steps (e.g. `PASS_INTERCEPT`).

## BB2025 surface (Phase E scope)
~40 behaviour classes under `ffb-server/.../skillbehaviour/bb2025` (+ `common`, `mixed`).
Examples to port: Block, Dodge, Catch, SureHands, Tackle, Guard, MightyBlow, Frenzy,
Juggernaut, StripBall, Wrestle, StandFirm, SideStep, Fend, Grab, Horns, Dauntless, ThickSkull,
Regeneration, Loner, Pro, Leader, DirtyPlayer, SneakyGit, …

## Rust representation (Phase E decision, behaviour-neutral)
Mirror the modifier-registry idea: each `Step` enum variant declares the hook points it fires;
a skill registry maps `SkillId` → handlers keyed by hook point. Keep the firing order and
dice consumption identical to Java (the modifiers run at precise points in a step). Per-skill
characterization tests per `TESTING.md`.

## Status
- [ ] Hook framework ported
- [ ] Per-behaviour entries authored
- [ ] BB2025 26-race ×100 tier-3 parity (the Phase E gate)
