# Phase AAF: Skill-Behaviour Hook Audit

Audit-only. No code changed. Scope: every file under
`crates/ffb-engine/src/skill_behaviour/{bb2016,bb2020,bb2025,mixed,common}/`
(excluding `mod.rs`, `registry.rs`, `dispatch.rs`, `step_hook.rs`).

## Key mechanism finding (read this first)

`registry.rs` registration and `TODO(hook-infra)` presence are **not** reliable
signals of "done" on their own. Three independent things determine whether a
skill's hook is real:

1. Does the file have `impl StepModifierTrait for XxxModifier` with a real body
   (not the dead default `execute_step_hook(&self, _game) -> bool { false }`)?
2. Is that modifier actually reachable — i.e. does the corresponding
   `step_xxx.rs` under `crates/ffb-engine/src/step/` call
   `dispatch::execute_step_hooks(...)`? **Only 7 step files do, today:**
   `step_horns.rs`, `step_bone_head.rs`, `step_really_stupid.rs`,
   `step_wild_animal.rs`, `step_pushback.rs`, `step_blood_lust.rs`,
   `step_take_root.rs` → StepIds `Horns, BoneHead, ReallyStupid, WildAnimal,
   Pushback, BloodLust, TakeRoot`.
3. **Separately**, many skills' actual game logic is implemented **directly
   inline** in their `step_xxx.rs` file (has_skill checks, dialogs, rolls) with
   *no* dependency on the `skill_behaviour/` mechanism at all — e.g.
   `step_wrestle.rs`, `step_dump_off.rs`, `step_stab.rs`, `step_bombardier.rs`
   directly reference `SkillId::Wrestle/DumpOff/Stab/Bombardier` and implement
   the roll/dialog logic themselves. For these, the matching
   `skill_behaviour/*.rs` file is **dead/duplicate code carrying a stale
   `TODO(hook-infra)` comment** — functionally done elsewhere, safe to clean up,
   not a real gap for this mechanism.
4. A further wrinkle discovered mid-audit: `registry.rs`'s `build_bb2016()` /
   `build_bb2020()` mostly **reuse the `bb2025::` module** for skills whose
   logic is edition-identical (Animosity, BloodLust, Bombardier, DumpOff,
   FoulAppearance, JumpUp, Shadowing, Stab, TakeRoot, Tentacles, Wrestle). That
   means the bb2016/bb2020-namespaced files of the *same name* are **never
   imported anywhere** — pure dead code, regardless of TODO status.

Net effect: the real gap surface is smaller than "77 files with TODO markers"
suggests, but concentrated in specific places — see tables below.

## Summary counts

- Files audited: 127 (bb2016: 33, bb2020: 38, bb2025: 41, mixed: 14, common: 1)
- Files carrying `TODO(hook-infra)`: 77
- Of those 77:
  - **False positive / dead duplicate / trivial stat-only skill** (no work
    needed, safe to delete stale comment): ~40
  - **DONE (direct-in-step)** — real logic already lives in the step file,
    skill_behaviour port is dead/duplicate, some with known internal stubs
    (dialogs auto-decline, tackler-search stubbed): ~14
  - **GENUINE GAP** — no real implementation exists anywhere yet (neither
    wired through `dispatch::execute_step_hooks` nor directly in the step
    file): **~20** (see table below; a handful more are ambiguous, flagged as
    "needs verification")
- Genuinely wired end-to-end via the mechanism (`StepModifierTrait` + step
  calls `dispatch::execute_step_hooks`): Horns, BoneHead (×3 editions),
  ReallyStupid (×3), WildAnimal (×2), BloodLust, TakeRoot, Dauntless(mixed,
  registered but see caveat below), plus Pushback-targeting EyeGouge/Grab/
  Sidestep/StandFirm (plumbing wired, but see note — EyeGouge's own hook body
  is still a stub).
- `mixed/abstract_dodging_behaviour.rs` has no TODO marker and is **fully
  wired** (StepBlockDodge calls dispatch) — correctly excluded from the gap
  list.

## Genuine gaps, grouped by target Java step class

| Target step (Java `StepXxx.StepState`) | Skill(s) | Rust file(s) | Java file(s) | What's missing | Size |
|---|---|---|---|---|---|
| **StepPushback** (largest cluster) | Grab | `bb2016/grab_behaviour.rs`, `bb2020/grab_behaviour.rs` | `GrabBehaviour.java` (all editions) | Attacker picks the pushback square when Grab is active; no `SkillId::Grab` check exists in any `step_pushback.rs` variant (bb2016/bb2020/bb2025 are 3 separate, divergent files) | medium |
| StepPushback | SideStep | `bb2016/side_step_behaviour.rs`, `bb2020/side_step_behaviour.rs` | `SideStepBehaviour.java` | Defender cancels the pushback unless attacker has a countering skill; no real check in any `step_pushback.rs` | medium |
| StepPushback | MonstrousMouth | `bb2016/monstrous_mouth_behaviour.rs`, `bb2020/monstrous_mouth_behaviour.rs`, `bb2025/monstrous_mouth_behaviour.rs` | `MonstrousMouthBehaviour.java` | Chomped-defender: forces push, clears pushback stack, blocks ball-strip. (Rust's own doc comment wrongly guesses this mirrors Catch — it doesn't.) Not implemented anywhere | small |
| StepPushback | StandFirm (bb2016) | `bb2016/stand_firm_behaviour.rs` | `StandFirmBehaviour.java` | bb2016 reuses bb2025's `step_pushback.rs`, which has **zero** StandFirm handling. (bb2020's own `step_pushback.rs` has a partial `has_skill` check — likely incomplete vs. the 87-line Java dialog/reroll logic; flag for follow-up verification rather than a hard gap) | medium |
| StepPushback | EyeGouge | `bb2025/eye_gouge_behaviour.rs` | `EyeGougeBehaviour.java` | Plumbing (registration + dispatch call) already works since Pushback is wired, but `handle_execute_step` body is a no-op stub — sets defender `eyeGouged` flag + skill-use report, gated on `canRemoveOpponentAssists` | small |
| **StepCatchScatterThrowIn** | Catch | `bb2016/catch_behaviour.rs`, `bb2020/catch_behaviour.rs`, `bb2025/catch_behaviour.rs` | `CatchBehaviour.java` (all editions) | Offer catcher a re-roll (`ReRolledActions.CATCH`) when they have Catch. **Unimplemented in all three editions** — even the "canonical" bb2025 file registers an empty container | small |
| **StepAnimalSavagery** | AnimalSavagery | `bb2020/animal_savagery_behaviour.rs`, `bb2025/animal_savagery_behaviour.rs` | `AnimalSavageryBehaviour.java` (344 lines) | Confusion roll, negatrait check, lash-out target selection/dialog, injury on teammate/opponent, fallback-action routing. No `SkillId::AnimalSavagery` anywhere in Rust step/mechanics/model | large |
| **StepDivingTackle** | DivingTackle | `bb2016/diving_tackle_behaviour.rs`, `bb2020/diving_tackle_behaviour.rs`, `bb2025/diving_tackle_behaviour.rs` | `DivingTackleBehaviour.java` | Dodge-modifier math (min-roll with/without Break Tackle), dialog. Partially ported directly into `step_diving_tackle.rs`, but `UtilPlayer.findEligibleDivingTacklers` equivalent is stubbed to always-empty — real, scoped gap | large |
| **StepShadowing** | Shadowing | `bb2025/shadowing_behaviour.rs` (+ dead-dup bb2016/bb2020 files) | `ShadowingBehaviour.java` | Eligible-shadower lookup/filter, dialog, 4+ roll w/ re-roll, moves shadower into vacated square. `step_shadowing.rs` variants exist but appear to only handle plumbing — **needs a closer follow-up check**, tentatively real gap | large |
| **StepTentacles** | Tentacles | `bb2025/tentacles_behaviour.rs` (+ dead-dup bb2016/bb2020) | `TentaclesBehaviour.java` | Eligible Tentacles-holder lookup (adjacent to *mover*, not acting player), ST-contest roll w/ re-roll, holds player in place | large |
| **StepUnchannelledFury** | UnchannelledFury | `bb2025/unchannelled_fury_behaviour.rs` (+ dead-dup bb2020) | `UnchannelledFuryBehaviour.java` (192 lines) | Confusion-style roll after Bone Head failure; on fail offers "second block" skill dialog, else cancels action. Some direct-in-step logic exists (`step_unchannelled_fury.rs`) but per agent finding may not cover the full hook body — **verify before starting**, likely medium not large once confirmed | large→medium |
| **StepJuggernaut** | Juggernaut | `bb2025/juggernaut_behaviour.rs`, `mixed/juggernaut_behaviour.rs` (BB2016/2020) | `JuggernautBehaviour.java` (bb2025 + mixed) | Both-Down→Pushback conversion dialog on Blitz. Identical logic duplicated per ruleset — port once, reuse | small |
| **StepDauntless** | Dauntless, Indomitable | `mixed/dauntless_behaviour.rs`, `mixed/indomitable_behaviour.rs` | `DauntlessBehaviour.java`, `IndomitableBehaviour.java` | Sequential dialog chain sharing one step: Dauntless die roll w/ re-roll (priority 2) → on success, Indomitable dialog doubles target ST (priority 3). `mixed/dauntless_behaviour.rs` already has a `StepModifierTrait` impl registered in `registry.rs` for all editions, but targets `StepId::Dauntless`, which is **not** one of the 7 wired StepIds — registered but unreachable | large |
| **StepFoulAppearance** / **StepFoulAppearanceMultiple** | FoulAppearance | `bb2025/foul_appearance_behaviour.rs` (+ dead-dup bb2016/bb2020) | `FoulAppearanceBehaviour.java` | Roll 2+ resist check w/ re-roll; failure drops attacker prone & ends action. Needs the generic multi-block base (below) for its "multiple" variant | medium |
| **AbstractStepModifierMultipleBlock** (shared base, blocks two step families) | — (base class used by FoulAppearance-multi and Dauntless-multi) | `mixed/abstract_step_modifier_multiple_block.rs` | `AbstractStepModifierMultipleBlock.java` | Generic template: first pass rolls each block target needing a roll + collects re-roll eligibility + shows dialog; second pass applies the chosen re-roll. Currently a hollow container in Rust — no trait/base logic at all | large |
| **StepJumpUp** | JumpUp | `mixed/jump_up_behaviour.rs` (+ dead-dup bb2016/bb2020/bb2025 which reuse bb2025's registered copy — confirm which is canonical) | `JumpUpBehaviour.java` (mixed) | Gathers JumpUpModifierFactory modifiers, agility check w/ re-roll; fail → prone + end action, success → free stand | medium |
| **StepAnimosity** | Animosity | `bb2025/animosity_behaviour.rs` (+ dead-dup bb2016/bb2020) | `AnimosityBehaviour.java` | Race-based animosity check, d6≥2 roll w/ re-roll. `step_animosity.rs` (242 lines) exists with `start`/`execute_step` but no direct `SkillId::Animosity` reference found by grep — **needs a closer look; may already be handled by caller-side gating rather than in-step, tentatively treat as a real but small gap** | small |
| **StepBombardier** | Bombardier | `bb2025/bombardier_behaviour.rs` (+ dead-dup bb2016/bb2020) | `BombardierBehaviour.java` | Mark skill used + switch TurnMode to BOMB_HOME/AWAY(_BLITZ). **Correction to initial agent read: `step_bombardier.rs` already references `SkillId::Bombardier` directly and marks it used** — likely a false positive / already direct-in-step; verify fully before scheduling | trivial (or none) |
| **StepDumpOff** | DumpOff | `bb2025/dump_off_behaviour.rs` (+ dead-dup bb2016/bb2020) | `DumpOffBehaviour.java` | `DumpOffStepModifier::applies_to` checks `StepId::BlockRoll` instead of `StepId::DumpOff` — its own `// TODO: map to correct StepId` flags a wiring bug. **However `step_dump_off.rs` already references `SkillId::DumpOff` directly with dialog gating and TurnMode switching** — likely already done directly in step; the skill_behaviour file is dead/miswired duplicate, not a real gap | trivial (verify) |
| **StepStab** | Stab | `bb2025/stab_behaviour.rs` (+ dead-dup bb2016/bb2020) | `StabBehaviour.java` | Armour-roll injury path instead of block dice. **`step_stab.rs` already references `SkillId::Stab` directly with dedicated tests** — likely already done directly in step; skill_behaviour file is dead duplicate, not a real gap | none (verify) |
| **StepWrestle** | Wrestle | `bb2025/wrestle_behaviour.rs` (+ dead-dup bb2016/bb2020) | `WrestleBehaviour.java` | Sequential attacker/defender skill-use dialogs, Both-Down handling, ball-and-chain injury. **Confirmed by direct read: `step_wrestle.rs` already implements this end-to-end** (with documented simplification stub: random agent always declines dialogs; drop is simplified). Not a real gap for this audit — skill_behaviour file is dead/duplicate | none |
| **StepCloudBurster** (not a StepModifier hook at all) | CloudBurster | `bb2020/cloud_burster_behaviour.rs` | `CloudBursterBehaviour.java` | Java registers a **whole standalone step** (`registerStep(StepId.CLOUD_BURSTER, StepCloudBurster.class)`), not a `StepModifier`. No Rust `StepCloudBurster` exists at all — different mechanism, needs its own step file, not just a hook port | large |

## False positives (stale TODO, safe to delete, no further work)

Dead duplicates of skills whose real logic already lives in the canonical
`bb2025::` module (reused directly by `registry.rs` for bb2016/bb2020) or
directly in a step file, with no distinct behavior of their own:
`bb2016/animosity_behaviour.rs`, `bb2016/blood_lust_behaviour.rs`,
`bb2016/bombardier_behaviour.rs`, `bb2016/dauntless_behaviour.rs`,
`bb2016/dump_off_behaviour.rs`, `bb2016/foul_appearance_behaviour.rs`,
`bb2016/jump_up_behaviour.rs`, `bb2016/pass_behaviour.rs`,
`bb2016/shadowing_behaviour.rs`, `bb2016/stab_behaviour.rs`,
`bb2016/take_root_behaviour.rs`, `bb2016/tentacles_behaviour.rs`,
`bb2016/wrestle_behaviour.rs`, `bb2020/abstract_pass_behaviour.rs` (Java
itself is a trivial passthrough), `bb2020/animosity_behaviour.rs`,
`bb2020/blood_lust_behaviour.rs`, `bb2020/bombardier_behaviour.rs`,
`bb2020/dump_off_behaviour.rs`, `bb2020/foul_appearance_behaviour.rs`,
`bb2020/pass_behaviour.rs`, `bb2020/shadowing_behaviour.rs`,
`bb2020/stab_behaviour.rs`, `bb2020/take_root_behaviour.rs`,
`bb2020/tentacles_behaviour.rs`, `bb2025/bombardier_behaviour.rs`\*,
`bb2025/dump_off_behaviour.rs`\*, `bb2025/stab_behaviour.rs`\*,
`bb2025/wrestle_behaviour.rs`\* (\*re-flagged during this audit from an
initial "genuine gap" read — see table above, `step_xxx.rs` already
implements the logic directly).

Trivial stat-only skills with no `registerModifier` in Java at all (comment
can just be deleted): `bb2020/slayer_behaviour.rs`
(`SlayerBehaviour.java`, 17 lines), `bb2020/toxin_connoisseur_behaviour.rs`
(`ToxinConnoisseurBehaviour.java`, 16 lines).

Intentional inert markers (Java itself is a no-op subclass, real logic lives
in a shared `mixed`/`abstract` base already wired): `bb2016/dodge_behaviour.rs`,
`bb2020/dodge_behaviour.rs` (both real logic is in
`mixed/abstract_dodging_behaviour.rs`, which **is** fully wired — no TODO
marker, dispatch is called from `step_block_dodge.rs`).

## Done (direct-in-step, not via this mechanism — no action needed for hook-infra, but some carry their own separate known stubs)

`bb2016/piling_on_behaviour.rs`, `bb2020/piling_on_behaviour.rs` (real
simplified logic in `step_drop_falling_players.rs`; known stub: agent always
declines the Piling On dialog, full team-reroll/double-KO paths pending),
`bb2016/swarming_behaviour.rs`, `bb2020/swarming_behaviour.rs` (real logic in
`step_swarming.rs`), `bb2016/throw_team_mate_behaviour.rs` (real logic in
`step_throw_team_mate.rs`; ThrowTeamMate is BB2016-only),
`bb2016/swoop_behaviour.rs`, `bb2020/swoop_behaviour.rs` (real logic in
`step_swoop.rs`; Swoop is BB2020/BB2025-only, bb2016 file is unreachable dead
code), `bb2016/leap_behaviour.rs` (real logic in `step_jump.rs`; Leap isn't
assigned to BB2016 per the skill edition table, file is unreachable),
`bb2016/sneaky_git_behaviour.rs`, `bb2020/sneaky_git_behaviour.rs` (checks
exist in `step_eject_player.rs` / `step_referee.rs`, not verified line-by-line
against full 131-line Java for parity), `bb2020/stand_firm_behaviour.rs`
(partial `has_skill` check in its own `step_pushback.rs`, not verified
complete vs. the 87-line Java dialog logic), `bb2020/the_ballista_behaviour.rs`,
`bb2020/throw_team_mate_behaviour.rs` (both presumed folded into
`step_throw_team_mate.rs`/`step_hail_mary_pass.rs`, not exhaustively verified),
`bb2020/unchannelled_fury_behaviour.rs` (real, substantial logic in
`step_unchannelled_fury.rs` — but see the bb2025 entry above flagging possible
incompleteness), `mixed/abstract_dodging_behaviour.rs` (fully wired, no TODO
marker, included here only for completeness).

## Phase AAI update (closed items 5 and 6)

Re-verification against source (Phase AAI) found items 5 and 6 were sized wrong:

- **Item 5 (`AbstractStepModifierMultipleBlock` base)**: NOT a from-scratch base-class port.
  Both concrete call sites (`step_dauntless_multiple.rs`, `step_foul_appearance_multiple.rs`)
  were already ~90% complete, tested, direct-in-step ports. Only two narrow pieces were
  missing: the re-roll-choice dialog (`decideNextStep`'s `showDialog` branch — now wired via a
  new `AgentPrompt::ReRollForTargets` variant + shared `build_reroll_prompt` helper in
  `abstract_step_multiple.rs`) and the auto-immediate-reroll-on-failure inside the first-run
  roll loop (ported for Dauntless via the same hardcoded Blind-Rage check already used by
  `step_dauntless.rs`; FoulAppearance's equivalent branch is correctly a no-op since no skill in
  this codebase registers a reroll source for `ReRolledActions.FOUL_APPEARANCE`). The hollow
  `skill_behaviour/mixed/abstract_step_modifier_multiple_block.rs` file needed no real logic —
  left in place, dead/unreferenced, per the established convention for confirmed-dead
  `skill_behaviour/*.rs` files. **Closed.**
- **Item 6 (StepJumpUp, StepAnimosity)**: both steps were already complete/tested; each had
  exactly one stubbed dependency one layer down. JumpUp needed a `JumpUpModifierFactory` (new,
  small — the collapsed Java logic reduces to just the edition's modifier collection, no
  skill/card ever registers a `JumpUpModifier`). Animosity needed a real
  `SkillMechanic::animosity_exists` per edition (bb2016 correctly always `false`; bb2020 matches
  raw `positionId`/`race` strings; bb2025 matches `Keyword`-normalized position keywords — the
  two editions have genuinely different `Animosity.Evaluator` implementations in Java, not a
  shared one). Also filled a small pre-existing gap: `SkillId::Animosity` had no `properties()`
  entry for `hasToRollToPassBallOn` (same shape as the EyeGouge gap fixed in Phase AAG). **Closed.**

Item 7 (StepDivingTackle) turned out **larger** than originally estimated once directly
verified: no dodge-modifier math, no dialog round-trip, and no eligible-tackler lookup exist
anywhere in Rust yet (contrary to a stale doc comment implying partial porting) — treat as its
own phase, not bundled with anything else.

## Phase AAJ update (closed item 7)

Item 7 (StepDivingTackle) is **closed**. Ported all three editions' dodge-modifier math, the
eligible-tackler lookup (`UtilPlayer::find_diving_tacklers`/`find_eligible_diving_tacklers`), and
the coach-choice dialog round-trip (extending `AgentPrompt::PlayerChoice` with a `descriptions`
field) directly into `step_diving_tackle.rs`. **Correction to this doc's own item-7 note**: the
"likely dependency on the also-unported StepDropDivingTackler" concern was stale —
`StepDropDivingTackler` (both variants) was already fully implemented, tested, and wired into
every move/blitz-move sequence generator; direct verification found no work needed there. See
`SESSION.md`'s Phase AAJ entry for full details, including one out-of-scope pre-existing gap
surfaced along the way (BB2020/BB2025's Break Tackle/Incorporeal stat-based dodge modifiers are
not wired into `step_move_dodge.rs`'s own base-dodge computation — `DivingTackle` reuses the same
primitives and inherits the limitation rather than fixing it).

## Recommended phase-batching order

Ordered roughly by (a) how many skills unlock per unit of work and (b) how
foundational/small the step is, per the task's batching goal:

1. **Wiring bug + verification pass first (near-zero cost, unblocks nothing
   new but prevents wasted work):** Fix `bb2025/dump_off_behaviour.rs`'s
   `applies_to` (`StepId::BlockRoll` → `StepId::DumpOff`), and do a quick
   direct-in-step verification pass on Bombardier/DumpOff/Stab/Wrestle (this
   audit found strong evidence they're already done directly in their step
   files — confirming that turns 4 "genuine gaps" into deletions of stale
   `skill_behaviour/` duplicates).
2. **StepPushback family** (5 skills share one step, and Pushback is already
   wired for dispatch — lowest marginal cost per skill): Grab, SideStep,
   MonstrousMouth, StandFirm(bb2016), finish EyeGouge's stub body. Unify the
   three divergent `step_pushback.rs` files while here.
3. **StepCatchScatterThrowIn / Catch**: single small, well-scoped feature
   (reroll offer), touches all 3 editions at once since none have it.
4. **StepDauntless / StepJuggernaut**: small-to-medium, each unlocks 2+ skills
   sharing one step (Dauntless+Indomitable; Juggernaut bb2025+mixed).
5. **AbstractStepModifierMultipleBlock base** — unlocks FoulAppearance-multi
   and Dauntless-multi simultaneously; do this once Dauntless/FoulAppearance
   singular paths are ported, so the abstraction has two concrete call sites
   to validate against.
6. **StepJumpUp, StepAnimosity** (small-medium, single skill each, but
   Animosity needs a closer look at `step_animosity.rs` first — may already
   be a false positive).
7. **StepDivingTackle** (medium — most of the port already exists, just needs
   the eligible-tackler lookup un-stubbed).
8. **StepFoulAppearance** (needs #5 above landed first for its multi-variant).
9. **Large, single-skill, currently fully-unimplemented items** — schedule
   last since they're isolated and don't block anything else: AnimalSavagery,
   Shadowing (verify first — may be partially done), Tentacles,
   UnchannelledFury (verify first — may be partially done), CloudBurster
   (different mechanism entirely, needs its own new step, treat as its own
   mini-project).

## Caveats for whoever picks this up next

- This audit was done via two parallel sub-agent passes plus direct
  spot-checks; several "DONE (direct-in-step)" and a few "GENUINE GAP"
  verdicts above are flagged **"needs verification"** because the two passes
  initially disagreed (e.g. Bombardier/DumpOff/Stab/Wrestle) and a quick
  direct read of `step_wrestle.rs` confirmed those are further along than
  first assessed. Re-verify each item's checkbox before starting work on it,
  the classifications given here are a strong prior, not gospel.
- The three divergent `step_pushback.rs` implementations (bb2016 re-exports
  bb2025's; bb2020 has its own) is an existing inconsistency independent of
  this audit — worth deciding whether to unify before adding Grab/SideStep/
  MonstrousMouth to all three.
- `crates/ffb-engine/src/step/engine.rs` is dead code (deleted from the mod
  tree per Phase ZR / CLAUDE.md) — any `SkillId` references found there do
  not count as a live implementation.
