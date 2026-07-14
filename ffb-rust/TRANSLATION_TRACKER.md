# FFB Java to Rust 1:1 Translation Tracker

<!-- Auto-generated skeleton from Java source. Update status cells manually as files are translated. -->
<!-- To regenerate: python scripts/gen_translation_tracker.py -->

## How to Use

This file tracks every Java class in ffb-common, ffb-server, and ffb-client-logic against its target Rust file.

1. When you start translating a file: change its status to `~`
2. When it matches the Java source 1:1 and parity is confirmed: change to `✓`
3. When a race passes T3b 100/100, all files exercised by that race should be `✓`

**Workflow per Java file:**
- Read the Java source in `ffb-java/<module>/src/main/java/<path>.java`
- Find or create the corresponding Rust file at the listed Rust Target path
- Translate method by method, matching dice consumption order, conditions, and state transitions exactly
- Run `cargo test` after each file
- Run `python scripts/parity_run.py --home amazon --away amazon --seeds 1-10` to catch regressions

## Status Legend

- `○` Not started -- no Rust equivalent exists
- `~` Partial -- Rust equivalent exists but not yet 1:1 with Java
- `✓` Done -- Rust matches Java line-by-line, parity confirmed
- `—` Not translating — AWT/Swing GUI only (`ffb-client` Java module, ~81 files). DB, WebSocket, cache, replay are all in scope.

---

## Progress Summary

**Phase AAP (closes 2 real gaps found via a from-scratch source audit of Phase AAO's own "what's
left" list — 2 of its 3 named items turned out to be stale, corrected here):**
1. **Real bug/gap closed: raise-dead never fired.** `UtilServerInjury.handleRaiseDead` +
   `raisePlayer`/`sendRaisedPlayer` (Necromantic/vampire-lord teams raising a RIP'd opponent as
   Zombie/Thrall, or Nurgle's-Rot teams raising as Rotter) was a documented no-op in all 3
   `step_apothecary.rs` files despite `InjuryMechanic::can_raise_dead`/`can_raise_infected_players`/
   `raise_type`/`raised_nurgle_type` already being real, tested, per-edition mechanics. Implemented
   the missing piece in `util_server_injury.rs::handle_raise_dead`: added `Roster::
   raised_position_id`/`raised_roster_position()` (the JSON data already carried
   `raised_position_id`, just never consumed), reused the existing `Player::from_position`/
   `find_roster`/`UtilBox::put_player_into_box` helpers. 5 new tests.
2. **Correction of Phase AAO's own top-priority item**: AAO's closing note recommended wiring
   `UtilSkillBehaviours::register_behaviours` into `GameState` construction. Direct investigation
   found this recommendation was itself wrong — it would build a *third* parallel skill-dispatch
   path alongside the already-live `skill_behaviour/registry.rs`/`dispatch::execute_step_hooks`
   system. A full audit of that live registry's ~30 `StepModifierTrait::handle_execute_step`
   bodies found: ~10 (Wrestle/Stab/Bombardier/Dauntless/Tentacles/Shadowing/JumpUp/Animosity/
   DumpOff/Juggernaut) are confirmed dead-duplicate stubs with real logic already in direct
   `step_xxx.rs` files (matches the established Phases AAG-AAI precedent, correctly left alone);
   the rest are genuinely real and working. **One true gap found**: `TheBallista`'s both step
   modifiers are real stubs blocked on step-specific re-roll-state plumbing
   (`reRolledAction`/`reRollSource` from the `UseSkill` command) that was never built anywhere —
   sized for its own future phase, not fixed here. Deleting the dead `SkillBehaviour` marker-trait
   system itself (~30 mixed files + 21 fully-orphaned bb2016/bb2020 duplicates + ~129 dead-trait
   test blocks) is a large, separate mechanical phase in its own right — scoped and documented,
   not executed this session.
3. **Correction: "InducementSet model port" (AAO's 2nd-priority item, flagged as the *largest*
   remaining gap) was stale — it was already fully done as of Phase ZA (2026-07-06).** No phase
   needed; the only real remainder in that cluster was item 1 above.
4. **BlockMode armour-modifier gating**: `injury_type_block.rs`'s `armour_roll`/`injury_roll` now
   gate the (still Mighty-Blow-only — full Claws/chainsaw/`CLAW_DOES_NOT_STACK` modifier-factory
   lookup remains a documented separate TODO) modifier addition on `BlockMode` + same-team check,
   matching `InjuryTypeBlock.java`'s real per-roll conditions exactly. Fixed 2 stale tests that had
   encoded the old always-false `Regular`-mode behavior as correct; added 5 new tests.

Tests: 17,621 → **17,634** (+13: +5 raise-dead, +3 `Roster::raised_roster_position`, +5 net
BlockMode). 0 failures. `cargo clippy --workspace --all-targets`: still 0 errors. No parity/
integration testing (per standing instruction). **Honest completion estimate: ~98.5-99%** (up from
AAO's ~98%). Full writeup: `SESSION.md` Current Status.

**Phase AAO (closes the skill-hook audit's last loose thread, fixes a real correctness bug,
reverses course on a planned dead-file cleanup):**
1. Fixed `step_reset_fumblerooskie.rs`: a lost `!isNextMovePossible` clause made its
   report-emission branch unreachable unless `end_player_action` was already true. Now matches
   Java's `StepResetFumblerooskie.start()` exactly. 2 new regression tests.
2. Fixed `step_eject_player.rs`'s stale `&& false` placeholder — the `sneakyGitBanToKo` option
   was already ported elsewhere; wired it in for real.
3. Fixed a genuine clippy `overly_complex_bool_expr` in
   `util_throw_team_mate_sequence.rs` (tautological test assertion) — `cargo clippy --workspace
   --all-targets` is now fully clean for the first time in this project.
4. Built the generic `insertHooks`/`PASS_INTERCEPT` mechanism (`skill_behaviour::step_hook::
   hooked_steps` + `Sequence::insert_hooks`), wired into all 3 pass generators. `StepSafeThrow`
   (BB2016) and `StepCloudBurster` (BB2020) — implemented since Phases AAF/AAN but unreachable —
   are now spliced into the live pass sequence. BB2025 correctly inserts nothing (matches real
   Java). 3 edition-level integration tests.
5. **Did not** delete the ~30 `skill_behaviour/*.rs` files the audit called "dead duplicates" —
   direct re-verification found every one is actually referenced by
   `UtilSkillBehaviours::register_behaviours`, a real, tested translation of Java's reflection
   registry that just isn't wired into `GameState` construction yet. Flagged as a new, more
   valuable future gap instead of deleting.

Tests: 17,609 → 17,621 (+12), 0 failures. `cargo clippy --workspace --all-targets`: 0 errors (down
from 2). No parity/integration testing (per instruction). Full writeup: `SESSION.md` Current
Status.

**Phase AAN (closed skill-hook-audit item 9's CloudBurster — audit fully complete, this
session):**
CloudBurster is a whole standalone step in Java (`registerStep`, not `StepModifier`) — no Rust
`StepCloudBurster` existed. Created `step/bb2020/pass/step_cloud_burster.rs`: deflection/
thrower/interceptor guard, `canForceInterceptionRerollOfLongPasses` skill-property lookup on the
thrower, passing-distance check via the existing `PassMechanic`, `cancelsSkill` check (via
`has_skill_property`, matching `StepSafeThrow`/`VeryLongLegs` precedent), and on success:
`ReportCloudBurster` + reset deflection + re-push `INTERCEPT` forwarding only
`GOTO_LABEL_ON_FAILURE`. Added `StepId::CloudBurster` + wiring in `driver.rs`/
`step_id_factory.rs`, the missing `SkillId::CloudBurster` properties entry, and extended
`VeryLongLegs`'s properties to the BB2016+BB2020 union. Two pre-existing, out-of-scope
limitations documented rather than fixed: the generic `PASS_INTERCEPT` hook-insertion mechanism
isn't wired into the bb2020 pass generator (matches the `StepSafeThrow` precedent), and Rust's
per-instance step fields mean a re-pushed `INTERCEPT` can't transparently resume the
previously-chosen interceptor the way Java's shared `PassState` does.

**This closes all 9 batching-order items in `docs/PHASE_AAF_SKILL_HOOK_AUDIT.md`** (1-7: Phases
AAG-AAJ; 8 + Shadowing/UnchannelledFury: AAK; AnimalSavagery: AAL; Tentacles: AAM; CloudBurster:
AAN).

Tests: 17,595 → 17,609 (+14), 0 failures. No parity/integration testing (deferred, per
instruction). **Honest completion estimate**: roughly ~99.9% true behavioral completion of
in-scope skill-hook logic — the skill-hook audit that has driven Phases AAG-AAN is now complete.
The natural next major workstream is Java/Rust parity/integration testing against the real
engine, currently the least-developed piece of this project (only 8 sample seeds in
`progress.html`/`parity/`, one known FAIL) — out of scope for this plan; needs its own separate
planning pass. Full writeup: `SESSION.md` Current Status.

**Phase AAM (closed skill-hook-audit item 9's Tentacles, this session):**
Mixed result: BB2016 (`step/bb2016/move_/step_tentacles.rs`) was already complete and correct, no
change needed. BB2020/BB2025 (`step/mixed/move_/step_tentacles.rs`) was a genuine from-scratch
gap — `execute_step` short-circuited to `NextStep` with no holder lookup/roll/hold-in-place logic.
Ported `TentaclesBehaviour.java` (bb2020+bb2025, byte-identical except one trigger condition)
directly into the step: eligible-holder lookup via `UtilPlayer::find_adjacent_opposing_players_with_skill`
centred on the mover's `coordinate_from`, `dodging || jumping` trigger (plus a BB2020-only
`has_blocked` extra trigger), 1d6 strength contest (`min_roll = max(6 - stDifference, 2)`) with
re-roll owned by the defender (Tentacles player) — a real edition difference from BB2016 where the
escaping player re-rolls — and hold-in-place resolution (cancel dodging/jumping, move mover+ball
back to `coordinate_from`). Reproduced one Java quirk: `goToLabelOnSuccess` is a mandatory but
dead init parameter in BB2020/2025 (always resolves via `NEXT_STEP`, unlike BB2016).

Tests: 17,585 → 17,595 (+10), 0 failures. No parity/integration testing
(deferred, per instruction). **Honest completion estimate**: roughly ~99.8% true behavioral
completion of in-scope logic — only CloudBurster remains; expect 1 more phase to close it, after
which parity/integration testing against the Java engine becomes the natural next major
workstream. Full writeup: `SESSION.md` Current Status.

**Phase AAL (closed skill-hook-audit item 9's AnimalSavagery, this session):**
Unlike Shadowing/UnchannelledFury in the prior phase, this item's "fully unimplemented" audit
claim was correct: `StepAnimalSavagery::execute_step()` was a no-op stub and both editions'
`skill_behaviour/*/animal_savagery_behaviour.rs` hooks always returned `false`, despite correct
surrounding scaffolding (step field plumbing, deferred commands, enums, reports, dice/injury/
adjacency helpers). Ported the full mechanic directly into `execute_step()` (direct-in-step
pattern, matching Dauntless/Wrestle/Stab/DumpOff — one modifier per step, no dispatch needed):
negatrait gate, confusion roll + skill/team re-roll chain, `canLashOutAgainstOpponents` skill-use
dialog + adjacent-target computation, multi-target `PlayerChoice` dialog, and `lash_out` (injury
via `handle_injury` with edition-correct `InjuryTypeBlock::Mode`, end-turn deferred-command wiring,
`fallbackAction`/`cancelPlayerAction` state machine). Added `BlockMode::DoNotUseModifiers`/
`UseArmourModifiersOnlyAgainstTeamMates` to `injury_type_block.rs` and
`SteadyFootingContext::from_drop_player_with_commands`. Reproduced 2 Java quirks bug-for-bug
(no-skill lash-out target pool defaults to own team; already-used-this-drive proceeds without
failure status).

Tests: 17,552 → 17,585 (+33), 0 failures. No parity/integration testing (deferred, per
instruction). **Honest completion estimate**: roughly ~99.6% true behavioral completion of
in-scope logic — only Tentacles and CloudBurster remain; expect 2 more phases to close the rest,
after which parity/integration testing against the Java engine becomes the natural next major
workstream. Full writeup: `SESSION.md` Current Status.

**Phase AAK (closed skill-hook-audit item 8, and Shadowing + UnchannelledFury of item 9, this
session):**
Planned as a verification-only pass on item 8 (FoulAppearance) plus a sizing check on Shadowing
and UnchannelledFury (both flagged "may already be partially done"). Item 8 confirmed already
fully complete — no code change needed. Shadowing and UnchannelledFury both turned out to have
small, real, fixable bugs rather than large missing features, so both closed in this same phase.
(1) **Shadowing**: all three `step_shadowing.rs` variants filtered eligible defenders by
`NamedProperties::CAN_ATTEMPT_TO_TACKLE_DODGING_PLAYER` (a `DivingTackle` property) instead of a
direct `SkillId::Shadowing` check — a real Shadowing-skill defender would essentially never be
found. Fixed via new `UtilPlayer::find_adjacent_opposing_players_with_skill` (1:1 with Java's
`findAdjacentOpposingPlayersWithSkill`) + a BB2025-only `movement_with_modifiers()` fix in the
`shadowingCount` filter. (2) **UnchannelledFury**: the main roll/re-roll/dialog loop was already
correct; `cancel_unchannelled_fury_action`'s turn-flag switch had drifted from Java in 4 small
ways (missing `allowsAdditionalFoul` guard, BB2025's separate `ttm_used` flag merged into
`pass_used`, missing BB2025 `PUNT`/`PUNT_MOVE` case, spurious `StandUpBlitz` in the blitz group) —
fixed with an edition-aware branch.

Tests: 17,544 → 17,552 (+8: 3 Shadowing + 5 UnchannelledFury regression tests), 0 failures. No
parity/integration testing (deferred, per instruction). **Honest completion estimate**: roughly
~99.3% true behavioral completion of in-scope logic — only AnimalSavagery, Tentacles, and
CloudBurster remain, all confirmed genuinely unimplemented (not sizing errors); expect 3 more
phases (one per skill) to close the rest, after which parity/integration testing against the Java
engine becomes the natural next major workstream. Full writeup: `SESSION.md` Current Status.

**Phase AAJ (closed skill-hook-audit batching item 7: Diving Tackle, this session):**
Ported Diving Tackle end-to-end for all three rule editions — the single largest itemized gap
left in `docs/PHASE_AAF_SKILL_HOOK_AUDIT.md`'s batching order (confirmed by direct research this
session: none of the eligibility lookup, dodge-modifier math, or dialog round-trip existed in
Rust before this phase, only stub files with descriptive comments). Following the established
Wrestle/Stab/DumpOff/Dauntless "logic lives directly in the step" convention: (1) added
`UtilPlayer::find_diving_tacklers`/`find_eligible_diving_tacklers` + `filter_thrower`/
`filter_attacker_and_defender` (`ffb-model/src/util/util_player.rs`) — BB2016/BB2020 share one
filter chain (differing only in `checkAbleToMove`); BB2025 adds the `GameOptionId::
DivingTackleLeavingTzOnly`-gated destination-adjacency exclusion, a genuine per-edition
difference. (2) Extended `AgentPrompt::PlayerChoice` with a `descriptions: Vec<String>` field
(matching `DialogPlayerChoiceParameter`'s flat, non-per-player message list — not indexed to
`eligible_players`) plus the matching `WireDialog`/`wire_prompt.rs` conversion arm, updating all
8 existing call sites. (3) Ported `step_diving_tackle.rs`'s real logic: BB2016's 3-way branch and
BB2020/BB2025's shared 4-way branch (an extra `StatBasedRollModifier` axis, in this codebase only
ever produced by BB2020's Gretchen-only `Incorporeal` skill — hardcoded since the generic
`Skill.stat_based_roll_modifier_factory` field is an unwired `String` placeholder across the
whole codebase, an out-of-scope pre-existing gap). Dodge-modifier minimum-roll math is computed
inline (matching the real per-edition `AgilityMechanic.minimumRollDodge` formulas exactly) rather
than via the `AgilityMechanic` trait, because `DodgeModifier` has no `Hash`/`Eq` impl and can't
populate the trait's `HashSet<DodgeModifier>` signature — the same reason `step_move_dodge.rs`
already bypasses the trait. Reproduced one bug-for-bug Java quirk (`strengthModifierCanBeAdded`
re-checking the wrong modifier set, always false in that branch) rather than "fixing" it, per
translation ground rules. (4) `StepDropDivingTackler` (both bb2016 and mixed/BB2020+BB2025
variants) turned out to already be fully implemented, tested, and wired into every move/blitz-move
sequence generator — a stale claim in the audit doc's item-7 writeup, corrected by direct
verification. (5) Corrected the 3 dead `skill_behaviour/*/diving_tackle_behaviour.rs` stub doc
comments to point at the real step-file implementation (left registered, matching the Wrestle/
Stab/DumpOff/Dauntless precedent).

Tests: 17,533 → 17,544 (+11), 0 failures. No parity/integration testing (deferred, per
instruction). **Honest completion estimate**: roughly ~98.5-99% true behavioral completion of
in-scope logic — item 8 (StepFoulAppearance's own gate) was already effectively closed as a
byproduct of Phase AAI's multi-block work; that leaves only item 9's large isolated skills
(AnimalSavagery, Shadowing, Tentacles, UnchannelledFury, CloudBurster) — expect 2-4 more phases to
close the rest, after which parity/integration testing against the Java engine becomes the
natural next major workstream. Full writeup: `SESSION.md` Current Status.

**Phase AAH (closed skill-hook-audit batching item 4: Dauntless/Indomitable/Juggernaut, this
session):**
Found the audit's "large"+"small" sizing for this item was stale — the real game logic already
lived directly in Rust step files (`step_dauntless.rs`, `step_dauntless_multiple.rs`,
`step_double_strength.rs`, `step_juggernaut.rs`), same direct-in-step pattern as Wrestle/Stab/
DumpOff/Bombardier. Closed 4 real gaps found by direct Java comparison: (1) ported Java's
single-block Indomitable dialog chain (priority-3 `StepModifier` sharing `StepDauntless` with
Dauntless's own priority-2 modifier — verified this distinction against source, not
`StepDoubleStrength` as initially assumed) directly into `step_dauntless.rs`; (2) wired the
already-defined-but-unused `Action::IndomitableChoice` into `step_double_strength.rs` so
multi-target Dauntless successes let the coach pick which target to double, instead of always the
first; (3) implemented Dauntless's silent Blind-Rage reroll (a real, data-present mechanic —
`"Blind Rage"` is already a registered `ReRollSources` entry); (4) investigated
`step_dauntless_multiple.rs`'s reroll-choice dialog and explicitly deferred it — closing it
properly needs the same aggregation logic as the still-unbuilt `AbstractStepModifierMultipleBlock`
base (audit item 5), so it wasn't force-fit into this phase. Corrected the 4 dead
`skill_behaviour/*.rs` stubs' doc comments (left registered, matching the confirmed Wrestle/Stab/
DumpOff precedent) rather than deleting them.

Tests: 17,492 → 17,500 (+8), 0 failures. No parity/integration testing (deferred, per instruction).
**Honest completion estimate**: roughly ~97% true behavioral completion (modest gain — most of this
item was already done); expect 3-6 more phases (audit items 5-9) to close the rest. Full writeup:
`SESSION.md` Current Status.

**Phase AAG (closed skill-hook-audit batching items 1-3: DumpOff bug fix, StepPushback skill
family, StepCatchScatterThrowIn Catch family, this session):**
Closed the first 3 (of 9) batching-order items from `docs/PHASE_AAF_SKILL_HOOK_AUDIT.md`. (1)
Fixed `bb2025/dump_off_behaviour.rs`'s `applies_to` bug (`StepId::BlockRoll` → `StepId::DumpOff`);
confirmed Bombardier/DumpOff/Stab/Wrestle's real logic already lives directly in their step files,
no further work needed. (2) Ported real `StepModifierTrait` impls for **Grab, SideStep, StandFirm**
across all 3 editions (BB2016/BB2020 were previously bare unregistered stubs; only BB2025 was real)
— migrated BB2020's `step_pushback.rs` off its hand-inlined hook methods onto the shared
`dispatch::execute_step_hooks` mechanism BB2025 already used, fixed a live priority bug (BB2025
Grab was `3`, Java registers `5`) and a stray-space typo in a cancelling-skill property check, and
replaced EyeGouge's stub body with a real translation. Also implemented BB2025's MonstrousMouth
(chomped-defender forced-push) for real — a prior research pass claimed it was "already fully real
and registered," which a direct read of the file disproved (still an unimplemented stub); corrected
after verifying against source directly rather than trusting the stale claim. Fixed one small
genuine pre-existing model gap found along the way: `SkillId::EyeGouge` had no `properties()` entry
for `"canRemoveOpponentAssists"` despite Java's `EyeGouge.java` granting it. (3) Wired
StepCatchScatterThrowIn's Catch/MonstrousMouth reroll family (new `StepCatchHookState` +
`dispatch::execute_step_hooks` call in BB2020's and BB2025's `catch_ball()`, BB2016 inherits via its
existing re-export) — verified the exact Java recursion semantics directly against source (fires
once per failed roll, guarded against infinite recursion, automatic reroll with no dialog) before
wiring; implemented real `CatchStepModifier`s (all 3 editions) and `MonstrousMouthStepModifier`s
(BB2016/BB2020 — confirmed via direct Java read to be exact Catch-twins, unrelated to BB2025's
same-named forced-push mechanic).

Tests: 17,417 → 17,492 (+75: +61 Pushback family/DumpOff, +14 Catch family), 0 failures. No parity/
integration testing (deferred, per instruction). **Honest completion estimate**: roughly ~96–97% true
behavioral completion of in-scope logic (up from ~94–96% pre-phase); expect 4–7 more similarly-scoped
phases (per the audit doc's remaining batching-order items 4-9) to close the rest. Full writeup:
`SESSION.md` Current Status.

**Phase AAF (closed the factory tracker-accuracy audit; discovered + closed the skill-hook-infra gap; Dodge-family reference implementation, this session):**
Three-track session: (1) closed the `ffb-model/factory` tracker-accuracy audit flagged unverified in
Phase AAE — re-checked all 82 `factory/` rows, fixed 47 (43 moved to their real location, mostly
`ffb-mechanics/src/modifiers/*` or an enum's `for_name` method; 3 genuinely missing —
`CardFactory`/`CardTypeFactory`/`InducementTypeFactory` — translated for real with 13 tests; 4
orphaned dead stubs corrected to `—`; 1 left `~`, `JumpUpModifierFactory`, pending a `step/`-side
wiring change). (2) **Discovered a bigger, tracker-invisible problem**: the skill-behaviour step-hook
mechanism (`SkillRegistry`/`register_into`/`StepModifierTrait`/`dispatch::execute_step_hooks`,
proven real by `HornsBehaviour`) is only actually reachable from **7 of ~584 step files** — every
other skill routed through it, registered or not, currently has no effect on gameplay regardless of
its tracker `✓` mark. Verified concretely with Dodge (zero effect on BB2020/BB2025 games before this
session — the step computed its geometric dodge-choice heuristic but never ran the hook that decides/
reports the outcome). A full audit of all 127 skill-behaviour files
(`docs/PHASE_AAF_SKILL_HOOK_AUDIT.md`) found ~20 genuine remaining gaps (fewer than the raw 77
`TODO(hook-infra)` count suggests — many are dead duplicates or have real logic already inline in
their step file), grouped by target step with a recommended batching order for follow-up phases. (3)
**Wired the Dodge family for real** as the reference implementation: `step/mixed/step_block_dodge.rs`
now calls `dispatch::execute_step_hooks`; `skill_behaviour/mixed/abstract_dodging_behaviour.rs` has
the real `AbstractDodgingStepModifier` logic (has-skill check, `requireUnusedSkill` gate,
tacklezones-based default, `ReportSkillUse`), registered for both Dodge and WatchOut in the BB2020/
BB2025 registries. BB2016's own hand-rolled `StepBlockDodge` needed no changes — its logic already
lived directly in the step (a valid alternate translation of Java's non-shared bb2016
`DodgeBehaviour`). Also verified `FumbblRequestLoadTeam`'s dispatch tail (flagged as a possible
remaining gap) was already closed in Phase AAE.

Tests: 17,387 → 17,417 (+30: +13 factory, +17 dodge-family), 0 failures. No parity/integration testing
(deferred, per instruction). **Honest completion estimate**: true behavioral completion (correcting
for the skill-hook gap) is roughly ~94–96% of in-scope logic, ~81% counting the permanently-skipped
Swing GUI — expect ~5–8 more similarly-scoped phases (per the audit doc's batching order) to close the
remaining ~20 gaps and reach ~99–100%. Full writeup: `SESSION.md` Current Status;
`docs/PHASE_AAF_SKILL_HOOK_AUDIT.md` for the follow-up worklist.

**Phase AAD (32/32 server handlers reachable + behavioral-approximation fixes, this session):**
Three-step follow-up to Phase ZV, closing the remaining non-file-level gaps rather than starting
parity testing early (deferred per instruction).
- **Step 1** (`fe03b161`): wired the last 3 dormant `ServerCommandHandler*` structs
  (`AddLoadedTeam`, `ApplyAutomatedPlayerMarkings`, `CalculateAutomaticPlayerMarkings`) into live
  dispatch — **32/32 handlers now reachable, up from 29/32**. Required giving
  `InternalServerCommandAddLoadedTeam` a real `Team` field (parsed from FUMBBL team XML via the
  existing `XmlHandler` path) and `InternalServerCommandApplyAutomatedPlayerMarkings`/
  `InternalServerCommandCalculateAutomaticPlayerMarkings` real `AutoMarkingConfig`/`Game` fields
  (added `to_json_value`/`from_json` to `AutoMarkingConfig`/`AutoMarkingRecord` using the real Java
  `IJsonOption` keys) instead of the opaque `String` placeholders they carried before. Documented,
  not fabricated: no `ServerRequestProcessor`→internal-command dispatch channel exists yet for the
  3 `FumbblRequest*` types that would construct these commands from a real HTTP response, so their
  callers still discard the newly-parsed result — a narrower, separately-scoped follow-up.
- **Step 2** (`24d81084`): fixed two behavioral approximations found inside already-`✓` files.
  `ActingPlayer.isMustCompleteAction()` was hardcoded `false` in both `bomb_logic_module.rs` files
  (bb2025/mixed) — added the real `must_complete_action` field to `ActingPlayer` with real
  getter/setter. `SwarmingLogicModule`'s bb2025 `squareHasSwarmingPlayer` LINEMAN keyword check was
  hardcoded `false` — added `is_lineman` to `Player`/`RosterPosition` mirroring the existing
  `is_big_guy` precomputed-flag pattern. (The `mixed`-edition `SwarmingLogicModule` was already
  correct — it uses a different Java check, `hasSkillProperty`, not the LINEMAN keyword.) Also
  confirmed the `unimplemented!()` bodies in `pass_block_logic_module.rs`/
  `kickoff_return_logic_module.rs` are correct 1:1 translations (Java itself throws
  `UnsupportedOperationException` there) — not gaps, left untouched.
- **Step 3** (`01a8fded`): audited the 5 files with the highest `// TODO` comment counts despite
  being `✓`. 4 of 5 had only stale/already-resolved TODOs (tightened the comments); found and fixed
  one real bug in `step_init_throw_team_mate.rs` — it always advanced to the next step once a TTM
  target coordinate was set, missing Java's `UtilRangeRuler` range-gate check (now uses
  `PassMechanic::find_passing_distance`, matching `step_throw_team_mate.rs`'s existing use of the
  same mechanic, to correctly wait instead of advance when the target is out of range).

Tests: 17,331 → 17,359 (+28), 0 failures throughout. No parity/integration testing (deferred, per
instruction). What's left, none of it "translation" anymore: (1) the `FumbblRequest*` dispatch-tail
gap named in Step 1; (2) parity/integration testing against the real Java engine — still "the
natural next phase"; (3) the standing decision on the 271 permanently-skipped Swing files; (4) live
production infra wiring (real MySQL, real Jetty↔axum wire compatibility). Full writeup:
`docs/PHASE_AAD_PLAN.md`.

**Phase ZW.5 (closed the last 3 `○` rows in the entire tracker, this session):**
`LogicPluginFactory.java` (`client/factory/`, previously deferred pending `LogicPlugin` itself) →
`client/factory/logic_plugin_factory.rs`: Java's `initialize(Game)` builds its `Map<LogicPlugin.Type,
LogicPlugin>` via a reflection-based `Scanner<LogicPlugin>` that auto-discovers every
`@RulesCollection`-annotated implementation; substituted per this project's established
Java-reflection-to-Rust convention (`ReportMessageType::report_id()`, Phase ZW.3) with an explicit
list of the two known concrete plugin sets already translated under
`client/state/logic/plugin/{bb2025,mixed}/` (`BaseLogicPlugin`/`BlockLogicExtensionPlugin`/
`MoveLogicPlugin`), selected by the game's `Rules` edition (`Bb2025` → `bb2025`, `Bb2020`/`Bb2016` →
`mixed`) instead of scanning annotations. `forType`/`forName` translated 1:1. `UtilClientTimeout.java`
(`client/util/`, previously deferred pending a headless `StatusReport` path) →
`client/util/util_client_timeout.rs`: `StatusReport` was made real/headless in Phase ZW.2 Batch C
and every report renderer already reaches it via an explicit `&mut StatusReport` parameter (not
through `getUserInterface()`) — `showTimeoutStatus` follows that same convention, taking
`&mut FantasyFootballClient` + `&mut StatusReport` explicitly; `game.isTimeoutPossible()`/
`isHomePlaying()` map directly to the already-public `Game::timeout_possible`/`home_playing` fields.
`UserInterface.java` (463 lines, `extends JFrame implements WindowListener, IDialogCloseListener`,
`JDesktopPane`/icon/font/sound-engine fields throughout) was mis-marked `○` instead of `—`
(genuinely Swing GUI, same category as the other permanently-skipped rows) — corrected, no Rust
written. Deleted the 2 dead orphaned PascalCase stub files these two targets previously sat at
(`UtilClientTimeout.rs`; `LogicPluginFactory.rs` never existed as a stub), matching the standing
per-batch convention. **This closes out the last remaining `○` rows in the entire tracker** — every
in-scope file (everything except the permanent Swing-GUI `—` rows) is now `✓`, except the one
intentionally-`~` `UtilServerHttpClient.java` (documented elsewhere; live HTTP wiring gap, not
touched this batch). Tests: 17,321 → 17,331 (+10, 5 per new file).

**Phase ZVE (Join/JoinApproved/SocketClosed wired — final batch of command-hierarchy reconciliation, this session):**
Closed the last 3 gaps from the ZVA–ZVD dispatch-wiring effort. `ClientCommand::ClientJoin`'s
arm in `ServerCommandHandlerFactory::handle_command` (which only fires for a *repeat* Join —
the very first Join on a connection is still special-cased in `command_socket.rs` before
enqueue, matching a Rust-specific optimization, not a Java behavior difference: Java's
`ServerCommandHandlerJoin` itself never special-cases the first message) now builds a real
`ffb_protocol::commands::client_command_join::ClientCommandJoin` from the wire `ClientJoin`
struct's fields (`game_id` → `game_name`, since `command_socket.rs` already treats that field as
a name, not a numeric id; `password_hash` → `password`; no `client_mode` field exists on the
wire struct, so a re-join can never resolve as REPLAY — a documented, narrow gap in the wire
struct itself) and dispatches to the real `join_handler`, whose success path already redispatches
`InternalServerCommandJoinApproved` back through `handle_internal_command` to the real
`join_approved_handler` (verified end-to-end, not just assumed from doc comments, by a new test
alongside the pre-existing `factory_dispatches_join_approved_through_real_handler` coverage).
`SocketClosed` isn't a `ClientCommand` — Java's `CommandSocket.onClose()` calls
`ServerCommunication.close(session)`, which enqueues `InternalServerCommandSocketClosed` onto the
same dispatch queue `onMessage()` uses, rather than calling `SessionManager.removeSession`
directly. `command_socket.rs`'s disconnect-cleanup previously called `sm.remove_session(...)`
directly, bypassing `ServerCommandHandlerSocketClosed`'s sketch-cleanup/leave-broadcast/replay-
control-handoff side effects entirely and duplicating the removal that handler already performs
internally; it now instead sends `AnyInternalServerCommand::SocketClosed` on
`AppState::dispatch_tx` (the same sender `ServerCommunication::new` already wires to a live
`dispatch_loop`), matching Java's real single-flow shape with no plumbing beyond the
already-present `dispatch_tx` field. Added 3 new tests: two proving the `ClientJoin` re-join arm
reaches `join_handler` (lobby-list and targeted-join-by-name cases), one proving
`command_socket.rs`'s exact cleanup statement reaches the real `ServerCommandHandlerSocketClosed`
(observed via its `serverLeave` broadcast side effect, not just session removal, to rule out a
bare `remove_session` masquerading as the fix). `cargo test --workspace`: 17,318 → 17,321 tests
(+3), 0 failures.

**This closes the command-hierarchy-reconciliation effort started in Phase ZVA.** Of the 32 real
`ServerCommandHandler*` structs under `ffb-server/src/handler` (excluding the factory itself),
29 are now reachable from live dispatch. **3 documented stragglers remain, pre-existing from
earlier phases, not touched by this batch:** `ServerCommandHandlerAddLoadedTeam` (constructed in
the factory, but its `AddLoadedTeam` dispatch arm stays a no-op — `InternalServerCommandAddLoadedTeam`
carries no typed `Team` payload to hand it); `ServerCommandHandlerApplyAutomatedPlayerMarkings` and
`ServerCommandHandlerCalculateAutomaticPlayerMarkings` (neither is even constructed in the
factory — both internal commands carry their `AutoMarkingConfig`/`Game` payloads as opaque,
undecoded `String`s with no serde impl to bridge them). All 3 are real and directly unit-tested
in their own modules; the gaps are documented `// java:` no-ops in `handle_internal_command`,
not fabricated logic. Closing them requires new wire-payload shapes (a `Team`/`AutoMarkingConfig`
JSON encoding that doesn't exist in Java or Rust today), which is real, separately-scoped
follow-up work — not a hierarchy-bridging gap like the 27+3 handlers this 5-batch effort closed.

**Phase ZVD (game-management handler family wired into live dispatch, this session):**
Wired the game-management handler family — `ServerCommandHandlerAddLoadedTeam`,
`ServerCommandHandlerFumbblTeamLoaded`, `ServerCommandHandlerFumbblGameChecked`,
`ServerCommandHandlerScheduleGame`, `ServerCommandHandlerCloseGame`,
`ServerCommandHandlerUploadGame`, `ServerCommandHandlerUserSettings` (all already real and
unit-tested) — into `ServerCommandHandlerFactory`'s live dispatch, following the Phase
ZVA/ZVB/ZVC pattern. Six are `AnyInternalServerCommand`s (variants already existed) wired into
`handle_internal_command`; `UserSettings` is a `ClientCommand`, so a new `ClientUserSettings`
variant (mirroring `ffb_protocol::commands::client_command_user_settings::ClientCommandUserSettings`'s
shape) was added to `ffb_protocol::client_commands::ClientCommand` and wired into
`handle_command`. `ScheduleGame`/`UploadGame`/`FumbblGameChecked` reuse the factory's existing
shared `game_cache`/`team_cache`/`roster_cache`/`markings_http_client` fields — no new
dependencies needed. `CloseGame` needed a real `&ServerCommunication`, which the factory
couldn't previously hold without a circular "factory owns a `ServerCommunication` that owns the
factory" loop (`ServerCommunication::new` spawns the factory's own dispatch task); added
`ServerCommunication::from_parts` (a `pub(crate)` constructor sharing an existing
`tx`/`session_manager`/`replay_session_manager` triple instead of spawning a second dispatch
loop) so the factory can build one for real. `AddLoadedTeam`'s dispatch arm stays a documented
no-op — `InternalServerCommandAddLoadedTeam` carries no `Team` payload to hand its handler (see
that handler's own doc comment), so `add_loaded_team_handler` is `pub(crate)` and exercised
directly by this file's own tests instead, matching the `join_handler`/
`load_automatic_player_markings_handler` precedent for handlers not reachable via dispatch.
Two pre-existing Send-future bugs surfaced now that these handlers run inside
`tokio::spawn(dispatch_loop(...))` for the first time: `ServerCommandHandlerUserSettings`
held a `std::sync::MutexGuard<DbConnectionManager>` across an `.await` (fixed by cloning the
manager out first, same pattern as `ServerCommandHandlerDeleteGame`); and
`ServerCommandHandlerFumbblTeamLoaded::handle_command` took `game_cache: &GameCache` and
implicitly held a reference into it (via `Box<dyn Step>`, non-`Sync`) across its own internal
`.await` — changed its signature to take an already-resolved, owned `Option<&Game>` instead
(`Game: Clone`), so the caller can lock, clone, and drop the `GameCache` lock before awaiting.
`cargo test --workspace`: 17310 -> 17318 tests, 0 failures.

**Phase ZVC (replay handler family wired into live dispatch, this session):**
Wired the replay-playback handler family — `ServerCommandHandlerJoinReplay`,
`ServerCommandHandlerReplay`, `ServerCommandHandlerReplayLoaded`, `ServerCommandHandlerReplayStatus`
(all already real and unit-tested from Phase AAB's replay-playback engine work) — into
`ServerCommandHandlerFactory::handle_command`'s live dispatch, following the Phase ZVA/ZVB
pattern. Adds `ClientJoinReplay`/`ClientReplay`/`ClientReplayStatus` variants to
`ffb_protocol::client_commands::ClientCommand` (mirroring the real `ffb_protocol::commands::*`
struct each handler was translated against); `ServerCommandHandlerReplayLoaded` is an
`AnyInternalServerCommand::ReplayLoaded` (that variant already existed) rather than a
`ClientCommand`, so it's wired into `handle_internal_command` instead. The factory gained a
real `Arc<Mutex<ReplayCache>>` (`replay_cache`, shared with `join_replay_handler` — the only
handler in this factory translated directly against `ReplayCache`) and a shared
`Arc<Mutex<ServerReplayer>>` (`replayer`, shared between `replay_handler` and
`replay_loaded_handler`, matching Java's single `getServer().getReplayer()`).
`ServerCommandHandlerReplayStatus` reuses the factory's existing ad hoc `replay_states` map
(now `pub(crate)` so this file's own tests can seed a cached `ReplayState`), the same
documented "no server-level `ReplayCache` wired in yet" stand-in `transfer_control_handler`/
`set_prevent_sketching_handler` already used — no new gap introduced, no fabricated logic.
`cargo test --workspace`: 17306 -> 17310 tests, 0 failures.

**Phase ZVB (sketch/marker handler family wired into live dispatch, this session):**
Wired the sketch/marker family of already-translated, already-unit-tested `ServerCommandHandler*`
structs into `ServerCommandHandlerFactory::handle_command`'s live dispatch, following the
Phase ZVA pattern: `ServerCommandHandlerAddSketch`, `ServerCommandHandlerClearSketches`,
`ServerCommandHandlerRemoveSketches`, `ServerCommandHandlerSketchAddCoordinate`,
`ServerCommandHandlerSketchSetColor`, `ServerCommandHandlerSketchSetLabel`,
`ServerCommandHandlerSetMarker`, `ServerCommandHandlerSetPreventSketching`,
`ServerCommandHandlerUpdatePlayerMarkings`, and `ServerCommandHandlerLoadAutomaticPlayerMarkings`
each got a new `ffb_protocol::client_commands::ClientCommand` variant (`ClientAddSketch`,
`ClientClearSketches`, `ClientRemoveSketches`, `ClientSketchAddCoordinate`,
`ClientSketchSetColor`, `ClientSketchSetLabel`, `ClientSetMarker`, `ClientSetPreventSketching`,
`ClientUpdatePlayerMarkings`, `ClientLoadAutomaticPlayerMarkings`) mirroring the field shape of
the `ffb_protocol::commands::*` struct each handler was originally translated against, plus a
factory dispatch arm and constructor wiring. All six sketch handlers now share the factory's one
`ServerSketchManager` (already built for `ServerCommandHandlerSocketClosed`), exposed as a new
`pub sketch_manager` field; `SetMarker` reuses the factory's existing `game_cache`/
`session_manager`; `SetPreventSketching` reuses the `replay_states` map already created for
`TransferControl` (same documented "no server-level `ReplayCache` yet" gap); `UpdatePlayerMarkings`/
`LoadAutomaticPlayerMarkings` share one new `ServerRequestProcessor` and HTTP client (URL template
defaults to empty, same "no server-startup config wiring yet" gap as `team_cache`/`roster_cache`).
`ServerCommandHandlerApplyAutomatedPlayerMarkings` and
`ServerCommandHandlerCalculateAutomaticPlayerMarkings` are `AnyInternalServerCommand`s whose
`InternalServerCommand*` payload carries `AutoMarkingConfig`/`Game` as opaque, undecoded `String`s
(no serde impl exists for `AutoMarkingConfig`); rather than inventing a decode format, these two
got explicit, documented no-op match arms in `handle_internal_command` (each handler itself
remains real and unit-tested on its own).

Fixed two more pre-existing `Send`-future bugs uncovered by making these handlers reachable from
`tokio::spawn(dispatch_loop(...))`: `ServerCommandHandlerUpdatePlayerMarkings::handle_command` now
takes `&Arc<Mutex<GameCache>>`/`&Arc<Mutex<SessionManager>>` instead of plain references, locking
and dropping around each `.await` individually instead of the factory holding both `MutexGuard`s
for the whole async call (same shape as `ServerCommandHandlerDeleteGame`'s existing
`DbConnectionManager` clone-out fix); and a new `LazyReqwestHttpClient`
(`crates/ffb-server/src/request/fumbbl/util_fumbbl_request.rs`) builds a transient
`reqwest::blocking::Client` per `fetch_page` call instead of the factory eagerly constructing one
`ReqwestHttpClient` up front — the latter panicked ("Cannot drop a runtime in a context where
blocking is not allowed") the moment any `#[tokio::test]` constructed *and dropped* a factory,
since `ServerRequestProcessor::run()` (the only real caller of `fetch_page`) runs off the async
dispatch path so a transient client there is safe. Added a unit test per newly-wired arm in
`server_command_handler_factory.rs` proving the real handler runs. `cargo test --workspace`:
17286 → 17306 tests, 0 failures.

**Phase ZVA (session/game-lifecycle handler family wired into live dispatch, this session):**
Wired the session/game-lifecycle family of already-translated, already-unit-tested
`ServerCommandHandler*` structs into `ServerCommandHandlerFactory::handle_command`'s live
dispatch, which previously only reached these via `Action`-decoded gameplay commands or the
`ClientPing` special case: `ServerCommandHandlerTalk`, `ServerCommandHandlerCloseSession`,
`ServerCommandHandlerTransferControl`, `ServerCommandHandlerRequestVersion`, and
`ServerCommandHandlerPasswordChallenge` each got a new `ffb_protocol::client_commands::ClientCommand`
variant (`ClientTalk`, `ClientCloseSession`, `ClientTransferReplayControl`, `ClientRequestVersion`,
`ClientPasswordChallenge`) mirroring the field shape of the `ffb_protocol::commands::*` struct each
handler was originally translated against, plus a factory dispatch arm and constructor wiring
(`ServerCommandHandlerFactory::with_replay_session_manager`). `ServerCommandHandlerDeleteGame` is an
`AnyInternalServerCommand` rather than a `ClientCommand`, so it was wired into
`handle_internal_command`'s match instead of getting a new wire variant. Along the way fixed two
pre-existing `Send`-future bugs that only surfaced once these handlers became reachable from
`tokio::spawn(dispatch_loop(...))`: `ServerCommandHandlerDeleteGame::handle_command` held a
`std::sync::MutexGuard` across an `.await` (fixed by cloning the already-`Clone`
`DbConnectionManager` out from behind the mutex first), and `ServerCommandHandlerPasswordChallenge`'s
call now runs inside `tokio::task::spawn_blocking` since building/dropping its
`reqwest::blocking::Client` from inside a live tokio runtime context panics. Added a unit test per
newly-wired arm in `server_command_handler_factory.rs` proving the real handler runs (not just that
it compiles). `cargo test --workspace`: 17275 → 17286 tests, 0 failures.

**Phase AAC (UploadGame HTTP-backup branch, completed, this session):**
Third of 3 sub-phases closing the last "translate more Java" gap: `ServerCommandHandlerUploadGame`'s
missing-game branch (Java: fetch the game from the backup service via `ServerRequestLoadReplay`,
rehydrate it, re-add to the cache, and redispatch `InternalServerCommandUploadGame`), previously a
`todo!("Phase ZZ: ...")`.

- Added `GameState::init_from(&mut self, json: &str) -> Result<(), String>`
  (`crates/ffb-server/src/game_state.rs`) — Java: `GameState.initFrom(IFactorySource, JsonValue)`.
  Deserializes `json` directly into the already-`Serialize`/`Deserialize`-derived `Game` struct
  and rehydrates via `DriverGameState::from_game`, resetting the command counter and game log —
  the same *control-flow* shape as Java's method (deserialize → rehydrate → reset transient
  step/log state). Documented narrowing: Java's real `initFrom` reconstructs `fCurrentStep`/
  `fStepStack`/`fGameLog`/`passState`/`blitzTurnState`/`prayerState`/`activeEffects` from a
  custom `IServerJsonOption`-keyed wire format that has no decoder in this crate; this uses this
  crate's own serde shape instead of matching real FUMBBL backup JSON byte-for-byte, which would
  be a separate, much larger follow-up.
- Made `ServerRequestLoadReplay` genuinely queueable: added `QueuedServerRequestLoadReplay`
  (`crates/ffb-server/src/request/server_request_load_replay.rs`), a `ServerRequest`-trait
  adapter around the existing fetch-only `ServerRequestLoadReplay::process(client, url)`, matching
  the established adapter pattern (`util/marker_loading_service.rs`'s
  `QueuedLoadPlayerMarkingsRequest`). Ports all three of Java's `process()` mode branches
  (`LOAD_GAME`/`DELETE_GAME`/`UPLOAD_GAME`, one shared Java method) since the dependencies
  (`GameCache`, a `dispatch_tx: mpsc::UnboundedSender<ReceivedCommand>` redispatch sink) are
  threaded through as adapter fields rather than reached via a `getServer()` singleton. The
  `LOAD_GAME`-not-found sub-case (`communication.sendStatus(session, REPLAY_UNAVAILABLE, "")`)
  is a documented no-op — no `SessionManager`/status-send dependency was threaded into this
  adapter, being out of scope for the `UPLOAD_GAME` branch this phase actually targets.
- Wired `ServerCommandHandlerUploadGame` (`crates/ffb-server/src/handler/server_command_handler_upload_game.rs`)
  with a `request_processor: Arc<Mutex<ServerRequestProcessor>>`, an `Arc<dyn HttpClient + Send +
  Sync>`, a `backup_url_load_template`, and a `dispatch_tx`, matching the DI convention already
  established by `ServerCommandHandlerUpdatePlayerMarkings`/`ServerCommandHandlerJoin`. The
  missing-game branch now builds the real `ServerRequestLoadReplay`, wraps it in
  `QueuedServerRequestLoadReplay`, and enqueues it via `ServerRequestProcessor::add` — closing the
  literal 4-line Java gap this sub-phase targeted. `handle_command` gained a `session_id`
  parameter (Java's `receivedCommand.getSession()`) since nothing called it in production yet
  (it isn't wired into `ServerCommandHandlerFactory`'s dispatch match — same pre-existing,
  separately-scoped gap as ~12 other internal command types documented in that file).
- Corrected a stale `TRANSLATION_TRACKER.md` mark: `server/util/UtilServerHttpClient.java` was
  marked `✓` despite all three methods being `todo!()` with no real caller anywhere in
  `ffb-engine` — per that crate's own architecture doc, `ffb-engine` is deliberately
  networking-free (`GameEvent`/`AgentPrompt` output channels instead of direct HTTP), with real
  HTTP already implemented one layer up in `ffb-server`'s `ReqwestHttpClient`. Left the `todo!()`s
  in place (documented why: no caller, and implementing them would duplicate `ReqwestHttpClient`
  rather than reuse it) and fixed the mark to `~`.
- No parity/integration testing this phase (per plan); unit tests only. New tests cover
  `GameState::init_from` (round-trip, command-nr/game-log reset, invalid-JSON error),
  `QueuedServerRequestLoadReplay`'s three mode branches plus the missing-backup/invalid-JSON/
  HTTP-error no-op paths, and `ServerCommandHandlerUploadGame`'s enqueue + full-round-trip
  (enqueue → drain via `ServerRequestProcessor::run()` → cache re-add → redispatch observed on
  the channel) paths. Tests: 17,249 → 17,259 (net +10), 0 failures.

**Phase AAB (replay playback engine, mostly completed, this session):**
Second of 3 planned sub-phases closing the last "translate more Java" gap: wiring the
replay-playback engine (`ServerReplayer`/`ServerReplay`/`ReplayCache`/`ReplayState`/
`ServerSketchManager`, all real in `ffb-engine` but never reachable from `ffb-server`) and
the 3 handlers that were blocked on it (`ServerCommandHandlerJoinReplay`/`Replay`/
`ReplayLoaded`).

- **Architectural decision:** `ServerReplayer::run()` needs to push a serialized replay
  batch out to a live session, which in this crate means going through `ffb-server`'s
  `SessionManager`. Since `ffb-engine` must not depend on `ffb-server` (only the reverse),
  `run()` is parametrized over a new `ffb_engine::server_replayer::ReplaySender` trait
  (`fn send(&self, session: u64, message: &str)`) instead of moving `ServerReplayer`/
  `ServerReplay`/`ReplayCache` into `ffb-server` — the dependency-inversion option the plan
  called out as preferred. `ffb-server`'s `SessionManager` implements `ReplaySender` by
  delegating to its existing `send_to` (`net/session_manager.rs`).
- Fixed `ffb-engine::replay_cache::ReplayCache`, which had accidentally grown its own
  duplicate 1-field `ReplayState` stub instead of using the real, already-fully-translated
  `replay_state::ReplayState` (name/command_nr/speed/running/forward/prevented-coaches) —
  merged onto the real type and added `replay_state_mut`.
- Rewrote `ffb-engine::server_replay::ServerReplay` to actually snapshot a `GameLog`: its
  single Java constructor (`ServerReplay(GameState, int, Session)`) decomposes into
  `(game_id, to_command_nr, session, &GameLog)` (a `GameLog` reference, not a whole
  `GameState`, since `GameLog` itself already lives in `ffb-engine`). Ported
  `orderCommands()` (Java renumbers every command in the shared array in place via
  `setCommandNr(i+1)`; since `AnyServerCommand` isn't `Clone` and only reachable through
  `GameLog`'s `Mutex` guard, each command is instead serialized once via
  `to_json_value()` with `commandNr` overwritten in that JSON copy — renumbered
  command_nr always equals array position, so no parallel index needs tracking) and
  `findRelevantCommandsInLog()` (filters those serialized strings by from/to command_nr).
- Implemented `ServerReplayer::run()` for real against the Java source: drains the queue
  (this crate's established "synchronous drain" convention for what Java runs as a
  blocking background thread — see `ServerRequestProcessor::run`'s doc comment), chunks
  each replay's relevant commands into `ServerCommandReplay::MAX_NR_OF_COMMANDS`-sized
  batches, detects marking-affecting commands (`SERVER_ADD_PLAYER` always; `SERVER_MODEL_SYNC`
  when its `modelChangeList.changes` contains one of the 7 `ModelChangeId`s Java's
  `markingAffectingChanges` set names) by inspecting each batch entry's parsed JSON rather
  than a typed `AnyServerCommand` match (the commands are already JSON strings by this
  point — see `ServerReplay`'s doc comment), and sends one `ServerCommandReplay`-shaped
  JSON message per batch via `ReplaySender::send`. Discovered and added a missing
  `ModelChangeId::FieldModelRemovePrayer` variant in `ffb-model` (Java's
  `FIELD_MODEL_REMOVE_PRAYER`, needed by the marking-affecting-changes set — falls through
  to the existing `ModelChangeDataType::String` default, so no new match arm needed).
- Completed `ffb-server::util::server_replay::start_server_replay` — replaced the
  documented no-op with a real `ServerReplay::new(...)` + `ServerReplayer::add(...)` call,
  now that both are reachable.
- **`ServerCommandHandlerJoinReplay` — closed.** Both branches (new `ReplayState`
  created and control granted; existing `ReplayState` resumed with status/control/
  prevent-sketching/add-sketches replayed to the joining session) are real, using the now-
  wired `ReplayCache`/`ReplayState`/`ServerSketchManager` and the `ServerCommandJoin`/
  `ServerCommandReplayControl`/`ServerCommandReplayStatus`/`ServerCommandSetPreventSketching`/
  `ServerCommandAddSketches` wire commands. One precise, documented fidelity gap: converting
  `ffb_engine::server_sketch_manager::Sketch` (id/coordinates/label/rgb) to
  `ffb_model::model::sketch::sketch::Sketch` (positions only — the type
  `ServerCommandAddSketches` actually carries, from an earlier phase) can only carry the
  coordinate list across; `id`/`label`/`rgb` have no field to land in and are dropped. Real
  data, not fabricated — just lossy on those 3 fields; unifying the two `Sketch` models is
  flagged as a separate follow-up.
- **`ServerCommandHandlerReplayLoaded` — closed.** The found-game branch now really sets
  `GameState::status` (a new field on `ffb-server`'s `GameState`, added to match Java's
  separate `fStatus` — distinct from `Game.status`, `GameStatus::Replaying` was already
  a defined variant) and starts the replay for real; the not-found branch (send
  `ERROR_UNKNOWN_GAME_ID`) was already real from before this phase.
- **`ServerCommandHandlerReplay` — partially closed.** The found-in-cache branch is real
  (starts the replay). The not-found branch's `GameCache::query_from_db` call is real, but
  per that method's own pre-existing doc comment it can only return raw gzipped blob bytes
  (no gunzip-to-`GameState` pipeline exists in this crate), so there's no reconstructed
  `GameState` to `addGame` even on a successful query — the final fallback (enqueue
  `ServerRequestLoadReplay` on `ServerRequestProcessor`) stays a documented `todo!()`:
  `ServerRequestLoadReplay` does not implement the `ServerRequest` trait (its `process`
  takes an `HttpClient` + URL template, not `ServerRequest::process(&self)`) — checked this
  session and confirmed still unfixed, that trait-signature fix is Phase AAC's scope.
- No parity/integration testing this phase (per plan); unit tests only, one per handler
  branch closed plus dedicated `ServerReplayer::run()` dispatch tests (enqueue real
  `GameLog` commands → run the real drain/batch loop → observe a mock `ReplaySender`).
  Tests: 17,249 → 17,265 (net +16). **Follow-up wiring pass, immediately after merging AAB and
  AAC together this session:** with Phase AAC's `QueuedServerRequestLoadReplay` adapter now
  landed, `ServerCommandHandlerReplay`'s not-found branch's final fallback was closed for real —
  see the merge note below for the exact change and final test count.

**Merge + final wiring pass (all 3 sub-phases, this session): 11/11 originally-deferred
`ffb-server` handlers now closed — 0 remain.** Merged Phase AAA → AAC → AAB sequentially
(AAC fast-forwarded cleanly onto AAA; AAB conflicted only in this file and `SESSION.md`,
both resolved by concatenating each sub-phase's entry — no source-code conflicts). With
both `ServerRequestLoadReplay`'s `ServerRequest`-trait fix (Phase AAC) and the replay
playback engine (Phase AAB) now on `main` together, `ServerCommandHandlerReplay`'s
not-found branch — the one piece both sub-phases had independently left as a documented
`todo!()`, each blocked on the other's half — was wired for real: it now builds a
`ServerRequestLoadReplay` (mode `LOAD_GAME`), wraps it in `QueuedServerRequestLoadReplay`,
and enqueues it on a `ServerRequestProcessor` the handler's constructor now receives
(matching the DI convention `ServerCommandHandlerUploadGame` established). Once the backup
service answers, that adapter re-adds the rehydrated `GameState` to the cache and
redispatches `InternalServerCommandReplayLoaded`, which `ServerCommandHandlerReplayLoaded`
picks up to actually start the replay — closing the loop Java's own `handleCommand` describes.
Fixed 3 pre-existing test failures surfaced by this change: 3 of this handler's
`#[tokio::test]`s constructed a real `ReqwestHttpClient` (which builds its own tokio runtime)
from inside an async test context, which panics on drop (`"Cannot drop a runtime in a
context where blocking is not allowed"`) — switched those to the `MockHttpClient`-backed
`handler_with` helper already used elsewhere in this crate's tests, matching the established
pattern; no production-code behavior changed by that fix. Every ffb-server handler that was
deliberately deferred as of Phase ZW.1 (35 files, 11 handlers) is now genuinely `todo!()`-free
and unit-tested — remaining known gaps are the separately-documented, out-of-scope items
(the two parallel command hierarchies not yet reconciled in `command_socket.rs`, and
`UtilServerHttpClient`'s deliberate stub since its concern is already covered by
`ReqwestHttpClient`). Tests: 17,265 (AAB alone) / 17,259 (AAC alone) → **17,275** after
merging both plus this closing pass, 0 failures.

**Phase AAA (GameLog wiring, completed, this session):**
First of 3 planned sub-phases closing the last "translate more Java" gap: wiring a typed,
replayable command log into `ffb-server`'s live `GameState`, instead of the untyped
`Vec<String>` placeholder it had carried since Phase ZT.

- Added `AnyServerCommand::get_command_nr(&self) -> i32` and `is_replayable(&self) -> bool` in
  `ffb-protocol` (`commands/any_server_command.rs`) — one match arm per of the 32 genuine
  `ServerCommand*` variants, delegating to each struct's own `command_nr` field. For
  `is_replayable`, 9 variants (`ServerAdminMessage`, `ServerAutomaticPlayerMarkings`,
  `ServerGameList`, `ServerGameState`, `ServerGameTime`, `ServerJoin`, `ServerLeave`,
  `ServerPasswordChallenge`, `ServerReplay`) have their own Java-side override and delegate to
  it; the other 23 have no override in Java and fall back to the `ServerCommand` base class's
  `true` default (`ServerCommandGameTime::is_replayable()` correctly returns `false`, exercised
  by a dedicated unit test).
- Rewrote `ffb_engine::GameLog` (`crates/ffb-engine/src/game_log.rs`) to store
  `Vec<AnyServerCommand>` instead of `Vec<String>`, fixing a pre-existing bug where `add()` had
  no `isReplayable()` guard (Java: `if (pServerCommand.isReplayable()) fServerCommands.add(...)`)
  and implementing the two previously-`todo!()`'d methods: `get_uncommitted_server_commands()`
  (filters by `command_nr > last_committed_command_nr`) and `find_max_command_nr()` (max
  `command_nr` across stored commands, `0` for an empty log, matching Java's `maxCommandNr = 0`
  seed). `get_server_commands()` now returns the locked `MutexGuard<Vec<AnyServerCommand>>`
  rather than a cloned `Vec` — `AnyServerCommand` isn't `Clone` (holds a non-`Clone`
  `ServerCommandModelSync` variant), and `get_uncommitted_server_commands()` returns `Vec<i32>`
  of command numbers rather than command references for the same reason, since nothing
  downstream yet needs the full payloads.
- Added `ffb-protocol` as a new `ffb-engine` dependency (checked first: no cycle, since
  `ffb-protocol` only depends on `ffb-model`).
- Wired the live `ffb-server::GameState` (`crates/ffb-server/src/game_state.rs`) to use
  `ffb_engine::GameLog` in place of its old `game_log: Vec<String>` field. Grepped
  `crates/ffb-server/src` for every `.game_log` call site: there were none outside the field's
  own declaration/initializer, so no downstream code needed to be ported to a real
  `AnyServerCommand`-constructing call site this phase — the field exists but nothing in the
  live server pushes to it yet. There was no pre-existing debug-string-pushing call site to
  replace, unlike the scenario the plan anticipated, so no `// java:` gap comment was needed at
  a call site.
- Left `ffb-engine::game_state::GameState` (the separate, dead struct used only by
  `ffb-engine::fantasy_football_server.rs`) untouched — it already held a `GameLog` field and
  never called `.add()`, so the type change didn't require touching it, confirming it really is
  unused dead code on this path.
- No parity/integration testing this phase (per plan); unit tests only, colocated with the
  changed files. Tests: 17,238 → 17,249 (net +11: new `AnyServerCommand` accessor tests plus
  rewritten/added `GameLog` tests for the replayable-guard, uncommitted-filter, and
  max-command-nr logic).

**Phase ZZ (async dispatch + Join/JoinApproved wiring, completed, 2026-07-11):**
Closed the two remaining server handler gaps Phase ZY narrowed but left blocked
(`ServerCommandHandlerJoin`/`JoinApproved`) by threading async/DB access through the
dispatch path they'd been waiting on since Phase ZX.

- `net::server_communication::dispatch_loop` is now `async` (awaits `factory.handle_command`
  inside its existing `tokio::spawn`), and `ServerCommunication::new`/
  `ServerCommandHandlerFactory::{new,with_replay_session_manager}` take a
  `db_connection_manager: Arc<Mutex<DbConnectionManager>>` parameter, threaded to two new
  factory fields (`team_cache`, `roster_cache`, `client_properties`, defaulted empty — no
  server-startup wiring exists yet to populate them for real) and two new handler instances
  (`join_handler`, `join_approved_handler`).
- `SessionManager::sender_for` (new accessor) exposes a registered session's outgoing sender
  by id, since Java re-queries a live `Session` object this crate's `SessionManager` doesn't
  hold — needed so the factory's new `AnyInternalServerCommand::JoinApproved` dispatch arm
  can supply `join_approved_handler.handle_command` its `sender` parameter.
- `ServerCommandHandlerJoin::handle_command` is now `async`: the targeted-join branch calls
  `DbPasswordForCoachQuery::execute` (gated by `pool_ready()`), then either redispatches
  `InternalServerCommandJoinApproved` through a cloned dispatch-queue sender or sends
  `ERROR_WRONG_PASSWORD`. FUMBBL-mode authorization stays a documented no-op (no
  `ServerMode`/request-processor plumbing here, same class of gap as `ScheduleGame`'s own
  FUMBBL branch).
- `ServerCommandHandlerJoinApproved::handle_command` is now `async` with all three
  previously-`todo!()`'d branches implemented: SPECTATOR calls the real `send_server_join`
  (removing the redundant manual `add_session` call — `send_server_join` already does this
  internally); REPLAY calls the real `send_user_settings`; PLAYER translates
  `joinWithoutTeam`/`joinWithTeam`/`sendTeamList` 1:1, resolving teams via
  `GameCache::get_team_by_id`/`TeamCache`/`RosterCache` and calling
  `join_game_as_player_and_check_if_ready_to_start` + `start_game`. `GameCache::take_game_state`
  (new) lets `start_game`'s awaited DB calls run without holding the cache's `Mutex` guard
  across an `.await` (required for the `tokio::spawn`ed dispatch loop to stay `Send`).
- Two structural gaps, documented rather than invented: (1) this crate's `GameState` has no
  `Game` at all until both teams are attached via `start_game` (no empty/skeleton `Game`
  slot, unlike Java's always-present blank `Game`), so a freshly-created-by-name game with no
  team id yet degrades to `sendTeamList`/`joinWithTeam` against blank coach fields — matching
  Java's practical outcome, just derived differently; (2) `GameState::is_started()` (the
  engine driver being initialized) doubles as the proxy for both Java's
  `GameStatus.SCHEDULED` gate and its separate `game.getStarted() != null` check, since this
  crate's `Game` has neither a `scheduled` nor `started` timestamp field (an existing,
  previously-documented gap elsewhere in this crate).
- `ServerRequestProcessor`'s queued-request trait object gained a `+ Send + Sync` bound
  (compile-time-only tightening — every existing `ServerRequest` implementor already stores
  only `Send + Sync` data) so the now-async `JoinApproved` path, reached from inside
  `dispatch_loop`'s `tokio::spawn`, type-checks; no behavior change.
- Cleanup: deleted 4 confirmed-orphaned `ffb-engine` stub files (`roster_cache.rs`,
  `team_cache.rs`, `util/util_server_replay.rs`, `util/marker_loading_service.rs` — dead
  duplicates superseded by the real, live `ffb-server` versions) and 55 confirmed-orphaned
  stub files under `ffb-model/src/factory/*` (superseded by the real implementations in
  `ffb-mechanics`; verified via grep that every real cross-crate usage of these names routes
  through `ffb_mechanics::modifiers::*`, never `ffb_model::factory::*` — more than the ~24
  the plan estimated, since independent re-verification found a larger safely-orphaned set).

Result: **7 of the 11 originally-deferred server handlers are now fully wired and
`todo!`-free** (adds `ServerCommandHandlerJoin`/`JoinApproved` to Phase ZX/ZY's 5). **4
remain genuinely blocked**: `JoinReplay`/`Replay`/`ReplayLoaded` (the ~1,330-Java-LOC
replay-record/playback engine) and `UploadGame`'s missing-game branch (HTTP backup-service
dispatch) — both named as the next major step, not started here. `command_socket.rs` still
special-cases `ClientJoin` inline before the factory ever sees it, so `ServerCommandHandlerJoin`
is not yet reachable from live WebSocket traffic — bridging the two parallel command
hierarchies remains the separately-documented, out-of-scope gap `ServerCommandHandlerFactory`'s
own doc comment already flagged. No parity/integration testing this phase (per plan). Tests:
17,399 → 17,238 (net -161: cleanup deleted more orphaned-stub tests than the phase added).

**Phase ZY (close 2 more server handler gaps + redispatch sink infra, completed, 2026-07-11):**
Followed up on Phase ZX's 8 remaining blocked `ServerCommandHandler*` files by building two
of the four named missing infra pieces and using them to close two more handlers for real.

- **ZY.1 — step-stack clear + EndGame dispatch** (`DriverStepStack::clear`,
  `DriverGameState::clear_step_stack`/`push_end_game_sequence` in `ffb-engine`, bridged onto
  the server `GameState` wrapper). Closes `ServerCommandHandlerUploadGame`'s known-game
  branch (sets `GameResult` conceded flags, clears the stack, pushes `EndGame`, drives to
  completion); the missing-game branch (`ServerRequestProcessor` + HTTP backup service)
  stays a separate, narrower `todo!()`.
- **ZY.2 — XML roster/team deserializer** (`ffb_model::xml`: `XmlHandler`/`IXmlReadable`/
  `UtilXml`, a 1:1 port of `com.fumbbl.ffb.xml` over `quick-xml` instead of SAX). Implements
  `IXmlReadable` on `Roster`, `RosterPosition`, `RosterSkeleton`, `Team`, `Player`,
  `TeamSkeleton`, plus `Team::update_roster`/`Player::update_position`. Rewrote
  `RosterCache`/`TeamCache` in `ffb-server` to return real `Roster`/`Team` objects (the
  `RosterXml`/`TeamXml` raw-string stopgaps and hand-rolled scanners are gone). This was the
  single largest remaining infra gap and the one most of the 8 handlers shared.
- **ZY.3 — wire two handlers onto the real objects.** `ServerCommandHandlerFumbblGameChecked`
  and `ServerCommandHandlerScheduleGame` are now fully `todo!`-free. Added
  `GameCache::get_team_by_id`/`get_teams_for_coach` (Java's `GameCache.getTeamById` +
  its private `updateRoster` helper: `TeamCache` lookup + `RosterCache.getRosterForTeam` +
  `Team::update_roster` + `UtilTeamValue` re-derivation). `ScheduleGame` resolves both teams
  and starts the engine in one `GameState::start_game` call rather than Java's two
  incremental `addTeamToGame` calls onto an empty `Game` — this crate's `Game`/
  `DriverGameState` need both teams at construction (no empty-`Game` state exists), so both
  resolved teams are supplied together, the same substitution other team-attaching handlers
  here already make. `ServerCommandHandlerJoinApproved`'s `todo!()`s were corrected (team
  loading is no longer the blocker; `join_game_as_player_and_check_if_ready_to_start`/
  `send_server_join`/`send_user_settings` are `async` + need a `DbConnectionManager` this
  handler's sync signature doesn't carry — a different, larger gap) but not closed.
- **ZY.4 — command redispatch sink.** `AnyInternalServerCommand` (sum type over the 14
  `internal_server_command_*` structs, same pattern as `ffb-protocol`'s `AnyServerCommand`/
  `AnyClientCommand`) + widened `ReceivedCommand`/`ReceivedNetCommand::{Client,Internal}` +
  `ServerCommunication::receive_internal` (Java: `handleCommand(InternalServerCommand)` →
  `fCommandQueue.offer(...)`) + a real `Internal` dispatch arm in
  `ServerCommandHandlerFactory` routing `SocketClosed` to a real handler instance (previously
  handled ad hoc in `command_socket.rs`, already flagged as a follow-up). An end-to-end test
  proves enqueue → the real `dispatch_loop` task → factory → handler. The other 13 internal
  command types fall through to a logged no-op (most need their own handler + dependencies
  wired in; several are async/DB-backed, which would need `handle_command` itself made async
  first). `ServerCommandHandlerJoin`'s targeted-join `todo!()` is corrected: the sink it would
  use to redispatch `InternalServerCommandJoinApproved` now exists, but the real blocker is
  `DbPasswordForCoachQuery` being async with no DB connection threaded through — same class
  of gap as `JoinApproved`, not resolved by the sink alone.

Result: **5 of the 11 handlers are now fully wired and `todo!`-free**
(`ServerCommandHandlerCloseGame`, `AddLoadedTeam`, `FumbblTeamLoaded` from Phase ZX, plus
`ScheduleGame` and `FumbblGameChecked` this phase). **6 remain genuinely blocked**:
`ServerCommandHandlerJoin`, `JoinApproved`, `JoinReplay`, `Replay`, `ReplayLoaded` (all now
narrowed to async DB/SessionManager plumbing or the replay-engine gap, not roster
resolution), and `UploadGame`'s missing-game branch (`ServerRequestProcessor` + HTTP backup
service). The replay/command-log playback engine (blocking `JoinReplay`/`Replay`/
`ReplayLoaded`) is deliberately deferred to a future **Phase ZZ** — it is the largest and
riskiest of the four originally-named infra pieces, and has a hidden prerequisite: nothing
in the live engine currently records a typed, replayable `ServerCommand` log for it to play
back. No parity testing this phase (per plan). Tests: 17,357 → 17,399 (+42).

**Phase ZX (server handler subsystem gaps, completed, 2026-07-11):**
Closed the majority of the last known genuine untranslated-Java gap in `ffb-server`: 11
`ServerCommandHandler*` files carrying `todo!("Phase ZV: ...")` calls, each blocked on a
named missing subsystem. Translated the 5 subsystem classes that unblock them —
`RosterCache.java`, `TeamCache.java`, `UtilServerReplay.java`, `MarkerLoadingService.java`
(all new files), and an extension of the existing in-memory `GameCache` MVP with
`addTeamToGame`/`closeGame`/`removeGame`/`queueDbUpdate`/`queueDbDelete`/`queryFromDb`/
`queueDbPlayerMarkersUpdate` — plus `UtilServerStartGame.java`'s
`joinGameAsPlayerAndCheckIfReadyToStart`/`sendServerJoin`/`sendUserSettings`/`startGame`
methods (its `addDefaultGameOptions` was already translated in a prior phase, in
`ffb-engine`). All follow the established per-class, explicit-dependency-injection
convention (no `FantasyFootballServer` god-object graph); most of the underlying DB/HTTP
primitives (`DbPasswordForCoachQuery`, `FumbblRequestCheckAuthorization`,
`ServerRequestProcessor`) were already translated and just needed wiring.

Result: **3 of the 11 handlers are now fully wired and `todo!`-free** —
`ServerCommandHandlerCloseGame`, `ServerCommandHandlerAddLoadedTeam`,
`ServerCommandHandlerFumbblTeamLoaded`. **8 remain genuinely blocked**, each now with a
narrower, more specific `todo!` than before: `ServerCommandHandlerJoin`,
`ServerCommandHandlerJoinApproved`, `ServerCommandHandlerJoinReplay`,
`ServerCommandHandlerReplay`, `ServerCommandHandlerReplayLoaded`,
`ServerCommandHandlerScheduleGame`, `ServerCommandHandlerUploadGame`,
`ServerCommandHandlerFumbblGameChecked`. The common blockers across these 8: no XML→`Team`
roster deserializer (the client already parses roster XML via a different, JSON-based data
pipeline — `RosterCache`/`TeamCache` in this phase return raw XML text rather than parsed
objects, since no SAX-equivalent `XmlHandler` exists in this crate), no server-side command
redispatch sink (`ServerCommandHandlerFactory`/`ServerCommunication`'s dispatch loop isn't
built), no replay/command-log playback engine, and no step-stack + `EndGame` sequence
dispatch. A future phase would need one or more of those four pieces of infrastructure
before the remaining 8 can close. No parity testing this phase (per plan). Tests:
17,305 → 17,357 (Phase ZX.1-2: +32; ZX.3-5: +25).

Also discovered (not fixed, flagged for a future cleanup pass): a second pocket of
fake-`✓` stub duplicates, this time in `ffb-engine` — `roster_cache.rs`, `team_cache.rs`,
`util/util_server_replay.rs`, `util/marker_loading_service.rs` already existed there as
10-line `todo!()` placeholders from the Phase ZT infra sweep, superseded by this phase's
real translations in `ffb-server`. Same shape as the 36 orphaned `ffb-model/src/factory/*`
stubs found during Phase ZX scoping (superseded by real `ffb-mechanics` implementations) —
neither pocket was deleted this phase to avoid scope creep, but both are safe, low-risk
cleanup candidates for a future session.

**Phase ZW.3 + ZW.4 (client/report renderers + docs closeout, completed, 2026-07-11):**
Translated the last major `ffb-client-logic` block: all 211 `client/report/*Message.java`
report-to-text renderers (55 root + 32 `bb2016/` + 26 `bb2020/` + 39 `bb2025/` + 57
`mixed/`), each a `ReportMessage` trait impl that builds styled text runs from an
already-translated `ffb-model` report object. Five batches translated in parallel
(one per subdirectory, isolated git worktrees), then merged and reconciled.

Prerequisite fix found before the batch could start: `client/StatusReport.java` (279
lines, the shared rendering sink every message calls through) was still a 10-line stub,
and its two style enums — `TextStyle`, `ParagraphStyle` — had been bulk-marked `—`
(permanent Swing skip) during the ZW.0 audit, mis-lumped in with genuinely-Swing root
files like `FontCache`/`IconCache` despite being plain 22-line string-keyed enums with
zero AWT dependency. Un-skipped both enums and gave `StatusReport` a real translation:
its one actual Swing call (`getUserInterface().getLog().append(...)`) became a headless
`rendered_runs: Vec<RenderedRun>` capture, making every renderer's output assertable in a
unit test without any UI. `ReportMessageBase` became a `ReportMessage` trait (Rust has no
inheritance); `ReportMessageType`'s annotation-based dispatch became a `report_id()`
method per renderer (Java's reflection-based `Scanner` registry has no faithful Rust
equivalent, so this is a mechanical substitution, not new logic).

Small legitimate gaps filled in `ffb-model` along the way (methods present on the Java
enums but not yet ported): `PlayerGender::dative()`/`self_word()`/`verb_form()` and
`PlayerAction::description()`, transcribed directly from `PlayerGender.java`/
`PlayerAction.java`. Where a renderer depended on data the report layer doesn't retain
(e.g. `RollModifier` magnitude/sign — the model only keeps resolved modifier *names*,
per the Phase ZU report-serialization decision) or on unported mechanic/factory lookups,
each file documents the gap inline with a `// java: <call>` comment rather than
fabricating values — see individual batch commits for the full list per file.

Tests: 16,412 → 17,305 (+893: ~250 root, ~126 bb2016, ~105 bb2020, ~179 bb2025, plus
mixed and the prerequisite files). `cargo test --workspace` green, `client/report/` (211
files) + the 5 prerequisite files now genuinely `✓`. This completes Phase ZW's
translation scope: **all 373 in-scope `ffb-client-logic` files are now genuinely
translated** (`client/model`, `client/util`, `client/factory`, `client/net`,
`client/handler`, `client/` root, `client/state/` 85 files, `client/report/` 211 files).
Remaining `—` rows (271 files, Swing dialog/ui/layer/animation/overlay/sound rendering)
stay permanently skipped per the ZW plan's triage. No parity work this phase, as planned.

**Phase ZW.2c (NetCommand wire-protocol layer, completed, 2026-07-10):** closed the blocker
Batch B flagged (below) — and found a second, smaller instance of the ZW.0 fake-✓-stub
pattern while doing it. `net/NetCommand.java`, `net/NetCommandFactory.java`, and all 123
`net/commands/ClientCommand*.java`/`ServerCommand*.java` classes were marked `✓` in the
tracker but `net_command.rs` was a one-variant `Unknown`-only enum (not a 1:1 translation of
the abstract `NetCommand` class) and none of the 123 leaf structs had `toJsonValue()`/
`initFrom()` — a ~126-file pocket of quietly-fake `✓` rows inside the previously-"genuinely
done" common+server count. Fixed for real this session:
- `net_command.rs` rewritten as a genuine `NetCommand` trait (`get_id()`/`get_context()`/
  `is_internal()`), matching Java's abstract class.
- `ClientCommand`/`ServerCommand` base structs got real `base_json_fields()`/
  `base_from_json()` helpers (entropy/command_nr + the shared `netCommandId` key).
- All 91 `ClientCommand*`/32 `ServerCommand*` leaf structs got the missing inherited
  `entropy`/`command_nr` field, a `NetCommand` impl, and real `to_json_value()`/`from_json()`
  matching Java's `toJsonValue()`/`initFrom()` field-for-field (wire keys verified against
  `IJsonOption.java`, not guessed) — following the Phase ZU report-serialization convention
  (manual `serde_json::json!` + manual field extraction + round-trip test per file).
- `AnyClientCommand`/`AnyServerCommand` (`ffb-protocol/src/commands/any_{client,server}
  _command.rs`) — real sum types mirroring `NetCommandId.createNetCommand()`'s switch, each
  implementing `NetCommand` and a `from_json(id, json)` dispatch constructor.
- `NetCommandFactory::for_json_value()` — the real `forJsonValue()` dispatch (reads
  `netCommandId`, routes to `AnyClientCommand`/`AnyServerCommand::from_json`). Added
  `NetCommandId::from_name()` (reverse of the pre-existing `name()`) to `ffb-model` to support
  this. This is additive: the pre-existing hand-rolled `client_commands`/`server_commands`
  simplification that the live WebSocket layer (`connection`, `network_encoder`) depends on
  today is untouched — reconciling the two hierarchies stays a separate, later, riskier
  decision (same treatment as other live-infra deferrals since Phase ZW.1).
- Unblocked `client/net/` (3 files: `ClientCommunication`'s ~90 `send*` methods, `ClientPingTask`,
  `CommandEndpoint`) and `client/handler/` (27 files: the incoming-`ServerCommand` dispatch
  factory + one handler per command, confusingly named `ClientCommandHandler*` in Java) — both
  translated for real this session, replacing all remaining PascalCase stub duplicates in
  those two directories.

Tests: 14,940 → 15,647 (+707: ~500 from the 123 struct JSON methods, ~10 from the dispatch
layer, +91 from `client/net`, +233 from `client/handler` minus the pre-existing 60). All of
`ffb-client-logic`'s `client/net/` and `client/handler/` directories (30 files) are now
genuinely `✓`, on top of the 126-file protocol-layer honesty fix. Remaining `ffb-client-logic`
`○` work: `client/state/` (85 files) and `client/report/` (211 files) — unchanged from
`docs/PHASE_ZW_PLAN.md`'s existing plan, recommended next.

**Phase ZW.0 (tracker truth reset, 2026-07-10):** the 644 `ffb-client-logic` rows were
previously all marked `✓` despite their Rust targets being ~10-line placeholder stubs, not
translations (spot check: `client/ActionKeyBindings.java` is 191 lines of key-binding logic;
its Rust file was an empty struct). `scripts/audit_client_stubs.py` reclassified all 644 rows
by directory: `dialog/`, `ui/`, `layer/`, `overlay/`, `sound/` (257 files) plus 22 root-level
Swing/AWT files (`ActionKey*`, `ClientLayout`, `Component`, `*DimensionProvider`,
`FantasyFootballClient`, `FieldComponent`, `FontCache`, `GameTitle`, `IconCache`,
`LayoutSettings`, `ParagraphStyle`, `RenderContext`, `StyleProvider`, `TextStyle`,
`UtilStyle`) → **279 files marked `—`** (no headless equivalent, permanent skip). The
remaining **365 files** (`animation/`, `factory/`, `handler/`, `model/`, `net/`, `report/`,
`state/`, `util/`, plus 9 root logic files: `ClientData`, `ClientParameters`,
`ClientReplayer`, `CoordinateConverter`, `IProgressListener`, `PlayerIconFactory`,
`ReplayControl`, `StatusReport`, `UserInterface`) → **marked `○` (not started)**, pending real
translation in ZW.2/ZW.3. No Rust code changed this step — bookkeeping only. Honest totals:

| Metric | Count (at ZW.0, before any translation) | Count (after ZW.2 Batch A) |
|--------|---|---|
| Total Java files tracked | 2979 | 2979 |
| Genuinely done (✓, common + server) | 2278 | 2278 |
| Genuinely done (✓, ffb-client-logic) | 0 | 7 |
| Not started (○, ffb-client-logic real translation, was falsely ✓) | 365 | 351 |
| Partial (~, all ffb-server infra — genuine subsystem gaps, see Phase ZW.1 note below) | 11 | 11 |
| Not translating (—, 46 server/common GUI-adjacent + 279/286 ffb-client-logic Swing/AWT) | 325 | 332 |

**Phase ZW.1 (server closeout, 2026-07-10):** closed 24 of the 35 `~` `ffb-server` rows —
the 4 missing lower-level APIs (`SoundId::all()` visibility, a real `GameOptionId` enum +
`GameOptionFactory`, `SeriousInjuryFactory`), all 6 `net/` servlet+background-task stubs
(plus a previously entirely-missing `ServerCommunication::send_to_replay_session`), and 14
of the 25 DB/HTTP-dependent handlers (including all 6 sketch handlers via the new
replay-broadcast wiring). The remaining **11 handlers stay `~` for a genuine reason, not a
narrow gap**: `add_loaded_team`, `close_game`, `fumbbl_game_checked`, `fumbbl_team_loaded`,
`join`, `join_approved`, `join_replay`, `schedule_game`, `upload_game`, `replay`,
`replay_loaded` each bottom out in a whole unported Java subsystem — `GameCache
.addTeamToGame` (team/roster/box-placement mutation), `RosterCache`/`TeamCache`,
`UtilServerStartGame.startGame`, `ReplayCache`/`ReplayState`/`ServerSketchManager`, or a full
replay-playback engine (`UtilServerReplay.startServerReplay`) — confirmed against the real
Java source, not assumed. Building those is real follow-up work, not a bounded-session task.
Tests: 14,794 → 14,911 (+117).

By LOC (recomputed in ZW.0 from actual per-directory Java LOC, not the earlier estimate):
GUI-skip is 279 files / ~33.8k LOC (dialog 17,993 + ui 9,316 + layer 1,971 + overlay 320 +
sound 188 + ~22 root GUI files ~3,991); real client logic to translate is 365 files /
~26.7k LOC (animation 1,325 + factory 40 + handler 1,659 + model 312 + net 794 +
report 9,187 + state 10,868 + util 858 + 9 root logic files ~1,631). In-scope Java
(common + server + client-logic-to-translate, excluding GUI/Swing app/tools) is therefore
~235.2k, not the prior ~238k estimate. **~88% of in-scope Java is genuinely translated**
(~207k of ~235.2k; ~74% of all 279k — this % is unchanged since no code moved, only the
denominator was corrected). Plan: `docs/PHASE_ZW_PLAN.md`.

**Phase ZW.2 Batch B follow-up (net/, completed, 2026-07-10):** the prerequisite flagged in
the original Batch B note below — a real dispatch/serialization layer over the 92 genuine
`ClientCommand*` structs — was built earlier this session (`commands::any_client_command`,
`commands::any_server_command`, `net_command_factory::NetCommandFactory`), so all 3
`client/net/` files were revisited and translated. `ClientCommunication.java`'s ~90 `send*`
methods now each construct the real, field-for-field-correct `ClientCommand*` struct and call
its genuine `.to_json_value()`; `ServerConnection::send` still only accepts the old hand-rolled
`ffb_protocol::client_commands::ClientCommand` enum, so the JSON is pushed onto a
`Vec<serde_json::Value>` outbox rather than a live socket — a documented, explicit follow-up
(not a silent fork), same "two parallel command hierarchies" gap as before, now isolated to
just the last-mile transport instead of the whole construction layer. Several `getClient()`-
sourced values (client mode, coach name, entropy) became explicit parameters since
`FantasyFootballClient` is the permanently-skipped GUI shell; `TeamSetup`'s missing
player-number/coordinate fields and `Sketch`'s missing `id` field (both pre-existing model
simplifications) meant `sendTeamSetupSave`/`sendAddSketch` also take extra explicit params.
`ClientPingTask` translated directly (`tick(is_open, communication, now)`, no live timer per
project's `TimerTask` convention). `CommandEndpoint`: real socket I/O is already covered by
`connection::ServerConnection` (`tokio-tungstenite`, per original note below); translated the
pure logic that IS extractable (compression-flag parsing, `isOpen()` state, the pong
ping-time arithmetic) as a standalone testable method. New files: `client/net/mod.rs`,
`client_communication.rs`, `client_ping_task.rs`, `command_endpoint.rs` (replacing the 3
orphaned PascalCase stubs); `pub mod net;` added to `client/mod.rs`. Tests: 15,474 → 15,565
(+91, all in `ffb-client`).

**Phase ZW.2 Batch B (net/, investigated, blocked, 2026-07-10):** all 3 `client/net/` files
bottom out in real gaps, not narrow ones. `ClientCommunication.java` (597 lines, ~90 `send*`
methods) needs a genuine `ClientCommand` dispatch enum over the **already-existing 92
`ffb-protocol/src/commands/client_command_*.rs` structs** — those structs are a faithful
field-for-field 1:1 translation of Java's real `com.fumbbl.ffb.net.commands.ClientCommand*`
classes (verified: `ClientCommandEndTurn` has `turn_mode`/`player_coordinates` matching Java's
`turnMode`/`playerCoordinates` exactly), but **no enum wraps them for dispatch, and no JSON
wire serialization exists for them yet** (no `to_json_value`/`from_json`, unlike the Phase ZU
report files). What `ServerConnection`/`network_encoder` actually use today is a second,
parallel `ffb-protocol/src/client_commands::ClientCommand` enum — a hand-rolled, **not 1:1**
simplification (invented field shapes, e.g. `ClientBuyInducements.purchases: Vec<(String,
i32)>` has no Java equivalent) built to get the WebSocket layer working without doing the
full per-class translation. This is the same "two parallel command hierarchies" gap flagged
since Phase ZV and never resolved. `ClientPingTask.java` and `CommandEndpoint.java` both also
depend on `FantasyFootballClient` (permanently-skipped GUI shell); `CommandEndpoint`'s actual
networking role is already covered by `connection::ServerConnection` (necessarily a different
tech stack — `tokio-tungstenite` vs. Java's `javax.websocket` — not a 1:1 line translation
candidate). **Sizing:** building a genuine `ClientCommand` dispatch enum over the 92 real
structs and translating `ClientCommunication`'s ~90 methods against it is comparable in scope
to Phase ZU's 191-file report-serialization phase — a dedicated sub-phase, not foldable into
this batch. All 3 files marked `~` (not `○`, not skipped) pending that prerequisite work.
Recommended next-session focus; not started this session.

**Phase ZW.2 Batch A (client core translation start, 2026-07-10):** translated the first 7
files: `client/model/` (4: `ChangeList`, `ControlAware`, `OnlineAware`, `VersionChangeList`)
and 2 of 11 `client/util/` files (`UtilClientActionKeys` → `action_keys.rs`, `UtilClientChat`
→ `chat.rs`, text-manipulation half only — its Swing `JTextComponent` half has no headless
equivalent). **Major discovery: the entire `crates/ffb-client/src/client/` tree (649 files,
including all 644 tracker rows) was never declared as a module anywhere — not wired into
`lib.rs`, so none of it compiled or ran, the same dead-code pattern found in `ffb-server/net/`
during Phase ZW.1.** Added `pub mod client;` to `lib.rs` and built out the module tree
(`client/mod.rs`, `client/model/mod.rs`, `client/util/mod.rs`) from scratch as files are
translated — snake_case filenames per project convention, replacing the orphaned PascalCase
stubs outright (no coexistence step needed since nothing referenced them). **Triage
correction found while translating:** `client/ActionKey.java` — classified `—` (GUI) in ZW.0
by directory-membership heuristic — is actually a plain enum with no Swing/AWT dependency
(keybinding identifiers + property-name strings); reclassified to `✓` and translated, since
`UtilClientActionKeys` genuinely needs it. **8 of the 11 `util/` files turned out to be
GUI-coupled despite `util/` being classified wholesale as logic in ZW.0** — `MarkerService`
(JDialog/JPanel), `MouseEntropySource` (AWT `MouseEvent`/`SwingUtilities`), `UtilClientCursor`
(AWT `Cursor`/`Toolkit`), `UtilClientGraphics` (`Graphics2D`), `UtilClientJTable` (Swing
`JTable`), `UtilClientPlayerDrag` (mouse-drag pixel math tied to `FantasyFootballClient`/
`UserInterface` rendering), `UtilClientReflection` (Java-version/Swing-JTable-reflection
workarounds with no Rust equivalent), `UtilClientThrowTeamMate` (pure UI-redraw trigger) —
reclassified `○`→`—`. `UtilClientTimeout` and `factory/LogicPluginFactory` stay `○` but are
**deferred**, not translated this batch: `UtilClientTimeout` needs the `UserInterface`/
`ClientData` headless-callback trait boundary (ZW.2 Batch D); `LogicPluginFactory` needs
`state/logic/plugin/LogicPlugin` (ZW.2 Batch E5, the last state batch) — both real dependency
gaps, confirmed against the Java source, not skipped for convenience. This is expected and
matches the plan's flagged risk that directory-level GUI/logic triage would need file-by-file
correction as translation actually touches each file. Tests: 14,911 → 14,940 (+29). Net
tracker effect: ○ 365→351 (-6 translated, -8 reclassified to —), — 279→286 (-1 reclassified
to ✓, +8 reclassified from ○), ✓ (client-logic) 0→7.

---

## Session History

| Session | Date | Tests | DEFERREDs | Notes |
|---------|------|-------|-----------|-------|
| Phase VIII | 2026-07-04 | 8,064 | ~540 | Modifier factory wiring complete (ArmorModifierFactory, InjuryModifierFactory, injury type wiring) |
| Phase IX | 2026-07-05 | 8,149 | 540 | DEFERRED sweep complete; remaining DEFERREDs blocked by dialog/report/card/persistence infrastructure |
| Phase X | 2026-07-05 | 8,775 | ~525 | Report system (~183 new report files, ~586 tests), SkillFactory (222-skill HashMap, 22 tests), dialog wiring (show_dialog/hide_dialog, 4 sites wired), step completions (step_reset_to_move, state_mechanic chef rolls, step_right_stuff BB2025, step_quick_bite adjacent-opponent branch). Functional completeness: 82% → ~85%. |
| Phase ZT | 2026-07-09 | 12,451 (start) | ~52 | Stub implementation sweep; TRACKER updated: 412 server/report entries moved from — to ○ |
| Phase ZU | 2026-07-10 | 14,322 | ~221 | Report serialization: 191 report files fully translated (fields, getters, to_json_value/from_json, round-trip tests) into `ffb-model/src/report/`. TRACKER had not been updated for this — corrected in Phase ZV below. |
| Phase ZV (start) | 2026-07-10 | 14,341 | 221 | Tracker correction: flipped 191 report rows ○→✓ (crate column corrected `ffb-server`→`ffb-model`); added 2 previously-missing trivial files (`ReportInjury` trait, `UtilReport` helper) discovered during the audit. Remaining 221 ○ rows are all genuine `ffb-server` handler/db/admin/request/commandline/net stubs — next up. |
| Phase ZV | 2026-07-10 | 14,794 | 35 | Real 1:1 translation of the `ffb-server` infrastructure layer, replacing `todo!()` stubs with genuine ported logic: (1) wired 5 orphaned module trees (`db`, `admin`, `request`, `commandline`, remaining `handler/` + `net/commands` files) into `lib.rs`/`mod.rs` so their code and tests actually compile/run; (2) ported `handler/*.rs` (36 files) and `handler/talk/*.rs` (70 files) — session/coach/game bookkeeping, talk-command dispatch, sketch/marker/replay handling — using `GameCache`/`SessionManager`/`ReplaySessionManager`; (3) ported `db/` (29 files: base registry classes + `query/`/`insert/`/`update/`) onto real `mysql_async`-backed execution, following the pre-existing `db/delete/*.rs` pattern (added `mysql_async` as a workspace dependency); (4) ported `admin/*.rs` (8) and `request/*.rs` + `request/fumbbl/*.rs` (20) behind a new `HttpClient` trait (mockable in tests, no live network wiring yet); (5) refactored `ServerCommandHandlerFactory` to delegate `ClientPing` to the real `ServerCommandHandlerPing` (documented remaining delegation as blocked on a pre-existing split between two parallel `ffb_protocol` command-type hierarchies — a real architectural gap, not fixed this phase); (6) expanded `net/wire.rs`'s `GameEvent → WireReport` coverage from 18 to 114 of ~128 variants, and added `net/wire_prompt.rs` — the previously entirely-missing `AgentPrompt → WireDialog` outgoing-encoding direction, covering all 35 `AgentPrompt` variants (unverified against a literal Java wire-format source, since no equivalent `ServerCommandSetDialogParameter`-style class exists in `ffb-java`; documented as a best-effort design following the `WireReport` convention). Tests: 12,451 → 14,794 (+2,343). **Remaining `~` (35 files, all genuinely infra-gated, not logic gaps):** live DB connection wiring for a few handler DB calls, live HTTP wiring for FUMBBL-auth/team-loading handlers, sketch-replay-broadcast plumbing, and a handful of missing lower-level APIs (`GameState` step-stack reset, `SoundId` enumeration, `GameOptionId` enumeration, `SeriousInjuryFactory.forAttribute`). |
| Phase ZW.1 | 2026-07-10 | 14,911 | 11 | Server closeout sweep on the 35 remaining `~` files from Phase ZV. Fixed the 4 lower-level API gaps: `SoundId::all()` made `pub`; built a real `GameOptionId` enum (127 variants) + `GameOptionFactory` (127-case port of Java's `createGameOption`) + `GameOptions::get_option_with_default`; implemented `SeriousInjuryFactory` (`for_name`/`initialize`/`dead`/`poison`/`for_attribute`) via a new `AnySeriousInjury` sum type over the edition `SeriousInjury` enums — unblocked all 4 `handler/talk/*.rs` files. Implemented all 6 `net/` servlet + background-task stubs (`command_servlet`/`file_servlet` as axum routes, 3 `tokio::time::interval`-based tasks, `server_network_entropy_task` feeding a shared `Fortuna`) and discovered/fixed that none of the 6 were declared in `net/mod.rs` (dead code, never compiled into any test run). Added `ServerCommunication::send_to_replay_session`/`close`/`send_game_time` — previously entirely missing, needed by the sketch/marker/replay handler family; gave `ReplaySessionManager` real sender storage. Wired `DbConnectionManager::init_pool()` into `fantasy_football_server.rs::run()` (env-var gated) and added a real `reqwest`-backed `HttpClient` impl (mocks kept for tests). Closed 14 of 25 DB/HTTP-dependent handlers this way (`set_marker`, `password_challenge`, `user_settings`, `delete_game`, `load_automatic_player_markings`, `update_player_markings`, all 6 sketch handlers, `replay_status`) plus `db_player_markers_insert_parameter_list.rs` (needed new `FieldModel` player/field-marker fields). **11 handlers investigated and left `~` on purpose** — each needs a whole unported Java subsystem (`GameCache.addTeamToGame`, `RosterCache`/`TeamCache`, `UtilServerStartGame.startGame`, `ReplayCache`/`ReplayState`/`ServerSketchManager`, or a full `UtilServerReplay.startServerReplay` playback engine) confirmed against the real Java source — building those is real follow-up work, not this session's scope. **Major discovery (not this phase's scope, flagged for follow-up):** an audit found all 644 `ffb-client-logic` tracker rows marked `✓` are actually ~10-line placeholder stubs, not translations — see the Progress Summary correction above and `docs/PHASE_ZW_PLAN.md` for the full remediation plan (ZW.0 tracker fix, ZW.2/ZW.3 real client translation, ~373 files / ~29.3k LOC). Tests: 14,794 → 14,911 (+117). |
| Phase ZW.0 | 2026-07-10 | 14,911 | 365 | Tracker truth reset (no code changes): ran new `scripts/audit_client_stubs.py` over all 644 `ffb-client-logic` rows, reclassifying by directory. 279 files (`dialog/`, `ui/`, `layer/`, `overlay/`, `sound/`, plus 22 root Swing/AWT files) → `—` (permanent skip, no headless equivalent). 365 files (`animation/`, `factory/`, `handler/`, `model/`, `net/`, `report/`, `state/`, `util/`, plus 9 root logic files) → `○` (not started — real translation next in ZW.2/ZW.3). Recomputed in-scope LOC denominator from actual per-directory Java line counts (~235.2k, not the prior ~238k estimate): honest completeness ~88% of in-scope, ~74% of all-Java, unchanged test count. Also fixed stale `engine.rs`-as-live-path references in `docs/step_port/TESTING.md` (engine.rs was deleted in Phase ZR; driver.rs is the live path). |
| Phase ZW.2 Batch A | 2026-07-10 | 14,940 | 351 | Client core translation start: 7 files done (`model/` × 4, `util/action_keys.rs`, `util/chat.rs`, root `action_key.rs`). **Discovered the entire `crates/ffb-client/src/client/` tree (649 files) was never wired into `lib.rs`** — added `pub mod client;` and built the module tree (`client/mod.rs`, `client/model/mod.rs`, `client/util/mod.rs`) from scratch. **Triage corrections found while translating (expected — flagged as a risk in the ZW plan):** `ActionKey.java` reclassified `—`→`✓` (plain enum, no Swing dependency, needed by `UtilClientActionKeys`); 8 of 11 `util/` files reclassified `○`→`—` (Swing/AWT-coupled despite `util/` being classified wholesale as logic: `MarkerService`, `MouseEntropySource`, `UtilClientCursor`, `UtilClientGraphics`, `UtilClientJTable`, `UtilClientPlayerDrag`, `UtilClientReflection`, `UtilClientThrowTeamMate`). `UtilClientTimeout` and `factory/LogicPluginFactory` deferred (real dependency gaps: UI trait boundary, `LogicPlugin` respectively) — see Progress Summary above for detail. Tests: 14,911 → 14,940 (+29). |
| Phase ZW.2 Batch B (investigation) | 2026-07-10 | 14,940 | 351 | Investigated `client/net/` (3 files); found all 3 blocked on real gaps, not narrow ones — see Progress Summary above for full detail. Headline: `ffb-protocol` has **two parallel `ClientCommand` hierarchies** — 92 genuine 1:1-translated structs (`commands::client_command_*.rs`, unwired, no dispatch enum, no wire serialization) vs. a hand-rolled 40-variant simplification (`client_commands::ClientCommand`) that the WebSocket layer (`connection`, `network_encoder`) actually uses today. Marked all 3 net/ files `~` with the dependency documented. No code translated this step (investigation only, 0 new tests). |
| Phase ZW.2c | 2026-07-10 | 15,647 | 11 | Built the real NetCommand wire-protocol layer flagged as the ZW.2 Batch B blocker: rewrote `net_command.rs` as a genuine `NetCommand` trait; gave the 91 `ClientCommand*`/32 `ServerCommand*` structs their missing inherited field, `NetCommand` impl, and `to_json_value`/`from_json` (wire keys verified against `IJsonOption.java`); built `AnyClientCommand`/`AnyServerCommand` sum types + a real `NetCommandFactory::for_json_value` dispatch + `NetCommandId::from_name`. Along the way found a second fake-✓-stub pocket (this same ~126-file net/commands set, previously counted in the "genuinely done" common+server total) — now genuinely done. Unblocked and translated `client/net/` (3 files) and `client/handler/` (27 files), both previously PascalCase stubs. See Progress Summary above for full detail. Tests: 14,940 → 15,647 (+707). |
| Phase ZW.2 Batch C (root files, part 1) | 2026-07-11 | 15,692 | 4 | Fresh inventory found `client/state/` (85 files) hard-depends on `FantasyFootballClient` — `ClientState<T,C>` is generically parameterized over it, and `LogicModule` imports it directly; the explicit-parameter trick used for `client/net`/`client/handler` doesn't scale to the ~1,000 `client.*` call sites across 85 state files. Promoted `FantasyFootballClient` from GUI-skip to a real hybrid struct (`fantasy_football_client.rs`) holding the logic-relevant fields (`client_data`, `game`, `mode`, `parameters`, `command_handler_factory`, `communication`, `command_endpoint`) with concrete methods translated 1:1 (`gameId`, `getGame`/`setGame`, `getMode`/`setMode`, `getParameters`, `getClientData`, `getCommandHandlerFactory`, `getCommunication`, `getCommandEndpoint`, `logError`/`logDebug`); `abstract` methods with no in-scope body (AWT-client-only or `ffb-ai`-only) are omitted, not invented, per `CLAUDE.md`. `updateClientState()` deferred to Step 3 (needs `ClientState`/`ClientStateFactory`, not yet translated). Also translated `ClientData` (plain data holder), `ClientParameters` (arg parsing + validation; added `ClientModeFactory::for_argument` to `ffb-model`, the missing counterpart to the existing `for_name`), and `IProgressListener` (trait). **Triage correction:** `ClientLayout` reclassified `—`→`✓` (plain data enum, misclassified by association with the Swing code that consumes it — same pattern as the `ActionKey` correction). **Triage correction (other direction):** `CoordinateConverter` reclassified `○`→`—` — its one method takes a Swing `MouseEvent` and calls into `UiDimensionProvider`/`PitchDimensionProvider`, both genuinely Swing-scale-bound GUI-skip types, not a narrow dependency to route around. Tests: 15,647 → 15,692 (+45, incl. 6 new `ClientModeFactory` tests in `ffb-model`). |
| Phase ZW.2 Batch C (root files, part 2 — triage only) | 2026-07-11 | 15,692 | -4/+3 | Read the actual Java source for the 4 remaining Batch-1 root file candidates; all 4 turned out more Swing-coupled than the ZW.1-era bulk triage assumed (expected risk, same pattern as the ZW.2 Batch A util/ corrections). **Reclassified `○`→`—` (permanent skip):** `PlayerIconFactory` (every method is `BufferedImage`/`Graphics2D` icon compositing) and `ReplayControl` (`extends JPanel implements MouseInputListener` — a real Swing widget). **Reclassified `○`→`~` (blocked, not skip — real follow-up work, not invented):** `StatusReport` (every print routes through `getUserInterface().getLog()`, GUI; `report()` dispatches to `client/report/ReportMessageBase` renderers that don't exist yet — blocked on Phase ZW.3) and `ClientReplayer` (644 LOC, `implements ActionListener` driven by a `javax.swing.Timer`, deep `getUserInterface()` calls for playback UI, and `createGame()`/`cloneGame()` reconstruct `Game` via the Java `Game(IFactorySource, FactoryManager)` constructor shape that this project's ported `Game::new(home, away, rules)` doesn't support — same gap documented in `fantasy_football_client.rs`'s doc comment). `client/state/` calls `getReplayer()` 24× — likely only needs a small logic-only subset (`isReplaying`/`hasControl`/replay-speed state), which is real follow-up work once `client/state/` translation surfaces exactly which methods are actually called, not before. No code changes this step (triage/tracker only); Batch 1's 9 candidate root files are now fully resolved: 5 translated (Batch C part 1), 2 confirmed permanent-skip, 2 documented-blocked. |
| Phase ZW.2 Batch D (state/interaction, part 1) | 2026-07-11 | 15,713 | 4 | Started translating `client/state/` (previously entirely unwired PascalCase stubs): translated the 2 prerequisite value types (`ClientAction` — 54-variant plain enum; `Influences` — 7-variant enum whose `get_influenced_actions()` maps each variant to its Java-hardcoded `ClientAction` list) plus `logic/interaction/ActionContext` (mutable `actions`/`influences`/`block_alternatives` lists with `add_*`/`merge`) and `logic/interaction/InteractionResult` (`Kind` enum + builder-style `with_*` methods + static factories `delegate`/`select_action`/`invalid`/`reset`/`perform`/`ignore`/`handled`/`preview_throw`). Built the module tree from scratch (`client/state/mod.rs`, `client/state/logic/mod.rs`, `client/state/logic/interaction/mod.rs`) and wired `pub mod state;` into `client/mod.rs`; only these 4 modules are declared so far, the other 81 stub files in the tree remain orphaned on disk for future batches. Tests: 15,692 → 15,713 (+21). |
| Phase ZW.2 Batch D (state/plugin) | 2026-07-11 | 15,734 | 10 | Translated `client/state/logic/plugin/` (10 files on disk — the plan's estimate of 13 was off by 3; no `plugin/` root files beyond the 4 actually confirmed). `LogicPlugin` → trait + top-level `LogicPluginType` enum (nested Java `Type` has no Rust nested-type equivalent); `BaseLogicPlugin`/`BlockLogicExtensionPlugin`/`MoveLogicPlugin` (root abstract classes) → traits, the latter two generic over the still-unwired `BlockLogicExtension`/`MoveLogicModule` logic-module types (default `()`) since those remain PascalCase stub files not yet declared in `logic/mod.rs` (blocked on the not-yet-translated `LogicModule.java`, part of a later batch). Concrete `bb2025`/`mixed` structs implement the traits with real, tested logic where the Java body only touches already-ported types (`get_type`/`get_name`, `player_can_not_move` via `PlayerState::is_pinned`/`is_rooted`, `available_actions`, the trivial pass-through `action_context`/`block_action_context` bodies); bodies that call untranslated `LogicModule`/`BlockLogicExtension`/`MoveLogicModule` availability checks (`isChompAvailable`, `isIncorporealAvailable`, `isThenIStartedBlastinAvailable`) or need the acting player's resolved `Player` object (unavailable from the id-only `ActingPlayer` struct) are left as documented-gap no-ops with per-call-site comments, matching the existing `client_communication.rs`/`command_endpoint.rs` gap-documentation convention — no invented logic. Deleted the 10 old PascalCase stub files, added `pub mod plugin;` to `logic/mod.rs`. Tests: 15,713 → 15,734 (+21). |
| Phase ZW.2 Batch D (state/logic root) | 2026-07-11 | 15,974 | 28 | Translated all 28 root-level files in `client/state/logic/` (the plan's ~30 estimate was off by 2; actual directory listing had exactly 28 after excluding the already-done `ClientAction`/`Influences` and the `bb2016/bb2020/bb2025/mixed/interaction/plugin/` subdirs). Anchor: `LogicModule.java` (753 lines) → a slim `pub trait LogicModule` (4 always-abstract methods: `get_id`/`available_actions`/`action_context`/`perform_available_action`; ~15 default lifecycle/interaction methods taking `&FantasyFootballClient`/`&mut FantasyFootballClient` explicitly rather than storing it, matching the `client/handler/*` convention) plus ~60 free `is_xxx_available(game, player)`/`_ap(game, acting_player)` predicate functions factored out for direct unit-testing (48 tests on this file alone). `RangeGridState`/`AbstractBlockLogicModule` (small helpers) and `MoveLogicModule`/`SetupLogicModule`/`BlockLogicExtension` (base/extension classes other concrete modules build on) translated next, then the 22 remaining concrete `*LogicModule` structs (Blitz/DumpOff/HighKick/IllegalSubstitution/Interception/Kickoff/KickoffReturn/Login/PassBlock/PlaceBall/Pushback/QuickSnap/Replay/Setup/SolidDefence/Spectate/StartGame/Swoop/ThrowTeamMate/Touchback/WaitForOpponent/WaitForSetup/Wizard), each a struct implementing `LogicModule` (composition over a held `MoveLogicModule`/`SetupLogicModule` where Java `extends` one of those, direct impl otherwise). Added `jump_mechanic_for`/`ttm_mechanic_for`/`pass_mechanic_for`/`on_the_ball_mechanic_for` dispatch helpers to `ffb-engine::mechanic` (mirroring the existing `game_mechanic_for` precedent) and a `game_mut()` accessor on `FantasyFootballClient` (mirroring `communication_mut()`) — both small, precedented infra additions, not new game logic. **Rust-specific pitfall discovered and documented in-file:** a trait default method taking `&self` silently shadows a same-named inherent `&mut self` method at every call site (method-resolution tries the `&self` step first) — worked around per-file with `Cell<T>` fields or by keeping the inherent method `&self` where only reads were needed. **Documented gaps** (conservative fallbacks, `// java:` comments, no invented logic): `LogicPluginFactory` (still not translated — every `plugin()`/`BaseLogicPlugin`/`MoveLogicPlugin`/`BlockLogicExtensionPlugin` resolution is a no-op/false/empty fallback); `ActingPlayer.getOldPlayerState()`/`hasOnlyStandingUpMove()` and `Player.hasActiveEnhancement()` (no Rust-model fields); `FieldModel` multi-occupancy `getPlayers(coordinate)` (model is 1:1 coordinate→player); `FantasyFootballClient.getProperty()`/`getOverlays()`/`replayInitialized()` (abstract, no in-scope body); `ClientCommunication.send_acting_player` requiring a non-optional `PlayerAction` (no null-action variant); `UtilPassing.findInterceptors` (per-edition-private in `ffb-engine`, not a public API); `PasswordChallenge.createResponse` (MD5 HMAC, no crypto crate dependency); `ffb_protocol::ServerCommand`'s documented not-1:1 simplification (blocks most of `ReplayLogicModule.handle_command`'s cases beyond `SERVER_GAME_STATE`). Tests: 15,734 → 15,974 (+240). |
| Phase ZW.2 Batch D (state/logic editions) | 2026-07-11 | 16,332 | 40 | Translated all 40 edition-specific logic modules under `client/state/logic/{bb2016,bb2020,bb2025,mixed}/` (the plan's ~40 estimate was exact: bb2016 1, bb2020 8, bb2025 13, mixed 18, confirmed via direct directory listing before starting). Parallelized across 5 isolated git worktrees (bb2016+bb2020; bb2025 split into two halves; mixed split into two halves) to avoid concurrent-compile contention in the shared `ffb-client` crate, then merged all worktree diffs back, hand-wrote the shared `bb2025/mod.rs`/`mixed/mod.rs` (each split across two agents, so neither owned the file), and wired `pub mod bb2016;`/`bb2020;`/`bb2025;`/`mixed;` into the parent `client/state/logic/mod.rs`. All 40 concrete structs implement `LogicModule`, composing over `MoveLogicModule`/`BlockLogicExtension`/`AbstractBlockLogicModule`/other Batch-D-root base types (a struct field, not inheritance) wherever the Java class `extends`/mixes one in, delegating to `logic_module`'s free predicate functions where available. Deleted all 40 old PascalCase stub files, corrected all 40 tracker rows to their snake_case Rust paths. **UI stub sites:** exactly the 2 documented in the batch plan (`bb2020::SelectBlitzTargetLogicModule::player_peek`, `bb2025::SelectBlitzTargetLogicModule::player_peek`) — both stubbed to skip only the `getUserInterface().getFieldComponent()...clearMovePath()` rendering side-effect line; no further `getUserInterface()` call sites were found anywhere else in the batch. **New documented gaps beyond the pre-existing list** (conservative fallbacks/local reimplementations, `// java:` comments, no invented logic): `ActingPlayer.isMustCompleteAction()` (no Rust field, conservatively `false`); `UtilPlayer.isFoulable(Game, Player)` (not in `ffb-model`, reimplemented locally in both `bb2025::FoulLogicModule` and `mixed::FoulLogicModule`); `UtilPlayer.canKickTeamMate`/`isKickable` (reimplemented locally in `bb2020::KickTeamMateLikeThrowLogicModule` and `mixed::KickEmBlitz/BlockLogicModule`); `FieldModel.findAdjacentCoordinates(coord, bounds, distance>1, withStart)` general form (no shared public helper beyond `adjacent_on_pitch`'s distance-1 case; reimplemented locally per call site, same pattern as the existing `logic_module.rs` private duplicate); `Game.playingTeamHasActingPLayer()`/`getDefender()`/`getOtherTeam(Team)` (no Rust `Game` equivalents, reimplemented locally in `mixed::ThenIStartedBlastinLogicModule`/`PutridRegurgitationBlitzLogicModule`); `Player.canDeclareSkillAction(property, playerState)` (no per-skill `DeclareCondition` data reachable from a bare `Player`; approximated as `has_unused_skill_with_property(property)` alone, matching the same simplification already made in `block_logic_extension.rs`); `Player.getPosition().getKeywords().contains(LINEMAN)` (`bb2025::SwarmingLogicModule`, no keyword lookup on `Player`, conservatively `false`); `MoveLogicModule`'s protected `actionAvailable` hook has no virtual-dispatch equivalent, so `bb2025::PuntLogicModule`'s acting-player interaction branch reimplements its own `action_available` directly; `mixed::BlockLogicModule` (the `Stab`/`KickEm*`/`MaximumCarnage`/`PutridRegurgitationBlock` superclass) was owned by a different parallel worktree than `bb2020::StabLogicModule`'s, so `Stab`'s unmodified inherited behavior was inlined directly (doc-commented) rather than composed over it; the other 3 sibling files' assumed `BlockLogicModule::new()`/inherent-method API was verified to match after merge (clean compile, no changes needed). Hit the known "`&self` trait default shadows `&mut self` inherent method" pitfall again in both `bb2020`'s and `bb2025`'s `SynchronousMultiBlockLogicModule` (mutable selection-state maps needed from a `player_interaction`-named method); worked around with `RefCell<HashMap<...>>` fields per the documented convention. Tests: 15,974 → 16,332 (+358, avg ~9/file across the 5 parallel batches: 72+41+84+79+90 reported individually, some overlap with shared-helper duplication not double counted in the final total). `cargo test --workspace`: 16,332 passed, 0 failed. |
| Phase ZW.2 Batch D (state root) | 2026-07-11 | 16,412 | 3 | Final batch of `client/state/` — the 3 remaining root files. `ClientState.java` (148 lines, abstract `ClientState<T extends LogicModule, C extends FantasyFootballClient>`) → `client_state.rs`: generic only over `L: LogicModule` (the `C`/`FantasyFootballClient` type param is dropped — one real instantiation in this crate — and the held `fClient` field is likewise dropped, passed explicitly instead, per the `LogicModule`/`client/handler` convention); `enterState`/`leaveState`/`endTurn` (`final` in Java) translated as real inherent methods delegating to the held `logic_module`; `hideSelectSquare`/`showSelectSquare` translate the real coordinate-state transition; `drawSelectSquare` (Java's one always-`abstract` method) has no in-scope concrete body anywhere in this crate (all `ClientStateXxx` Swing subclasses live in `ffb-client`'s AWT layer, out of scope) — left as a documented no-op rather than invented or exposed as an unimplementable trait requirement; AWT `MouseEvent` handlers skipped, the non-`MouseEvent` default bodies (`actionKeyPressed`, drag/drop predicates) translated for real since Java's own bodies are trivial in-scope logic. `ClientStateFactory.java` (368 lines) → `client_state_factory.rs`: `registerStates()`/`registerStatesForRules()` (per-edition `abstract`, no in-scope concrete subclass) reduced to a documented no-op registry shell; the real translation target, `getStateForGame()`/`findPassiveState()` (a pure `Game`-state → `ClientStateId` dispatcher switching on `ClientMode`/`TurnMode`/`ActingPlayer.getPlayerAction()`/pushback squares), is ported branch-by-branch in full, including the `TtmMechanic.handleKickLikeThrow()` real mechanic dispatch (reusing the existing `ttm_mechanic_for` helper) and the `MULTIPLE_BLOCK`→`canBlockTwoAtOnce` skill-property ternary. Two documented gaps: `game.getFinished() != null` has no separate `Date` field on the Rust `Game` model, mapped to `game.status == GameStatus::Finished` instead; `getReplayer().isReplaying()` has no in-scope body (`ClientReplayer` remains a blocked stub) and is conservatively `false`. `IPlayerPopupMenuKeys.java` → `i_player_popup_menu_keys.rs`: 45 `KEY_*` AWT `VK_*` constant aliases, reproduced directly as their standard JDK virtual-key-code values (no AWT dependency in this crate). Corrected `state_dispatch::current_state`'s doc comment (it was already correctly *implemented* as a deliberately coarser TurnMode-only helper, but its old doc comment claimed to "mirror `ClientStateFactory.java`", which was never true and is now fixed) — kept as a separate helper, not merged. Deleted the 3 stale `// client-only: ... superseded by crate::state_dispatch::mod.` comments (never accurate) along with the 3 old PascalCase stub files. Wired all 3 into `client/state/mod.rs`, updated `client/mod.rs`'s stale "not yet wired in" comment. **`client/state/` is now 100% complete**: 85 files total across all 5 batches (4 interaction/value-type + 10 plugin + 28 logic-root + 40 logic-editions + 3 this batch — the original ~85 estimate landed exactly on the nose after all 5 batches' individual reconciliations). Total new tests across the whole `client/state/` effort: 21 + 21 + 240 + 358 + 80 = 720. Tests: 16,332 → 16,412 (+80: 77 in `client_state`/`client_state_factory` incl. one test per `TurnMode` branch and one per `PlayerAction` sub-branch, 3 in `i_player_popup_menu_keys`). `cargo test --workspace`: 16,412 passed, 0 failed. |
| Phase ZW.3 + ZW.4 | 2026-07-11 | 17,305 | 0 | Translated all 211 `client/report/*Message.java` renderers (55 root + 32 bb2016 + 26 bb2020 + 39 bb2025 + 57 mixed) in 5 parallel git-worktree batches, plus the prerequisite `StatusReport`/`TextStyle`/`ParagraphStyle`/`ReportMessageBase`/`ReportMessageType` (un-skipping the two style enums, miscategorized as Swing in ZW.0). See Progress Summary above for full detail. Tests: 16,412 → 17,305 (+893). `cargo test --workspace`: 17,305 passed, 0 failed. **This completes Phase ZW: all 373 in-scope `ffb-client-logic` files are genuinely translated.** Docs closeout (this row + Progress Summary + `docs/PHASE_ZW_PLAN.md` final numbers) done same session. |
| ZW.1 (partial) | 2026-07-10 | 14,904 | 29 | Closed out 6 of the 35 remaining `~` `ffb-server/net` stub files: `CommandServlet`/`FileServlet` (Jetty servlet → axum route/handler, wired into `FantasyFootballServer::run()`'s router), `ServerDbKeepAliveTask`/`ServerGameTimeTask`/`ServerNetworkEntropyTask`/`SessionTimeoutTask` (`TimerTask` → `tokio::time::interval` loops, spawned from `run()`, gated by new `FFB_TIMER_*`/`FFB_SESSION_TIMEOUT_*` env vars mirroring the Java `IServerProperty` gates). **Discovered these 6 files (plus their existing stub tests) were never wired into `net/mod.rs` — dead code not compiled or counted** — fixed as part of this closeout. Along the way: added `Fortuna` to `FantasyFootballServer` (`getFortuna()`), gave `DbConnectionManager` a `Clone` impl (async tasks pull an owned copy out from behind `std::sync::Mutex` before `.await`, avoiding non-`Send` futures), and added the previously-missing `ServerCommunication.sendToReplaySession`/`close`/`sendGameTime` equivalents (`ServerCommunication` now owns a shared `ReplaySessionManager`, given `register_sender`/`send_to` so replay broadcasts have somewhere to write — `ServerCommunication.java` was already tracked `✓` before this, so its row is unchanged). Tests: 14,794 → 14,904 (+110). Remaining: 29 `~` files. |

---

## Module: ffb-common

### bb2016/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2016/SeriousInjury.java` | `ffb-model` | `src/bb2016/serious_injury.rs` | ✓ |

### bb2020/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2020/InjuryDescription.java` | `ffb-model` | `src/bb2020/injury_description.rs` | ✓ |
| `bb2020/SeriousInjury.java` | `ffb-model` | `src/bb2020/serious_injury.rs` | ✓ |

### bb2025/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `bb2025/SeriousInjury.java` | `ffb-model` | `src/bb2025/serious_injury.rs` | ✓ |

### dialog/ (70 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `dialog/DialogApothecaryChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_apothecary_choice_parameter.rs` | ✓ |
| `dialog/DialogArgueTheCallParameter.java` | `ffb-model` | `src/dialog/dialog_argue_the_call_parameter.rs` | ✓ |
| `dialog/DialogBlockRollParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_parameter.rs` | ✓ |
| `dialog/DialogBlockRollPartialReRollParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_partial_re_roll_parameter.rs` | ✓ |
| `dialog/DialogBlockRollPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_block_roll_properties_parameter.rs` | ✓ |
| `dialog/DialogBloodlustActionParameter.java` | `ffb-model` | `src/dialog/dialog_bloodlust_action_parameter.rs` | ✓ |
| `dialog/DialogBriberyAndCorruptionParameter.java` | `ffb-model` | `src/dialog/dialog_bribery_and_corruption_parameter.rs` | ✓ |
| `dialog/DialogBribesParameter.java` | `ffb-model` | `src/dialog/dialog_bribes_parameter.rs` | ✓ |
| `dialog/DialogBuyCardsAndInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_cards_and_inducements_parameter.rs` | ✓ |
| `dialog/DialogBuyCardsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_cards_parameter.rs` | ✓ |
| `dialog/DialogBuyInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_inducements_parameter.rs` | ✓ |
| `dialog/DialogBuyPrayersAndInducementsParameter.java` | `ffb-model` | `src/dialog/dialog_buy_prayers_and_inducements_parameter.rs` | ✓ |
| `dialog/DialogCoinChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_coin_choice_parameter.rs` | ✓ |
| `dialog/DialogConcedeGameParameter.java` | `ffb-model` | `src/dialog/dialog_concede_game_parameter.rs` | ✓ |
| `dialog/DialogConfirmEndActionParameter.java` | `ffb-model` | `src/dialog/dialog_confirm_end_action_parameter.rs` | ✓ |
| `dialog/DialogDefenderActionParameter.java` | `ffb-model` | `src/dialog/dialog_defender_action_parameter.rs` | ✓ |
| `dialog/DialogFollowupChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_followup_choice_parameter.rs` | ✓ |
| `dialog/DialogGameStatisticsParameter.java` | `ffb-model` | `src/dialog/dialog_game_statistics_parameter.rs` | ✓ |
| `dialog/DialogId.java` | `ffb-model` | `src/dialog/dialog_id.rs` | ✓ |
| `dialog/DialogInformationOkayParameter.java` | `ffb-model` | `src/dialog/dialog_information_okay_parameter.rs` | ✓ |
| `dialog/DialogInterceptionParameter.java` | `ffb-model` | `src/dialog/dialog_interception_parameter.rs` | ✓ |
| `dialog/DialogInvalidSolidDefenceParameter.java` | `ffb-model` | `src/dialog/dialog_invalid_solid_defence_parameter.rs` | ✓ |
| `dialog/DialogJoinParameter.java` | `ffb-model` | `src/dialog/dialog_join_parameter.rs` | ✓ |
| `dialog/DialogJourneymenParameter.java` | `ffb-model` | `src/dialog/dialog_journeymen_parameter.rs` | ✓ |
| `dialog/DialogKickOffResultParameter.java` | `ffb-model` | `src/dialog/dialog_kick_off_result_parameter.rs` | ✓ |
| `dialog/DialogKickoffReturnParameter.java` | `ffb-model` | `src/dialog/dialog_kickoff_return_parameter.rs` | ✓ |
| `dialog/DialogKickSkillParameter.java` | `ffb-model` | `src/dialog/dialog_kick_skill_parameter.rs` | ✓ |
| `dialog/DialogOpponentBlockSelectionParameter.java` | `ffb-model` | `src/dialog/dialog_opponent_block_selection_parameter.rs` | ✓ |
| `dialog/DialogOpponentBlockSelectionPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_opponent_block_selection_properties_parameter.rs` | ✓ |
| `dialog/DialogParameterFactory.java` | `ffb-model` | `src/dialog/dialog_parameter_factory.rs` | ✓ |
| `dialog/DialogPassBlockParameter.java` | `ffb-model` | `src/dialog/dialog_pass_block_parameter.rs` | ✓ |
| `dialog/DialogPenaltyShootoutParameter.java` | `ffb-model` | `src/dialog/dialog_penalty_shootout_parameter.rs` | ✓ |
| `dialog/DialogPettyCashParameter.java` | `ffb-model` | `src/dialog/dialog_petty_cash_parameter.rs` | ✓ |
| `dialog/DialogPickUpChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_pick_up_choice_parameter.rs` | ✓ |
| `dialog/DialogPileDriverParameter.java` | `ffb-model` | `src/dialog/dialog_pile_driver_parameter.rs` | ✓ |
| `dialog/DialogPilingOnParameter.java` | `ffb-model` | `src/dialog/dialog_piling_on_parameter.rs` | ✓ |
| `dialog/DialogPlayerChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_player_choice_parameter.rs` | ✓ |
| `dialog/DialogPuntToCrowdParameter.java` | `ffb-model` | `src/dialog/dialog_punt_to_crowd_parameter.rs` | ✓ |
| `dialog/DialogReceiveChoiceParameter.java` | `ffb-model` | `src/dialog/dialog_receive_choice_parameter.rs` | ✓ |
| `dialog/DialogReRollBlockForTargetsParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_block_for_targets_parameter.rs` | ✓ |
| `dialog/DialogReRollBlockForTargetsPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_block_for_targets_properties_parameter.rs` | ✓ |
| `dialog/DialogReRollForTargetsParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_for_targets_parameter.rs` | ✓ |
| `dialog/DialogReRollParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_parameter.rs` | ✓ |
| `dialog/DialogReRollPropertiesParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_properties_parameter.rs` | ✓ |
| `dialog/DialogReRollRegenerationMultipleParameter.java` | `ffb-model` | `src/dialog/dialog_re_roll_regeneration_multiple_parameter.rs` | ✓ |
| `dialog/DialogSelectBlitzTargetParameter.java` | `ffb-model` | `src/dialog/dialog_select_blitz_target_parameter.rs` | ✓ |
| `dialog/DialogSelectGazeTargetParameter.java` | `ffb-model` | `src/dialog/dialog_select_gaze_target_parameter.rs` | ✓ |
| `dialog/DialogSelectKeywordParameter.java` | `ffb-model` | `src/dialog/dialog_select_keyword_parameter.rs` | ✓ |
| `dialog/DialogSelectPositionParameter.java` | `ffb-model` | `src/dialog/dialog_select_position_parameter.rs` | ✓ |
| `dialog/DialogSelectSkillParameter.java` | `ffb-model` | `src/dialog/dialog_select_skill_parameter.rs` | ✓ |
| `dialog/DialogSelectWeatherParameter.java` | `ffb-model` | `src/dialog/dialog_select_weather_parameter.rs` | ✓ |
| `dialog/DialogSetupErrorParameter.java` | `ffb-model` | `src/dialog/dialog_setup_error_parameter.rs` | ✓ |
| `dialog/DialogSkillUseParameter.java` | `ffb-model` | `src/dialog/dialog_skill_use_parameter.rs` | ✓ |
| `dialog/DialogStartGameParameter.java` | `ffb-model` | `src/dialog/dialog_start_game_parameter.rs` | ✓ |
| `dialog/DialogSwarmingErrorParameter.java` | `ffb-model` | `src/dialog/dialog_swarming_error_parameter.rs` | ✓ |
| `dialog/DialogSwarmingPlayersParameter.java` | `ffb-model` | `src/dialog/dialog_swarming_players_parameter.rs` | ✓ |
| `dialog/DialogTeamSetupParameter.java` | `ffb-model` | `src/dialog/dialog_team_setup_parameter.rs` | ✓ |
| `dialog/DialogTouchbackParameter.java` | `ffb-model` | `src/dialog/dialog_touchback_parameter.rs` | ✓ |
| `dialog/DialogUseApothecariesParameter.java` | `ffb-model` | `src/dialog/dialog_use_apothecaries_parameter.rs` | ✓ |
| `dialog/DialogUseApothecaryParameter.java` | `ffb-model` | `src/dialog/dialog_use_apothecary_parameter.rs` | ✓ |
| `dialog/DialogUseChainsawParameter.java` | `ffb-model` | `src/dialog/dialog_use_chainsaw_parameter.rs` | ✓ |
| `dialog/DialogUseIgorParameter.java` | `ffb-model` | `src/dialog/dialog_use_igor_parameter.rs` | ✓ |
| `dialog/DialogUseIgorsParameter.java` | `ffb-model` | `src/dialog/dialog_use_igors_parameter.rs` | ✓ |
| `dialog/DialogUseInducementParameter.java` | `ffb-model` | `src/dialog/dialog_use_inducement_parameter.rs` | ✓ |
| `dialog/DialogUseMortuaryAssistantParameter.java` | `ffb-model` | `src/dialog/dialog_use_mortuary_assistant_parameter.rs` | ✓ |
| `dialog/DialogUseMortuaryAssistantsParameter.java` | `ffb-model` | `src/dialog/dialog_use_mortuary_assistants_parameter.rs` | ✓ |
| `dialog/DialogWinningsReRollParameter.java` | `ffb-model` | `src/dialog/dialog_winnings_re_roll_parameter.rs` | ✓ |
| `dialog/DialogWithoutParameter.java` | `ffb-model` | `src/dialog/dialog_without_parameter.rs` | ✓ |
| `dialog/DialogWizardSpellParameter.java` | `ffb-model` | `src/dialog/dialog_wizard_spell_parameter.rs` | ✓ |
| `dialog/UtilDialogParameter.java` | `ffb-model` | `src/dialog/util_dialog_parameter.rs` | ✓ |

### factory/ (82 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `factory/AnimationTypeFactory.java` | `ffb-model` | `src/model/animation_type.rs` (`AnimationType::for_name`) | ✓ |
| `factory/ApothecaryModeFactory.java` | `ffb-model` | `src/enums/apothecary.rs` (`ApothecaryMode::from_name`) | ✓ |
| `factory/ApothecaryStatusFactory.java` | `ffb-model` | `src/enums/apothecary.rs` (`ApothecaryStatus::from_name`) | ✓ |
| `factory/application/NetCommandIdFactory.java` | `ffb-model` | `src/enums/net.rs` (`NetCommandId::from_name`) | ✓ |
| `factory/ArmorModifierFactory.java` | `ffb-mechanics` | `src/modifiers/armor_modifier_factory.rs` | ✓ |
| `factory/ArmorModifiers.java` | `ffb-mechanics` | `src/modifiers/armor_modifiers.rs` | ✓ |
| `factory/bb2016/ArmorModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2016/armor_modifiers.rs` | ✓ |
| `factory/bb2016/InjuryModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2016/injury_modifiers.rs` | ✓ |
| `factory/bb2016/JumpModifierFactory.java` | `ffb-mechanics` | `src/modifiers/jump_modifier_factory.rs` | `~` |
| `factory/bb2020/ArmorModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2020/armor_modifiers.rs` | ✓ |
| `factory/bb2020/InjuryModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2020/injury_modifiers.rs` | ✓ |
| `factory/bb2020/PrayerFactory.java` | `ffb-model` | `src/factory/bb2020/prayer_factory.rs` | ✓ |
| `factory/bb2025/ArmorModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2025/armor_modifiers.rs` | ✓ |
| `factory/bb2025/InjuryModifiers.java` | `ffb-mechanics` | `src/modifiers/bb2025/injury_modifiers.rs` | ✓ |
| `factory/bb2025/PrayerFactory.java` | `ffb-model` | `src/factory/bb2025/prayer_factory.rs` | ✓ |
| `factory/BlockResultFactory.java` | `ffb-model` | `src/factory/block_result_factory.rs` | ✓ |
| `factory/CardEffectFactory.java` | `ffb-model` | `src/enums/card.rs` (`CardEffect::from_name`) | ✓ |
| `factory/CardFactory.java` | `ffb-model` | `src/factory/card_factory.rs` | ✓ |
| `factory/CardTypeFactory.java` | `ffb-model` | `src/factory/card_type_factory.rs` | ✓ |
| `factory/CatchModifierFactory.java` | `ffb-mechanics` | `src/modifiers/catch_modifier_factory.rs` | ✓ |
| `factory/CatchScatterThrowInModeFactory.java` | `ffb-model` | `src/model/catch_scatter_throw_in_mode.rs` (`CatchScatterThrowInMode::for_name`) | ✓ |
| `factory/ClientModeFactory.java` | `ffb-model` | `src/factory/client_mode_factory.rs` | ✓ |
| `factory/ClientStateIdFactory.java` | `ffb-model` | `src/enums/client.rs` (`ClientStateId::from_name`) | ✓ |
| `factory/common/GoForItModifierFactory.java` | `ffb-mechanics` | `src/modifiers/go_for_it_modifier_factory.rs` | ✓ |
| `factory/ConcedeGameStatusFactory.java` | `ffb-model` | `src/model/concede_game_status.rs` (`ConcedeGameStatus::from_name`) | ✓ |
| `factory/DialogIdFactory.java` | `ffb-model` | `src/dialog/dialog_id.rs` (`DialogId::for_name`) | ✓ |
| `factory/DirectionFactory.java` | `ffb-model` | `src/enums/direction.rs` (`Direction::from_name`) | ✓ |
| `factory/DodgeModifierFactory.java` | `ffb-mechanics` | `src/modifiers/dodge_modifier_factory.rs` | ✓ |
| `factory/FoulAssistArmorModifier.java` | `ffb-mechanics` | `src/modifiers/foul_assist_armor_modifier.rs` | ✓ |
| `factory/GameOptionFactory.java` | `ffb-model` | `src/factory/game_option_factory.rs` | ✓ |
| `factory/GameOptionIdFactory.java` | `ffb-model` | `src/factory/game_option_id_factory.rs` | ✓ |
| `factory/GameStatusFactory.java` | `ffb-model` | `src/enums/game.rs` (`GameStatus::from_name`) | ✓ |
| `factory/GazeModifierFactory.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_collection.rs` (`GazeModifierCollection::find_applicable`, called directly from `ffb-engine`'s `step_hypnotic_gaze.rs` per edition — no separate factory indirection needed, same pattern as the other `Generified*`-based modifiers) | ✓ |
| `factory/GenerifiedModifierFactory.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_collection.rs` (shared resolution logic inlined per modifier-type `*_collection.rs::find_applicable`, e.g. `catch_modifier_collection.rs`/`dodge_modifier_collection.rs`/`interception_modifier_collection.rs` — Rust has no runtime-generic factory base the way Java's abstract `<C,V,R>` class does, so each concrete collection carries its own `find_applicable`) | ✓ |
| `factory/IFactorySource.java` | `ffb-model` | `src/factory/i_factory_source.rs` | ✓ |
| `factory/ILoggingFacade.java` | `ffb-model` | `src/factory/i_logging_facade.rs` | ✓ |
| `factory/INamedObjectFactory.java` | `ffb-model` | `src/factory/i_named_object_factory.rs` | ✓ |
| `factory/InducementPhaseFactory.java` | `ffb-model` | `src/enums/card.rs` (`InducementPhase::from_name`) | ✓ |
| `factory/InducementTypeFactory.java` | `ffb-model` | `src/factory/inducement_type_factory.rs` | ✓ |
| `factory/InjuryModifierFactory.java` | `ffb-mechanics` | `src/modifiers/injury_modifier_factory.rs` | ✓ |
| `factory/InjuryModifiers.java` | `ffb-mechanics` | `src/modifiers/injury_modifiers.rs` | ✓ |
| `factory/InjuryTypeFactory.java` | — | — | — (dead: no live `ffb-server` callers found; Java-side is superseded server-side by the distinct `server.factory.InjuryTypeServerFactory`, not this class — permanently skipped) |
| `factory/InterceptionModifierFactory.java` | `ffb-mechanics` | `src/modifiers/interception_modifier_factory.rs` | ✓ |
| `factory/IRollModifierFactory.java` | `ffb-model` | `src/factory/i_roll_modifier_factory.rs` | ✓ |
| `factory/JumpModifierFactory.java` | `ffb-mechanics` | `src/modifiers/jump_modifier_factory.rs` | ✓ |
| `factory/JumpUpModifierFactory.java` | `ffb-mechanics` | `src/modifiers/jump_up_modifier_factory.rs` | ✓ (Phase AAI: wired into `step_jump_up.rs`, replacing the "modifiers = empty" stub) |
| `factory/KickoffResultFactory.java` | `ffb-model` | `src/enums/kickoff_result.rs` (`KickoffResult::from_name`) | ✓ |
| `factory/LeaderStateFactory.java` | `ffb-model` | `src/enums/reroll.rs` (`LeaderState::from_name`) | ✓ |
| `factory/MechanicsFactory.java` | `ffb-engine` | `src/mechanic/mod.rs` (`*_mechanic_for(rules)` per-`Mechanic.Type` edition dispatch helpers, e.g. `pass_mechanic_for`/`ttm_mechanic_for`/`state_mechanic_for` — replaces Java's reflection-populated `Map<Mechanic.Type, Mechanic>` registry with direct edition-keyed construction) | ✓ |
| `factory/mixed/CasualtyModifierFactory.java` | `ffb-mechanics` | `src/modifiers/casualty_modifier_factory.rs` | ✓ |
| `factory/mixed/JumpModifierFactory.java` | `ffb-mechanics` | `src/modifiers/jump_modifier_factory.rs` | ✓ |
| `factory/ModelChangeDataTypeFactory.java` | `ffb-model` | `src/enums/model_change.rs` (`ModelChangeDataType::for_name`) | ✓ |
| `factory/ModelChangeIdFactory.java` | `ffb-model` | `src/enums/model_change.rs` (`ModelChangeId::for_name`) | ✓ |
| `factory/PassingDistanceFactory.java` | `ffb-model` | `src/enums/pass.rs` (`PassingDistance::from_name`) | ✓ |
| `factory/PassModifierFactory.java` | `ffb-mechanics` | `src/modifiers/pass_modifier_factory.rs` | ✓ |
| `factory/PassResultFactory.java` | `ffb-model` | `src/enums/pass.rs` (`PassResult::from_name`) | ✓ |
| `factory/PickupModifierFactory.java` | `ffb-mechanics` | `src/modifiers/pickup_modifier_factory.rs` | ✓ |
| `factory/PlayerActionFactory.java` | `ffb-model` | `src/enums/player.rs` (`PlayerAction::from_name`) | ✓ |
| `factory/PlayerChoiceModeFactory.java` | `ffb-model` | `src/model/player_choice_mode.rs` (`PlayerChoiceMode::for_name`) | ✓ |
| `factory/PlayerGenderFactory.java` | `ffb-model` | `src/factory/player_gender_factory.rs` | ✓ |
| `factory/PlayerTypeFactory.java` | `ffb-model` | `src/factory/player_type_factory.rs` | ✓ |
| `factory/PrayerFactory.java` | `ffb-model` | `src/factory/prayer_factory.rs` | ✓ |
| `factory/PushbackModeFactory.java` | `ffb-model` | `src/model/pushback_mode.rs` (`PushbackMode::for_name`) | ✓ |
| `factory/ReportFactory.java` | `ffb-engine` | `src/mechanic/state_mechanic.rs` (+ `bb2025`/`mixed` variants: `build_report_injury` and sibling direct `Report*::new` construction sites — replaces Java's reflection-populated `Map<ReportId, Constructor<? extends IReport>>` with direct construction at each call site) | ✓ |
| `factory/ReportIdFactory.java` | `ffb-model` | `src/enums/report.rs` (`ReportId::from_name`) | ✓ |
| `factory/ReRolledActionFactory.java` | `ffb-model` | `src/model/re_rolled_action.rs` (`ReRolledAction::new`, constructed directly by name at each `ffb-engine` skill-behaviour/step call site — replaces Java's `Map<String,ReRolledAction>` registry lookup with direct construction, same pattern as `ReportFactory`) | ✓ |
| `factory/ReRollPropertyFactory.java` | `ffb-model` | `src/enums/reroll.rs` (`ReRollProperty::from_name`) | ✓ |
| `factory/ReRollSourceFactory.java` | `ffb-model` | `src/factory/re_roll_source_factory.rs` | ✓ |
| `factory/RightStuffModifierFactory.java` | `ffb-mechanics` | `src/modifiers/right_stuff_modifier_factory.rs` | ✓ |
| `factory/SendToBoxReasonFactory.java` | `ffb-model` | `src/enums/team.rs` (`SendToBoxReason::from_name`) | ✓ |
| `factory/SeriousInjuryFactory.java` | `ffb-model` | `src/factory/serious_injury_factory.rs` | ✓ |
| `factory/ServerStatusFactory.java` | `ffb-model` | `src/enums/net.rs` (`ServerStatus::from_name`) | ✓ |
| `factory/SkillCategoryFactory.java` | `ffb-model` | `src/factory/skill_category_factory.rs` | ✓ |
| `factory/SkillFactory.java` | `ffb-model` | `src/factory/skill_factory.rs` | ✓ |
| `factory/SkillPropertiesFactory.java` | — | — | — (dead: no live `ffb-server` callers found; reflection-populated registry over `NamedProperties` static fields, GUI/tooling-only — permanently skipped) |
| `factory/SkillUseFactory.java` | `ffb-model` | `src/model/skill_use.rs` (`SkillUse::for_name`) | ✓ |
| `factory/SoundIdFactory.java` | `ffb-model` | `src/factory/sound_id_factory.rs` | ✓ |
| `factory/SpecialEffectFactory.java` | `ffb-model` | `src/model/special_effect.rs` (`SpecialEffect::for_name`) | ✓ |
| `factory/TeamStatusFactory.java` | `ffb-model` | `src/enums/team.rs` (`TeamStatus::from_name`) | ✓ |
| `factory/TemporaryStatModifierFactory.java` | — | — | — (dead: no live `ffb-server` callers found; Java's `forName` reflectively instantiates a `TemporaryStatModifier` via `Class.forName` + constructor lookup, which has no Rust equivalent and nothing in-scope needs it — permanently skipped) |
| `factory/TurnModeFactory.java` | `ffb-model` | `src/enums/turn.rs` (`TurnMode::from_name`) | ✓ |
| `factory/WeatherFactory.java` | `ffb-model` | `src/factory/weather_factory.rs` | ✓ |

### inducement/ (29 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `inducement/bb2016/CardHandlerKey.java` | `ffb-model` | `src/inducement/bb2016/card_handler_key.rs` | ✓ |
| `inducement/bb2016/Cards.java` | `ffb-model` | `src/inducement/bb2016/cards.rs` | ✓ |
| `inducement/bb2016/CardType.java` | `ffb-model` | `src/inducement/bb2016/card_type.rs` | ✓ |
| `inducement/bb2016/InducementCollection.java` | `ffb-model` | `src/inducement/bb2016/inducement_collection.rs` | ✓ |
| `inducement/bb2020/CardHandlerKey.java` | `ffb-model` | `src/inducement/bb2020/card_handler_key.rs` | ✓ |
| `inducement/bb2020/Cards.java` | `ffb-model` | `src/inducement/bb2020/cards.rs` | ✓ |
| `inducement/bb2020/CardType.java` | `ffb-model` | `src/inducement/bb2020/card_type.rs` | ✓ |
| `inducement/bb2020/InducementCollection.java` | `ffb-model` | `src/inducement/bb2020/inducement_collection.rs` | ✓ |
| `inducement/bb2020/Prayer.java` | `ffb-model` | `src/inducement/bb2020/prayer.rs` | ✓ |
| `inducement/bb2020/Prayers.java` | `ffb-model` | `src/inducement/bb2020/prayers.rs` | ✓ |
| `inducement/bb2025/InducementCollection.java` | `ffb-model` | `src/inducement/bb2025/inducement_collection.rs` | ✓ |
| `inducement/bb2025/Prayer.java` | `ffb-model` | `src/inducement/bb2025/prayer.rs` | ✓ |
| `inducement/bb2025/Prayers.java` | `ffb-model` | `src/inducement/bb2025/prayers.rs` | ✓ |
| `inducement/BriberyAndCorruptionAction.java` | `ffb-model` | `src/inducement/bribery_and_corruption_action.rs` | ✓ |
| `inducement/Card.java` | `ffb-model` | `src/inducement/card.rs` | ✓ |
| `inducement/CardChoice.java` | `ffb-model` | `src/inducement/card_choice.rs` | ✓ |
| `inducement/CardChoices.java` | `ffb-model` | `src/inducement/card_choices.rs` | ✓ |
| `inducement/CardHandlerKey.java` | `ffb-model` | `src/inducement/card_handler_key.rs` | ✓ |
| `inducement/CardReport.java` | `ffb-model` | `src/inducement/card_report.rs` | ✓ |
| `inducement/Cards.java` | `ffb-model` | `src/inducement/cards.rs` | ✓ |
| `inducement/CardType.java` | `ffb-model` | `src/inducement/card_type.rs` | ✓ |
| `inducement/EnhancementProvider.java` | `ffb-model` | `src/inducement/enhancement_provider.rs` | ✓ |
| `inducement/Inducement.java` | `ffb-model` | `src/inducement/inducement.rs` | ✓ |
| `inducement/InducementCollection.java` | `ffb-model` | `src/inducement/inducement_collection.rs` | ✓ |
| `inducement/InducementDuration.java` | `ffb-model` | `src/inducement/inducement_duration.rs` | ✓ |
| `inducement/InducementPhase.java` | `ffb-model` | `src/inducement/inducement_phase.rs` | ✓ |
| `inducement/InducementType.java` | `ffb-model` | `src/inducement/inducement_type.rs` | ✓ |
| `inducement/Prayer.java` | `ffb-model` | `src/inducement/prayer.rs` | ✓ |
| `inducement/Usage.java` | `ffb-model` | `src/inducement/usage.rs` | ✓ |

### injury/ (52 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `injury/BallAndChain.java` | `ffb-model` | `src/injury/ball_and_chain.rs` | ✓ |
| `injury/Bitten.java` | `ffb-model` | `src/injury/bitten.rs` | ✓ |
| `injury/Block.java` | `ffb-model` | `src/injury/block.rs` | ✓ |
| `injury/BlockProne.java` | `ffb-model` | `src/injury/block_prone.rs` | ✓ |
| `injury/BlockProneForSpp.java` | `ffb-model` | `src/injury/block_prone_for_spp.rs` | ✓ |
| `injury/BlockStunned.java` | `ffb-model` | `src/injury/block_stunned.rs` | ✓ |
| `injury/BlockStunnedForSpp.java` | `ffb-model` | `src/injury/block_stunned_for_spp.rs` | ✓ |
| `injury/Bomb.java` | `ffb-model` | `src/injury/bomb.rs` | ✓ |
| `injury/BombForSpp.java` | `ffb-model` | `src/injury/bomb_for_spp.rs` | ✓ |
| `injury/BreatheFire.java` | `ffb-model` | `src/injury/breathe_fire.rs` | ✓ |
| `injury/BreatheFireForSpp.java` | `ffb-model` | `src/injury/breathe_fire_for_spp.rs` | ✓ |
| `injury/Chainsaw.java` | `ffb-model` | `src/injury/chainsaw.rs` | ✓ |
| `injury/ChainsawForSpp.java` | `ffb-model` | `src/injury/chainsaw_for_spp.rs` | ✓ |
| `injury/context/IInjuryContextModification.java` | `ffb-model` | `src/injury/context/i_injury_context_modification.rs` | ✓ |
| `injury/context/InjuryContext.java` | `ffb-model` | `src/injury/context/injury_context.rs` | ✓ |
| `injury/context/InjuryModification.java` | `ffb-model` | `src/injury/context/injury_modification.rs` | ✓ |
| `injury/context/ModifiedInjuryContext.java` | `ffb-model` | `src/injury/context/modified_injury_context.rs` | ✓ |
| `injury/CrowdPush.java` | `ffb-model` | `src/injury/crowd_push.rs` | ✓ |
| `injury/CrowdPushForSpp.java` | `ffb-model` | `src/injury/crowd_push_for_spp.rs` | ✓ |
| `injury/DropDodge.java` | `ffb-model` | `src/injury/drop_dodge.rs` | ✓ |
| `injury/DropDodgeForSpp.java` | `ffb-model` | `src/injury/drop_dodge_for_spp.rs` | ✓ |
| `injury/DropGFI.java` | `ffb-model` | `src/injury/drop_gfi.rs` | ✓ |
| `injury/DropJump.java` | `ffb-model` | `src/injury/drop_jump.rs` | ✓ |
| `injury/EatPlayer.java` | `ffb-model` | `src/injury/eat_player.rs` | ✓ |
| `injury/Fireball.java` | `ffb-model` | `src/injury/fireball.rs` | ✓ |
| `injury/Foul.java` | `ffb-model` | `src/injury/foul.rs` | ✓ |
| `injury/FoulForSpp.java` | `ffb-model` | `src/injury/foul_for_spp.rs` | ✓ |
| `injury/FoulForSppWithChainsaw.java` | `ffb-model` | `src/injury/foul_for_spp_with_chainsaw.rs` | ✓ |
| `injury/FoulWithChainsaw.java` | `ffb-model` | `src/injury/foul_with_chainsaw.rs` | ✓ |
| `injury/InjuryType.java` | `ffb-model` | `src/injury/injury_type.rs` | ✓ |
| `injury/KegHit.java` | `ffb-model` | `src/injury/keg_hit.rs` | ✓ |
| `injury/KTMCrowd.java` | `ffb-model` | `src/injury/ktm_crowd.rs` | ✓ |
| `injury/KTMFumbleApoKoInjury.java` | `ffb-model` | `src/injury/ktm_fumble_apo_ko_injury.rs` | ✓ |
| `injury/KTMFumbleInjury.java` | `ffb-model` | `src/injury/ktm_fumble_injury.rs` | ✓ |
| `injury/KTMInjury.java` | `ffb-model` | `src/injury/ktm_injury.rs` | ✓ |
| `injury/Lightning.java` | `ffb-model` | `src/injury/lightning.rs` | ✓ |
| `injury/PilingOnArmour.java` | `ffb-model` | `src/injury/piling_on_armour.rs` | ✓ |
| `injury/PilingOnInjury.java` | `ffb-model` | `src/injury/piling_on_injury.rs` | ✓ |
| `injury/PilingOnKnockedOut.java` | `ffb-model` | `src/injury/piling_on_knocked_out.rs` | ✓ |
| `injury/ProjectileVomit.java` | `ffb-model` | `src/injury/projectile_vomit.rs` | ✓ |
| `injury/QuickBite.java` | `ffb-model` | `src/injury/quick_bite.rs` | ✓ |
| `injury/Sabotaged.java` | `ffb-model` | `src/injury/sabotaged.rs` | ✓ |
| `injury/Saboteur.java` | `ffb-model` | `src/injury/saboteur.rs` | ✓ |
| `injury/Stab.java` | `ffb-model` | `src/injury/stab.rs` | ✓ |
| `injury/StabForSpp.java` | `ffb-model` | `src/injury/stab_for_spp.rs` | ✓ |
| `injury/ThenIStartedBlastin.java` | `ffb-model` | `src/injury/then_i_started_blastin.rs` | ✓ |
| `injury/ThrowARock.java` | `ffb-model` | `src/injury/throw_a_rock.rs` | ✓ |
| `injury/TrapDoorFall.java` | `ffb-model` | `src/injury/trap_door_fall.rs` | ✓ |
| `injury/TrapDoorFallForSpp.java` | `ffb-model` | `src/injury/trap_door_fall_for_spp.rs` | ✓ |
| `injury/TTMHitPlayer.java` | `ffb-model` | `src/injury/ttm_hit_player.rs` | ✓ |
| `injury/TTMHitPlayerForSpp.java` | `ffb-model` | `src/injury/ttm_hit_player_for_spp.rs` | ✓ |
| `injury/TTMLanding.java` | `ffb-model` | `src/injury/ttm_landing.rs` | ✓ |

### json/ (35 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `json/IJsonOption.java` | `—` | `—` | — |
| `json/IJsonReadable.java` | `—` | `—` | — |
| `json/IJsonSerializable.java` | `—` | `—` | — |
| `json/IJsonWriteable.java` | `—` | `—` | — |
| `json/JsonAbstractOption.java` | `—` | `—` | — |
| `json/JsonArrayOption.java` | `—` | `—` | — |
| `json/JsonBooleanArrayOption.java` | `—` | `—` | — |
| `json/JsonBooleanMapOption.java` | `—` | `—` | — |
| `json/JsonBooleanOption.java` | `—` | `—` | — |
| `json/JsonDateOption.java` | `—` | `—` | — |
| `json/JsonEnumWithNameOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateArrayOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateMapOption.java` | `—` | `—` | — |
| `json/JsonFieldCoordinateOption.java` | `—` | `—` | — |
| `json/JsonIntArrayOption.java` | `—` | `—` | — |
| `json/JsonIntegerListMapOption.java` | `—` | `—` | — |
| `json/JsonIntegerMapOption.java` | `—` | `—` | — |
| `json/JsonIntOption.java` | `—` | `—` | — |
| `json/JsonLegacySkillValuesOption.java` | `—` | `—` | — |
| `json/JsonLongOption.java` | `—` | `—` | — |
| `json/JsonObjectOption.java` | `—` | `—` | — |
| `json/JsonPlayerStateOption.java` | `—` | `—` | — |
| `json/JsonSkillPropertiesMapOption.java` | `—` | `—` | — |
| `json/JsonSkillValuesMapOption.java` | `—` | `—` | — |
| `json/JsonSkillWithValuesMapOption.java` | `—` | `—` | — |
| `json/JsonStringArrayOption.java` | `—` | `—` | — |
| `json/JsonStringListMapOption.java` | `—` | `—` | — |
| `json/JsonStringMapListOption.java` | `—` | `—` | — |
| `json/JsonStringMapOption.java` | `—` | `—` | — |
| `json/JsonStringOption.java` | `—` | `—` | — |
| `json/JsonTemporaryModifiersMapOption.java` | `—` | `—` | — |
| `json/JsonValueOption.java` | `—` | `—` | — |
| `json/LZString.java` | `—` | `—` | — |
| `json/MissingKeyException.java` | `—` | `—` | — |
| `json/UtilJson.java` | `—` | `—` | — |

### kickoff/ (8 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `kickoff/bb2016/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2016/kickoff_result.rs` | ✓ |
| `kickoff/bb2016/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2016/kickoff_result_mapping.rs` | ✓ |
| `kickoff/bb2020/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2020/kickoff_result.rs` | ✓ |
| `kickoff/bb2020/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2020/kickoff_result_mapping.rs` | ✓ |
| `kickoff/bb2025/KickoffResult.java` | `ffb-model` | `src/kickoff/bb2025/kickoff_result.rs` | ✓ |
| `kickoff/bb2025/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/bb2025/kickoff_result_mapping.rs` | ✓ |
| `kickoff/KickoffResult.java` | `ffb-model` | `src/kickoff/kickoff_result.rs` | ✓ |
| `kickoff/KickoffResultMapping.java` | `ffb-model` | `src/kickoff/kickoff_result_mapping.rs` | ✓ |

### marking/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `marking/FieldMarker.java` | `ffb-model` | `src/marking/field_marker.rs` | ✓ |
| `marking/PlayerMarker.java` | `ffb-model` | `src/marking/player_marker.rs` | ✓ |
| `marking/SortMode.java` | `ffb-model` | `src/marking/sort_mode.rs` | ✓ |
| `marking/TransientPlayerMarker.java` | `ffb-model` | `src/marking/transient_player_marker.rs` | ✓ |

### mechanics/ (50 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `mechanics/AgilityMechanic.java` | `ffb-mechanics` | `src/agility_mechanic.rs` | ✓ |
| `mechanics/ApothecaryMechanic.java` | `ffb-mechanics` | `src/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2016/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2016/agility_mechanic.rs` | ✓ |
| `mechanics/bb2016/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2016/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2016/GameMechanic.java` | `ffb-mechanics` | `src/bb2016/game_mechanic.rs` | ✓ |
| `mechanics/bb2016/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2016/injury_mechanic.rs` | ✓ |
| `mechanics/bb2016/JumpMechanic.java` | `ffb-mechanics` | `src/bb2016/jump_mechanic.rs` | ✓ |
| `mechanics/bb2016/OnTheBallMechanic.java` | `ffb-mechanics` | `src/bb2016/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/bb2016/PassMechanic.java` | `ffb-mechanics` | `src/bb2016/pass_mechanic.rs` | ✓ |
| `mechanics/bb2016/SkillMechanic.java` | `ffb-mechanics` | `src/bb2016/skill_mechanic.rs` | ✓ |
| `mechanics/bb2016/SppMechanic.java` | `ffb-mechanics` | `src/bb2016/spp_mechanic.rs` | ✓ |
| `mechanics/bb2016/StatsMechanic.java` | `ffb-mechanics` | `src/bb2016/stats_mechanic.rs` | ✓ |
| `mechanics/bb2016/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2016/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2016/TtmMechanic.java` | `ffb-mechanics` | `src/bb2016/ttm_mechanic.rs` | ✓ |
| `mechanics/bb2020/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2020/agility_mechanic.rs` | ✓ |
| `mechanics/bb2020/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2020/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2020/GameMechanic.java` | `ffb-mechanics` | `src/bb2020/game_mechanic.rs` | ✓ |
| `mechanics/bb2020/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2020/injury_mechanic.rs` | ✓ |
| `mechanics/bb2020/JumpMechanic.java` | `ffb-mechanics` | `src/bb2020/jump_mechanic.rs` | ✓ |
| `mechanics/bb2020/PassMechanic.java` | `ffb-mechanics` | `src/bb2020/pass_mechanic.rs` | ✓ |
| `mechanics/bb2020/SkillMechanic.java` | `ffb-mechanics` | `src/bb2020/skill_mechanic.rs` | ✓ |
| `mechanics/bb2020/SppMechanic.java` | `ffb-mechanics` | `src/bb2020/spp_mechanic.rs` | ✓ |
| `mechanics/bb2020/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2020/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2020/TtmMechanic.java` | `ffb-mechanics` | `src/bb2020/ttm_mechanic.rs` | ✓ |
| `mechanics/bb2025/AgilityMechanic.java` | `ffb-mechanics` | `src/bb2025/agility_mechanic.rs` | ✓ |
| `mechanics/bb2025/ApothecaryMechanic.java` | `ffb-mechanics` | `src/bb2025/apothecary_mechanic.rs` | ✓ |
| `mechanics/bb2025/GameMechanic.java` | `ffb-mechanics` | `src/bb2025/game_mechanic.rs` | ✓ |
| `mechanics/bb2025/InjuryMechanic.java` | `ffb-mechanics` | `src/bb2025/injury_mechanic.rs` | ✓ |
| `mechanics/bb2025/JumpMechanic.java` | `ffb-mechanics` | `src/bb2025/jump_mechanic.rs` | ✓ |
| `mechanics/bb2025/PassMechanic.java` | `ffb-mechanics` | `src/bb2025/pass_mechanic.rs` | ✓ |
| `mechanics/bb2025/SkillMechanic.java` | `ffb-mechanics` | `src/bb2025/skill_mechanic.rs` | ✓ |
| `mechanics/bb2025/SppMechanic.java` | `ffb-mechanics` | `src/bb2025/spp_mechanic.rs` | ✓ |
| `mechanics/bb2025/ThrowInMechanic.java` | `ffb-mechanics` | `src/bb2025/throw_in_mechanic.rs` | ✓ |
| `mechanics/bb2025/TtmMechanic.java` | `ffb-mechanics` | `src/bb2025/ttm_mechanic.rs` | ✓ |
| `mechanics/GameMechanic.java` | `ffb-mechanics` | `src/game_mechanic.rs` | ✓ |
| `mechanics/InjuryMechanic.java` | `ffb-mechanics` | `src/injury_mechanic.rs` | ✓ |
| `mechanics/JumpMechanic.java` | `ffb-mechanics` | `src/jump_mechanic.rs` | ✓ |
| `mechanics/Mechanic.java` | `ffb-mechanics` | `src/mechanic.rs` | ✓ |
| `mechanics/mixed/OnTheBallMechanic.java` | `ffb-mechanics` | `src/mixed/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/mixed/StatsMechanic.java` | `ffb-mechanics` | `src/mixed/stats_mechanic.rs` | ✓ |
| `mechanics/OnTheBallMechanic.java` | `ffb-mechanics` | `src/on_the_ball_mechanic.rs` | ✓ |
| `mechanics/PassMechanic.java` | `ffb-mechanics` | `src/pass_mechanic.rs` | ✓ |
| `mechanics/PassResult.java` | `ffb-mechanics` | `src/pass_result.rs` | ✓ |
| `mechanics/SkillMechanic.java` | `ffb-mechanics` | `src/skill_mechanic.rs` | ✓ |
| `mechanics/SppMechanic.java` | `ffb-mechanics` | `src/spp_mechanic.rs` | ✓ |
| `mechanics/StatsDrawingModifier.java` | `ffb-mechanics` | `src/stats_drawing_modifier.rs` | ✓ |
| `mechanics/StatsMechanic.java` | `ffb-mechanics` | `src/stats_mechanic.rs` | ✓ |
| `mechanics/ThrowInMechanic.java` | `ffb-mechanics` | `src/throw_in_mechanic.rs` | ✓ |
| `mechanics/TtmMechanic.java` | `ffb-mechanics` | `src/ttm_mechanic.rs` | ✓ |
| `mechanics/Wording.java` | `ffb-mechanics` | `src/wording.rs` | ✓ |

### model/ (61 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `model/ActingPlayer.java` | `ffb-model` | `src/model/acting_player.rs` | ✓ |
| `model/Animation.java` | `ffb-model` | `src/model/animation.rs` | ✓ |
| `model/AnimationType.java` | `ffb-model` | `src/model/animation_type.rs` | ✓ |
| `model/BlitzTurnState.java` | `ffb-model` | `src/model/blitz_turn_state.rs` | ✓ |
| `model/BlockKind.java` | `ffb-model` | `src/model/block_kind.rs` | ✓ |
| `model/BlockRoll.java` | `ffb-model` | `src/model/block_roll.rs` | ✓ |
| `model/BlockRollProperties.java` | `ffb-model` | `src/model/block_roll_properties.rs` | ✓ |
| `model/BlockTarget.java` | `ffb-model` | `src/model/block_target.rs` | ✓ |
| `model/change/IModelChangeObserver.java` | `ffb-model` | `src/model/change/i_model_change_observer.rs` | ✓ |
| `model/change/ModelChange.java` | `ffb-model` | `src/model/change/model_change.rs` | ✓ |
| `model/change/ModelChangeDataType.java` | `ffb-model` | `src/model/change/model_change_data_type.rs` | ✓ |
| `model/change/ModelChangeId.java` | `ffb-model` | `src/model/change/model_change_id.rs` | ✓ |
| `model/change/ModelChangeList.java` | `ffb-model` | `src/model/change/model_change_list.rs` | ✓ |
| `model/change/ModelChangeObservable.java` | `ffb-model` | `src/model/change/model_change_observable.rs` | ✓ |
| `model/change/ModelChangeProcessor.java` | `ffb-model` | `src/model/change/model_change_processor.rs` | ✓ |
| `model/EnhancementRegistry.java` | `ffb-model` | `src/model/enhancement_registry.rs` | ✓ |
| `model/FieldModel.java` | `ffb-model` | `src/model/field_model.rs` | ✓ |
| `model/Game.java` | `ffb-model` | `src/model/game.rs` | ✓ |
| `model/GameOptions.java` | `ffb-model` | `src/model/game_options.rs` | ✓ |
| `model/GameResult.java` | `ffb-model` | `src/model/game_result.rs` | ✓ |
| `model/GameRules.java` | `ffb-model` | `src/model/game_rules.rs` | ✓ |
| `model/InducementSet.java` | `ffb-model` | `src/model/inducement_set.rs` | ✓ |
| `model/InjuryTypeConstants.java` | `ffb-model` | `src/model/injury_type_constants.rs` | ✓ |
| `model/ISkillBehaviour.java` | `ffb-model` | `src/model/i_skill_behaviour.rs` | ✓ |
| `model/Keyword.java` | `ffb-model` | `src/model/keyword.rs` | ✓ |
| `model/KickTeamMateRange.java` | `ffb-model` | `src/model/kick_team_mate_range.rs` | ✓ |
| `model/Player.java` | `ffb-model` | `src/model/player.rs` | ✓ |
| `model/PlayerModifier.java` | `ffb-model` | `src/model/player_modifier.rs` | ✓ |
| `model/PlayerResult.java` | `ffb-model` | `src/model/player_result.rs` | ✓ |
| `model/PlayerStats.java` | `ffb-model` | `src/model/player_stats.rs` | ✓ |
| `model/PlayerStatus.java` | `ffb-model` | `src/model/player_status.rs` | ✓ |
| `model/Position.java` | `ffb-model` | `src/model/position.rs` | ✓ |
| `model/property/CancelSkillProperty.java` | `ffb-model` | `src/model/property/cancel_skill_property.rs` | ✓ |
| `model/property/ISkillProperty.java` | `ffb-model` | `src/model/property/i_skill_property.rs` | ✓ |
| `model/property/NamedProperties.java` | `ffb-model` | `src/model/property/named_properties.rs` | ✓ |
| `model/property/NamedProperty.java` | `ffb-model` | `src/model/property/named_property.rs` | ✓ |
| `model/property/PassingProperty.java` | `ffb-model` | `src/model/property/passing_property.rs` | ✓ |
| `model/Roster.java` | `ffb-model` | `src/model/roster.rs` | ✓ |
| `model/RosterPlayer.java` | `ffb-model` | `src/model/roster_player.rs` | ✓ |
| `model/RosterPosition.java` | `ffb-model` | `src/model/roster_position.rs` | ✓ |
| `model/RosterSkeleton.java` | `ffb-model` | `src/model/roster_skeleton.rs` | ✓ |
| `model/sketch/Sketch.java` | `ffb-model` | `src/model/sketch/sketch.rs` | ✓ |
| `model/sketch/SketchState.java` | `ffb-model` | `src/model/sketch/sketch_state.rs` | ✓ |
| `model/skill/AnimosityValueEvaluator.java` | `ffb-model` | `src/model/skill/animosity_value_evaluator.rs` | ✓ |
| `model/skill/DeclareCondition.java` | `ffb-model` | `src/model/skill/declare_condition.rs` | ✓ |
| `model/skill/Skill.java` | `ffb-model` | `src/model/skill/skill.rs` | ✓ |
| `model/skill/SkillClassWithValue.java` | `ffb-model` | `src/model/skill/skill_class_with_value.rs` | ✓ |
| `model/skill/SkillDisplayInfo.java` | `ffb-model` | `src/model/skill/skill_display_info.rs` | ✓ |
| `model/skill/SkillUsageType.java` | `ffb-model` | `src/model/skill/skill_usage_type.rs` | ✓ |
| `model/skill/SkillValueEvaluator.java` | `ffb-model` | `src/model/skill/skill_value_evaluator.rs` | ✓ |
| `model/skill/SkillWithValue.java` | `ffb-model` | `src/model/skill/skill_with_value.rs` | ✓ |
| `model/SpecialRule.java` | `ffb-model` | `src/model/special_rule.rs` | ✓ |
| `model/stadium/OnPitchEnhancement.java` | `ffb-model` | `src/model/stadium/on_pitch_enhancement.rs` | ✓ |
| `model/stadium/TrapDoor.java` | `ffb-model` | `src/model/stadium/trap_door.rs` | ✓ |
| `model/TargetSelectionState.java` | `ffb-model` | `src/model/target_selection_state.rs` | ✓ |
| `model/Team.java` | `ffb-model` | `src/model/team.rs` | ✓ |
| `model/TeamResult.java` | `ffb-model` | `src/model/team_result.rs` | ✓ |
| `model/TeamSkeleton.java` | `ffb-model` | `src/model/team_skeleton.rs` | ✓ |
| `model/TurnData.java` | `ffb-model` | `src/model/turn_data.rs` | ✓ |
| `model/ZappedPlayer.java` | `ffb-model` | `src/model/zapped_player.rs` | ✓ |
| `model/ZappedPosition.java` | `ffb-model` | `src/model/zapped_position.rs` | ✓ |

### modifiers/ (82 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `modifiers/ArmorModifier.java` | `ffb-mechanics` | `src/modifiers/armor_modifier.rs` | ✓ |
| `modifiers/ArmorModifierContext.java` | `ffb-mechanics` | `src/modifiers/armor_modifier_context.rs` | ✓ |
| `modifiers/bb2016/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/dodge_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/gaze_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/jump_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/pass_modifier_collection.rs` | ✓ |
| `modifiers/bb2016/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2016/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/CasualtyModifier.java` | `ffb-mechanics` | `src/modifiers/bb2020/casualty_modifier.rs` | ✓ |
| `modifiers/bb2020/CasualtyNigglingModifier.java` | `ffb-mechanics` | `src/modifiers/bb2020/casualty_niggling_modifier.rs` | ✓ |
| `modifiers/bb2020/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/gaze_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2020/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2020/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/catch_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/interception_modifier_collection.rs` | ✓ |
| `modifiers/bb2025/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/bb2025/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/CatchContext.java` | `ffb-mechanics` | `src/modifiers/catch_context.rs` | ✓ |
| `modifiers/CatchModifier.java` | `ffb-mechanics` | `src/modifiers/catch_modifier.rs` | ✓ |
| `modifiers/CatchModifierCollection.java` | `ffb-mechanics` | `src/modifiers/catch_modifier_collection.rs` | ✓ |
| `modifiers/DodgeContext.java` | `ffb-mechanics` | `src/modifiers/dodge_context.rs` | ✓ |
| `modifiers/DodgeModifier.java` | `ffb-mechanics` | `src/modifiers/dodge_modifier.rs` | ✓ |
| `modifiers/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/dodge_modifier_collection.rs` | ✓ |
| `modifiers/GazeModifier.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier.rs` | ✓ |
| `modifiers/GazeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_collection.rs` | ✓ |
| `modifiers/GazeModifierContext.java` | `ffb-mechanics` | `src/modifiers/gaze_modifier_context.rs` | ✓ |
| `modifiers/GoForItContext.java` | `ffb-mechanics` | `src/modifiers/go_for_it_context.rs` | ✓ |
| `modifiers/GoForItModifier.java` | `ffb-mechanics` | `src/modifiers/go_for_it_modifier.rs` | ✓ |
| `modifiers/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/InjuryModifier.java` | `ffb-mechanics` | `src/modifiers/injury_modifier.rs` | ✓ |
| `modifiers/InjuryModifierContext.java` | `ffb-mechanics` | `src/modifiers/injury_modifier_context.rs` | ✓ |
| `modifiers/InterceptionContext.java` | `ffb-mechanics` | `src/modifiers/interception_context.rs` | ✓ |
| `modifiers/InterceptionModifier.java` | `ffb-mechanics` | `src/modifiers/interception_modifier.rs` | ✓ |
| `modifiers/InterceptionModifierCollection.java` | `ffb-mechanics` | `src/modifiers/interception_modifier_collection.rs` | ✓ |
| `modifiers/IRegistrationAwareModifier.java` | `ffb-mechanics` | `src/modifiers/i_registration_aware_modifier.rs` | ✓ |
| `modifiers/JumpContext.java` | `ffb-mechanics` | `src/modifiers/jump_context.rs` | ✓ |
| `modifiers/JumpModifier.java` | `ffb-mechanics` | `src/modifiers/jump_modifier.rs` | ✓ |
| `modifiers/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/jump_modifier_collection.rs` | ✓ |
| `modifiers/JumpUpContext.java` | `ffb-mechanics` | `src/modifiers/jump_up_context.rs` | ✓ |
| `modifiers/JumpUpModifier.java` | `ffb-mechanics` | `src/modifiers/jump_up_modifier.rs` | ✓ |
| `modifiers/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/mixed/DodgeModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/dodge_modifier_collection.rs` | ✓ |
| `modifiers/mixed/GoForItModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/go_for_it_modifier_collection.rs` | ✓ |
| `modifiers/mixed/JumpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/jump_modifier_collection.rs` | ✓ |
| `modifiers/mixed/JumpUpModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/jump_up_modifier_collection.rs` | ✓ |
| `modifiers/mixed/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/mixed/pass_modifier_collection.rs` | ✓ |
| `modifiers/ModifierAggregator.java` | `ffb-mechanics` | `src/modifiers/modifier_aggregator.rs` | ✓ |
| `modifiers/ModifierCollection.java` | `ffb-mechanics` | `src/modifiers/modifier_collection.rs` | ✓ |
| `modifiers/ModifierContext.java` | `ffb-mechanics` | `src/modifiers/modifier_context.rs` | ✓ |
| `modifiers/ModifierType.java` | `ffb-mechanics` | `src/modifiers/modifier_type.rs` | ✓ |
| `modifiers/PassContext.java` | `ffb-mechanics` | `src/modifiers/pass_context.rs` | ✓ |
| `modifiers/PassModifier.java` | `ffb-mechanics` | `src/modifiers/pass_modifier.rs` | ✓ |
| `modifiers/PassModifierCollection.java` | `ffb-mechanics` | `src/modifiers/pass_modifier_collection.rs` | ✓ |
| `modifiers/PickupContext.java` | `ffb-mechanics` | `src/modifiers/pickup_context.rs` | ✓ |
| `modifiers/PickupModifier.java` | `ffb-mechanics` | `src/modifiers/pickup_modifier.rs` | ✓ |
| `modifiers/PickupModifierCollection.java` | `ffb-mechanics` | `src/modifiers/pickup_modifier_collection.rs` | ✓ |
| `modifiers/PlayerStatKey.java` | `ffb-mechanics` | `src/modifiers/player_stat_key.rs` | ✓ |
| `modifiers/PlayerStatLimit.java` | `ffb-mechanics` | `src/modifiers/player_stat_limit.rs` | ✓ |
| `modifiers/RegistrationAwareModifier.java` | `ffb-mechanics` | `src/modifiers/registration_aware_modifier.rs` | ✓ |
| `modifiers/RightStuffContext.java` | `ffb-mechanics` | `src/modifiers/right_stuff_context.rs` | ✓ |
| `modifiers/RightStuffModifier.java` | `ffb-mechanics` | `src/modifiers/right_stuff_modifier.rs` | ✓ |
| `modifiers/RightStuffModifierCollection.java` | `ffb-mechanics` | `src/modifiers/right_stuff_modifier_collection.rs` | ✓ |
| `modifiers/RollModifier.java` | `ffb-mechanics` | `src/modifiers/roll_modifier.rs` | ✓ |
| `modifiers/SpecialEffectArmourModifier.java` | `ffb-mechanics` | `src/modifiers/special_effect_armour_modifier.rs` | ✓ |
| `modifiers/SpecialEffectInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/special_effect_injury_modifier.rs` | ✓ |
| `modifiers/StatBasedRollModifier.java` | `ffb-mechanics` | `src/modifiers/stat_based_roll_modifier.rs` | ✓ |
| `modifiers/StatBasedRollModifierFactory.java` | `ffb-mechanics` | `src/modifiers/stat_based_roll_modifier_factory.rs` | ✓ |
| `modifiers/StaticArmourModifier.java` | `ffb-mechanics` | `src/modifiers/static_armour_modifier.rs` | ✓ |
| `modifiers/StaticInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier.rs` | ✓ |
| `modifiers/StaticInjuryModifierAttacker.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier_attacker.rs` | ✓ |
| `modifiers/StaticInjuryModifierDefender.java` | `ffb-mechanics` | `src/modifiers/static_injury_modifier_defender.rs` | ✓ |
| `modifiers/TemporaryEnhancements.java` | `ffb-mechanics` | `src/modifiers/temporary_enhancements.rs` | ✓ |
| `modifiers/TemporaryStatDecrementer.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_decrementer.rs` | ✓ |
| `modifiers/TemporaryStatIncrementer.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_incrementer.rs` | ✓ |
| `modifiers/TemporaryStatModifier.java` | `ffb-mechanics` | `src/modifiers/temporary_stat_modifier.rs` | ✓ |
| `modifiers/VariableArmourModifier.java` | `ffb-mechanics` | `src/modifiers/variable_armour_modifier.rs` | ✓ |
| `modifiers/VariableInjuryModifier.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier.rs` | ✓ |
| `modifiers/VariableInjuryModifierAttacker.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier_attacker.rs` | ✓ |
| `modifiers/VariableInjuryModifierDefender.java` | `ffb-mechanics` | `src/modifiers/variable_injury_modifier_defender.rs` | ✓ |

### net/ (137 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `net/commands/ClientCommand.java` | `ffb-protocol` | `src/commands/client_command.rs` | ✓ |
| `net/commands/ClientCommandActingPlayer.java` | `ffb-protocol` | `src/commands/client_command_acting_player.rs` | ✓ |
| `net/commands/ClientCommandAddSketch.java` | `ffb-protocol` | `src/commands/client_command_add_sketch.rs` | ✓ |
| `net/commands/ClientCommandApothecaryChoice.java` | `ffb-protocol` | `src/commands/client_command_apothecary_choice.rs` | ✓ |
| `net/commands/ClientCommandArgueTheCall.java` | `ffb-protocol` | `src/commands/client_command_argue_the_call.rs` | ✓ |
| `net/commands/ClientCommandBlitzMove.java` | `ffb-protocol` | `src/commands/client_command_blitz_move.rs` | ✓ |
| `net/commands/ClientCommandBlock.java` | `ffb-protocol` | `src/commands/client_command_block.rs` | ✓ |
| `net/commands/ClientCommandBlockChoice.java` | `ffb-protocol` | `src/commands/client_command_block_choice.rs` | ✓ |
| `net/commands/ClientCommandBlockOrReRollChoiceForTarget.java` | `ffb-protocol` | `src/commands/client_command_block_or_re_roll_choice_for_target.rs` | ✓ |
| `net/commands/ClientCommandBloodlustAction.java` | `ffb-protocol` | `src/commands/client_command_bloodlust_action.rs` | ✓ |
| `net/commands/ClientCommandBuyCard.java` | `ffb-protocol` | `src/commands/client_command_buy_card.rs` | ✓ |
| `net/commands/ClientCommandBuyInducements.java` | `ffb-protocol` | `src/commands/client_command_buy_inducements.rs` | ✓ |
| `net/commands/ClientCommandClearSketches.java` | `ffb-protocol` | `src/commands/client_command_clear_sketches.rs` | ✓ |
| `net/commands/ClientCommandCloseSession.java` | `ffb-protocol` | `src/commands/client_command_close_session.rs` | ✓ |
| `net/commands/ClientCommandCoinChoice.java` | `ffb-protocol` | `src/commands/client_command_coin_choice.rs` | ✓ |
| `net/commands/ClientCommandConcedeGame.java` | `ffb-protocol` | `src/commands/client_command_concede_game.rs` | ✓ |
| `net/commands/ClientCommandConfirm.java` | `ffb-protocol` | `src/commands/client_command_confirm.rs` | ✓ |
| `net/commands/ClientCommandDebugClientState.java` | `ffb-protocol` | `src/commands/client_command_debug_client_state.rs` | ✓ |
| `net/commands/ClientCommandEndTurn.java` | `ffb-protocol` | `src/commands/client_command_end_turn.rs` | ✓ |
| `net/commands/ClientCommandFieldCoordinate.java` | `ffb-protocol` | `src/commands/client_command_field_coordinate.rs` | ✓ |
| `net/commands/ClientCommandFollowupChoice.java` | `ffb-protocol` | `src/commands/client_command_followup_choice.rs` | ✓ |
| `net/commands/ClientCommandFoul.java` | `ffb-protocol` | `src/commands/client_command_foul.rs` | ✓ |
| `net/commands/ClientCommandGaze.java` | `ffb-protocol` | `src/commands/client_command_gaze.rs` | ✓ |
| `net/commands/ClientCommandHandOver.java` | `ffb-protocol` | `src/commands/client_command_hand_over.rs` | ✓ |
| `net/commands/ClientCommandIllegalProcedure.java` | `ffb-protocol` | `src/commands/client_command_illegal_procedure.rs` | ✓ |
| `net/commands/ClientCommandInterceptorChoice.java` | `ffb-protocol` | `src/commands/client_command_interceptor_choice.rs` | ✓ |
| `net/commands/ClientCommandJoin.java` | `ffb-protocol` | `src/commands/client_command_join.rs` | ✓ |
| `net/commands/ClientCommandJoinReplay.java` | `ffb-protocol` | `src/commands/client_command_join_replay.rs` | ✓ |
| `net/commands/ClientCommandJourneymen.java` | `ffb-protocol` | `src/commands/client_command_journeymen.rs` | ✓ |
| `net/commands/ClientCommandKeywordSelection.java` | `ffb-protocol` | `src/commands/client_command_keyword_selection.rs` | ✓ |
| `net/commands/ClientCommandKickoff.java` | `ffb-protocol` | `src/commands/client_command_kickoff.rs` | ✓ |
| `net/commands/ClientCommandKickOffResultChoice.java` | `ffb-protocol` | `src/commands/client_command_kick_off_result_choice.rs` | ✓ |
| `net/commands/ClientCommandKickTeamMate.java` | `ffb-protocol` | `src/commands/client_command_kick_team_mate.rs` | ✓ |
| `net/commands/ClientCommandLoadAutomaticPlayerMarkings.java` | `ffb-protocol` | `src/commands/client_command_load_automatic_player_markings.rs` | ✓ |
| `net/commands/ClientCommandMove.java` | `ffb-protocol` | `src/commands/client_command_move.rs` | ✓ |
| `net/commands/ClientCommandPass.java` | `ffb-protocol` | `src/commands/client_command_pass.rs` | ✓ |
| `net/commands/ClientCommandPasswordChallenge.java` | `ffb-protocol` | `src/commands/client_command_password_challenge.rs` | ✓ |
| `net/commands/ClientCommandPettyCash.java` | `ffb-protocol` | `src/commands/client_command_petty_cash.rs` | ✓ |
| `net/commands/ClientCommandPickUpChoice.java` | `ffb-protocol` | `src/commands/client_command_pick_up_choice.rs` | ✓ |
| `net/commands/ClientCommandPileDriver.java` | `ffb-protocol` | `src/commands/client_command_pile_driver.rs` | ✓ |
| `net/commands/ClientCommandPing.java` | `ffb-protocol` | `src/commands/client_command_ping.rs` | ✓ |
| `net/commands/ClientCommandPlayerChoice.java` | `ffb-protocol` | `src/commands/client_command_player_choice.rs` | ✓ |
| `net/commands/ClientCommandPositionSelection.java` | `ffb-protocol` | `src/commands/client_command_position_selection.rs` | ✓ |
| `net/commands/ClientCommandPuntToCrowd.java` | `ffb-protocol` | `src/commands/client_command_punt_to_crowd.rs` | ✓ |
| `net/commands/ClientCommandPushback.java` | `ffb-protocol` | `src/commands/client_command_pushback.rs` | ✓ |
| `net/commands/ClientCommandReceiveChoice.java` | `ffb-protocol` | `src/commands/client_command_receive_choice.rs` | ✓ |
| `net/commands/ClientCommandRemoveSketches.java` | `ffb-protocol` | `src/commands/client_command_remove_sketches.rs` | ✓ |
| `net/commands/ClientCommandReplay.java` | `ffb-protocol` | `src/commands/client_command_replay.rs` | ✓ |
| `net/commands/ClientCommandReplayStatus.java` | `ffb-protocol` | `src/commands/client_command_replay_status.rs` | ✓ |
| `net/commands/ClientCommandRequestVersion.java` | `ffb-protocol` | `src/commands/client_command_request_version.rs` | ✓ |
| `net/commands/ClientCommandSelectCardToBuy.java` | `ffb-protocol` | `src/commands/client_command_select_card_to_buy.rs` | ✓ |
| `net/commands/ClientCommandSelectWeather.java` | `ffb-protocol` | `src/commands/client_command_select_weather.rs` | ✓ |
| `net/commands/ClientCommandSetBlockTargetSelection.java` | `ffb-protocol` | `src/commands/client_command_set_block_target_selection.rs` | ✓ |
| `net/commands/ClientCommandSetMarker.java` | `ffb-protocol` | `src/commands/client_command_set_marker.rs` | ✓ |
| `net/commands/ClientCommandSetPreventSketching.java` | `ffb-protocol` | `src/commands/client_command_set_prevent_sketching.rs` | ✓ |
| `net/commands/ClientCommandSetupPlayer.java` | `ffb-protocol` | `src/commands/client_command_setup_player.rs` | ✓ |
| `net/commands/ClientCommandSketchAddCoordinate.java` | `ffb-protocol` | `src/commands/client_command_sketch_add_coordinate.rs` | ✓ |
| `net/commands/ClientCommandSketchSetColor.java` | `ffb-protocol` | `src/commands/client_command_sketch_set_color.rs` | ✓ |
| `net/commands/ClientCommandSketchSetLabel.java` | `ffb-protocol` | `src/commands/client_command_sketch_set_label.rs` | ✓ |
| `net/commands/ClientCommandSkillSelection.java` | `ffb-protocol` | `src/commands/client_command_skill_selection.rs` | ✓ |
| `net/commands/ClientCommandStartGame.java` | `ffb-protocol` | `src/commands/client_command_start_game.rs` | ✓ |
| `net/commands/ClientCommandSwoop.java` | `ffb-protocol` | `src/commands/client_command_swoop.rs` | ✓ |
| `net/commands/ClientCommandSynchronousMultiBlock.java` | `ffb-protocol` | `src/commands/client_command_synchronous_multi_block.rs` | ✓ |
| `net/commands/ClientCommandTalk.java` | `ffb-protocol` | `src/commands/client_command_talk.rs` | ✓ |
| `net/commands/ClientCommandTargetSelected.java` | `ffb-protocol` | `src/commands/client_command_target_selected.rs` | ✓ |
| `net/commands/ClientCommandTeamSetupDelete.java` | `ffb-protocol` | `src/commands/client_command_team_setup_delete.rs` | ✓ |
| `net/commands/ClientCommandTeamSetupLoad.java` | `ffb-protocol` | `src/commands/client_command_team_setup_load.rs` | ✓ |
| `net/commands/ClientCommandTeamSetupSave.java` | `ffb-protocol` | `src/commands/client_command_team_setup_save.rs` | ✓ |
| `net/commands/ClientCommandThrowKeg.java` | `ffb-protocol` | `src/commands/client_command_throw_keg.rs` | ✓ |
| `net/commands/ClientCommandThrowTeamMate.java` | `ffb-protocol` | `src/commands/client_command_throw_team_mate.rs` | ✓ |
| `net/commands/ClientCommandTouchback.java` | `ffb-protocol` | `src/commands/client_command_touchback.rs` | ✓ |
| `net/commands/ClientCommandTransferReplayControl.java` | `ffb-protocol` | `src/commands/client_command_transfer_replay_control.rs` | ✓ |
| `net/commands/ClientCommandUnsetBlockTargetSelection.java` | `ffb-protocol` | `src/commands/client_command_unset_block_target_selection.rs` | ✓ |
| `net/commands/ClientCommandUpdatePlayerMarkings.java` | `ffb-protocol` | `src/commands/client_command_update_player_markings.rs` | ✓ |
| `net/commands/ClientCommandUseApothecaries.java` | `ffb-protocol` | `src/commands/client_command_use_apothecaries.rs` | ✓ |
| `net/commands/ClientCommandUseApothecary.java` | `ffb-protocol` | `src/commands/client_command_use_apothecary.rs` | ✓ |
| `net/commands/ClientCommandUseBrawler.java` | `ffb-protocol` | `src/commands/client_command_use_brawler.rs` | ✓ |
| `net/commands/ClientCommandUseChainsaw.java` | `ffb-protocol` | `src/commands/client_command_use_chainsaw.rs` | ✓ |
| `net/commands/ClientCommandUseConsummateReRollForBlock.java` | `ffb-protocol` | `src/commands/client_command_use_consummate_re_roll_for_block.rs` | ✓ |
| `net/commands/ClientCommandUseFumblerooskie.java` | `ffb-protocol` | `src/commands/client_command_use_fumblerooskie.rs` | ✓ |
| `net/commands/ClientCommandUseHatred.java` | `ffb-protocol` | `src/commands/client_command_use_hatred.rs` | ✓ |
| `net/commands/ClientCommandUseIgors.java` | `ffb-protocol` | `src/commands/client_command_use_igors.rs` | ✓ |
| `net/commands/ClientCommandUseInducement.java` | `ffb-protocol` | `src/commands/client_command_use_inducement.rs` | ✓ |
| `net/commands/ClientCommandUseMultiBlockDiceReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_multi_block_dice_re_roll.rs` | ✓ |
| `net/commands/ClientCommandUseProReRollForBlock.java` | `ffb-protocol` | `src/commands/client_command_use_pro_re_roll_for_block.rs` | ✓ |
| `net/commands/ClientCommandUseReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_re_roll.rs` | ✓ |
| `net/commands/ClientCommandUseReRollForTarget.java` | `ffb-protocol` | `src/commands/client_command_use_re_roll_for_target.rs` | ✓ |
| `net/commands/ClientCommandUserSettings.java` | `ffb-protocol` | `src/commands/client_command_user_settings.rs` | ✓ |
| `net/commands/ClientCommandUseSingleBlockDieReRoll.java` | `ffb-protocol` | `src/commands/client_command_use_single_block_die_re_roll.rs` | ✓ |
| `net/commands/ClientCommandUseSkill.java` | `ffb-protocol` | `src/commands/client_command_use_skill.rs` | ✓ |
| `net/commands/ClientCommandUseTeamMatesWisdom.java` | `ffb-protocol` | `src/commands/client_command_use_team_mates_wisdom.rs` | ✓ |
| `net/commands/ClientCommandWizardSpell.java` | `ffb-protocol` | `src/commands/client_command_wizard_spell.rs` | ✓ |
| `net/commands/ClientSketchCommand.java` | `ffb-protocol` | `src/commands/client_sketch_command.rs` | ✓ |
| `net/commands/ICommandWithActingPlayer.java` | `ffb-protocol` | `src/commands/i_command_with_acting_player.rs` | ✓ |
| `net/commands/ServerCommand.java` | `ffb-protocol` | `src/commands/server_command.rs` | ✓ |
| `net/commands/ServerCommandAddPlayer.java` | `ffb-protocol` | `src/commands/server_command_add_player.rs` | ✓ |
| `net/commands/ServerCommandAddSketches.java` | `ffb-protocol` | `src/commands/server_command_add_sketches.rs` | ✓ |
| `net/commands/ServerCommandAdminMessage.java` | `ffb-protocol` | `src/commands/server_command_admin_message.rs` | ✓ |
| `net/commands/ServerCommandAutomaticPlayerMarkings.java` | `ffb-protocol` | `src/commands/server_command_automatic_player_markings.rs` | ✓ |
| `net/commands/ServerCommandClearSketches.java` | `ffb-protocol` | `src/commands/server_command_clear_sketches.rs` | ✓ |
| `net/commands/ServerCommandGameList.java` | `ffb-protocol` | `src/commands/server_command_game_list.rs` | ✓ |
| `net/commands/ServerCommandGameState.java` | `ffb-protocol` | `src/commands/server_command_game_state.rs` | ✓ |
| `net/commands/ServerCommandGameTime.java` | `ffb-protocol` | `src/commands/server_command_game_time.rs` | ✓ |
| `net/commands/ServerCommandJoin.java` | `ffb-protocol` | `src/commands/server_command_join.rs` | ✓ |
| `net/commands/ServerCommandLeave.java` | `ffb-protocol` | `src/commands/server_command_leave.rs` | ✓ |
| `net/commands/ServerCommandModelSync.java` | `ffb-protocol` | `src/commands/server_command_model_sync.rs` | ✓ |
| `net/commands/ServerCommandPasswordChallenge.java` | `ffb-protocol` | `src/commands/server_command_password_challenge.rs` | ✓ |
| `net/commands/ServerCommandPong.java` | `ffb-protocol` | `src/commands/server_command_pong.rs` | ✓ |
| `net/commands/ServerCommandRemovePlayer.java` | `ffb-protocol` | `src/commands/server_command_remove_player.rs` | ✓ |
| `net/commands/ServerCommandRemoveSketches.java` | `ffb-protocol` | `src/commands/server_command_remove_sketches.rs` | ✓ |
| `net/commands/ServerCommandReplay.java` | `ffb-protocol` | `src/commands/server_command_replay.rs` | ✓ |
| `net/commands/ServerCommandReplayControl.java` | `ffb-protocol` | `src/commands/server_command_replay_control.rs` | ✓ |
| `net/commands/ServerCommandReplayStatus.java` | `ffb-protocol` | `src/commands/server_command_replay_status.rs` | ✓ |
| `net/commands/ServerCommandSetPreventSketching.java` | `ffb-protocol` | `src/commands/server_command_set_prevent_sketching.rs` | ✓ |
| `net/commands/ServerCommandSketchAddCoordinate.java` | `ffb-protocol` | `src/commands/server_command_sketch_add_coordinate.rs` | ✓ |
| `net/commands/ServerCommandSketchSetColor.java` | `ffb-protocol` | `src/commands/server_command_sketch_set_color.rs` | ✓ |
| `net/commands/ServerCommandSketchSetLabel.java` | `ffb-protocol` | `src/commands/server_command_sketch_set_label.rs` | ✓ |
| `net/commands/ServerCommandSound.java` | `ffb-protocol` | `src/commands/server_command_sound.rs` | ✓ |
| `net/commands/ServerCommandStatus.java` | `ffb-protocol` | `src/commands/server_command_status.rs` | ✓ |
| `net/commands/ServerCommandTalk.java` | `ffb-protocol` | `src/commands/server_command_talk.rs` | ✓ |
| `net/commands/ServerCommandTeamList.java` | `ffb-protocol` | `src/commands/server_command_team_list.rs` | ✓ |
| `net/commands/ServerCommandTeamSetupList.java` | `ffb-protocol` | `src/commands/server_command_team_setup_list.rs` | ✓ |
| `net/commands/ServerCommandUnzapPlayer.java` | `ffb-protocol` | `src/commands/server_command_unzap_player.rs` | ✓ |
| `net/commands/ServerCommandUpdateLocalPlayerMarkers.java` | `ffb-protocol` | `src/commands/server_command_update_local_player_markers.rs` | ✓ |
| `net/commands/ServerCommandUserSettings.java` | `ffb-protocol` | `src/commands/server_command_user_settings.rs` | ✓ |
| `net/commands/ServerCommandVersion.java` | `ffb-protocol` | `src/commands/server_command_version.rs` | ✓ |
| `net/commands/ServerCommandZapPlayer.java` | `ffb-protocol` | `src/commands/server_command_zap_player.rs` | ✓ |
| `net/commands/UtilNetCommand.java` | `ffb-protocol` | `src/commands/util_net_command.rs` | ✓ |
| `net/GameCoach.java` | `ffb-protocol` | `src/game_coach.rs` | ✓ |
| `net/IConnectionListener.java` | `ffb-protocol` | `src/i_connection_listener.rs` | ✓ |
| `net/INetCommandHandler.java` | `ffb-protocol` | `src/i_net_command_handler.rs` | ✓ |
| `net/NetCommand.java` | `ffb-protocol` | `src/net_command.rs` | ✓ |
| `net/NetCommandFactory.java` | `ffb-protocol` | `src/net_command_factory.rs` | ✓ |
| `net/NetCommandId.java` | `ffb-protocol` | `src/net_command_id.rs` | ✓ |
| `net/NetCommandLog.java` | `ffb-protocol` | `src/net_command_log.rs` | ✓ |
| `net/ServerStatus.java` | `ffb-protocol` | `src/server_status.rs` | ✓ |
| `net/SocketChangeRequest.java` | `ffb-protocol` | `src/socket_change_request.rs` | ✓ |

### option/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `option/GameOptionAbstract.java` | `ffb-model` | `src/option/game_option_abstract.rs` | ✓ |
| `option/GameOptionBoolean.java` | `ffb-model` | `src/option/game_option_boolean.rs` | ✓ |
| `option/GameOptionId.java` | `ffb-model` | `src/option/game_option_id.rs` | ✓ |
| `option/GameOptionInt.java` | `ffb-model` | `src/option/game_option_int.rs` | ✓ |
| `option/GameOptionString.java` | `ffb-model` | `src/option/game_option_string.rs` | ✓ |
| `option/IGameOption.java` | `ffb-model` | `src/option/i_game_option.rs` | ✓ |
| `option/UtilGameOption.java` | `ffb-model` | `src/option/util_game_option.rs` | ✓ |

### report/ (191 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `report/bb2016/ReportApothecaryRoll.java` | `ffb-model` | `src/report/bb2016/report_apothecary_roll.rs` | ✓ |
| `report/bb2016/ReportArgueTheCallRoll.java` | `ffb-model` | `src/report/bb2016/report_argue_the_call_roll.rs` | ✓ |
| `report/bb2016/ReportCardsBought.java` | `ffb-model` | `src/report/bb2016/report_cards_bought.rs` | ✓ |
| `report/bb2016/ReportDodgeRoll.java` | `ffb-model` | `src/report/bb2016/report_dodge_roll.rs` | ✓ |
| `report/bb2016/ReportFanFactorRollPostMatch.java` | `ffb-model` | `src/report/bb2016/report_fan_factor_roll_post_match.rs` | ✓ |
| `report/bb2016/ReportHypnoticGazeRoll.java` | `ffb-model` | `src/report/bb2016/report_hypnotic_gaze_roll.rs` | ✓ |
| `report/bb2016/ReportInducementsBought.java` | `ffb-model` | `src/report/bb2016/report_inducements_bought.rs` | ✓ |
| `report/bb2016/ReportInjury.java` | `ffb-model` | `src/report/bb2016/report_injury.rs` | ✓ |
| `report/bb2016/ReportKickoffExtraReRoll.java` | `ffb-model` | `src/report/bb2016/report_kickoff_extra_re_roll.rs` | ✓ |
| `report/bb2016/ReportKickoffPitchInvasion.java` | `ffb-model` | `src/report/bb2016/report_kickoff_pitch_invasion.rs` | ✓ |
| `report/bb2016/ReportKickoffRiot.java` | `ffb-model` | `src/report/bb2016/report_kickoff_riot.rs` | ✓ |
| `report/bb2016/ReportKickoffThrowARock.java` | `ffb-model` | `src/report/bb2016/report_kickoff_throw_a_rock.rs` | ✓ |
| `report/bb2016/ReportKickTeamMateRoll.java` | `ffb-model` | `src/report/bb2016/report_kick_team_mate_roll.rs` | ✓ |
| `report/bb2016/ReportNervesOfSteel.java` | `ffb-model` | `src/report/bb2016/report_nerves_of_steel.rs` | ✓ |
| `report/bb2016/ReportNoPlayersToField.java` | `ffb-model` | `src/report/bb2016/report_no_players_to_field.rs` | ✓ |
| `report/bb2016/ReportPassRoll.java` | `ffb-model` | `src/report/bb2016/report_pass_roll.rs` | ✓ |
| `report/bb2016/ReportPenaltyShootout.java` | `ffb-model` | `src/report/bb2016/report_penalty_shootout.rs` | ✓ |
| `report/bb2016/ReportReferee.java` | `ffb-model` | `src/report/bb2016/report_referee.rs` | ✓ |
| `report/bb2016/ReportSpectators.java` | `ffb-model` | `src/report/bb2016/report_spectators.rs` | ✓ |
| `report/bb2016/ReportSwoopPlayer.java` | `ffb-model` | `src/report/bb2016/report_swoop_player.rs` | ✓ |
| `report/bb2016/ReportTentaclesShadowingRoll.java` | `ffb-model` | `src/report/bb2016/report_tentacles_shadowing_roll.rs` | ✓ |
| `report/bb2016/ReportThrowTeamMateRoll.java` | `ffb-model` | `src/report/bb2016/report_throw_team_mate_roll.rs` | ✓ |
| `report/bb2016/ReportTurnEnd.java` | `ffb-model` | `src/report/bb2016/report_turn_end.rs` | ✓ |
| `report/bb2016/ReportWinningsRoll.java` | `ffb-model` | `src/report/bb2016/report_winnings_roll.rs` | ✓ |
| `report/bb2020/ReportCardsAndInducementsBought.java` | `ffb-model` | `src/report/bb2020/report_cards_and_inducements_bought.rs` | ✓ |
| `report/bb2020/ReportCheeringFans.java` | `ffb-model` | `src/report/bb2020/report_cheering_fans.rs` | ✓ |
| `report/bb2020/ReportKickoffOfficiousRef.java` | `ffb-model` | `src/report/bb2020/report_kickoff_officious_ref.rs` | ✓ |
| `report/bb2020/ReportOfficiousRefRoll.java` | `ffb-model` | `src/report/bb2020/report_officious_ref_roll.rs` | ✓ |
| `report/bb2020/ReportPrayerRoll.java` | `ffb-model` | `src/report/bb2020/report_prayer_roll.rs` | ✓ |
| `report/bb2020/ReportSkillUseOtherPlayer.java` | `ffb-model` | `src/report/bb2020/report_skill_use_other_player.rs` | ✓ |
| `report/bb2020/ReportSwoopPlayer.java` | `ffb-model` | `src/report/bb2020/report_swoop_player.rs` | ✓ |
| `report/bb2020/ReportTwoForOne.java` | `ffb-model` | `src/report/bb2020/report_two_for_one.rs` | ✓ |
| `report/bb2025/ReportCheeringFans.java` | `ffb-model` | `src/report/bb2025/report_cheering_fans.rs` | ✓ |
| `report/bb2025/ReportChompRemoved.java` | `ffb-model` | `src/report/bb2025/report_chomp_removed.rs` | ✓ |
| `report/bb2025/ReportChompRoll.java` | `ffb-model` | `src/report/bb2025/report_chomp_roll.rs` | ✓ |
| `report/bb2025/ReportDodgySnackRoll.java` | `ffb-model` | `src/report/bb2025/report_dodgy_snack_roll.rs` | ✓ |
| `report/bb2025/ReportGettingEvenRoll.java` | `ffb-model` | `src/report/bb2025/report_getting_even_roll.rs` | ✓ |
| `report/bb2025/ReportKickoffDodgySnack.java` | `ffb-model` | `src/report/bb2025/report_kickoff_dodgy_snack.rs` | ✓ |
| `report/bb2025/ReportMascotUsed.java` | `ffb-model` | `src/report/bb2025/report_mascot_used.rs` | ✓ |
| `report/bb2025/ReportPickupRoll.java` | `ffb-model` | `src/report/bb2025/report_pickup_roll.rs` | ✓ |
| `report/bb2025/ReportPrayerRoll.java` | `ffb-model` | `src/report/bb2025/report_prayer_roll.rs` | ✓ |
| `report/bb2025/ReportPrayersAndInducementsBought.java` | `ffb-model` | `src/report/bb2025/report_prayers_and_inducements_bought.rs` | ✓ |
| `report/bb2025/ReportPuntDirection.java` | `ffb-model` | `src/report/bb2025/report_punt_direction.rs` | ✓ |
| `report/bb2025/ReportPuntDistance.java` | `ffb-model` | `src/report/bb2025/report_punt_distance.rs` | ✓ |
| `report/bb2025/ReportSaboteurRoll.java` | `ffb-model` | `src/report/bb2025/report_saboteur_roll.rs` | ✓ |
| `report/bb2025/ReportSteadyFootingRoll.java` | `ffb-model` | `src/report/bb2025/report_steady_footing_roll.rs` | ✓ |
| `report/bb2025/ReportSwarmingRoll.java` | `ffb-model` | `src/report/bb2025/report_swarming_roll.rs` | ✓ |
| `report/bb2025/ReportSwoopDirection.java` | `ffb-model` | `src/report/bb2025/report_swoop_direction.rs` | ✓ |
| `report/bb2025/ReportSwoopPlayer.java` | `ffb-model` | `src/report/bb2025/report_swoop_player.rs` | ✓ |
| `report/bb2025/ReportTeamCaptainRoll.java` | `ffb-model` | `src/report/bb2025/report_team_captain_roll.rs` | ✓ |
| `report/bb2025/ReportTeamEvent.java` | `ffb-model` | `src/report/bb2025/report_team_event.rs` | ✓ |
| `report/bb2025/ReportThrowAtPlayer.java` | `ffb-model` | `src/report/bb2025/report_throw_at_player.rs` | ✓ |
| `report/IReport.java` | `ffb-model` | `src/report/i_report.rs` | ✓ |
| `report/logcontrol/SkipInjuryParts.java` | `ffb-model` | `src/report/skip_injury_parts.rs` | ✓ |
| `report/mixed/ReportAllYouCanEatRoll.java` | `ffb-model` | `src/report/mixed/report_all_you_can_eat_roll.rs` | ✓ |
| `report/mixed/ReportAnimalSavagery.java` | `ffb-model` | `src/report/mixed/report_animal_savagery.rs` | ✓ |
| `report/mixed/ReportApothecaryRoll.java` | `ffb-model` | `src/report/mixed/report_apothecary_roll.rs` | ✓ |
| `report/mixed/ReportArgueTheCallRoll.java` | `ffb-model` | `src/report/mixed/report_argue_the_call_roll.rs` | ✓ |
| `report/mixed/ReportBalefulHexRoll.java` | `ffb-model` | `src/report/mixed/report_baleful_hex_roll.rs` | ✓ |
| `report/mixed/ReportBiasedRef.java` | `ffb-model` | `src/report/mixed/report_biased_ref.rs` | ✓ |
| `report/mixed/ReportBlitzRoll.java` | `ffb-model` | `src/report/mixed/report_blitz_roll.rs` | ✓ |
| `report/mixed/ReportBlockReRoll.java` | `ffb-model` | `src/report/mixed/report_block_re_roll.rs` | ✓ |
| `report/mixed/ReportBreatheFire.java` | `ffb-model` | `src/report/mixed/report_breathe_fire.rs` | ✓ |
| `report/mixed/ReportBriberyAndCorruptionReRoll.java` | `ffb-model` | `src/report/mixed/report_bribery_and_corruption_re_roll.rs` | ✓ |
| `report/mixed/ReportBrilliantCoachingReRollsLost.java` | `ffb-model` | `src/report/mixed/report_brilliant_coaching_re_rolls_lost.rs` | ✓ |
| `report/mixed/ReportCatchOfTheDayRoll.java` | `ffb-model` | `src/report/mixed/report_catch_of_the_day_roll.rs` | ✓ |
| `report/mixed/ReportCloudBurster.java` | `ffb-model` | `src/report/mixed/report_cloud_burster.rs` | ✓ |
| `report/mixed/ReportDedicatedFans.java` | `ffb-model` | `src/report/mixed/report_dedicated_fans.rs` | ✓ |
| `report/mixed/ReportDodgeRoll.java` | `ffb-model` | `src/report/mixed/report_dodge_roll.rs` | ✓ |
| `report/mixed/ReportDoubleHiredStaff.java` | `ffb-model` | `src/report/mixed/report_double_hired_staff.rs` | ✓ |
| `report/mixed/ReportEvent.java` | `ffb-model` | `src/report/mixed/report_event.rs` | ✓ |
| `report/mixed/ReportFanFactor.java` | `ffb-model` | `src/report/mixed/report_fan_factor.rs` | ✓ |
| `report/mixed/ReportFreePettyCash.java` | `ffb-model` | `src/report/mixed/report_free_petty_cash.rs` | ✓ |
| `report/mixed/ReportFumblerooskie.java` | `ffb-model` | `src/report/mixed/report_fumblerooskie.rs` | ✓ |
| `report/mixed/ReportHitAndRun.java` | `ffb-model` | `src/report/mixed/report_hit_and_run.rs` | ✓ |
| `report/mixed/ReportHypnoticGazeRoll.java` | `ffb-model` | `src/report/mixed/report_hypnotic_gaze_roll.rs` | ✓ |
| `report/mixed/ReportIndomitable.java` | `ffb-model` | `src/report/mixed/report_indomitable.rs` | ✓ |
| `report/mixed/ReportInjury.java` | `ffb-model` | `src/report/mixed/report_injury.rs` | ✓ |
| `report/mixed/ReportKickoffExtraReRoll.java` | `ffb-model` | `src/report/mixed/report_kickoff_extra_re_roll.rs` | ✓ |
| `report/mixed/ReportKickoffPitchInvasion.java` | `ffb-model` | `src/report/mixed/report_kickoff_pitch_invasion.rs` | ✓ |
| `report/mixed/ReportKickoffSequenceActivationsCount.java` | `ffb-model` | `src/report/mixed/report_kickoff_sequence_activations_count.rs` | ✓ |
| `report/mixed/ReportKickoffSequenceActivationsExhausted.java` | `ffb-model` | `src/report/mixed/report_kickoff_sequence_activations_exhausted.rs` | ✓ |
| `report/mixed/ReportKickoffTimeout.java` | `ffb-model` | `src/report/mixed/report_kickoff_timeout.rs` | ✓ |
| `report/mixed/ReportKickTeamMateFumble.java` | `ffb-model` | `src/report/mixed/report_kick_team_mate_fumble.rs` | ✓ |
| `report/mixed/ReportLookIntoMyEyesRoll.java` | `ffb-model` | `src/report/mixed/report_look_into_my_eyes_roll.rs` | ✓ |
| `report/mixed/ReportModifiedDodgeResultSuccessful.java` | `ffb-model` | `src/report/mixed/report_modified_dodge_result_successful.rs` | ✓ |
| `report/mixed/ReportModifiedPassResult.java` | `ffb-model` | `src/report/mixed/report_modified_pass_result.rs` | ✓ |
| `report/mixed/ReportNervesOfSteel.java` | `ffb-model` | `src/report/mixed/report_nerves_of_steel.rs` | ✓ |
| `report/mixed/ReportOldPro.java` | `ffb-model` | `src/report/mixed/report_old_pro.rs` | ✓ |
| `report/mixed/ReportPassRoll.java` | `ffb-model` | `src/report/mixed/report_pass_roll.rs` | ✓ |
| `report/mixed/ReportPenaltyShootout.java` | `ffb-model` | `src/report/mixed/report_penalty_shootout.rs` | ✓ |
| `report/mixed/ReportPickMeUp.java` | `ffb-model` | `src/report/mixed/report_pick_me_up.rs` | ✓ |
| `report/mixed/ReportPickupRoll.java` | `ffb-model` | `src/report/mixed/report_pickup_roll.rs` | ✓ |
| `report/mixed/ReportPlaceBallDirection.java` | `ffb-model` | `src/report/mixed/report_place_ball_direction.rs` | ✓ |
| `report/mixed/ReportPlayerEvent.java` | `ffb-model` | `src/report/mixed/report_player_event.rs` | ✓ |
| `report/mixed/ReportPrayerAmount.java` | `ffb-model` | `src/report/mixed/report_prayer_amount.rs` | ✓ |
| `report/mixed/ReportPrayerEnd.java` | `ffb-model` | `src/report/mixed/report_prayer_end.rs` | ✓ |
| `report/mixed/ReportPrayerWasted.java` | `ffb-model` | `src/report/mixed/report_prayer_wasted.rs` | ✓ |
| `report/mixed/ReportProjectileVomit.java` | `ffb-model` | `src/report/mixed/report_projectile_vomit.rs` | ✓ |
| `report/mixed/ReportPumpUpTheCrowdReRoll.java` | `ffb-model` | `src/report/mixed/report_pump_up_the_crowd_re_roll.rs` | ✓ |
| `report/mixed/ReportPumpUpTheCrowdReRollsLost.java` | `ffb-model` | `src/report/mixed/report_pump_up_the_crowd_re_rolls_lost.rs` | ✓ |
| `report/mixed/ReportQuickSnapRoll.java` | `ffb-model` | `src/report/mixed/report_quick_snap_roll.rs` | ✓ |
| `report/mixed/ReportRaidingParty.java` | `ffb-model` | `src/report/mixed/report_raiding_party.rs` | ✓ |
| `report/mixed/ReportReferee.java` | `ffb-model` | `src/report/mixed/report_referee.rs` | ✓ |
| `report/mixed/ReportSelectBlitzTarget.java` | `ffb-model` | `src/report/mixed/report_select_blitz_target.rs` | ✓ |
| `report/mixed/ReportSelectGazeTarget.java` | `ffb-model` | `src/report/mixed/report_select_gaze_target.rs` | ✓ |
| `report/mixed/ReportShowStarReRoll.java` | `ffb-model` | `src/report/mixed/report_show_star_re_roll.rs` | ✓ |
| `report/mixed/ReportShowStarReRollsLost.java` | `ffb-model` | `src/report/mixed/report_show_star_re_rolls_lost.rs` | ✓ |
| `report/mixed/ReportSkillWasted.java` | `ffb-model` | `src/report/mixed/report_skill_wasted.rs` | ✓ |
| `report/mixed/ReportSolidDefenceRoll.java` | `ffb-model` | `src/report/mixed/report_solid_defence_roll.rs` | ✓ |
| `report/mixed/ReportStallerDetected.java` | `ffb-model` | `src/report/mixed/report_staller_detected.rs` | ✓ |
| `report/mixed/ReportSwarmingRoll.java` | `ffb-model` | `src/report/mixed/report_swarming_roll.rs` | ✓ |
| `report/mixed/ReportTentaclesShadowingRoll.java` | `ffb-model` | `src/report/mixed/report_tentacles_shadowing_roll.rs` | ✓ |
| `report/mixed/ReportThenIStartedBlastin.java` | `ffb-model` | `src/report/mixed/report_then_i_started_blastin.rs` | ✓ |
| `report/mixed/ReportThrowAtStallingPlayer.java` | `ffb-model` | `src/report/mixed/report_throw_at_stalling_player.rs` | ✓ |
| `report/mixed/ReportThrownKeg.java` | `ffb-model` | `src/report/mixed/report_thrown_keg.rs` | ✓ |
| `report/mixed/ReportThrowTeamMateRoll.java` | `ffb-model` | `src/report/mixed/report_throw_team_mate_roll.rs` | ✓ |
| `report/mixed/ReportTrapDoor.java` | `ffb-model` | `src/report/mixed/report_trap_door.rs` | ✓ |
| `report/mixed/ReportTurnEnd.java` | `ffb-model` | `src/report/mixed/report_turn_end.rs` | ✓ |
| `report/mixed/ReportWeatherMageResult.java` | `ffb-model` | `src/report/mixed/report_weather_mage_result.rs` | ✓ |
| `report/mixed/ReportWeatherMageRoll.java` | `ffb-model` | `src/report/mixed/report_weather_mage_roll.rs` | ✓ |
| `report/mixed/ReportWinnings.java` | `ffb-model` | `src/report/mixed/report_winnings.rs` | ✓ |
| `report/NoDiceReport.java` | `ffb-model` | `src/report/no_dice_report.rs` | ✓ |
| `report/ReportAlwaysHungryRoll.java` | `ffb-model` | `src/report/report_always_hungry_roll.rs` | ✓ |
| `report/ReportAnimosityRoll.java` | `ffb-model` | `src/report/report_animosity_roll.rs` | ✓ |
| `report/ReportApothecaryChoice.java` | `ffb-model` | `src/report/report_apothecary_choice.rs` | ✓ |
| `report/ReportBiteSpectator.java` | `ffb-model` | `src/report/report_bite_spectator.rs` | ✓ |
| `report/ReportBlock.java` | `ffb-model` | `src/report/report_block.rs` | ✓ |
| `report/ReportBlockChoice.java` | `ffb-model` | `src/report/report_block_choice.rs` | ✓ |
| `report/ReportBlockRoll.java` | `ffb-model` | `src/report/report_block_roll.rs` | ✓ |
| `report/ReportBloodLustRoll.java` | `ffb-model` | `src/report/report_blood_lust_roll.rs` | ✓ |
| `report/ReportBombExplodesAfterCatch.java` | `ffb-model` | `src/report/report_bomb_explodes_after_catch.rs` | ✓ |
| `report/ReportBombOutOfBounds.java` | `ffb-model` | `src/report/report_bomb_out_of_bounds.rs` | ✓ |
| `report/ReportBribesRoll.java` | `ffb-model` | `src/report/report_bribes_roll.rs` | ✓ |
| `report/ReportCardDeactivated.java` | `ffb-model` | `src/report/report_card_deactivated.rs` | ✓ |
| `report/ReportCardEffectRoll.java` | `ffb-model` | `src/report/report_card_effect_roll.rs` | ✓ |
| `report/ReportCatchRoll.java` | `ffb-model` | `src/report/report_catch_roll.rs` | ✓ |
| `report/ReportChainsawRoll.java` | `ffb-model` | `src/report/report_chainsaw_roll.rs` | ✓ |
| `report/ReportCoinThrow.java` | `ffb-model` | `src/report/report_coin_throw.rs` | ✓ |
| `report/ReportConfusionRoll.java` | `ffb-model` | `src/report/report_confusion_roll.rs` | ✓ |
| `report/ReportDauntlessRoll.java` | `ffb-model` | `src/report/report_dauntless_roll.rs` | ✓ |
| `report/ReportDefectingPlayers.java` | `ffb-model` | `src/report/report_defecting_players.rs` | ✓ |
| `report/ReportDoubleHiredStarPlayer.java` | `ffb-model` | `src/report/report_double_hired_star_player.rs` | ✓ |
| `report/ReportEscapeRoll.java` | `ffb-model` | `src/report/report_escape_roll.rs` | ✓ |
| `report/ReportFoul.java` | `ffb-model` | `src/report/report_foul.rs` | ✓ |
| `report/ReportFoulAppearanceRoll.java` | `ffb-model` | `src/report/report_foul_appearance_roll.rs` | ✓ |
| `report/ReportFumbblResultUpload.java` | `ffb-model` | `src/report/report_fumbbl_result_upload.rs` | ✓ |
| `report/ReportGameOptions.java` | `ffb-model` | `src/report/report_game_options.rs` | ✓ |
| `report/ReportGoForItRoll.java` | `ffb-model` | `src/report/report_go_for_it_roll.rs` | ✓ |
| `report/ReportHandOver.java` | `ffb-model` | `src/report/report_hand_over.rs` | ✓ |
| `report/ReportId.java` | `ffb-model` | `src/report/report_id.rs` | ✓ |
| `report/ReportInducement.java` | `ffb-model` | `src/report/report_inducement.rs` | ✓ |
| `report/ReportInjury.java` | `ffb-model` | `src/report/report_injury.rs` | ✓ |
| `report/ReportInterceptionRoll.java` | `ffb-model` | `src/report/report_interception_roll.rs` | ✓ |
| `report/ReportJumpRoll.java` | `ffb-model` | `src/report/report_jump_roll.rs` | ✓ |
| `report/ReportJumpUpRoll.java` | `ffb-model` | `src/report/report_jump_up_roll.rs` | ✓ |
| `report/ReportKickoffResult.java` | `ffb-model` | `src/report/report_kickoff_result.rs` | ✓ |
| `report/ReportKickoffScatter.java` | `ffb-model` | `src/report/report_kickoff_scatter.rs` | ✓ |
| `report/ReportLeader.java` | `ffb-model` | `src/report/report_leader.rs` | ✓ |
| `report/ReportList.java` | `ffb-model` | `src/report/report_list.rs` | ✓ |
| `report/ReportMasterChefRoll.java` | `ffb-model` | `src/report/report_master_chef_roll.rs` | ✓ |
| `report/ReportMostValuablePlayers.java` | `ffb-model` | `src/report/report_most_valuable_players.rs` | ✓ |
| `report/ReportPassBlock.java` | `ffb-model` | `src/report/report_pass_block.rs` | ✓ |
| `report/ReportPassDeviate.java` | `ffb-model` | `src/report/report_pass_deviate.rs` | ✓ |
| `report/ReportPettyCash.java` | `ffb-model` | `src/report/report_petty_cash.rs` | ✓ |
| `report/ReportPickupRoll.java` | `ffb-model` | `src/report/report_pickup_roll.rs` | ✓ |
| `report/ReportPilingOn.java` | `ffb-model` | `src/report/report_piling_on.rs` | ✓ |
| `report/ReportPlayCard.java` | `ffb-model` | `src/report/report_play_card.rs` | ✓ |
| `report/ReportPlayerAction.java` | `ffb-model` | `src/report/report_player_action.rs` | ✓ |
| `report/ReportPushback.java` | `ffb-model` | `src/report/report_pushback.rs` | ✓ |
| `report/ReportRaiseDead.java` | `ffb-model` | `src/report/report_raise_dead.rs` | ✓ |
| `report/ReportReceiveChoice.java` | `ffb-model` | `src/report/report_receive_choice.rs` | ✓ |
| `report/ReportRegenerationRoll.java` | `ffb-model` | `src/report/report_regeneration_roll.rs` | ✓ |
| `report/ReportReRoll.java` | `ffb-model` | `src/report/report_re_roll.rs` | ✓ |
| `report/ReportRightStuffRoll.java` | `ffb-model` | `src/report/report_right_stuff_roll.rs` | ✓ |
| `report/ReportRiotousRookies.java` | `ffb-model` | `src/report/report_riotous_rookies.rs` | ✓ |
| `report/ReportSafeThrowRoll.java` | `ffb-model` | `src/report/report_safe_throw_roll.rs` | ✓ |
| `report/ReportScatterBall.java` | `ffb-model` | `src/report/report_scatter_ball.rs` | ✓ |
| `report/ReportScatterPlayer.java` | `ffb-model` | `src/report/report_scatter_player.rs` | ✓ |
| `report/ReportSecretWeaponBan.java` | `ffb-model` | `src/report/report_secret_weapon_ban.rs` | ✓ |
| `report/ReportSkillRoll.java` | `ffb-model` | `src/report/report_skill_roll.rs` | ✓ |
| `report/ReportSkillUse.java` | `ffb-model` | `src/report/report_skill_use.rs` | ✓ |
| `report/ReportSpecialEffectRoll.java` | `ffb-model` | `src/report/report_special_effect_roll.rs` | ✓ |
| `report/ReportStandUpRoll.java` | `ffb-model` | `src/report/report_stand_up_roll.rs` | ✓ |
| `report/ReportStartHalf.java` | `ffb-model` | `src/report/report_start_half.rs` | ✓ |
| `report/ReportThrowIn.java` | `ffb-model` | `src/report/report_throw_in.rs` | ✓ |
| `report/ReportTimeoutEnforced.java` | `ffb-model` | `src/report/report_timeout_enforced.rs` | ✓ |
| `report/ReportWeather.java` | `ffb-model` | `src/report/report_weather.rs` | ✓ |
| `report/ReportWeepingDaggerRoll.java` | `ffb-model` | `src/report/report_weeping_dagger_roll.rs` | ✓ |
| `report/ReportWizardUse.java` | `ffb-model` | `src/report/report_wizard_use.rs` | ✓ |
| `report/UtilReport.java` | `ffb-model` | `src/report/util_report.rs` | ✓ |

### root/ (86 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `ApothecaryMode.java` | `ffb-model` | `src/model/apothecary_mode.rs` | ✓ |
| `ApothecaryStatus.java` | `ffb-model` | `src/model/apothecary_status.rs` | ✓ |
| `ApothecaryType.java` | `ffb-model` | `src/model/apothecary_type.rs` | ✓ |
| `BlockDiceCategory.java` | `ffb-model` | `src/model/block_dice_category.rs` | ✓ |
| `BlockResult.java` | `ffb-model` | `src/model/block_result.rs` | ✓ |
| `BloodSpot.java` | `ffb-model` | `src/model/blood_spot.rs` | ✓ |
| `BoxType.java` | `ffb-model` | `src/model/box_type.rs` | ✓ |
| `BreatheFireResult.java` | `ffb-model` | `src/model/breathe_fire_result.rs` | ✓ |
| `CardEffect.java` | `ffb-model` | `src/model/card_effect.rs` | ✓ |
| `CardTarget.java` | `ffb-model` | `src/model/card_target.rs` | ✓ |
| `CatchScatterThrowInMode.java` | `ffb-model` | `src/model/catch_scatter_throw_in_mode.rs` | ✓ |
| `ChatCommand.java` | `ffb-model` | `src/model/chat_command.rs` | ✓ |
| `ClientMode.java` | `ffb-model` | `src/model/client_mode.rs` | ✓ |
| `ClientStateId.java` | `ffb-model` | `src/model/client_state_id.rs` | ✓ |
| `CommonProperty.java` | `ffb-model` | `src/model/common_property.rs` | ✓ |
| `CommonPropertyValue.java` | `ffb-model` | `src/model/common_property_value.rs` | ✓ |
| `ConcedeGameStatus.java` | `ffb-model` | `src/model/concede_game_status.rs` | ✓ |
| `Constant.java` | `ffb-model` | `src/model/constant.rs` | ✓ |
| `DefenderAction.java` | `ffb-model` | `src/model/defender_action.rs` | ✓ |
| `DiceCategory.java` | `ffb-model` | `src/model/dice_category.rs` | ✓ |
| `DiceCategoryFactory.java` | `ffb-model` | `src/model/dice_category_factory.rs` | ✓ |
| `DiceDecoration.java` | `ffb-model` | `src/model/dice_decoration.rs` | ✓ |
| `Direction.java` | `ffb-model` | `src/model/direction.rs` | ✓ |
| `DirectionDiceCategory.java` | `ffb-model` | `src/model/direction_dice_category.rs` | ✓ |
| `FactoryManager.java` | `ffb-model` | `src/model/factory_manager.rs` | ✓ |
| `FactoryType.java` | `ffb-model` | `src/model/factory_type.rs` | ✓ |
| `FantasyFootballConstants.java` | `ffb-model` | `src/model/fantasy_football_constants.rs` | ✓ |
| `FantasyFootballException.java` | `ffb-model` | `src/model/fantasy_football_exception.rs` | ✓ |
| `FieldCoordinate.java` | `ffb-model` | `src/model/field_coordinate.rs` | ✓ |
| `FieldCoordinateBounds.java` | `ffb-model` | `src/model/field_coordinate_bounds.rs` | ✓ |
| `FieldModelChangeEvent.java` | `ffb-model` | `src/model/field_model_change_event.rs` | ✓ |
| `GameList.java` | `ffb-model` | `src/model/game_list.rs` | ✓ |
| `GameListEntry.java` | `ffb-model` | `src/model/game_list_entry.rs` | ✓ |
| `GameStatus.java` | `ffb-model` | `src/model/game_status.rs` | ✓ |
| `HasReRollProperties.java` | `ffb-model` | `src/model/has_re_roll_properties.rs` | ✓ |
| `HeatExhaustion.java` | `ffb-model` | `src/model/heat_exhaustion.rs` | ✓ |
| `IClientProperty.java` | `ffb-model` | `src/model/i_client_property.rs` | ✓ |
| `IClientPropertyValue.java` | `ffb-model` | `src/model/i_client_property_value.rs` | ✓ |
| `IDialogParameter.java` | `ffb-model` | `src/model/i_dialog_parameter.rs` | ✓ |
| `IFieldModelChangeListener.java` | `ffb-model` | `src/model/i_field_model_change_listener.rs` | ✓ |
| `IIconProperty.java` | `ffb-model` | `src/model/i_icon_property.rs` | ✓ |
| `IKeyedItem.java` | `ffb-model` | `src/model/i_keyed_item.rs` | ✓ |
| `IKickOffResult.java` | `ffb-model` | `src/model/i_kick_off_result.rs` | ✓ |
| `INamedObject.java` | `ffb-model` | `src/model/i_named_object.rs` | ✓ |
| `InjuryAttribute.java` | `ffb-model` | `src/model/injury_attribute.rs` | ✓ |
| `KeyedItemRegistry.java` | `ffb-model` | `src/model/keyed_item_registry.rs` | ✓ |
| `KeywordChoiceMode.java` | `ffb-model` | `src/model/keyword_choice_mode.rs` | ✓ |
| `KnockoutRecovery.java` | `ffb-model` | `src/model/knockout_recovery.rs` | ✓ |
| `LeaderState.java` | `ffb-model` | `src/model/leader_state.rs` | ✓ |
| `MoveSquare.java` | `ffb-model` | `src/model/move_square.rs` | ✓ |
| `Pair.java` | `ffb-model` | `src/model/pair.rs` | ✓ |
| `PassingDistance.java` | `ffb-model` | `src/model/passing_distance.rs` | ✓ |
| `PasswordChallenge.java` | `ffb-model` | `src/model/password_challenge.rs` | ✓ |
| `PlayerAction.java` | `ffb-model` | `src/model/player_action.rs` | ✓ |
| `PlayerChoiceMode.java` | `ffb-model` | `src/model/player_choice_mode.rs` | ✓ |
| `PlayerGender.java` | `ffb-model` | `src/model/player_gender.rs` | ✓ |
| `PlayerState.java` | `ffb-model` | `src/model/player_state.rs` | ✓ |
| `PlayerType.java` | `ffb-model` | `src/model/player_type.rs` | ✓ |
| `PositionChoiceMode.java` | `ffb-model` | `src/model/position_choice_mode.rs` | ✓ |
| `Pushback.java` | `ffb-model` | `src/model/pushback.rs` | ✓ |
| `PushbackMode.java` | `ffb-model` | `src/model/pushback_mode.rs` | ✓ |
| `PushbackSquare.java` | `ffb-model` | `src/model/pushback_square.rs` | ✓ |
| `RangeRuler.java` | `ffb-model` | `src/model/range_ruler.rs` | ✓ |
| `ReRolledAction.java` | `ffb-model` | `src/model/re_rolled_action.rs` | ✓ |
| `ReRolledActions.java` | `ffb-model` | `src/model/re_rolled_actions.rs` | ✓ |
| `ReRollOptions.java` | `ffb-model` | `src/model/re_roll_options.rs` | ✓ |
| `ReRollProperty.java` | `ffb-model` | `src/model/re_roll_property.rs` | ✓ |
| `ReRollSource.java` | `ffb-model` | `src/model/re_roll_source.rs` | ✓ |
| `ReRollSources.java` | `ffb-model` | `src/model/re_roll_sources.rs` | ✓ |
| `RulesCollection.java` | `ffb-model` | `src/model/rules_collection.rs` | ✓ |
| `RulesCollections.java` | `ffb-model` | `src/model/rules_collections.rs` | ✓ |
| `SendToBoxReason.java` | `ffb-model` | `src/model/send_to_box_reason.rs` | ✓ |
| `SeriousInjury.java` | `ffb-model` | `src/model/serious_injury.rs` | ✓ |
| `SkillCategory.java` | `ffb-model` | `src/model/skill_category.rs` | ✓ |
| `SkillChoiceMode.java` | `ffb-model` | `src/model/skill_choice_mode.rs` | ✓ |
| `SkillUse.java` | `ffb-model` | `src/model/skill_use.rs` | ✓ |
| `SoundId.java` | `ffb-model` | `src/model/sound_id.rs` | ✓ |
| `SpecialEffect.java` | `ffb-model` | `src/model/special_effect.rs` | ✓ |
| `StatusType.java` | `ffb-model` | `src/model/status_type.rs` | ✓ |
| `TeamList.java` | `ffb-model` | `src/model/team_list.rs` | ✓ |
| `TeamListEntry.java` | `ffb-model` | `src/model/team_list_entry.rs` | ✓ |
| `TeamSetup.java` | `ffb-model` | `src/model/team_setup.rs` | ✓ |
| `TeamStatus.java` | `ffb-model` | `src/model/team_status.rs` | ✓ |
| `TrackNumber.java` | `ffb-model` | `src/model/track_number.rs` | ✓ |
| `TurnMode.java` | `ffb-model` | `src/model/turn_mode.rs` | ✓ |
| `Weather.java` | `ffb-model` | `src/model/weather.rs` | ✓ |

### skill/ (297 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `skill/ArmourIncrease.java` | `ffb-model` | `src/skill/armour_increase.rs` | ✓ |
| `skill/bb2016/Accurate.java` | `ffb-model` | `src/skill/bb2016/accurate.rs` | ✓ |
| `skill/bb2016/AlwaysHungry.java` | `ffb-model` | `src/skill/bb2016/always_hungry.rs` | ✓ |
| `skill/bb2016/Animosity.java` | `ffb-model` | `src/skill/bb2016/animosity.rs` | ✓ |
| `skill/bb2016/ArmourIncrease.java` | `ffb-model` | `src/skill/bb2016/armour_increase.rs` | ✓ |
| `skill/bb2016/BallAndChain.java` | `ffb-model` | `src/skill/bb2016/ball_and_chain.rs` | ✓ |
| `skill/bb2016/BloodLust.java` | `ffb-model` | `src/skill/bb2016/blood_lust.rs` | ✓ |
| `skill/bb2016/Bombardier.java` | `ffb-model` | `src/skill/bb2016/bombardier.rs` | ✓ |
| `skill/bb2016/BoneHead.java` | `ffb-model` | `src/skill/bb2016/bone_head.rs` | ✓ |
| `skill/bb2016/BreakTackle.java` | `ffb-model` | `src/skill/bb2016/break_tackle.rs` | ✓ |
| `skill/bb2016/Chainsaw.java` | `ffb-model` | `src/skill/bb2016/chainsaw.rs` | ✓ |
| `skill/bb2016/Claw.java` | `ffb-model` | `src/skill/bb2016/claw.rs` | ✓ |
| `skill/bb2016/Decay.java` | `ffb-model` | `src/skill/bb2016/decay.rs` | ✓ |
| `skill/bb2016/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2016/dirty_player.rs` | ✓ |
| `skill/bb2016/Disposable.java` | `ffb-model` | `src/skill/bb2016/disposable.rs` | ✓ |
| `skill/bb2016/DivingTackle.java` | `ffb-model` | `src/skill/bb2016/diving_tackle.rs` | ✓ |
| `skill/bb2016/FanFavourite.java` | `ffb-model` | `src/skill/bb2016/fan_favourite.rs` | ✓ |
| `skill/bb2016/Frenzy.java` | `ffb-model` | `src/skill/bb2016/frenzy.rs` | ✓ |
| `skill/bb2016/Grab.java` | `ffb-model` | `src/skill/bb2016/grab.rs` | ✓ |
| `skill/bb2016/Guard.java` | `ffb-model` | `src/skill/bb2016/guard.rs` | ✓ |
| `skill/bb2016/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2016/hypnotic_gaze.rs` | ✓ |
| `skill/bb2016/KickOffReturn.java` | `ffb-model` | `src/skill/bb2016/kick_off_return.rs` | ✓ |
| `skill/bb2016/KickTeamMate.java` | `ffb-model` | `src/skill/bb2016/kick_team_mate.rs` | ✓ |
| `skill/bb2016/Leap.java` | `ffb-model` | `src/skill/bb2016/leap.rs` | ✓ |
| `skill/bb2016/Loner.java` | `ffb-model` | `src/skill/bb2016/loner.rs` | ✓ |
| `skill/bb2016/MightyBlow.java` | `ffb-model` | `src/skill/bb2016/mighty_blow.rs` | ✓ |
| `skill/bb2016/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2016/monstrous_mouth.rs` | ✓ |
| `skill/bb2016/MultipleBlock.java` | `ffb-model` | `src/skill/bb2016/multiple_block.rs` | ✓ |
| `skill/bb2016/NervesOfSteel.java` | `ffb-model` | `src/skill/bb2016/nerves_of_steel.rs` | ✓ |
| `skill/bb2016/NoHands.java` | `ffb-model` | `src/skill/bb2016/no_hands.rs` | ✓ |
| `skill/bb2016/NurglesRot.java` | `ffb-model` | `src/skill/bb2016/nurgles_rot.rs` | ✓ |
| `skill/bb2016/PassBlock.java` | `ffb-model` | `src/skill/bb2016/pass_block.rs` | ✓ |
| `skill/bb2016/PilingOn.java` | `ffb-model` | `src/skill/bb2016/piling_on.rs` | ✓ |
| `skill/bb2016/PrehensileTail.java` | `ffb-model` | `src/skill/bb2016/prehensile_tail.rs` | ✓ |
| `skill/bb2016/ReallyStupid.java` | `ffb-model` | `src/skill/bb2016/really_stupid.rs` | ✓ |
| `skill/bb2016/Regeneration.java` | `ffb-model` | `src/skill/bb2016/regeneration.rs` | ✓ |
| `skill/bb2016/RightStuff.java` | `ffb-model` | `src/skill/bb2016/right_stuff.rs` | ✓ |
| `skill/bb2016/SafeThrow.java` | `ffb-model` | `src/skill/bb2016/safe_throw.rs` | ✓ |
| `skill/bb2016/SecretWeapon.java` | `ffb-model` | `src/skill/bb2016/secret_weapon.rs` | ✓ |
| `skill/bb2016/Shadowing.java` | `ffb-model` | `src/skill/bb2016/shadowing.rs` | ✓ |
| `skill/bb2016/SideStep.java` | `ffb-model` | `src/skill/bb2016/side_step.rs` | ✓ |
| `skill/bb2016/SneakyGit.java` | `ffb-model` | `src/skill/bb2016/sneaky_git.rs` | ✓ |
| `skill/bb2016/Stab.java` | `ffb-model` | `src/skill/bb2016/stab.rs` | ✓ |
| `skill/bb2016/Stakes.java` | `ffb-model` | `src/skill/bb2016/stakes.rs` | ✓ |
| `skill/bb2016/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2016/strength_increase.rs` | ✓ |
| `skill/bb2016/StrongArm.java` | `ffb-model` | `src/skill/bb2016/strong_arm.rs` | ✓ |
| `skill/bb2016/Stunty.java` | `ffb-model` | `src/skill/bb2016/stunty.rs` | ✓ |
| `skill/bb2016/SureFeet.java` | `ffb-model` | `src/skill/bb2016/sure_feet.rs` | ✓ |
| `skill/bb2016/Swarming.java` | `ffb-model` | `src/skill/bb2016/swarming.rs` | ✓ |
| `skill/bb2016/Swoop.java` | `ffb-model` | `src/skill/bb2016/swoop.rs` | ✓ |
| `skill/bb2016/TakeRoot.java` | `ffb-model` | `src/skill/bb2016/take_root.rs` | ✓ |
| `skill/bb2016/ThrowTeamMate.java` | `ffb-model` | `src/skill/bb2016/throw_team_mate.rs` | ✓ |
| `skill/bb2016/Timmmber.java` | `ffb-model` | `src/skill/bb2016/timmmber.rs` | ✓ |
| `skill/bb2016/Titchy.java` | `ffb-model` | `src/skill/bb2016/titchy.rs` | ✓ |
| `skill/bb2016/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2016/very_long_legs.rs` | ✓ |
| `skill/bb2016/WeepingDagger.java` | `ffb-model` | `src/skill/bb2016/weeping_dagger.rs` | ✓ |
| `skill/bb2016/WildAnimal.java` | `ffb-model` | `src/skill/bb2016/wild_animal.rs` | ✓ |
| `skill/bb2020/Animosity.java` | `ffb-model` | `src/skill/bb2020/animosity.rs` | ✓ |
| `skill/bb2020/BallAndChain.java` | `ffb-model` | `src/skill/bb2020/ball_and_chain.rs` | ✓ |
| `skill/bb2020/Bombardier.java` | `ffb-model` | `src/skill/bb2020/bombardier.rs` | ✓ |
| `skill/bb2020/BoneHead.java` | `ffb-model` | `src/skill/bb2020/bone_head.rs` | ✓ |
| `skill/bb2020/Brawler.java` | `ffb-model` | `src/skill/bb2020/brawler.rs` | ✓ |
| `skill/bb2020/BreakTackle.java` | `ffb-model` | `src/skill/bb2020/break_tackle.rs` | ✓ |
| `skill/bb2020/BreatheFire.java` | `ffb-model` | `src/skill/bb2020/breathe_fire.rs` | ✓ |
| `skill/bb2020/Chainsaw.java` | `ffb-model` | `src/skill/bb2020/chainsaw.rs` | ✓ |
| `skill/bb2020/CloudBurster.java` | `ffb-model` | `src/skill/bb2020/cloud_burster.rs` | ✓ |
| `skill/bb2020/Defensive.java` | `ffb-model` | `src/skill/bb2020/defensive.rs` | ✓ |
| `skill/bb2020/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2020/dirty_player.rs` | ✓ |
| `skill/bb2020/Fumblerooskie.java` | `ffb-model` | `src/skill/bb2020/fumblerooskie.rs` | ✓ |
| `skill/bb2020/HitAndRun.java` | `ffb-model` | `src/skill/bb2020/hit_and_run.rs` | ✓ |
| `skill/bb2020/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2020/hypnotic_gaze.rs` | ✓ |
| `skill/bb2020/Leap.java` | `ffb-model` | `src/skill/bb2020/leap.rs` | ✓ |
| `skill/bb2020/MightyBlow.java` | `ffb-model` | `src/skill/bb2020/mighty_blow.rs` | ✓ |
| `skill/bb2020/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2020/monstrous_mouth.rs` | ✓ |
| `skill/bb2020/NoHands.java` | `ffb-model` | `src/skill/bb2020/no_hands.rs` | ✓ |
| `skill/bb2020/PassingIncrease.java` | `ffb-model` | `src/skill/bb2020/passing_increase.rs` | ✓ |
| `skill/bb2020/PileDriver.java` | `ffb-model` | `src/skill/bb2020/pile_driver.rs` | ✓ |
| `skill/bb2020/PilingOn.java` | `ffb-model` | `src/skill/bb2020/piling_on.rs` | ✓ |
| `skill/bb2020/PogoStick.java` | `ffb-model` | `src/skill/bb2020/pogo_stick.rs` | ✓ |
| `skill/bb2020/ProjectileVomit.java` | `ffb-model` | `src/skill/bb2020/projectile_vomit.rs` | ✓ |
| `skill/bb2020/ReallyStupid.java` | `ffb-model` | `src/skill/bb2020/really_stupid.rs` | ✓ |
| `skill/bb2020/Regeneration.java` | `ffb-model` | `src/skill/bb2020/regeneration.rs` | ✓ |
| `skill/bb2020/RightStuff.java` | `ffb-model` | `src/skill/bb2020/right_stuff.rs` | ✓ |
| `skill/bb2020/RunningPass.java` | `ffb-model` | `src/skill/bb2020/running_pass.rs` | ✓ |
| `skill/bb2020/Shadowing.java` | `ffb-model` | `src/skill/bb2020/shadowing.rs` | ✓ |
| `skill/bb2020/SideStep.java` | `ffb-model` | `src/skill/bb2020/side_step.rs` | ✓ |
| `skill/bb2020/SneakyGit.java` | `ffb-model` | `src/skill/bb2020/sneaky_git.rs` | ✓ |
| `skill/bb2020/special/ASneakyPair.java` | `ffb-model` | `src/skill/bb2020/special/a_sneaky_pair.rs` | ✓ |
| `skill/bb2020/special/BlastIt.java` | `ffb-model` | `src/skill/bb2020/special/blast_it.rs` | ✓ |
| `skill/bb2020/special/BrutalBlock.java` | `ffb-model` | `src/skill/bb2020/special/brutal_block.rs` | ✓ |
| `skill/bb2020/special/BurstOfSpeed.java` | `ffb-model` | `src/skill/bb2020/special/burst_of_speed.rs` | ✓ |
| `skill/bb2020/special/ConsummateProfessional.java` | `ffb-model` | `src/skill/bb2020/special/consummate_professional.rs` | ✓ |
| `skill/bb2020/special/DwarfenScourge.java` | `ffb-model` | `src/skill/bb2020/special/dwarfen_scourge.rs` | ✓ |
| `skill/bb2020/special/ExcuseMeAreYouAZoat.java` | `ffb-model` | `src/skill/bb2020/special/excuse_me_are_you_a_zoat.rs` | ✓ |
| `skill/bb2020/special/FrenziedRush.java` | `ffb-model` | `src/skill/bb2020/special/frenzied_rush.rs` | ✓ |
| `skill/bb2020/special/GhostlyFlames.java` | `ffb-model` | `src/skill/bb2020/special/ghostly_flames.rs` | ✓ |
| `skill/bb2020/special/Incorporeal.java` | `ffb-model` | `src/skill/bb2020/special/incorporeal.rs` | ✓ |
| `skill/bb2020/special/LordOfChaos.java` | `ffb-model` | `src/skill/bb2020/special/lord_of_chaos.rs` | ✓ |
| `skill/bb2020/special/MasterAssassin.java` | `ffb-model` | `src/skill/bb2020/special/master_assassin.rs` | ✓ |
| `skill/bb2020/special/MesmerizingDance.java` | `ffb-model` | `src/skill/bb2020/special/mesmerizing_dance.rs` | ✓ |
| `skill/bb2020/special/PumpUpTheCrowd.java` | `ffb-model` | `src/skill/bb2020/special/pump_up_the_crowd.rs` | ✓ |
| `skill/bb2020/special/PutridRegurgitation.java` | `ffb-model` | `src/skill/bb2020/special/putrid_regurgitation.rs` | ✓ |
| `skill/bb2020/special/TheBallista.java` | `ffb-model` | `src/skill/bb2020/special/the_ballista.rs` | ✓ |
| `skill/bb2020/special/ThenIStartedBlastin.java` | `ffb-model` | `src/skill/bb2020/special/then_i_started_blastin.rs` | ✓ |
| `skill/bb2020/special/TwoForOne.java` | `ffb-model` | `src/skill/bb2020/special/two_for_one.rs` | ✓ |
| `skill/bb2020/special/WhirlingDervish.java` | `ffb-model` | `src/skill/bb2020/special/whirling_dervish.rs` | ✓ |
| `skill/bb2020/special/WisdomOfTheWhiteDwarf.java` | `ffb-model` | `src/skill/bb2020/special/wisdom_of_the_white_dwarf.rs` | ✓ |
| `skill/bb2020/Stab.java` | `ffb-model` | `src/skill/bb2020/stab.rs` | ✓ |
| `skill/bb2020/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2020/strength_increase.rs` | ✓ |
| `skill/bb2020/SureFeet.java` | `ffb-model` | `src/skill/bb2020/sure_feet.rs` | ✓ |
| `skill/bb2020/Swarming.java` | `ffb-model` | `src/skill/bb2020/swarming.rs` | ✓ |
| `skill/bb2020/Swoop.java` | `ffb-model` | `src/skill/bb2020/swoop.rs` | ✓ |
| `skill/bb2020/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2020/very_long_legs.rs` | ✓ |
| `skill/bb2025/AgilityIncrease.java` | `ffb-model` | `src/skill/bb2025/agility_increase.rs` | ✓ |
| `skill/bb2025/Animosity.java` | `ffb-model` | `src/skill/bb2025/animosity.rs` | ✓ |
| `skill/bb2025/BallAndChain.java` | `ffb-model` | `src/skill/bb2025/ball_and_chain.rs` | ✓ |
| `skill/bb2025/BigHand.java` | `ffb-model` | `src/skill/bb2025/big_hand.rs` | ✓ |
| `skill/bb2025/Bombardier.java` | `ffb-model` | `src/skill/bb2025/bombardier.rs` | ✓ |
| `skill/bb2025/BoneHead.java` | `ffb-model` | `src/skill/bb2025/bone_head.rs` | ✓ |
| `skill/bb2025/Brawler.java` | `ffb-model` | `src/skill/bb2025/brawler.rs` | ✓ |
| `skill/bb2025/BreakTackle.java` | `ffb-model` | `src/skill/bb2025/break_tackle.rs` | ✓ |
| `skill/bb2025/BreatheFire.java` | `ffb-model` | `src/skill/bb2025/breathe_fire.rs` | ✓ |
| `skill/bb2025/Bullseye.java` | `ffb-model` | `src/skill/bb2025/bullseye.rs` | ✓ |
| `skill/bb2025/Chainsaw.java` | `ffb-model` | `src/skill/bb2025/chainsaw.rs` | ✓ |
| `skill/bb2025/CloudBurster.java` | `ffb-model` | `src/skill/bb2025/cloud_burster.rs` | ✓ |
| `skill/bb2025/Defensive.java` | `ffb-model` | `src/skill/bb2025/defensive.rs` | ✓ |
| `skill/bb2025/DirtyPlayer.java` | `ffb-model` | `src/skill/bb2025/dirty_player.rs` | ✓ |
| `skill/bb2025/Dodge.java` | `ffb-model` | `src/skill/bb2025/dodge.rs` | ✓ |
| `skill/bb2025/EyeGouge.java` | `ffb-model` | `src/skill/bb2025/eye_gouge.rs` | ✓ |
| `skill/bb2025/Fumblerooski.java` | `ffb-model` | `src/skill/bb2025/fumblerooski.rs` | ✓ |
| `skill/bb2025/GiveAndGo.java` | `ffb-model` | `src/skill/bb2025/give_and_go.rs` | ✓ |
| `skill/bb2025/Hatred.java` | `ffb-model` | `src/skill/bb2025/hatred.rs` | ✓ |
| `skill/bb2025/HitAndRun.java` | `ffb-model` | `src/skill/bb2025/hit_and_run.rs` | ✓ |
| `skill/bb2025/HypnoticGaze.java` | `ffb-model` | `src/skill/bb2025/hypnotic_gaze.rs` | ✓ |
| `skill/bb2025/Insignificant.java` | `ffb-model` | `src/skill/bb2025/insignificant.rs` | ✓ |
| `skill/bb2025/Juggernaut.java` | `ffb-model` | `src/skill/bb2025/juggernaut.rs` | ✓ |
| `skill/bb2025/Kick.java` | `ffb-model` | `src/skill/bb2025/kick.rs` | ✓ |
| `skill/bb2025/Leader.java` | `ffb-model` | `src/skill/bb2025/leader.rs` | ✓ |
| `skill/bb2025/Leap.java` | `ffb-model` | `src/skill/bb2025/leap.rs` | ✓ |
| `skill/bb2025/LethalFlight.java` | `ffb-model` | `src/skill/bb2025/lethal_flight.rs` | ✓ |
| `skill/bb2025/LoneFouler.java` | `ffb-model` | `src/skill/bb2025/lone_fouler.rs` | ✓ |
| `skill/bb2025/MightyBlow.java` | `ffb-model` | `src/skill/bb2025/mighty_blow.rs` | ✓ |
| `skill/bb2025/MonstrousMouth.java` | `ffb-model` | `src/skill/bb2025/monstrous_mouth.rs` | ✓ |
| `skill/bb2025/NoBall.java` | `ffb-model` | `src/skill/bb2025/no_ball.rs` | ✓ |
| `skill/bb2025/PassingIncrease.java` | `ffb-model` | `src/skill/bb2025/passing_increase.rs` | ✓ |
| `skill/bb2025/PileDriver.java` | `ffb-model` | `src/skill/bb2025/pile_driver.rs` | ✓ |
| `skill/bb2025/Pogo.java` | `ffb-model` | `src/skill/bb2025/pogo.rs` | ✓ |
| `skill/bb2025/Pro.java` | `ffb-model` | `src/skill/bb2025/pro.rs` | ✓ |
| `skill/bb2025/ProjectileVomit.java` | `ffb-model` | `src/skill/bb2025/projectile_vomit.rs` | ✓ |
| `skill/bb2025/Punt.java` | `ffb-model` | `src/skill/bb2025/punt.rs` | ✓ |
| `skill/bb2025/PutTheBootIn.java` | `ffb-model` | `src/skill/bb2025/put_the_boot_in.rs` | ✓ |
| `skill/bb2025/QuickFoul.java` | `ffb-model` | `src/skill/bb2025/quick_foul.rs` | ✓ |
| `skill/bb2025/ReallyStupid.java` | `ffb-model` | `src/skill/bb2025/really_stupid.rs` | ✓ |
| `skill/bb2025/Regeneration.java` | `ffb-model` | `src/skill/bb2025/regeneration.rs` | ✓ |
| `skill/bb2025/RightStuff.java` | `ffb-model` | `src/skill/bb2025/right_stuff.rs` | ✓ |
| `skill/bb2025/Saboteur.java` | `ffb-model` | `src/skill/bb2025/saboteur.rs` | ✓ |
| `skill/bb2025/Shadowing.java` | `ffb-model` | `src/skill/bb2025/shadowing.rs` | ✓ |
| `skill/bb2025/Sidestep.java` | `ffb-model` | `src/skill/bb2025/sidestep.rs` | ✓ |
| `skill/bb2025/SneakyGit.java` | `ffb-model` | `src/skill/bb2025/sneaky_git.rs` | ✓ |
| `skill/bb2025/special/ASneakyPair.java` | `ffb-model` | `src/skill/bb2025/special/a_sneaky_pair.rs` | ✓ |
| `skill/bb2025/special/BlastinSolvesEverything.java` | `ffb-model` | `src/skill/bb2025/special/blastin_solves_everything.rs` | ✓ |
| `skill/bb2025/special/BlastIt.java` | `ffb-model` | `src/skill/bb2025/special/blast_it.rs` | ✓ |
| `skill/bb2025/special/DwarvenScourge.java` | `ffb-model` | `src/skill/bb2025/special/dwarven_scourge.rs` | ✓ |
| `skill/bb2025/special/ExcuseMeAreYouAZoat.java` | `ffb-model` | `src/skill/bb2025/special/excuse_me_are_you_a_zoat.rs` | ✓ |
| `skill/bb2025/special/FrenziedRush.java` | `ffb-model` | `src/skill/bb2025/special/frenzied_rush.rs` | ✓ |
| `skill/bb2025/special/Incorporeal.java` | `ffb-model` | `src/skill/bb2025/special/incorporeal.rs` | ✓ |
| `skill/bb2025/special/KrumpAndSmash.java` | `ffb-model` | `src/skill/bb2025/special/krump_and_smash.rs` | ✓ |
| `skill/bb2025/special/LordOfChaos.java` | `ffb-model` | `src/skill/bb2025/special/lord_of_chaos.rs` | ✓ |
| `skill/bb2025/special/MasterAssassin.java` | `ffb-model` | `src/skill/bb2025/special/master_assassin.rs` | ✓ |
| `skill/bb2025/special/MesmerisingDance.java` | `ffb-model` | `src/skill/bb2025/special/mesmerising_dance.rs` | ✓ |
| `skill/bb2025/special/PumpUpTheCrowd.java` | `ffb-model` | `src/skill/bb2025/special/pump_up_the_crowd.rs` | ✓ |
| `skill/bb2025/special/PutridRegurgitation.java` | `ffb-model` | `src/skill/bb2025/special/putrid_regurgitation.rs` | ✓ |
| `skill/bb2025/special/SlashingNails.java` | `ffb-model` | `src/skill/bb2025/special/slashing_nails.rs` | ✓ |
| `skill/bb2025/special/TeamCaptain.java` | `ffb-model` | `src/skill/bb2025/special/team_captain.rs` | ✓ |
| `skill/bb2025/special/TheBallista.java` | `ffb-model` | `src/skill/bb2025/special/the_ballista.rs` | ✓ |
| `skill/bb2025/special/WhirlingDervish.java` | `ffb-model` | `src/skill/bb2025/special/whirling_dervish.rs` | ✓ |
| `skill/bb2025/special/WisdomOfTheWhiteDwarf.java` | `ffb-model` | `src/skill/bb2025/special/wisdom_of_the_white_dwarf.rs` | ✓ |
| `skill/bb2025/special/WoodlandFury.java` | `ffb-model` | `src/skill/bb2025/special/woodland_fury.rs` | ✓ |
| `skill/bb2025/special/WorkingInTandem.java` | `ffb-model` | `src/skill/bb2025/special/working_in_tandem.rs` | ✓ |
| `skill/bb2025/Stab.java` | `ffb-model` | `src/skill/bb2025/stab.rs` | ✓ |
| `skill/bb2025/SteadyFooting.java` | `ffb-model` | `src/skill/bb2025/steady_footing.rs` | ✓ |
| `skill/bb2025/StrengthIncrease.java` | `ffb-model` | `src/skill/bb2025/strength_increase.rs` | ✓ |
| `skill/bb2025/SureFeet.java` | `ffb-model` | `src/skill/bb2025/sure_feet.rs` | ✓ |
| `skill/bb2025/Swoop.java` | `ffb-model` | `src/skill/bb2025/swoop.rs` | ✓ |
| `skill/bb2025/Taunt.java` | `ffb-model` | `src/skill/bb2025/taunt.rs` | ✓ |
| `skill/bb2025/Unsteady.java` | `ffb-model` | `src/skill/bb2025/unsteady.rs` | ✓ |
| `skill/bb2025/VeryLongLegs.java` | `ffb-model` | `src/skill/bb2025/very_long_legs.rs` | ✓ |
| `skill/bb2025/ViolentInnovator.java` | `ffb-model` | `src/skill/bb2025/violent_innovator.rs` | ✓ |
| `skill/common/Block.java` | `ffb-model` | `src/skill/common/block.rs` | ✓ |
| `skill/common/Catch.java` | `ffb-model` | `src/skill/common/catch.rs` | ✓ |
| `skill/common/Dauntless.java` | `ffb-model` | `src/skill/common/dauntless.rs` | ✓ |
| `skill/common/DisturbingPresence.java` | `ffb-model` | `src/skill/common/disturbing_presence.rs` | ✓ |
| `skill/common/DivingCatch.java` | `ffb-model` | `src/skill/common/diving_catch.rs` | ✓ |
| `skill/common/DumpOff.java` | `ffb-model` | `src/skill/common/dump_off.rs` | ✓ |
| `skill/common/ExtraArms.java` | `ffb-model` | `src/skill/common/extra_arms.rs` | ✓ |
| `skill/common/Fend.java` | `ffb-model` | `src/skill/common/fend.rs` | ✓ |
| `skill/common/FoulAppearance.java` | `ffb-model` | `src/skill/common/foul_appearance.rs` | ✓ |
| `skill/common/HailMaryPass.java` | `ffb-model` | `src/skill/common/hail_mary_pass.rs` | ✓ |
| `skill/common/Horns.java` | `ffb-model` | `src/skill/common/horns.rs` | ✓ |
| `skill/common/JumpUp.java` | `ffb-model` | `src/skill/common/jump_up.rs` | ✓ |
| `skill/common/MovementIncrease.java` | `ffb-model` | `src/skill/common/movement_increase.rs` | ✓ |
| `skill/common/Pass.java` | `ffb-model` | `src/skill/common/pass.rs` | ✓ |
| `skill/common/Sprint.java` | `ffb-model` | `src/skill/common/sprint.rs` | ✓ |
| `skill/common/StandFirm.java` | `ffb-model` | `src/skill/common/stand_firm.rs` | ✓ |
| `skill/common/StripBall.java` | `ffb-model` | `src/skill/common/strip_ball.rs` | ✓ |
| `skill/common/SureHands.java` | `ffb-model` | `src/skill/common/sure_hands.rs` | ✓ |
| `skill/common/Tackle.java` | `ffb-model` | `src/skill/common/tackle.rs` | ✓ |
| `skill/common/Tentacles.java` | `ffb-model` | `src/skill/common/tentacles.rs` | ✓ |
| `skill/common/ThickSkull.java` | `ffb-model` | `src/skill/common/thick_skull.rs` | ✓ |
| `skill/common/TwoHeads.java` | `ffb-model` | `src/skill/common/two_heads.rs` | ✓ |
| `skill/common/Wrestle.java` | `ffb-model` | `src/skill/common/wrestle.rs` | ✓ |
| `skill/mixed/Accurate.java` | `ffb-model` | `src/skill/mixed/accurate.rs` | ✓ |
| `skill/mixed/AgilityIncrease.java` | `ffb-model` | `src/skill/mixed/agility_increase.rs` | ✓ |
| `skill/mixed/AlwaysHungry.java` | `ffb-model` | `src/skill/mixed/always_hungry.rs` | ✓ |
| `skill/mixed/AnimalSavagery.java` | `ffb-model` | `src/skill/mixed/animal_savagery.rs` | ✓ |
| `skill/mixed/ArmBar.java` | `ffb-model` | `src/skill/mixed/arm_bar.rs` | ✓ |
| `skill/mixed/ArmourIncrease.java` | `ffb-model` | `src/skill/mixed/armour_increase.rs` | ✓ |
| `skill/mixed/BigHand.java` | `ffb-model` | `src/skill/mixed/big_hand.rs` | ✓ |
| `skill/mixed/Bloodlust.java` | `ffb-model` | `src/skill/mixed/bloodlust.rs` | ✓ |
| `skill/mixed/Cannoneer.java` | `ffb-model` | `src/skill/mixed/cannoneer.rs` | ✓ |
| `skill/mixed/Claws.java` | `ffb-model` | `src/skill/mixed/claws.rs` | ✓ |
| `skill/mixed/Decay.java` | `ffb-model` | `src/skill/mixed/decay.rs` | ✓ |
| `skill/mixed/DivingTackle.java` | `ffb-model` | `src/skill/mixed/diving_tackle.rs` | ✓ |
| `skill/mixed/Dodge.java` | `ffb-model` | `src/skill/mixed/dodge.rs` | ✓ |
| `skill/mixed/Drunkard.java` | `ffb-model` | `src/skill/mixed/drunkard.rs` | ✓ |
| `skill/mixed/Frenzy.java` | `ffb-model` | `src/skill/mixed/frenzy.rs` | ✓ |
| `skill/mixed/Grab.java` | `ffb-model` | `src/skill/mixed/grab.rs` | ✓ |
| `skill/mixed/Guard.java` | `ffb-model` | `src/skill/mixed/guard.rs` | ✓ |
| `skill/mixed/IronHardSkin.java` | `ffb-model` | `src/skill/mixed/iron_hard_skin.rs` | ✓ |
| `skill/mixed/Juggernaut.java` | `ffb-model` | `src/skill/mixed/juggernaut.rs` | ✓ |
| `skill/mixed/Kick.java` | `ffb-model` | `src/skill/mixed/kick.rs` | ✓ |
| `skill/mixed/KickTeamMate.java` | `ffb-model` | `src/skill/mixed/kick_team_mate.rs` | ✓ |
| `skill/mixed/Leader.java` | `ffb-model` | `src/skill/mixed/leader.rs` | ✓ |
| `skill/mixed/Loner.java` | `ffb-model` | `src/skill/mixed/loner.rs` | ✓ |
| `skill/mixed/MultipleBlock.java` | `ffb-model` | `src/skill/mixed/multiple_block.rs` | ✓ |
| `skill/mixed/MyBall.java` | `ffb-model` | `src/skill/mixed/my_ball.rs` | ✓ |
| `skill/mixed/NervesOfSteel.java` | `ffb-model` | `src/skill/mixed/nerves_of_steel.rs` | ✓ |
| `skill/mixed/OnTheBall.java` | `ffb-model` | `src/skill/mixed/on_the_ball.rs` | ✓ |
| `skill/mixed/PickMeUp.java` | `ffb-model` | `src/skill/mixed/pick_me_up.rs` | ✓ |
| `skill/mixed/PlagueRidden.java` | `ffb-model` | `src/skill/mixed/plague_ridden.rs` | ✓ |
| `skill/mixed/PrehensileTail.java` | `ffb-model` | `src/skill/mixed/prehensile_tail.rs` | ✓ |
| `skill/mixed/Pro.java` | `ffb-model` | `src/skill/mixed/pro.rs` | ✓ |
| `skill/mixed/SafePairOfHands.java` | `ffb-model` | `src/skill/mixed/safe_pair_of_hands.rs` | ✓ |
| `skill/mixed/SafePass.java` | `ffb-model` | `src/skill/mixed/safe_pass.rs` | ✓ |
| `skill/mixed/SecretWeapon.java` | `ffb-model` | `src/skill/mixed/secret_weapon.rs` | ✓ |
| `skill/mixed/special/AllYouCanEat.java` | `ffb-model` | `src/skill/mixed/special/all_you_can_eat.rs` | ✓ |
| `skill/mixed/special/BalefulHex.java` | `ffb-model` | `src/skill/mixed/special/baleful_hex.rs` | ✓ |
| `skill/mixed/special/BeerBarrelBash.java` | `ffb-model` | `src/skill/mixed/special/beer_barrel_bash.rs` | ✓ |
| `skill/mixed/special/BlackInk.java` | `ffb-model` | `src/skill/mixed/special/black_ink.rs` | ✓ |
| `skill/mixed/special/BlindRage.java` | `ffb-model` | `src/skill/mixed/special/blind_rage.rs` | ✓ |
| `skill/mixed/special/BoundingLeap.java` | `ffb-model` | `src/skill/mixed/special/bounding_leap.rs` | ✓ |
| `skill/mixed/special/BugmansXXXXXX.java` | `ffb-model` | `src/skill/mixed/special/bugmans_xxxxxx.rs` | ✓ |
| `skill/mixed/special/CatchOfTheDay.java` | `ffb-model` | `src/skill/mixed/special/catch_of_the_day.rs` | ✓ |
| `skill/mixed/special/CrushingBlow.java` | `ffb-model` | `src/skill/mixed/special/crushing_blow.rs` | ✓ |
| `skill/mixed/special/FuriousOutburst.java` | `ffb-model` | `src/skill/mixed/special/furious_outburst.rs` | ✓ |
| `skill/mixed/special/FuryOfTheBloodGod.java` | `ffb-model` | `src/skill/mixed/special/fury_of_the_blood_god.rs` | ✓ |
| `skill/mixed/special/GoredByTheBull.java` | `ffb-model` | `src/skill/mixed/special/gored_by_the_bull.rs` | ✓ |
| `skill/mixed/special/HalflingLuck.java` | `ffb-model` | `src/skill/mixed/special/halfling_luck.rs` | ✓ |
| `skill/mixed/special/IllBeBack.java` | `ffb-model` | `src/skill/mixed/special/ill_be_back.rs` | ✓ |
| `skill/mixed/special/Indomitable.java` | `ffb-model` | `src/skill/mixed/special/indomitable.rs` | ✓ |
| `skill/mixed/special/Kaboom.java` | `ffb-model` | `src/skill/mixed/special/kaboom.rs` | ✓ |
| `skill/mixed/special/KeenPlayer.java` | `ffb-model` | `src/skill/mixed/special/keen_player.rs` | ✓ |
| `skill/mixed/special/KickEmWhileTheyReDown.java` | `ffb-model` | `src/skill/mixed/special/kick_em_while_they_re_down.rs` | ✓ |
| `skill/mixed/special/LookIntoMyEyes.java` | `ffb-model` | `src/skill/mixed/special/look_into_my_eyes.rs` | ✓ |
| `skill/mixed/special/MaximumCarnage.java` | `ffb-model` | `src/skill/mixed/special/maximum_carnage.rs` | ✓ |
| `skill/mixed/special/OldPro.java` | `ffb-model` | `src/skill/mixed/special/old_pro.rs` | ✓ |
| `skill/mixed/special/PrimalSavagery.java` | `ffb-model` | `src/skill/mixed/special/primal_savagery.rs` | ✓ |
| `skill/mixed/special/QuickBite.java` | `ffb-model` | `src/skill/mixed/special/quick_bite.rs` | ✓ |
| `skill/mixed/special/RaidingParty.java` | `ffb-model` | `src/skill/mixed/special/raiding_party.rs` | ✓ |
| `skill/mixed/special/Ram.java` | `ffb-model` | `src/skill/mixed/special/ram.rs` | ✓ |
| `skill/mixed/special/Reliable.java` | `ffb-model` | `src/skill/mixed/special/reliable.rs` | ✓ |
| `skill/mixed/special/SavageBlow.java` | `ffb-model` | `src/skill/mixed/special/savage_blow.rs` | ✓ |
| `skill/mixed/special/SavageMauling.java` | `ffb-model` | `src/skill/mixed/special/savage_mauling.rs` | ✓ |
| `skill/mixed/special/ShotToNothing.java` | `ffb-model` | `src/skill/mixed/special/shot_to_nothing.rs` | ✓ |
| `skill/mixed/special/Slayer.java` | `ffb-model` | `src/skill/mixed/special/slayer.rs` | ✓ |
| `skill/mixed/special/SneakiestOfTheLot.java` | `ffb-model` | `src/skill/mixed/special/sneakiest_of_the_lot.rs` | ✓ |
| `skill/mixed/special/StarOfTheShow.java` | `ffb-model` | `src/skill/mixed/special/star_of_the_show.rs` | ✓ |
| `skill/mixed/special/StrongPassingGame.java` | `ffb-model` | `src/skill/mixed/special/strong_passing_game.rs` | ✓ |
| `skill/mixed/special/SwiftAsTheBreeze.java` | `ffb-model` | `src/skill/mixed/special/swift_as_the_breeze.rs` | ✓ |
| `skill/mixed/special/TastyMorsel.java` | `ffb-model` | `src/skill/mixed/special/tasty_morsel.rs` | ✓ |
| `skill/mixed/special/TheFlashingBlade.java` | `ffb-model` | `src/skill/mixed/special/the_flashing_blade.rs` | ✓ |
| `skill/mixed/special/ThinkingMansTroll.java` | `ffb-model` | `src/skill/mixed/special/thinking_mans_troll.rs` | ✓ |
| `skill/mixed/special/ToxinConnoisseur.java` | `ffb-model` | `src/skill/mixed/special/toxin_connoisseur.rs` | ✓ |
| `skill/mixed/special/Treacherous.java` | `ffb-model` | `src/skill/mixed/special/treacherous.rs` | ✓ |
| `skill/mixed/special/UnstoppableMomentum.java` | `ffb-model` | `src/skill/mixed/special/unstoppable_momentum.rs` | ✓ |
| `skill/mixed/special/ViciousVines.java` | `ffb-model` | `src/skill/mixed/special/vicious_vines.rs` | ✓ |
| `skill/mixed/special/WatchOut.java` | `ffb-model` | `src/skill/mixed/special/watch_out.rs` | ✓ |
| `skill/mixed/special/Yoink.java` | `ffb-model` | `src/skill/mixed/special/yoink.rs` | ✓ |
| `skill/mixed/StrongArm.java` | `ffb-model` | `src/skill/mixed/strong_arm.rs` | ✓ |
| `skill/mixed/Stunty.java` | `ffb-model` | `src/skill/mixed/stunty.rs` | ✓ |
| `skill/mixed/TakeRoot.java` | `ffb-model` | `src/skill/mixed/take_root.rs` | ✓ |
| `skill/mixed/ThrowTeamMate.java` | `ffb-model` | `src/skill/mixed/throw_team_mate.rs` | ✓ |
| `skill/mixed/Timmmber.java` | `ffb-model` | `src/skill/mixed/timmmber.rs` | ✓ |
| `skill/mixed/Titchy.java` | `ffb-model` | `src/skill/mixed/titchy.rs` | ✓ |
| `skill/mixed/Trickster.java` | `ffb-model` | `src/skill/mixed/trickster.rs` | ✓ |
| `skill/mixed/UnchannelledFury.java` | `ffb-model` | `src/skill/mixed/unchannelled_fury.rs` | ✓ |
| `skill/StrengthIncrease.java` | `ffb-model` | `src/skill/strength_increase.rs` | ✓ |

### stats/ (6 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `stats/DicePoolStat.java` | `—` | `—` | — |
| `stats/DieBase.java` | `—` | `—` | — |
| `stats/DieStat.java` | `—` | `—` | — |
| `stats/DoubleDiceStat.java` | `—` | `—` | — |
| `stats/SingleDieStat.java` | `—` | `—` | — |
| `stats/TeamMapping.java` | `—` | `—` | — |

### util/ (27 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `util/ArrayTool.java` | `ffb-model` | `src/util/array_tool.rs` | ✓ |
| `util/DateTool.java` | `ffb-model` | `src/util/date_tool.rs` | ✓ |
| `util/FileIterator.java` | `ffb-model` | `src/util/file_iterator.rs` | ✓ |
| `util/ListTool.java` | `ffb-model` | `src/util/list_tool.rs` | ✓ |
| `util/NaturalOrderComparator.java` | `ffb-model` | `src/util/natural_order_comparator.rs` | ✓ |
| `util/pathfinding/PathFindContext.java` | `ffb-model` | `src/util/pathfinding/path_find_context.rs` | ✓ |
| `util/pathfinding/PathFindData.java` | `ffb-model` | `src/util/pathfinding/path_find_data.rs` | ✓ |
| `util/pathfinding/PathFinderExtension.java` | `ffb-model` | `src/util/pathfinding/path_finder_extension.rs` | ✓ |
| `util/pathfinding/PathFinderWithMultiJump.java` | `ffb-model` | `src/util/pathfinding/path_finder_with_multi_jump.rs` | ✓ |
| `util/pathfinding/PathFinderWithPassBlockSupport.java` | `ffb-model` | `src/util/pathfinding/path_finder_with_pass_block_support.rs` | ✓ |
| `util/pathfinding/PathFindNode.java` | `ffb-model` | `src/util/pathfinding/path_find_node.rs` | ✓ |
| `util/pathfinding/PathFindState.java` | `ffb-model` | `src/util/pathfinding/path_find_state.rs` | ✓ |
| `util/RaiseType.java` | `ffb-model` | `src/util/raise_type.rs` | ✓ |
| `util/RawScanner.java` | `ffb-model` | `src/util/raw_scanner.rs` | ✓ |
| `util/rng/EntropySource.java` | `ffb-model` | `src/util/rng/entropy_source.rs` | ✓ |
| `util/Scanner.java` | `ffb-model` | `src/util/scanner.rs` | ✓ |
| `util/ScannerSingleton.java` | `ffb-model` | `src/util/scanner_singleton.rs` | ✓ |
| `util/StringTool.java` | `ffb-model` | `src/util/string_tool.rs` | ✓ |
| `util/UtilActingPlayer.java` | `ffb-model` | `src/util/util_acting_player.rs` | ✓ |
| `util/UtilBox.java` | `ffb-model` | `src/util/util_box.rs` | ✓ |
| `util/UtilCards.java` | `ffb-model` | `src/util/util_cards.rs` | ✓ |
| `util/UtilDisturbingPresence.java` | `ffb-model` | `src/util/util_disturbing_presence.rs` | ✓ |
| `util/UtilPassing.java` | `ffb-model` | `src/util/util_passing.rs` | ✓ |
| `util/UtilPlayer.java` | `ffb-model` | `src/util/util_player.rs` | ✓ |
| `util/UtilRangeRuler.java` | `ffb-model` | `src/util/util_range_ruler.rs` | ✓ |
| `util/UtilTeamValue.java` | `ffb-model` | `src/util/util_team_value.rs` | ✓ |
| `util/UtilUrl.java` | `ffb-model` | `src/util/util_url.rs` | ✓ |

### xml/ (5 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `xml/IXmlReadable.java` | `ffb-model` | `src/xml/i_xml_readable.rs` | ✓ |
| `xml/IXmlSerializable.java` | `—` | `—` | — |
| `xml/IXmlWriteable.java` | `—` | `—` | — |
| `xml/UtilXml.java` | `ffb-model` | `src/xml/util_xml.rs` | ✓ |
| `xml/XmlHandler.java` | `ffb-model` | `src/xml/xml_handler.rs` | ✓ |

## Module: ffb-server

### server/admin/ (9 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/admin/AdminConnector.java` | `ffb-server` | `src/admin/admin_connector.rs` | ✓ |
| `server/admin/AdminList.java` | `ffb-server` | `src/admin/admin_list.rs` | ✓ |
| `server/admin/AdminListEntry.java` | `ffb-server` | `src/admin/admin_list_entry.rs` | ✓ |
| `server/admin/AdminServlet.java` | `ffb-server` | `src/admin/admin_servlet.rs` | ✓ |
| `server/admin/BackupServlet.java` | `ffb-server` | `src/admin/backup_servlet.rs` | ✓ |
| `server/admin/GameStateConnector.java` | `ffb-server` | `src/admin/game_state_connector.rs` | ✓ |
| `server/admin/GameStateService.java` | `ffb-server` | `src/admin/game_state_service.rs` | ✓ |
| `server/admin/GameStateServlet.java` | `ffb-server` | `src/admin/game_state_servlet.rs` | ✓ |
| `server/admin/UtilBackup.java` | `ffb-server` | `src/admin/util_backup.rs` | ✓ |

### server/commandline/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/commandline/InifileParamFilter.java` | `ffb-server` | `src/commandline/inifile_param_filter.rs` | ✓ |
| `server/commandline/InifileParamFilterResult.java` | `ffb-server` | `src/commandline/inifile_param_filter_result.rs` | ✓ |

### server/db/ (55 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/db/DbConnectionManager.java` | `ffb-server` | `src/db/db_connection_manager.rs` | ✓ |
| `server/db/DbInitializer.java` | `ffb-server` | `src/db/db_initializer.rs` | ✓ |
| `server/db/DbQueryFactory.java` | `ffb-server` | `src/db/db_query_factory.rs` | ✓ |
| `server/db/DbStatement.java` | `ffb-server` | `src/db/db_statement.rs` | ✓ |
| `server/db/DbStatementId.java` | `ffb-server` | `src/db/db_statement_id.rs` | ✓ |
| `server/db/DbTransaction.java` | `ffb-server` | `src/db/db_transaction.rs` | ✓ |
| `server/db/DbUpdateFactory.java` | `ffb-server` | `src/db/db_update_factory.rs` | ✓ |
| `server/db/DbUpdateStatement.java` | `ffb-server` | `src/db/db_update_statement.rs` | ✓ |
| `server/db/DefaultDbUpdateParameter.java` | `ffb-server` | `src/db/default_db_update_parameter.rs` | ✓ |
| `server/db/delete/DbGamesInfoDelete.java` | `ffb-server` | `src/db/delete/db_games_info_delete.rs` | ✓ |
| `server/db/delete/DbGamesInfoDeleteParameter.java` | `ffb-server` | `src/db/delete/db_games_info_delete_parameter.rs` | ✓ |
| `server/db/delete/DbGamesSerializedDelete.java` | `ffb-server` | `src/db/delete/db_games_serialized_delete.rs` | ✓ |
| `server/db/delete/DbGamesSerializedDeleteParameter.java` | `ffb-server` | `src/db/delete/db_games_serialized_delete_parameter.rs` | ✓ |
| `server/db/delete/DbPlayerMarkersDelete.java` | `ffb-server` | `src/db/delete/db_player_markers_delete.rs` | ✓ |
| `server/db/delete/DbPlayerMarkersDeleteParameter.java` | `ffb-server` | `src/db/delete/db_player_markers_delete_parameter.rs` | ✓ |
| `server/db/delete/DbTeamSetupsDelete.java` | `ffb-server` | `src/db/delete/db_team_setups_delete.rs` | ✓ |
| `server/db/delete/DbTeamSetupsDeleteParameter.java` | `ffb-server` | `src/db/delete/db_team_setups_delete_parameter.rs` | ✓ |
| `server/db/delete/DbUserSettingsDelete.java` | `ffb-server` | `src/db/delete/db_user_settings_delete.rs` | ✓ |
| `server/db/delete/DbUserSettingsDeleteParameter.java` | `ffb-server` | `src/db/delete/db_user_settings_delete_parameter.rs` | ✓ |
| `server/db/delete/DefaultDbUpdateParameter.java` | `ffb-server` | `src/db/delete/default_db_update_parameter.rs` | ✓ |
| `server/db/IDbStatementFactory.java` | `ffb-server` | `src/db/i_db_statement_factory.rs` | ✓ |
| `server/db/IDbTableCoaches.java` | `ffb-server` | `src/db/i_db_table_coaches.rs` | ✓ |
| `server/db/IDbTableGamesInfo.java` | `ffb-server` | `src/db/i_db_table_games_info.rs` | ✓ |
| `server/db/IDbTableGamesSerialized.java` | `ffb-server` | `src/db/i_db_table_games_serialized.rs` | ✓ |
| `server/db/IDbTablePlayerMarkers.java` | `ffb-server` | `src/db/i_db_table_player_markers.rs` | ✓ |
| `server/db/IDbTableTeamSetups.java` | `ffb-server` | `src/db/i_db_table_team_setups.rs` | ✓ |
| `server/db/IDbTableUserSettings.java` | `ffb-server` | `src/db/i_db_table_user_settings.rs` | ✓ |
| `server/db/IDbUpdateParameter.java` | `ffb-server` | `src/db/i_db_update_parameter.rs` | ✓ |
| `server/db/IDbUpdateParameterList.java` | `ffb-server` | `src/db/i_db_update_parameter_list.rs` | ✓ |
| `server/db/IDbUpdateWithGameState.java` | `ffb-server` | `src/db/i_db_update_with_game_state.rs` | ✓ |
| `server/db/insert/DbGamesSerializedInsert.java` | `ffb-server` | `src/db/insert/db_games_serialized_insert.rs` | ✓ |
| `server/db/insert/DbGamesSerializedInsertParameter.java` | `ffb-server` | `src/db/insert/db_games_serialized_insert_parameter.rs` | ✓ |
| `server/db/insert/DbPlayerMarkersInsert.java` | `ffb-server` | `src/db/insert/db_player_markers_insert.rs` | ✓ |
| `server/db/insert/DbPlayerMarkersInsertParameter.java` | `ffb-server` | `src/db/insert/db_player_markers_insert_parameter.rs` | ✓ |
| `server/db/insert/DbPlayerMarkersInsertParameterList.java` | `ffb-server` | `src/db/insert/db_player_markers_insert_parameter_list.rs` | ✓ |
| `server/db/insert/DbTeamSetupsInsert.java` | `ffb-server` | `src/db/insert/db_team_setups_insert.rs` | ✓ |
| `server/db/insert/DbTeamSetupsInsertParameter.java` | `ffb-server` | `src/db/insert/db_team_setups_insert_parameter.rs` | ✓ |
| `server/db/insert/DbUserSettingsInsert.java` | `ffb-server` | `src/db/insert/db_user_settings_insert.rs` | ✓ |
| `server/db/insert/DbUserSettingsInsertParameter.java` | `ffb-server` | `src/db/insert/db_user_settings_insert_parameter.rs` | ✓ |
| `server/db/insert/DbUserSettingsInsertParameterList.java` | `ffb-server` | `src/db/insert/db_user_settings_insert_parameter_list.rs` | ✓ |
| `server/db/query/DbAdminListByIdQuery.java` | `ffb-server` | `src/db/query/db_admin_list_by_id_query.rs` | ✓ |
| `server/db/query/DbAdminListByStatusQuery.java` | `ffb-server` | `src/db/query/db_admin_list_by_status_query.rs` | ✓ |
| `server/db/query/DbGameListQueryOpenGamesByCoach.java` | `ffb-server` | `src/db/query/db_game_list_query_open_games_by_coach.rs` | ✓ |
| `server/db/query/DbGamesInfoInsertQuery.java` | `ffb-server` | `src/db/query/db_games_info_insert_query.rs` | ✓ |
| `server/db/query/DbGamesSerializedQuery.java` | `ffb-server` | `src/db/query/db_games_serialized_query.rs` | ✓ |
| `server/db/query/DbPasswordForCoachQuery.java` | `ffb-server` | `src/db/query/db_password_for_coach_query.rs` | ✓ |
| `server/db/query/DbPlayerMarkersQuery.java` | `ffb-server` | `src/db/query/db_player_markers_query.rs` | ✓ |
| `server/db/query/DbTeamSetupsForTeamQuery.java` | `ffb-server` | `src/db/query/db_team_setups_for_team_query.rs` | ✓ |
| `server/db/query/DbTeamSetupsQuery.java` | `ffb-server` | `src/db/query/db_team_setups_query.rs` | ✓ |
| `server/db/query/DbTestGameListQuery.java` | `ffb-server` | `src/db/query/db_test_game_list_query.rs` | ✓ |
| `server/db/query/DbUserSettingsQuery.java` | `ffb-server` | `src/db/query/db_user_settings_query.rs` | ✓ |
| `server/db/update/DbGamesInfoUpdate.java` | `ffb-server` | `src/db/update/db_games_info_update.rs` | ✓ |
| `server/db/update/DbGamesInfoUpdateParameter.java` | `ffb-server` | `src/db/update/db_games_info_update_parameter.rs` | ✓ |
| `server/db/update/DbGamesSerializedUpdate.java` | `ffb-server` | `src/db/update/db_games_serialized_update.rs` | ✓ |
| `server/db/update/DbGamesSerializedUpdateParameter.java` | `ffb-server` | `src/db/update/db_games_serialized_update_parameter.rs` | ✓ |

### server/factory/ (9 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/factory/bb2025/DeferredCommandFactory.java` | `ffb-engine` | `src/factory/bb2025/deferred_command_factory.rs` | ✓ |
| `server/factory/bb2025/DeferredCommandIdFactory.java` | `ffb-engine` | `src/factory/bb2025/deferred_command_id_factory.rs` | ✓ |
| `server/factory/CardHandlerFactory.java` | `ffb-engine` | `src/factory/card_handler_factory.rs` | ✓ |
| `server/factory/InjuryTypeServerFactory.java` | `ffb-engine` | `src/factory/injury_type_server_factory.rs` | ✓ |
| `server/factory/mixed/PrayerHandlerFactory.java` | `ffb-engine` | `src/factory/mixed/prayer_handler_factory.rs` | ✓ |
| `server/factory/ObserverFactory.java` | `ffb-engine` | `src/factory/observer_factory.rs` | ✓ |
| `server/factory/SequenceGeneratorFactory.java` | `ffb-engine` | `src/factory/sequence_generator_factory.rs` | ✓ |
| `server/factory/StepActionFactory.java` | `ffb-engine` | `src/factory/step_action_factory.rs` | ✓ |
| `server/factory/StepIdFactory.java` | `ffb-engine` | `src/factory/step_id_factory.rs` | ✓ |

### server/handler/ (108 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/handler/AbstractServerCommandHandlerSketch.java` | `ffb-server` | `src/handler/abstract_server_command_handler_sketch.rs` | ✓ |
| `server/handler/IReceivedCommandHandler.java` | `ffb-server` | `src/handler/i_received_command_handler.rs` | ✓ |
| `server/handler/RedeployHandler.java` | `ffb-server` | `src/handler/redeploy_handler.rs` | ✓ |
| `server/handler/ServerCommandHandler.java` | `ffb-server` | `src/handler/server_command_handler.rs` | ✓ |
| `server/handler/ServerCommandHandlerAddLoadedTeam.java` | `ffb-server` | `src/handler/server_command_handler_add_loaded_team.rs` | ✓ |
| `server/handler/ServerCommandHandlerAddSketch.java` | `ffb-server` | `src/handler/server_command_handler_add_sketch.rs` | ✓ |
| `server/handler/ServerCommandHandlerApplyAutomatedPlayerMarkings.java` | `ffb-server` | `src/handler/server_command_handler_apply_automated_player_markings.rs` | ✓ |
| `server/handler/ServerCommandHandlerCalculateAutomaticPlayerMarkings.java` | `ffb-server` | `src/handler/server_command_handler_calculate_automatic_player_markings.rs` | ✓ |
| `server/handler/ServerCommandHandlerClearSketches.java` | `ffb-server` | `src/handler/server_command_handler_clear_sketches.rs` | ✓ |
| `server/handler/ServerCommandHandlerCloseGame.java` | `ffb-server` | `src/handler/server_command_handler_close_game.rs` | ✓ |
| `server/handler/ServerCommandHandlerCloseSession.java` | `ffb-server` | `src/handler/server_command_handler_close_session.rs` | ✓ |
| `server/handler/ServerCommandHandlerDeleteGame.java` | `ffb-server` | `src/handler/server_command_handler_delete_game.rs` | ✓ |
| `server/handler/ServerCommandHandlerFactory.java` | `ffb-server` | `src/handler/server_command_handler_factory.rs` | ✓ |
| `server/handler/ServerCommandHandlerFumbblGameChecked.java` | `ffb-server` | `src/handler/server_command_handler_fumbbl_game_checked.rs` | ✓ |
| `server/handler/ServerCommandHandlerFumbblTeamLoaded.java` | `ffb-server` | `src/handler/server_command_handler_fumbbl_team_loaded.rs` | ✓ |
| `server/handler/ServerCommandHandlerJoin.java` | `ffb-server` | `src/handler/server_command_handler_join.rs` | ✓ |
| `server/handler/ServerCommandHandlerJoinApproved.java` | `ffb-server` | `src/handler/server_command_handler_join_approved.rs` | ✓ |
| `server/handler/ServerCommandHandlerJoinReplay.java` | `ffb-server` | `src/handler/server_command_handler_join_replay.rs` | ✓ |
| `server/handler/ServerCommandHandlerLoadAutomaticPlayerMarkings.java` | `ffb-server` | `src/handler/server_command_handler_load_automatic_player_markings.rs` | ✓ |
| `server/handler/ServerCommandHandlerPasswordChallenge.java` | `ffb-server` | `src/handler/server_command_handler_password_challenge.rs` | ✓ |
| `server/handler/ServerCommandHandlerPing.java` | `ffb-server` | `src/handler/server_command_handler_ping.rs` | ✓ |
| `server/handler/ServerCommandHandlerRemoveSketches.java` | `ffb-server` | `src/handler/server_command_handler_remove_sketches.rs` | ✓ |
| `server/handler/ServerCommandHandlerReplay.java` | `ffb-server` | `src/handler/server_command_handler_replay.rs` | ✓ |
| `server/handler/ServerCommandHandlerReplayLoaded.java` | `ffb-server` | `src/handler/server_command_handler_replay_loaded.rs` | ✓ |
| `server/handler/ServerCommandHandlerReplayStatus.java` | `ffb-server` | `src/handler/server_command_handler_replay_status.rs` | ✓ |
| `server/handler/ServerCommandHandlerRequestVersion.java` | `ffb-server` | `src/handler/server_command_handler_request_version.rs` | ✓ |
| `server/handler/ServerCommandHandlerScheduleGame.java` | `ffb-server` | `src/handler/server_command_handler_schedule_game.rs` | ✓ |
| `server/handler/ServerCommandHandlerSetMarker.java` | `ffb-server` | `src/handler/server_command_handler_set_marker.rs` | ✓ |
| `server/handler/ServerCommandHandlerSetPreventSketching.java` | `ffb-server` | `src/handler/server_command_handler_set_prevent_sketching.rs` | ✓ |
| `server/handler/ServerCommandHandlerSketchAddCoordinate.java` | `ffb-server` | `src/handler/server_command_handler_sketch_add_coordinate.rs` | ✓ |
| `server/handler/ServerCommandHandlerSketchSetColor.java` | `ffb-server` | `src/handler/server_command_handler_sketch_set_color.rs` | ✓ |
| `server/handler/ServerCommandHandlerSketchSetLabel.java` | `ffb-server` | `src/handler/server_command_handler_sketch_set_label.rs` | ✓ |
| `server/handler/ServerCommandHandlerSocketClosed.java` | `ffb-server` | `src/handler/server_command_handler_socket_closed.rs` | ✓ |
| `server/handler/ServerCommandHandlerTalk.java` | `ffb-server` | `src/handler/server_command_handler_talk.rs` | ✓ |
| `server/handler/ServerCommandHandlerTransferControl.java` | `ffb-server` | `src/handler/server_command_handler_transfer_control.rs` | ✓ |
| `server/handler/ServerCommandHandlerUpdatePlayerMarkings.java` | `ffb-server` | `src/handler/server_command_handler_update_player_markings.rs` | ✓ |
| `server/handler/ServerCommandHandlerUploadGame.java` | `ffb-server` | `src/handler/server_command_handler_upload_game.rs` | ✓ |
| `server/handler/ServerCommandHandlerUserSettings.java` | `ffb-server` | `src/handler/server_command_handler_user_settings.rs` | ✓ |
| `server/handler/talk/CommandAdapter.java` | `ffb-server` | `src/handler/talk/command_adapter.rs` | ✓ |
| `server/handler/talk/DecoratingCommandAdapter.java` | `ffb-server` | `src/handler/talk/decorating_command_adapter.rs` | ✓ |
| `server/handler/talk/IdentityCommandAdapter.java` | `ffb-server` | `src/handler/talk/identity_command_adapter.rs` | ✓ |
| `server/handler/talk/TalkHandler.java` | `ffb-server` | `src/handler/talk/talk_handler.rs` | ✓ |
| `server/handler/talk/TalkHandlerActivated.java` | `ffb-server` | `src/handler/talk/talk_handler_activated.rs` | ✓ |
| `server/handler/talk/TalkHandlerActivatedLive.java` | `ffb-server` | `src/handler/talk/talk_handler_activated_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerActivatedTest.java` | `ffb-server` | `src/handler/talk/talk_handler_activated_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerBox.java` | `ffb-server` | `src/handler/talk/talk_handler_box.rs` | ✓ |
| `server/handler/talk/TalkHandlerBoxLive.java` | `ffb-server` | `src/handler/talk/talk_handler_box_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerBoxTest.java` | `ffb-server` | `src/handler/talk/talk_handler_box_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerCard.java` | `ffb-server` | `src/handler/talk/talk_handler_card.rs` | ✓ |
| `server/handler/talk/TalkHandlerEmote.java` | `ffb-server` | `src/handler/talk/talk_handler_emote.rs` | ✓ |
| `server/handler/talk/TalkHandlerGameId.java` | `ffb-server` | `src/handler/talk/talk_handler_game_id.rs` | ✓ |
| `server/handler/talk/TalkHandlerGames.java` | `ffb-server` | `src/handler/talk/talk_handler_games.rs` | ✓ |
| `server/handler/talk/TalkHandlerInjury.java` | `ffb-server` | `src/handler/talk/talk_handler_injury.rs` | ✓ |
| `server/handler/talk/TalkHandlerInjuryLive.java` | `ffb-server` | `src/handler/talk/talk_handler_injury_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerInjuryTest.java` | `ffb-server` | `src/handler/talk/talk_handler_injury_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerMessage.java` | `ffb-server` | `src/handler/talk/talk_handler_message.rs` | ✓ |
| `server/handler/talk/TalkHandlerMoveBall.java` | `ffb-server` | `src/handler/talk/talk_handler_move_ball.rs` | ✓ |
| `server/handler/talk/TalkHandlerMoveBallLive.java` | `ffb-server` | `src/handler/talk/talk_handler_move_ball_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerMoveBallTest.java` | `ffb-server` | `src/handler/talk/talk_handler_move_ball_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerMovePlayer.java` | `ffb-server` | `src/handler/talk/talk_handler_move_player.rs` | ✓ |
| `server/handler/talk/TalkHandlerMovePlayerLive.java` | `ffb-server` | `src/handler/talk/talk_handler_move_player_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerMovePlayerTest.java` | `ffb-server` | `src/handler/talk/talk_handler_move_player_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerOption.java` | `ffb-server` | `src/handler/talk/talk_handler_option.rs` | ✓ |
| `server/handler/talk/TalkHandlerOptions.java` | `ffb-server` | `src/handler/talk/talk_handler_options.rs` | ✓ |
| `server/handler/talk/TalkHandlerPlayingLive.java` | `ffb-server` | `src/handler/talk/talk_handler_playing_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerPrayer.java` | `ffb-server` | `src/handler/talk/talk_handler_prayer.rs` | ✓ |
| `server/handler/talk/TalkHandlerProne.java` | `ffb-server` | `src/handler/talk/talk_handler_prone.rs` | ✓ |
| `server/handler/talk/TalkHandlerProneLive.java` | `ffb-server` | `src/handler/talk/talk_handler_prone_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerProneTest.java` | `ffb-server` | `src/handler/talk/talk_handler_prone_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerRedeploy.java` | `ffb-server` | `src/handler/talk/talk_handler_redeploy.rs` | ✓ |
| `server/handler/talk/TalkHandlerReRoll.java` | `ffb-server` | `src/handler/talk/talk_handler_re_roll.rs` | ✓ |
| `server/handler/talk/TalkHandlerReRollLive.java` | `ffb-server` | `src/handler/talk/talk_handler_re_roll_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerReRollTest.java` | `ffb-server` | `src/handler/talk/talk_handler_re_roll_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerResetStateLive.java` | `ffb-server` | `src/handler/talk/talk_handler_reset_state_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerRoll.java` | `ffb-server` | `src/handler/talk/talk_handler_roll.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetBall.java` | `ffb-server` | `src/handler/talk/talk_handler_set_ball.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetBallLive.java` | `ffb-server` | `src/handler/talk/talk_handler_set_ball_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetBallTest.java` | `ffb-server` | `src/handler/talk/talk_handler_set_ball_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetPlayer.java` | `ffb-server` | `src/handler/talk/talk_handler_set_player.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetPlayerLive.java` | `ffb-server` | `src/handler/talk/talk_handler_set_player_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerSetPlayerTest.java` | `ffb-server` | `src/handler/talk/talk_handler_set_player_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerSkill.java` | `ffb-server` | `src/handler/talk/talk_handler_skill.rs` | ✓ |
| `server/handler/talk/TalkHandlerSkillLive.java` | `ffb-server` | `src/handler/talk/talk_handler_skill_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerSkillTest.java` | `ffb-server` | `src/handler/talk/talk_handler_skill_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerSound.java` | `ffb-server` | `src/handler/talk/talk_handler_sound.rs` | ✓ |
| `server/handler/talk/TalkHandlerSounds.java` | `ffb-server` | `src/handler/talk/talk_handler_sounds.rs` | ✓ |
| `server/handler/talk/TalkHandlerSpecs.java` | `ffb-server` | `src/handler/talk/talk_handler_specs.rs` | ✓ |
| `server/handler/talk/TalkHandlerStandup.java` | `ffb-server` | `src/handler/talk/talk_handler_standup.rs` | ✓ |
| `server/handler/talk/TalkHandlerStandupLive.java` | `ffb-server` | `src/handler/talk/talk_handler_standup_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerStandupTest.java` | `ffb-server` | `src/handler/talk/talk_handler_standup_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerStat.java` | `ffb-server` | `src/handler/talk/talk_handler_stat.rs` | ✓ |
| `server/handler/talk/TalkHandlerStatLive.java` | `ffb-server` | `src/handler/talk/talk_handler_stat_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerStatTest.java` | `ffb-server` | `src/handler/talk/talk_handler_stat_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerStun.java` | `ffb-server` | `src/handler/talk/talk_handler_stun.rs` | ✓ |
| `server/handler/talk/TalkHandlerStunLive.java` | `ffb-server` | `src/handler/talk/talk_handler_stun_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerStunTest.java` | `ffb-server` | `src/handler/talk/talk_handler_stun_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerTurnLive.java` | `ffb-server` | `src/handler/talk/talk_handler_turn_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerTurnMode.java` | `ffb-server` | `src/handler/talk/talk_handler_turn_mode.rs` | ✓ |
| `server/handler/talk/TalkHandlerTurnModeLive.java` | `ffb-server` | `src/handler/talk/talk_handler_turn_mode_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerTurnModelTest.java` | `ffb-server` | `src/handler/talk/talk_handler_turn_model_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerTurnTest.java` | `ffb-server` | `src/handler/talk/talk_handler_turn_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerUsedActions.java` | `ffb-server` | `src/handler/talk/talk_handler_used_actions.rs` | ✓ |
| `server/handler/talk/TalkHandlerUsedActionsLive.java` | `ffb-server` | `src/handler/talk/talk_handler_used_actions_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerUsedActionsTest.java` | `ffb-server` | `src/handler/talk/talk_handler_used_actions_test.rs` | ✓ |
| `server/handler/talk/TalkHandlerWeather.java` | `ffb-server` | `src/handler/talk/talk_handler_weather.rs` | ✓ |
| `server/handler/talk/TalkHandlerWeatherLive.java` | `ffb-server` | `src/handler/talk/talk_handler_weather_live.rs` | ✓ |
| `server/handler/talk/TalkHandlerWeatherTest.java` | `ffb-server` | `src/handler/talk/talk_handler_weather_test.rs` | ✓ |
| `server/handler/talk/TalkRequirements.java` | `ffb-server` | `src/handler/talk/talk_requirements.rs` | ✓ |

### server/inducements/ (75 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/inducements/bb2016/cards/ChopBlockHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/chop_block_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/CustardPieHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/custard_pie_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/DistractHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/distract_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/ForceShieldHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/force_shield_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/IllegalSubstitutionHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/illegal_substitution_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/PitTrapHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/pit_trap_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/RabbitsFootHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/rabbits_foot_handler.rs` | ✓ |
| `server/inducements/bb2016/cards/WitchBrewHandler.java` | `ffb-engine` | `src/inducements/bb2016/cards/witch_brew_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/ChopBlockHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/chop_block_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/CustardPieHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/custard_pie_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/DistractHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/distract_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/ForceShieldHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/force_shield_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/IllegalSubstitutionHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/illegal_substitution_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/PitTrapHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/pit_trap_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/RabbitsFootHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/rabbits_foot_handler.rs` | ✓ |
| `server/inducements/bb2020/cards/WitchBrewHandler.java` | `ffb-engine` | `src/inducements/bb2020/cards/witch_brew_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/bad_habits_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/blessed_statue_of_nuffle_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/fan_interaction_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/fouling_frenzy_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/friends_with_the_ref_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/greasy_cleats_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/intensive_training_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/iron_man_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/knuckle_dusters_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/moles_under_the_pitch_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/NecessaryViolenceHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/necessary_violence_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/OpponentPlayerSelector.java` | `ffb-engine` | `src/inducements/bb2020/prayers/opponent_player_selector.rs` | ✓ |
| `server/inducements/bb2020/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/perfect_passing_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/bb2020/prayers/player_selector.rs` | ✓ |
| `server/inducements/bb2020/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/stiletto_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/throw_a_rock_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/treacherous_trapdoor_handler.rs` | ✓ |
| `server/inducements/bb2020/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/bb2020/prayers/under_scrutiny_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/bad_habits_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/blessed_statue_of_nuffle_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/DazzlingCatchingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/dazzling_catching_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/fan_interaction_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/fouling_frenzy_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/friends_with_the_ref_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/greasy_cleats_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/intensive_training_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/iron_man_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/knuckle_dusters_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/moles_under_the_pitch_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/OpponentPlayerSelector.java` | `ffb-engine` | `src/inducements/bb2025/prayers/opponent_player_selector.rs` | ✓ |
| `server/inducements/bb2025/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/perfect_passing_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/bb2025/prayers/player_selector.rs` | ✓ |
| `server/inducements/bb2025/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/stiletto_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/throw_a_rock_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/treacherous_trapdoor_handler.rs` | ✓ |
| `server/inducements/bb2025/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/bb2025/prayers/under_scrutiny_handler.rs` | ✓ |
| `server/inducements/CardHandler.java` | `ffb-engine` | `src/inducements/card_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/BadHabitsHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/bad_habits_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/BlessedStatueOfNuffleHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/blessed_statue_of_nuffle_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/DialogPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/dialog_prayer_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/EnhancementRemover.java` | `ffb-engine` | `src/inducements/mixed/prayers/enhancement_remover.rs` | ✓ |
| `server/inducements/mixed/prayers/FanInteractionHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/fan_interaction_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/FoulingFrenzyHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/fouling_frenzy_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/FriendsWithTheRefHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/friends_with_the_ref_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/GreasyCleatsHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/greasy_cleats_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/IntensiveTrainingHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/intensive_training_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/IronManHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/iron_man_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/KnuckleDustersHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/knuckle_dusters_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/MolesUnderThePitchHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/moles_under_the_pitch_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/PerfectPassingHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/perfect_passing_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/PlayerSelector.java` | `ffb-engine` | `src/inducements/mixed/prayers/player_selector.rs` | ✓ |
| `server/inducements/mixed/prayers/PrayerDialogSelection.java` | `ffb-engine` | `src/inducements/mixed/prayers/prayer_dialog_selection.rs` | ✓ |
| `server/inducements/mixed/prayers/PrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/prayer_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/RandomSelectionPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/random_selection_prayer_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/SelectPlayerPrayerHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/select_player_prayer_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/StilettoHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/stiletto_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/ThrowARockHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/throw_a_rock_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/TreacherousTrapdoorHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/treacherous_trapdoor_handler.rs` | ✓ |
| `server/inducements/mixed/prayers/UnderScrutinyHandler.java` | `ffb-engine` | `src/inducements/mixed/prayers/under_scrutiny_handler.rs` | ✓ |

### server/injury/ (69 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/injury/injuryType/AbstractInjuryTypeBombWithModifier.java` | `ffb-engine` | `src/injury/injuryType/abstract_injury_type_bomb_with_modifier.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBallAndChain.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ball_and_chain.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBitten.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bitten.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlock.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockProne.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_prone.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockProneForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_prone_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockStunned.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_stunned.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBlockStunnedForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_block_stunned_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBomb.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBombWithModifier.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb_with_modifier.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBombWithModifierForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_bomb_with_modifier_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBreatheFire.java` | `ffb-engine` | `src/injury/injuryType/injury_type_breathe_fire.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeBreatheFireForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_breathe_fire_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeChainsaw.java` | `ffb-engine` | `src/injury/injuryType/injury_type_chainsaw.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeChainsawForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_chainsaw_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowd.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowdPush.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd_push.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeCrowdPushForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_crowd_push_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropDodge.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_dodge.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropDodgeForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_dodge_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropGFI.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_gfi.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeDropJump.java` | `ffb-engine` | `src/injury/injuryType/injury_type_drop_jump.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeEatPlayer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_eat_player.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFireball.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fireball.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFoul.java` | `ffb-engine` | `src/injury/injuryType/injury_type_foul.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFoulForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_foul_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFumbledKtm.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fumbled_ktm.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeFumbledKtmApoKo.java` | `ffb-engine` | `src/injury/injuryType/injury_type_fumbled_ktm_apo_ko.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKegHit.java` | `ffb-engine` | `src/injury/injuryType/injury_type_keg_hit.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKTMCrowd.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ktm_crowd.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeKTMInjury.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ktm_injury.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeLightning.java` | `ffb-engine` | `src/injury/injuryType/injury_type_lightning.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnArmour.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_armour.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnInjury.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_injury.rs` | ✓ |
| `server/injury/injuryType/InjuryTypePilingOnKnockedOut.java` | `ffb-engine` | `src/injury/injuryType/injury_type_piling_on_knocked_out.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeProjectileVomit.java` | `ffb-engine` | `src/injury/injuryType/injury_type_projectile_vomit.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeQuickBite.java` | `ffb-engine` | `src/injury/injuryType/injury_type_quick_bite.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeSabotaged.java` | `ffb-engine` | `src/injury/injuryType/injury_type_sabotaged.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeSaboteur.java` | `ffb-engine` | `src/injury/injuryType/injury_type_saboteur.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeServer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_server.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeStab.java` | `ffb-engine` | `src/injury/injuryType/injury_type_stab.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeStabForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_stab_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThenIStartedBlastin.java` | `ffb-engine` | `src/injury/injuryType/injury_type_then_i_started_blastin.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThrowARock.java` | `ffb-engine` | `src/injury/injuryType/injury_type_throw_a_rock.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeThrowARockStalling.java` | `ffb-engine` | `src/injury/injuryType/injury_type_throw_a_rock_stalling.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTrapDoorFall.java` | `ffb-engine` | `src/injury/injuryType/injury_type_trap_door_fall.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTrapDoorFallForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_trap_door_fall_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMHitPlayer.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_hit_player.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMHitPlayerForSpp.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_hit_player_for_spp.rs` | ✓ |
| `server/injury/injuryType/InjuryTypeTTMLanding.java` | `ffb-engine` | `src/injury/injuryType/injury_type_ttm_landing.rs` | ✓ |
| `server/injury/injuryType/ModificationAwareInjuryTypeServer.java` | `ffb-engine` | `src/injury/injuryType/modification_aware_injury_type_server.rs` | ✓ |
| `server/injury/modification/AvOrInjModification.java` | `ffb-engine` | `src/injury/modification/av_or_inj_modification.rs` | ✓ |
| `server/injury/modification/bb2020/SlayerModification.java` | `ffb-engine` | `src/injury/modification/bb2020/slayer_modification.rs` | ✓ |
| `server/injury/modification/bb2020/ToxinConnoisseurModification.java` | `ffb-engine` | `src/injury/modification/bb2020/toxin_connoisseur_modification.rs` | ✓ |
| `server/injury/modification/bb2025/KrumpAndSmashModification.java` | `ffb-engine` | `src/injury/modification/bb2025/krump_and_smash_modification.rs` | ✓ |
| `server/injury/modification/bb2025/LoneFoulerModification.java` | `ffb-engine` | `src/injury/modification/bb2025/lone_fouler_modification.rs` | ✓ |
| `server/injury/modification/bb2025/MasterAssassinModification.java` | `ffb-engine` | `src/injury/modification/bb2025/master_assassin_modification.rs` | ✓ |
| `server/injury/modification/bb2025/RerollArmourModification.java` | `ffb-engine` | `src/injury/modification/bb2025/reroll_armour_modification.rs` | ✓ |
| `server/injury/modification/bb2025/SlayerModification.java` | `ffb-engine` | `src/injury/modification/bb2025/slayer_modification.rs` | ✓ |
| `server/injury/modification/bb2025/ToxinConnoisseurModification.java` | `ffb-engine` | `src/injury/modification/bb2025/toxin_connoisseur_modification.rs` | ✓ |
| `server/injury/modification/BrutalBlockModification.java` | `ffb-engine` | `src/injury/modification/brutal_block_modification.rs` | ✓ |
| `server/injury/modification/CrushingBlowModification.java` | `ffb-engine` | `src/injury/modification/crushing_blow_modification.rs` | ✓ |
| `server/injury/modification/GhostlyFlamesModification.java` | `ffb-engine` | `src/injury/modification/ghostly_flames_modification.rs` | ✓ |
| `server/injury/modification/InjuryContextModification.java` | `ffb-engine` | `src/injury/modification/injury_context_modification.rs` | ✓ |
| `server/injury/modification/MasterAssassinModification.java` | `ffb-engine` | `src/injury/modification/master_assassin_modification.rs` | ✓ |
| `server/injury/modification/ModificationParams.java` | `ffb-engine` | `src/injury/modification/modification_params.rs` | ✓ |
| `server/injury/modification/OldProModification.java` | `ffb-engine` | `src/injury/modification/old_pro_modification.rs` | ✓ |
| `server/injury/modification/OldProModificationParams.java` | `ffb-engine` | `src/injury/modification/old_pro_modification_params.rs` | ✓ |
| `server/injury/modification/SavageMaulingModification.java` | `ffb-engine` | `src/injury/modification/savage_mauling_modification.rs` | ✓ |

### server/marking/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/marking/ApplyTo.java` | `ffb-engine` | `src/marking/apply_to.rs` | ✓ |
| `server/marking/AutoMarkingConfig.java` | `ffb-engine` | `src/marking/auto_marking_config.rs` | ✓ |
| `server/marking/AutoMarkingRecord.java` | `ffb-engine` | `src/marking/auto_marking_record.rs` | ✓ |
| `server/marking/MarkerGenerator.java` | `ffb-engine` | `src/marking/marker_generator.rs` | ✓ |

### server/mechanic/ (16 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/mechanic/ArmorModifierValues.java` | `ffb-engine` | `src/mechanic/armor_modifier_values.rs` | ✓ |
| `server/mechanic/bb2016/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2016/roll_mechanic.rs` | ✓ |
| `server/mechanic/bb2020/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2020/roll_mechanic.rs` | ✓ |
| `server/mechanic/bb2025/RollMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/roll_mechanic.rs` | ✓ |
| `server/mechanic/bb2025/SetupMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/setup_mechanic.rs` | ✓ |
| `server/mechanic/bb2025/StateMechanic.java` | `ffb-engine` | `src/mechanic/bb2025/state_mechanic.rs` | ✓ |
| `server/mechanic/CasualtyCalc.java` | `ffb-engine` | `src/mechanic/casualty_calc.rs` | ✓ |
| `server/mechanic/InjuryCalc.java` | `ffb-engine` | `src/mechanic/injury_calc.rs` | ✓ |
| `server/mechanic/InjuryModifierValues.java` | `ffb-engine` | `src/mechanic/injury_modifier_values.rs` | ✓ |
| `server/mechanic/mixed/SetupMechanic.java` | `ffb-engine` | `src/mechanic/mixed/setup_mechanic.rs` | ✓ |
| `server/mechanic/mixed/StateMechanic.java` | `ffb-engine` | `src/mechanic/mixed/state_mechanic.rs` | ✓ |
| `server/mechanic/RollMechanic.java` | `ffb-engine` | `src/mechanic/roll_mechanic.rs` | ✓ |
| `server/mechanic/SetupMechanic.java` | `ffb-engine` | `src/mechanic/setup_mechanic.rs` | ✓ |
| `server/mechanic/SppCalc.java` | `ffb-engine` | `src/mechanic/spp_calc.rs` | ✓ |
| `server/mechanic/StateMechanic.java` | `ffb-engine` | `src/mechanic/state_mechanic.rs` | ✓ |
| `server/mechanic/WeatherModifierValues.java` | `ffb-engine` | `src/mechanic/weather_modifier_values.rs` | ✓ |

### server/model/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/model/change/ChompRemovalObserver.java` | `ffb-engine` | `src/model/change/chomp_removal_observer.rs` | ✓ |
| `server/model/change/ConditionalModelChangeObserver.java` | `ffb-engine` | `src/model/change/conditional_model_change_observer.rs` | ✓ |
| `server/model/DropPlayerContext.java` | `ffb-engine` | `src/model/drop_player_context.rs` | ✓ |
| `server/model/DropPlayerContextBuilder.java` | `ffb-engine` | `src/model/drop_player_context_builder.rs` | ✓ |
| `server/model/SkillBehaviour.java` | `ffb-engine` | `src/model/skill_behaviour.rs` | ✓ |
| `server/model/SteadyFootingContext.java` | `ffb-engine` | `src/model/steady_footing_context.rs` | ✓ |
| `server/model/StepModifier.java` | `ffb-engine` | `src/model/step_modifier.rs` | ✓ |

### server/net/ (26 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/net/commands/InternalServerCommand.java` | `ffb-server` | `src/net/commands/internal_server_command.rs` | ✓ |
| `server/net/commands/InternalServerCommandAddLoadedTeam.java` | `ffb-server` | `src/net/commands/internal_server_command_add_loaded_team.rs` | ✓ |
| `server/net/commands/InternalServerCommandApplyAutomatedPlayerMarkings.java` | `ffb-server` | `src/net/commands/internal_server_command_apply_automated_player_markings.rs` | ✓ |
| `server/net/commands/InternalServerCommandCalculateAutomaticPlayerMarkings.java` | `ffb-server` | `src/net/commands/internal_server_command_calculate_automatic_player_markings.rs` | ✓ |
| `server/net/commands/InternalServerCommandClearCache.java` | `ffb-server` | `src/net/commands/internal_server_command_clear_cache.rs` | ✓ |
| `server/net/commands/InternalServerCommandCloseGame.java` | `ffb-server` | `src/net/commands/internal_server_command_close_game.rs` | ✓ |
| `server/net/commands/InternalServerCommandDeleteGame.java` | `ffb-server` | `src/net/commands/internal_server_command_delete_game.rs` | ✓ |
| `server/net/commands/InternalServerCommandFumbblGameChecked.java` | `ffb-server` | `src/net/commands/internal_server_command_fumbbl_game_checked.rs` | ✓ |
| `server/net/commands/InternalServerCommandFumbblGameCreated.java` | `ffb-server` | `src/net/commands/internal_server_command_fumbbl_game_created.rs` | ✓ |
| `server/net/commands/InternalServerCommandFumbblTeamLoaded.java` | `ffb-server` | `src/net/commands/internal_server_command_fumbbl_team_loaded.rs` | ✓ |
| `server/net/commands/InternalServerCommandJoinApproved.java` | `ffb-server` | `src/net/commands/internal_server_command_join_approved.rs` | ✓ |
| `server/net/commands/InternalServerCommandReplayLoaded.java` | `ffb-server` | `src/net/commands/internal_server_command_replay_loaded.rs` | ✓ |
| `server/net/commands/InternalServerCommandScheduleGame.java` | `ffb-server` | `src/net/commands/internal_server_command_schedule_game.rs` | ✓ |
| `server/net/commands/InternalServerCommandSocketClosed.java` | `ffb-server` | `src/net/commands/internal_server_command_socket_closed.rs` | ✓ |
| `server/net/commands/InternalServerCommandUploadGame.java` | `ffb-server` | `src/net/commands/internal_server_command_upload_game.rs` | ✓ |
| `server/net/CommandServlet.java` | `ffb-server` | `src/net/command_servlet.rs` | ✓ |
| `server/net/CommandSocket.java` | `ffb-server` | `src/net/command_socket.rs` | ✓ |
| `server/net/FileServlet.java` | `ffb-server` | `src/net/file_servlet.rs` | ✓ |
| `server/net/ReceivedCommand.java` | `ffb-server` | `src/net/received_command.rs` | ✓ |
| `server/net/ReplaySessionManager.java` | `ffb-server` | `src/net/replay_session_manager.rs` | ✓ |
| `server/net/ServerCommunication.java` | `ffb-server` | `src/net/server_communication.rs` | ✓ |
| `server/net/ServerDbKeepAliveTask.java` | `ffb-server` | `src/net/server_db_keep_alive_task.rs` | ✓ |
| `server/net/ServerGameTimeTask.java` | `ffb-server` | `src/net/server_game_time_task.rs` | ✓ |
| `server/net/ServerNetworkEntropyTask.java` | `ffb-server` | `src/net/server_network_entropy_task.rs` | ✓ |
| `server/net/SessionManager.java` | `ffb-server` | `src/net/session_manager.rs` | ✓ |
| `server/net/SessionTimeoutTask.java` | `ffb-server` | `src/net/session_timeout_task.rs` | ✓ |

### server/request/ (21 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/request/fumbbl/AbstractFumbblRequestLoadPlayerMarkings.java` | `ffb-server` | `src/request/fumbbl/abstract_fumbbl_request_load_player_markings.rs` | ✓ |
| `server/request/fumbbl/FumbblGameState.java` | `ffb-server` | `src/request/fumbbl/fumbbl_game_state.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestCheckAuthorization.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_check_authorization.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestCheckGamestate.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_check_gamestate.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestCreateGamestate.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_create_gamestate.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestLoadPlayerMarkings.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_load_player_markings.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestLoadPlayerMarkingsForGameVersion.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_load_player_markings_for_game_version.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestLoadTeam.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_load_team.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestLoadTeamList.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_load_team_list.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestPasswordChallenge.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_password_challenge.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestRemoveGamestate.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_remove_gamestate.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestResumeGamestate.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_resume_gamestate.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestUpdateGamestate.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_update_gamestate.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestUploadResults.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_upload_results.rs` | ✓ |
| `server/request/fumbbl/FumbblRequestUploadTalk.java` | `ffb-server` | `src/request/fumbbl/fumbbl_request_upload_talk.rs` | ✓ |
| `server/request/fumbbl/FumbblResult.java` | `ffb-server` | `src/request/fumbbl/fumbbl_result.rs` | ✓ |
| `server/request/fumbbl/UtilFumbblRequest.java` | `ffb-server` | `src/request/fumbbl/util_fumbbl_request.rs` | ✓ |
| `server/request/ServerRequest.java` | `ffb-server` | `src/request/server_request.rs` | ✓ |
| `server/request/ServerRequestLoadReplay.java` | `ffb-server` | `src/request/server_request_load_replay.rs` | ✓ |
| `server/request/ServerRequestProcessor.java` | `ffb-server` | `src/request/server_request_processor.rs` | ✓ |
| `server/request/ServerRequestSaveReplay.java` | `ffb-server` | `src/request/server_request_save_replay.rs` | ✓ |

### server/root/ (31 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/ActionStatus.java` | `ffb-engine` | `src/action_status.rs` | ✓ |
| `server/ActiveEffects.java` | `ffb-engine` | `src/active_effects.rs` | ✓ |
| `server/CardDeck.java` | `ffb-engine` | `src/card_deck.rs` | ✓ |
| `server/DbUpdater.java` | `ffb-engine` | `src/db_updater.rs` | ✓ |
| `server/DebugLog.java` | `ffb-engine` | `src/debug_log.rs` | ✓ |
| `server/DiceInterpreter.java` | `ffb-engine` | `src/dice_interpreter.rs` | ✓ |
| `server/DiceRoller.java` | `ffb-engine` | `src/dice_roller.rs` | ✓ |
| `server/FantasyFootballServer.java` | `ffb-engine` | `src/fantasy_football_server.rs` | ✓ |
| `server/GameCache.java` | `ffb-server` | `src/game_cache.rs` | ✓ |
| `server/GameLog.java` | `ffb-engine` | `src/game_log.rs` | ✓ |
| `server/GameStartMode.java` | `ffb-engine` | `src/game_start_mode.rs` | ✓ |
| `server/GameState.java` | `ffb-engine` | `src/game_state.rs` | ✓ |
| `server/IdGenerator.java` | `ffb-engine` | `src/id_generator.rs` | ✓ |
| `server/IGameIdListener.java` | `ffb-engine` | `src/i_game_id_listener.rs` | ✓ |
| `server/InjuryResult.java` | `ffb-engine` | `src/injury_result.rs` | ✓ |
| `server/IServerJsonOption.java` | `ffb-engine` | `src/i_server_json_option.rs` | ✓ |
| `server/IServerLogLevel.java` | `ffb-engine` | `src/i_server_log_level.rs` | ✓ |
| `server/IServerProperty.java` | `ffb-engine` | `src/i_server_property.rs` | ✓ |
| `server/PrayerState.java` | `ffb-engine` | `src/prayer_state.rs` | ✓ |
| `server/ReplayCache.java` | `ffb-engine` | `src/replay_cache.rs` | ✓ |
| `server/ReplayState.java` | `ffb-engine` | `src/replay_state.rs` | ✓ |
| `server/RosterCache.java` | `ffb-server` | `src/roster_cache.rs` | ✓ |
| `server/ServerMode.java` | `ffb-engine` | `src/server_mode.rs` | ✓ |
| `server/ServerReplay.java` | `ffb-engine` | `src/server_replay.rs` | ✓ |
| `server/ServerReplayer.java` | `ffb-engine` | `src/server_replayer.rs` | ✓ |
| `server/ServerSketchManager.java` | `ffb-engine` | `src/server_sketch_manager.rs` | ✓ |
| `server/ServerUrlProperty.java` | `ffb-engine` | `src/server_url_property.rs` | ✓ |
| `server/SessionMode.java` | `ffb-engine` | `src/session_mode.rs` | ✓ |
| `server/Talk.java` | `ffb-engine` | `src/talk.rs` | ✓ |
| `server/TeamCache.java` | `ffb-server` | `src/team_cache.rs` | ✓ |
| `server/TeamSetupCache.java` | `ffb-engine` | `src/team_setup_cache.rs` | ✓ |

### server/skillbehaviour/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/StepHook.java` | `ffb-engine` | `src/skill_behaviour/step_hook.rs` | ✓ |

### server/skillbehaviour/bb2016/ (34 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2016/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/agility_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/animosity_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/ArmourIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/armour_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/blood_lust_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/bombardier_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/bone_head_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/catch_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/DauntlessBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dauntless_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/diving_tackle_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dodge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/dump_off_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/foul_appearance_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/grab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/JumpUpBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/jump_up_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/LeapBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/leap_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/monstrous_mouth_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/MovementIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/movement_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/pass_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/PilingOnBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/piling_on_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/really_stupid_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/SafeThrowBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/safe_throw_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/shadowing_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/SideStepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/side_step_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/sneaky_git_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/stab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/stand_firm_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/strength_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/SwarmingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/swarming_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/swoop_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/take_root_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/tentacles_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/throw_team_mate_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/WildAnimalBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/wild_animal_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2016/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2016/wrestle_behaviour.rs` | ✓ |

### server/skillbehaviour/bb2020/ (39 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2020/AbstractPassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/abstract_pass_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/agility_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/AnimalSavageryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/animal_savagery_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/animosity_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/blood_lust_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/bombardier_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/bone_head_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/BrutalBlockBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/brutal_block_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/catch_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/CloudBursterBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/cloud_burster_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/diving_tackle_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dodge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dump_off_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/DwarfenScourgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/dwarfen_scourge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/foul_appearance_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/GhostlyFlamesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/ghostly_flames_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/grab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/MasterAssassinBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/master_assassin_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/monstrous_mouth_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/pass_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/PassingIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/passing_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/PilingOnBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/piling_on_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/really_stupid_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/shadowing_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/SideStepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/side_step_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/SlayerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/slayer_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/sneaky_git_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/stab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/stand_firm_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/strength_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/SwarmingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/swarming_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/swoop_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/take_root_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/tentacles_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/TheBallistaBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/the_ballista_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/throw_team_mate_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/ToxinConnoisseurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/toxin_connoisseur_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/UnchannelledFuryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/unchannelled_fury_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2020/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2020/wrestle_behaviour.rs` | ✓ |

### server/skillbehaviour/bb2025/ (40 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/bb2025/AbstractPassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/abstract_pass_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/AgilityIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/agility_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/AnimalSavageryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/animal_savagery_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/AnimosityBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/animosity_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/BloodLustBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/blood_lust_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/BombardierBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bombardier_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/BoneHeadBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bone_head_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/BullseyeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/bullseye_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/CatchBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/catch_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/DivingTackleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/diving_tackle_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/DodgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dodge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/DumpOffBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dump_off_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/DwarvenScourgeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/dwarven_scourge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/EyeGougeBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/eye_gouge_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/FoulAppearanceBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/foul_appearance_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/GrabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/grab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/JuggernautBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/juggernaut_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/KrumpAndSmashBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/krump_and_smash_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/LoneFoulerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/lone_fouler_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/MasterAssassinBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/master_assassin_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/MonstrousMouthBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/monstrous_mouth_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/PassBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/pass_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/PassingIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/passing_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/ReallyStupidBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/really_stupid_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/SaboteurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/saboteur_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/ShadowingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/shadowing_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/SidestepBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/sidestep_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/SlayerBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/slayer_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/SneakyGitBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/sneaky_git_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/StabBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/stab_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/StandFirmBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/stand_firm_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/StrengthIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/strength_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/SwoopBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/swoop_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/TakeRootBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/take_root_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/TentaclesBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/tentacles_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/TheBallistaBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/the_ballista_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/ThrowTeamMateBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/throw_team_mate_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/ToxinConnoisseurBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/toxin_connoisseur_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/UnchannelledFuryBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/unchannelled_fury_behaviour.rs` | ✓ |
| `server/skillbehaviour/bb2025/WrestleBehaviour.java` | `ffb-engine` | `src/skill_behaviour/bb2025/wrestle_behaviour.rs` | ✓ |

### server/skillbehaviour/common/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/common/HornsBehaviour.java` | `ffb-engine` | `src/skill_behaviour/common/horns_behaviour.rs` | ✓ |

### server/skillbehaviour/mixed/ (14 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/skillbehaviour/mixed/AbstractDodgingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/abstract_dodging_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/AbstractStepModifierMultipleBlock.java` | `ffb-engine` | `src/skill_behaviour/mixed/abstract_step_modifier_multiple_block.rs` | ✓ |
| `server/skillbehaviour/mixed/ArmourIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/armour_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/BlindRageBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/blind_rage_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/CrushingBlowBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/crushing_blow_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/DauntlessBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/dauntless_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/IndomitableBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/indomitable_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/JuggernautBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/juggernaut_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/JumpUpBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/jump_up_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/MovementIncreaseBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/movement_increase_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/OldProBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/old_pro_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/RamBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/ram_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/SavageMaulingBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/savage_mauling_behaviour.rs` | ✓ |
| `server/skillbehaviour/mixed/WatchOutBehaviour.java` | `ffb-engine` | `src/skill_behaviour/mixed/watch_out_behaviour.rs` | ✓ |

### server/step/ (23 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/AbstractStep.java` | `ffb-engine` | `src/step/abstract_step.rs` | ✓ |
| `server/step/AbstractStepWithReRoll.java` | `ffb-engine` | `src/step/abstract_step_with_re_roll.rs` | ✓ |
| `server/step/DeferredCommand.java` | `ffb-engine` | `src/step/deferred_command.rs` | ✓ |
| `server/step/DeferredCommandId.java` | `ffb-engine` | `src/step/deferred_command_id.rs` | ✓ |
| `server/step/HasIdForSingleUseReRoll.java` | `ffb-engine` | `src/step/has_id_for_single_use_re_roll.rs` | ✓ |
| `server/step/IStackModifier.java` | `ffb-engine` | `src/step/i_stack_modifier.rs` | ✓ |
| `server/step/IStep.java` | `ffb-engine` | `src/step/i_step.rs` | ✓ |
| `server/step/IStepLabel.java` | `ffb-engine` | `src/step/i_step_label.rs` | ✓ |
| `server/step/StepAction.java` | `ffb-engine` | `src/step/step_action.rs` | ✓ |
| `server/step/StepCommandStatus.java` | `ffb-engine` | `src/step/step_command_status.rs` | ✓ |
| `server/step/StepException.java` | `ffb-engine` | `src/step/step_exception.rs` | ✓ |
| `server/step/StepFactory.java` | `ffb-engine` | `src/step/step_factory.rs` | ✓ |
| `server/step/StepGotoLabel.java` | `ffb-engine` | `src/step/step_goto_label.rs` | ✓ |
| `server/step/StepId.java` | `ffb-engine` | `src/step/step_id.rs` | ✓ |
| `server/step/StepNextStep.java` | `ffb-engine` | `src/step/step_next_step.rs` | ✓ |
| `server/step/StepNextStepAndRepeat.java` | `ffb-engine` | `src/step/step_next_step_and_repeat.rs` | ✓ |
| `server/step/StepParameter.java` | `ffb-engine` | `src/step/step_parameter.rs` | ✓ |
| `server/step/StepParameterKey.java` | `ffb-engine` | `src/step/step_parameter_key.rs` | ✓ |
| `server/step/StepParameterSet.java` | `ffb-engine` | `src/step/step_parameter_set.rs` | ✓ |
| `server/step/StepResetToMove.java` | `ffb-engine` | `src/step/step_reset_to_move.rs` | ✓ |
| `server/step/StepResult.java` | `ffb-engine` | `src/step/step_result.rs` | ✓ |
| `server/step/StepStack.java` | `ffb-engine` | `src/step/step_stack.rs` | ✓ |
| `server/step/UtilServerSteps.java` | `ffb-engine` | `src/step/util_server_steps.rs` | ✓ |

### server/step/action/ (24 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/action/block/StepBlockStatistics.java` | `ffb-engine` | `src/step/action/block/step_block_statistics.rs` | ✓ |
| `server/step/action/block/StepDauntless.java` | `ffb-engine` | `src/step/action/block/step_dauntless.rs` | ✓ |
| `server/step/action/block/StepDropFallingPlayers.java` | `ffb-engine` | `src/step/action/block/step_drop_falling_players.rs` | ✓ |
| `server/step/action/block/StepDumpOff.java` | `ffb-engine` | `src/step/action/block/step_dump_off.rs` | ✓ |
| `server/step/action/block/StepHorns.java` | `ffb-engine` | `src/step/action/block/step_horns.rs` | ✓ |
| `server/step/action/block/StepJuggernaut.java` | `ffb-engine` | `src/step/action/block/step_juggernaut.rs` | ✓ |
| `server/step/action/block/StepStab.java` | `ffb-engine` | `src/step/action/block/step_stab.rs` | ✓ |
| `server/step/action/block/StepWrestle.java` | `ffb-engine` | `src/step/action/block/step_wrestle.rs` | ✓ |
| `server/step/action/block/UtilBlockSequence.java` | `ffb-engine` | `src/step/action/block/util_block_sequence.rs` | ✓ |
| `server/step/action/common/StepBoneHead.java` | `ffb-engine` | `src/step/action/common/step_bone_head.rs` | ✓ |
| `server/step/action/common/StepReallyStupid.java` | `ffb-engine` | `src/step/action/common/step_really_stupid.rs` | ✓ |
| `server/step/action/foul/StepReferee.java` | `ffb-engine` | `src/step/action/foul/step_referee.rs` | ✓ |
| `server/step/action/ktm/StepEndKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_end_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepInitKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_init_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepKickTeamMate.java` | `ffb-engine` | `src/step/action/ktm/step_kick_team_mate.rs` | ✓ |
| `server/step/action/ktm/StepKickTeamMateDoubleRolled.java` | `ffb-engine` | `src/step/action/ktm/step_kick_team_mate_double_rolled.rs` | ✓ |
| `server/step/action/move/StepDivingTackle.java` | `ffb-engine` | `src/step/action/move/step_diving_tackle.rs` | ✓ |
| `server/step/action/pass/StepAnimosity.java` | `ffb-engine` | `src/step/action/pass/step_animosity.rs` | ✓ |
| `server/step/action/pass/StepBombardier.java` | `ffb-engine` | `src/step/action/pass/step_bombardier.rs` | ✓ |
| `server/step/action/pass/StepDispatchPassing.java` | `ffb-engine` | `src/step/action/pass/step_dispatch_passing.rs` | ✓ |
| `server/step/action/pass/StepHandOver.java` | `ffb-engine` | `src/step/action/pass/step_hand_over.rs` | ✓ |
| `server/step/action/select/StepJumpUp.java` | `ffb-engine` | `src/step/action/select/step_jump_up.rs` | ✓ |
| `server/step/action/ttm/StepEatTeamMate.java` | `ffb-engine` | `src/step/action/ttm/step_eat_team_mate.rs` | ✓ |
| `server/step/action/ttm/UtilThrowTeamMateSequence.java` | `ffb-engine` | `src/step/action/ttm/util_throw_team_mate_sequence.rs` | ✓ |

### server/step/bb2016/ (78 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2016/block/StepBlockBallAndChain.java` | `ffb-engine` | `src/step/bb2016/block/step_block_ball_and_chain.rs` | ✓ |
| `server/step/bb2016/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2016/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2016/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2016/block/step_block_choice.rs` | ✓ |
| `server/step/bb2016/block/StepBlockDodge.java` | `ffb-engine` | `src/step/bb2016/block/step_block_dodge.rs` | ✓ |
| `server/step/bb2016/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2016/block/step_block_roll.rs` | ✓ |
| `server/step/bb2016/block/StepBothDown.java` | `ffb-engine` | `src/step/bb2016/block/step_both_down.rs` | ✓ |
| `server/step/bb2016/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2016/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2016/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2016/block/step_followup.rs` | ✓ |
| `server/step/bb2016/end/StepFanFactor.java` | `ffb-engine` | `src/step/bb2016/end/step_fan_factor.rs` | ✓ |
| `server/step/bb2016/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2016/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2016/end/StepMvp.java` | `ffb-engine` | `src/step/bb2016/end/step_mvp.rs` | ✓ |
| `server/step/bb2016/end/StepPenaltyShootout.java` | `ffb-engine` | `src/step/bb2016/end/step_penalty_shootout.rs` | ✓ |
| `server/step/bb2016/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2016/end/step_player_loss.rs` | ✓ |
| `server/step/bb2016/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2016/end/step_winnings.rs` | ✓ |
| `server/step/bb2016/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2016/foul/step_bribes.rs` | ✓ |
| `server/step/bb2016/foul/StepEjectPlayer.java` | `ffb-engine` | `src/step/bb2016/foul/step_eject_player.rs` | ✓ |
| `server/step/bb2016/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2016/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2016/foul/StepFoul.java` | `ffb-engine` | `src/step/bb2016/foul/step_foul.rs` | ✓ |
| `server/step/bb2016/foul/StepFoulChainsaw.java` | `ffb-engine` | `src/step/bb2016/foul/step_foul_chainsaw.rs` | ✓ |
| `server/step/bb2016/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2016/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2016/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2016/move/step_end_moving.rs` | ✓ |
| `server/step/bb2016/move/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2016/move/step_end_selecting.rs` | ✓ |
| `server/step/bb2016/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2016/move/step_go_for_it.rs` | ✓ |
| `server/step/bb2016/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2016/move/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2016/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2016/move/step_init_moving.rs` | ✓ |
| `server/step/bb2016/move/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2016/move/step_init_selecting.rs` | ✓ |
| `server/step/bb2016/move/StepJump.java` | `ffb-engine` | `src/step/bb2016/move/step_jump.rs` | ✓ |
| `server/step/bb2016/move/StepMove.java` | `ffb-engine` | `src/step/bb2016/move/step_move.rs` | ✓ |
| `server/step/bb2016/move/StepMoveBallAndChain.java` | `ffb-engine` | `src/step/bb2016/move/step_move_ball_and_chain.rs` | ✓ |
| `server/step/bb2016/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2016/move/step_move_dodge.rs` | ✓ |
| `server/step/bb2016/move/StepTentacles.java` | `ffb-engine` | `src/step/bb2016/move/step_tentacles.rs` | ✓ |
| `server/step/bb2016/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2016/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2016/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepInitPassing.java` | `ffb-engine` | `src/step/bb2016/pass/step_init_passing.rs` | ✓ |
| `server/step/bb2016/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2016/pass/step_intercept.rs` | ✓ |
| `server/step/bb2016/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepPass.java` | `ffb-engine` | `src/step/bb2016/pass/step_pass.rs` | ✓ |
| `server/step/bb2016/pass/StepPassBlock.java` | `ffb-engine` | `src/step/bb2016/pass/step_pass_block.rs` | ✓ |
| `server/step/bb2016/pass/StepSafeThrow.java` | `ffb-engine` | `src/step/bb2016/pass/step_safe_throw.rs` | ✓ |
| `server/step/bb2016/special/StepEndBomb.java` | `ffb-engine` | `src/step/bb2016/special/step_end_bomb.rs` | ✓ |
| `server/step/bb2016/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2016/special/step_init_bomb.rs` | ✓ |
| `server/step/bb2016/start/StepBuyCards.java` | `ffb-engine` | `src/step/bb2016/start/step_buy_cards.rs` | ✓ |
| `server/step/bb2016/start/StepBuyInducements.java` | `ffb-engine` | `src/step/bb2016/start/step_buy_inducements.rs` | ✓ |
| `server/step/bb2016/start/StepPettyCash.java` | `ffb-engine` | `src/step/bb2016/start/step_petty_cash.rs` | ✓ |
| `server/step/bb2016/start/StepSpectators.java` | `ffb-engine` | `src/step/bb2016/start/step_spectators.rs` | ✓ |
| `server/step/bb2016/StepApothecary.java` | `ffb-engine` | `src/step/bb2016/step_apothecary.rs` | ✓ |
| `server/step/bb2016/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2016/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2016/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2016/step_blitz_turn.rs` | ✓ |
| `server/step/bb2016/StepBloodLust.java` | `ffb-engine` | `src/step/bb2016/step_blood_lust.rs` | ✓ |
| `server/step/bb2016/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2016/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2016/StepDropDivingTackler.java` | `ffb-engine` | `src/step/bb2016/step_drop_diving_tackler.rs` | ✓ |
| `server/step/bb2016/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2016/step_end_feeding.rs` | ✓ |
| `server/step/bb2016/StepEndInducement.java` | `ffb-engine` | `src/step/bb2016/step_end_inducement.rs` | ✓ |
| `server/step/bb2016/StepEndTurn.java` | `ffb-engine` | `src/step/bb2016/step_end_turn.rs` | ✓ |
| `server/step/bb2016/StepFallDown.java` | `ffb-engine` | `src/step/bb2016/step_fall_down.rs` | ✓ |
| `server/step/bb2016/StepFoulAppearance.java` | `ffb-engine` | `src/step/bb2016/step_foul_appearance.rs` | ✓ |
| `server/step/bb2016/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2016/step_init_blocking.rs` | ✓ |
| `server/step/bb2016/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2016/step_init_feeding.rs` | ✓ |
| `server/step/bb2016/StepInitInducement.java` | `ffb-engine` | `src/step/bb2016/step_init_inducement.rs` | ✓ |
| `server/step/bb2016/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2016/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2016/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2016/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2016/StepPickUp.java` | `ffb-engine` | `src/step/bb2016/step_pick_up.rs` | ✓ |
| `server/step/bb2016/StepPushback.java` | `ffb-engine` | `src/step/bb2016/step_pushback.rs` | ✓ |
| `server/step/bb2016/StepSetup.java` | `ffb-engine` | `src/step/bb2016/step_setup.rs` | ✓ |
| `server/step/bb2016/StepShadowing.java` | `ffb-engine` | `src/step/bb2016/step_shadowing.rs` | ✓ |
| `server/step/bb2016/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2016/step_special_effect.rs` | ✓ |
| `server/step/bb2016/StepStandUp.java` | `ffb-engine` | `src/step/bb2016/step_stand_up.rs` | ✓ |
| `server/step/bb2016/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2016/step_take_root.rs` | ✓ |
| `server/step/bb2016/StepWildAnimal.java` | `ffb-engine` | `src/step/bb2016/step_wild_animal.rs` | ✓ |
| `server/step/bb2016/StepWizard.java` | `ffb-engine` | `src/step/bb2016/step_wizard.rs` | ✓ |
| `server/step/bb2016/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2016/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2016/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2016/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2016/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2016/ttm/StepFumbleTtmPass.java` | `ffb-engine` | `src/step/bb2016/ttm/step_fumble_ttm_pass.rs` | ✓ |
| `server/step/bb2016/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2016/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2016/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2016/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2016/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2016/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2016/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/bb2020/ (89 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2020/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2020/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2020/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2020/block/step_block_choice.rs` | ✓ |
| `server/step/bb2020/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2020/block/step_block_roll.rs` | ✓ |
| `server/step/bb2020/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2020/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2020/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2020/block/step_followup.rs` | ✓ |
| `server/step/bb2020/block/StepHitAndRun.java` | `ffb-engine` | `src/step/bb2020/block/step_hit_and_run.rs` | ✓ |
| `server/step/bb2020/block/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2020/block/step_init_blocking.rs` | ✓ |
| `server/step/bb2020/block/StepPushback.java` | `ffb-engine` | `src/step/bb2020/block/step_pushback.rs` | ✓ |
| `server/step/bb2020/block/StepTrickster.java` | `ffb-engine` | `src/step/bb2020/block/step_trickster.rs` | ✓ |
| `server/step/bb2020/end/StepAssignTouchdowns.java` | `ffb-engine` | `src/step/bb2020/end/step_assign_touchdowns.rs` | ✓ |
| `server/step/bb2020/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2020/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2020/end/StepMvp.java` | `ffb-engine` | `src/step/bb2020/end/step_mvp.rs` | ✓ |
| `server/step/bb2020/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2020/end/step_player_loss.rs` | ✓ |
| `server/step/bb2020/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2020/end/step_winnings.rs` | ✓ |
| `server/step/bb2020/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2020/foul/step_bribes.rs` | ✓ |
| `server/step/bb2020/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2020/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2020/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2020/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2020/gaze/StepSelectGazeTarget.java` | `ffb-engine` | `src/step/bb2020/gaze/step_select_gaze_target.rs` | ✓ |
| `server/step/bb2020/gaze/StepSelectGazeTargetEnd.java` | `ffb-engine` | `src/step/bb2020/gaze/step_select_gaze_target_end.rs` | ✓ |
| `server/step/bb2020/inducements/StepEndInducement.java` | `ffb-engine` | `src/step/bb2020/inducements/step_end_inducement.rs` | ✓ |
| `server/step/bb2020/inducements/StepInitInducement.java` | `ffb-engine` | `src/step/bb2020/inducements/step_init_inducement.rs` | ✓ |
| `server/step/bb2020/inducements/StepWeatherMage.java` | `ffb-engine` | `src/step/bb2020/inducements/step_weather_mage.rs` | ✓ |
| `server/step/bb2020/kickoff/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2020/kickoff/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2020/kickoff/StepSetup.java` | `ffb-engine` | `src/step/bb2020/kickoff/step_setup.rs` | ✓ |
| `server/step/bb2020/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2020/move/step_end_moving.rs` | ✓ |
| `server/step/bb2020/move/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2020/move/step_end_selecting.rs` | ✓ |
| `server/step/bb2020/move/StepFallDown.java` | `ffb-engine` | `src/step/bb2020/move/step_fall_down.rs` | ✓ |
| `server/step/bb2020/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2020/move/step_go_for_it.rs` | ✓ |
| `server/step/bb2020/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2020/move/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2020/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2020/move/step_init_moving.rs` | ✓ |
| `server/step/bb2020/move/StepJump.java` | `ffb-engine` | `src/step/bb2020/move/step_jump.rs` | ✓ |
| `server/step/bb2020/move/StepMove.java` | `ffb-engine` | `src/step/bb2020/move/step_move.rs` | ✓ |
| `server/step/bb2020/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2020/move/step_move_dodge.rs` | ✓ |
| `server/step/bb2020/move/StepPickUp.java` | `ffb-engine` | `src/step/bb2020/move/step_pick_up.rs` | ✓ |
| `server/step/bb2020/move/StepShadowing.java` | `ffb-engine` | `src/step/bb2020/move/step_shadowing.rs` | ✓ |
| `server/step/bb2020/move/StepStandUp.java` | `ffb-engine` | `src/step/bb2020/move/step_stand_up.rs` | ✓ |
| `server/step/bb2020/multiblock/StepApothecaryMultiple.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_apothecary_multiple.rs` | ✓ |
| `server/step/bb2020/multiblock/StepBlockRollMultiple.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_block_roll_multiple.rs` | ✓ |
| `server/step/bb2020/multiblock/StepMultipleBlockFork.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_multiple_block_fork.rs` | ✓ |
| `server/step/bb2020/multiblock/StepReportStabInjury.java` | `ffb-engine` | `src/step/bb2020/multiblock/step_report_stab_injury.rs` | ✓ |
| `server/step/bb2020/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2020/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2020/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2020/pass/step_intercept.rs` | ✓ |
| `server/step/bb2020/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepPass.java` | `ffb-engine` | `src/step/bb2020/pass/step_pass.rs` | ✓ |
| `server/step/bb2020/pass/StepResolvePass.java` | `ffb-engine` | `src/step/bb2020/pass/step_resolve_pass.rs` | ✓ |
| `server/step/bb2020/shared/StepBloodLust.java` | `ffb-engine` | `src/step/bb2020/shared/step_blood_lust.rs` | ✓ |
| `server/step/bb2020/shared/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2020/shared/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2020/shared/StepCheckStalling.java` | `ffb-engine` | `src/step/bb2020/shared/step_check_stalling.rs` | ✓ |
| `server/step/bb2020/shared/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2020/shared/step_end_feeding.rs` | ✓ |
| `server/step/bb2020/shared/StepInitActivation.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_activation.rs` | ✓ |
| `server/step/bb2020/shared/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_feeding.rs` | ✓ |
| `server/step/bb2020/shared/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2020/shared/step_init_selecting.rs` | ✓ |
| `server/step/bb2020/shared/StepPlaceBall.java` | `ffb-engine` | `src/step/bb2020/shared/step_place_ball.rs` | ✓ |
| `server/step/bb2020/shared/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2020/shared/step_take_root.rs` | ✓ |
| `server/step/bb2020/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2020/special/step_init_bomb.rs` | ✓ |
| `server/step/bb2020/start/StepBuyCardsAndInducements.java` | `ffb-engine` | `src/step/bb2020/start/step_buy_cards_and_inducements.rs` | ✓ |
| `server/step/bb2020/StepApothecary.java` | `ffb-engine` | `src/step/bb2020/step_apothecary.rs` | ✓ |
| `server/step/bb2020/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2020/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2020/StepBalefulHex.java` | `ffb-engine` | `src/step/bb2020/step_baleful_hex.rs` | ✓ |
| `server/step/bb2020/StepBlackInk.java` | `ffb-engine` | `src/step/bb2020/step_black_ink.rs` | ✓ |
| `server/step/bb2020/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2020/step_blitz_turn.rs` | ✓ |
| `server/step/bb2020/StepBreatheFire.java` | `ffb-engine` | `src/step/bb2020/step_breathe_fire.rs` | ✓ |
| `server/step/bb2020/StepCatchOfTheDay.java` | `ffb-engine` | `src/step/bb2020/step_catch_of_the_day.rs` | ✓ |
| `server/step/bb2020/StepEndFuriousOutburst.java` | `ffb-engine` | `src/step/bb2020/step_end_furious_outburst.rs` | ✓ |
| `server/step/bb2020/StepEndTurn.java` | `ffb-engine` | `src/step/bb2020/step_end_turn.rs` | ✓ |
| `server/step/bb2020/StepHandleDropPlayerContext.java` | `ffb-engine` | `src/step/bb2020/step_handle_drop_player_context.rs` | ✓ |
| `server/step/bb2020/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2020/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2020/StepLookIntoMyEyes.java` | `ffb-engine` | `src/step/bb2020/step_look_into_my_eyes.rs` | ✓ |
| `server/step/bb2020/StepPrayer.java` | `ffb-engine` | `src/step/bb2020/step_prayer.rs` | ✓ |
| `server/step/bb2020/StepPrayers.java` | `ffb-engine` | `src/step/bb2020/step_prayers.rs` | ✓ |
| `server/step/bb2020/StepRaidingParty.java` | `ffb-engine` | `src/step/bb2020/step_raiding_party.rs` | ✓ |
| `server/step/bb2020/StepSelectBlitzTarget.java` | `ffb-engine` | `src/step/bb2020/step_select_blitz_target.rs` | ✓ |
| `server/step/bb2020/StepSetActingPlayerAndTeam.java` | `ffb-engine` | `src/step/bb2020/step_set_acting_player_and_team.rs` | ✓ |
| `server/step/bb2020/StepSetActingTeam.java` | `ffb-engine` | `src/step/bb2020/step_set_acting_team.rs` | ✓ |
| `server/step/bb2020/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2020/step_special_effect.rs` | ✓ |
| `server/step/bb2020/StepStallingPlayer.java` | `ffb-engine` | `src/step/bb2020/step_stalling_player.rs` | ✓ |
| `server/step/bb2020/StepStateMultipleRolls.java` | `ffb-engine` | `src/step/bb2020/step_state_multiple_rolls.rs` | ✓ |
| `server/step/bb2020/StepThenIStartedBlastin.java` | `ffb-engine` | `src/step/bb2020/step_then_i_started_blastin.rs` | ✓ |
| `server/step/bb2020/StepTreacherous.java` | `ffb-engine` | `src/step/bb2020/step_treacherous.rs` | ✓ |
| `server/step/bb2020/StepWisdomOfTheWhiteDwarf.java` | `ffb-engine` | `src/step/bb2020/step_wisdom_of_the_white_dwarf.rs` | ✓ |
| `server/step/bb2020/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2020/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2020/ttm/StepDispatchScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_dispatch_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2020/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2020/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2020/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2020/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2020/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2020/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2020/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/bb2025/ (109 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/bb2025/block/StepBlockChainsaw.java` | `ffb-engine` | `src/step/bb2025/block/step_block_chainsaw.rs` | ✓ |
| `server/step/bb2025/block/StepBlockChoice.java` | `ffb-engine` | `src/step/bb2025/block/step_block_choice.rs` | ✓ |
| `server/step/bb2025/block/StepBlockRoll.java` | `ffb-engine` | `src/step/bb2025/block/step_block_roll.rs` | ✓ |
| `server/step/bb2025/block/StepBreatheFire.java` | `ffb-engine` | `src/step/bb2025/block/step_breathe_fire.rs` | ✓ |
| `server/step/bb2025/block/StepChomp.java` | `ffb-engine` | `src/step/bb2025/block/step_chomp.rs` | ✓ |
| `server/step/bb2025/block/StepEndBlocking.java` | `ffb-engine` | `src/step/bb2025/block/step_end_blocking.rs` | ✓ |
| `server/step/bb2025/block/StepFollowup.java` | `ffb-engine` | `src/step/bb2025/block/step_followup.rs` | ✓ |
| `server/step/bb2025/block/StepHitAndRun.java` | `ffb-engine` | `src/step/bb2025/block/step_hit_and_run.rs` | ✓ |
| `server/step/bb2025/block/StepInitBlocking.java` | `ffb-engine` | `src/step/bb2025/block/step_init_blocking.rs` | ✓ |
| `server/step/bb2025/block/StepPushback.java` | `ffb-engine` | `src/step/bb2025/block/step_pushback.rs` | ✓ |
| `server/step/bb2025/block/StepTrickster.java` | `ffb-engine` | `src/step/bb2025/block/step_trickster.rs` | ✓ |
| `server/step/bb2025/command/AnimalSavageryCancelActionCommand.java` | `ffb-engine` | `src/step/bb2025/command/animal_savagery_cancel_action_command.rs` | ✓ |
| `server/step/bb2025/command/AnimalSavageryControlCommand.java` | `ffb-engine` | `src/step/bb2025/command/animal_savagery_control_command.rs` | ✓ |
| `server/step/bb2025/command/DropPlayerCommand.java` | `ffb-engine` | `src/step/bb2025/command/drop_player_command.rs` | ✓ |
| `server/step/bb2025/command/DropPlayerFromBombCommand.java` | `ffb-engine` | `src/step/bb2025/command/drop_player_from_bomb_command.rs` | ✓ |
| `server/step/bb2025/command/HitPlayerTurnOverCommand.java` | `ffb-engine` | `src/step/bb2025/command/hit_player_turn_over_command.rs` | ✓ |
| `server/step/bb2025/command/RightStuffCommand.java` | `ffb-engine` | `src/step/bb2025/command/right_stuff_command.rs` | ✓ |
| `server/step/bb2025/command/StandingUpCommand.java` | `ffb-engine` | `src/step/bb2025/command/standing_up_command.rs` | ✓ |
| `server/step/bb2025/end/StepInitEndGame.java` | `ffb-engine` | `src/step/bb2025/end/step_init_end_game.rs` | ✓ |
| `server/step/bb2025/end/StepMvp.java` | `ffb-engine` | `src/step/bb2025/end/step_mvp.rs` | ✓ |
| `server/step/bb2025/end/StepPlayerLoss.java` | `ffb-engine` | `src/step/bb2025/end/step_player_loss.rs` | ✓ |
| `server/step/bb2025/end/StepWinnings.java` | `ffb-engine` | `src/step/bb2025/end/step_winnings.rs` | ✓ |
| `server/step/bb2025/foul/StepBribes.java` | `ffb-engine` | `src/step/bb2025/foul/step_bribes.rs` | ✓ |
| `server/step/bb2025/foul/StepEndFouling.java` | `ffb-engine` | `src/step/bb2025/foul/step_end_fouling.rs` | ✓ |
| `server/step/bb2025/foul/StepInitFouling.java` | `ffb-engine` | `src/step/bb2025/foul/step_init_fouling.rs` | ✓ |
| `server/step/bb2025/inducements/StepEndInducement.java` | `ffb-engine` | `src/step/bb2025/inducements/step_end_inducement.rs` | ✓ |
| `server/step/bb2025/inducements/StepInitInducement.java` | `ffb-engine` | `src/step/bb2025/inducements/step_init_inducement.rs` | ✓ |
| `server/step/bb2025/inducements/StepThrowARock.java` | `ffb-engine` | `src/step/bb2025/inducements/step_throw_a_rock.rs` | ✓ |
| `server/step/bb2025/inducements/StepWeatherMage.java` | `ffb-engine` | `src/step/bb2025/inducements/step_weather_mage.rs` | ✓ |
| `server/step/bb2025/kickoff/StepApplyKickoffResult.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_apply_kickoff_result.rs` | ✓ |
| `server/step/bb2025/kickoff/StepBlitzTurn.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_blitz_turn.rs` | ✓ |
| `server/step/bb2025/kickoff/StepInitKickoff.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_init_kickoff.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoff.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffResultRoll.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_result_roll.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffScatterRoll.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_scatter_roll.rs` | ✓ |
| `server/step/bb2025/kickoff/StepKickoffScatterRollAskAfter.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_kickoff_scatter_roll_ask_after.rs` | ✓ |
| `server/step/bb2025/kickoff/StepSetup.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_setup.rs` | ✓ |
| `server/step/bb2025/kickoff/StepSwarming.java` | `ffb-engine` | `src/step/bb2025/kickoff/step_swarming.rs` | ✓ |
| `server/step/bb2025/move/StepEndMoving.java` | `ffb-engine` | `src/step/bb2025/move_/step_end_moving.rs` | ✓ |
| `server/step/bb2025/move/StepFallDown.java` | `ffb-engine` | `src/step/bb2025/move_/step_fall_down.rs` | ✓ |
| `server/step/bb2025/move/StepGoForIt.java` | `ffb-engine` | `src/step/bb2025/move_/step_go_for_it.rs` | ✓ |
| `server/step/bb2025/move/StepHypnoticGaze.java` | `ffb-engine` | `src/step/bb2025/move_/step_hypnotic_gaze.rs` | ✓ |
| `server/step/bb2025/move/StepInitMoving.java` | `ffb-engine` | `src/step/bb2025/move_/step_init_moving.rs` | ✓ |
| `server/step/bb2025/move/StepJump.java` | `ffb-engine` | `src/step/bb2025/move_/step_jump.rs` | ✓ |
| `server/step/bb2025/move/StepMove.java` | `ffb-engine` | `src/step/bb2025/move_/step_move.rs` | ✓ |
| `server/step/bb2025/move/StepMoveDodge.java` | `ffb-engine` | `src/step/bb2025/move_/step_move_dodge.rs` | ✓ |
| `server/step/bb2025/move/StepPickUp.java` | `ffb-engine` | `src/step/bb2025/move_/step_pick_up.rs` | ✓ |
| `server/step/bb2025/move/StepShadowing.java` | `ffb-engine` | `src/step/bb2025/move_/step_shadowing.rs` | ✓ |
| `server/step/bb2025/move/StepStandUp.java` | `ffb-engine` | `src/step/bb2025/move_/step_stand_up.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepApothecaryMultiple.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_apothecary_multiple.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepBlockRollMultiple.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_block_roll_multiple.rs` | ✓ |
| `server/step/bb2025/mutliblock/StepMultipleBlockFork.java` | `ffb-engine` | `src/step/bb2025/mutliblock/step_multiple_block_fork.rs` | ✓ |
| `server/step/bb2025/pass/StepEndPassing.java` | `ffb-engine` | `src/step/bb2025/pass/step_end_passing.rs` | ✓ |
| `server/step/bb2025/pass/StepHailMaryPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_hail_mary_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepHandOver.java` | `ffb-engine` | `src/step/bb2025/pass/step_hand_over.rs` | ✓ |
| `server/step/bb2025/pass/StepIntercept.java` | `ffb-engine` | `src/step/bb2025/pass/step_intercept.rs` | ✓ |
| `server/step/bb2025/pass/StepMissedPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_missed_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepPass.java` | `ffb-engine` | `src/step/bb2025/pass/step_pass.rs` | ✓ |
| `server/step/bb2025/pass/StepResolvePass.java` | `ffb-engine` | `src/step/bb2025/pass/step_resolve_pass.rs` | ✓ |
| `server/step/bb2025/punt/StepEndPunt.java` | `ffb-engine` | `src/step/bb2025/punt/step_end_punt.rs` | ✓ |
| `server/step/bb2025/punt/StepInitPunt.java` | `ffb-engine` | `src/step/bb2025/punt/step_init_punt.rs` | ✓ |
| `server/step/bb2025/punt/StepPuntDirection.java` | `ffb-engine` | `src/step/bb2025/punt/step_punt_direction.rs` | ✓ |
| `server/step/bb2025/punt/StepPuntDistance.java` | `ffb-engine` | `src/step/bb2025/punt/step_punt_distance.rs` | ✓ |
| `server/step/bb2025/shared/StallingExtension.java` | `ffb-engine` | `src/step/bb2025/shared/stalling_extension.rs` | ✓ |
| `server/step/bb2025/shared/StepApothecary.java` | `ffb-engine` | `src/step/bb2025/shared/step_apothecary.rs` | ✓ |
| `server/step/bb2025/shared/StepBloodLust.java` | `ffb-engine` | `src/step/bb2025/shared/step_blood_lust.rs` | ✓ |
| `server/step/bb2025/shared/StepCatchScatterThrowIn.java` | `ffb-engine` | `src/step/bb2025/shared/step_catch_scatter_throw_in.rs` | ✓ |
| `server/step/bb2025/shared/StepDropFallingPlayers.java` | `ffb-engine` | `src/step/bb2025/shared/step_drop_falling_players.rs` | ✓ |
| `server/step/bb2025/shared/StepEndFeeding.java` | `ffb-engine` | `src/step/bb2025/shared/step_end_feeding.rs` | ✓ |
| `server/step/bb2025/shared/StepEndSelecting.java` | `ffb-engine` | `src/step/bb2025/shared/step_end_selecting.rs` | ✓ |
| `server/step/bb2025/shared/StepForgoneStalling.java` | `ffb-engine` | `src/step/bb2025/shared/step_forgone_stalling.rs` | ✓ |
| `server/step/bb2025/shared/StepGettingEven.java` | `ffb-engine` | `src/step/bb2025/shared/step_getting_even.rs` | ✓ |
| `server/step/bb2025/shared/StepHandleDropPlayerContext.java` | `ffb-engine` | `src/step/bb2025/shared/step_handle_drop_player_context.rs` | ✓ |
| `server/step/bb2025/shared/StepInitActivation.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_activation.rs` | ✓ |
| `server/step/bb2025/shared/StepInitFeeding.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_feeding.rs` | ✓ |
| `server/step/bb2025/shared/StepInitSelecting.java` | `ffb-engine` | `src/step/bb2025/shared/step_init_selecting.rs` | ✓ |
| `server/step/bb2025/shared/StepPlaceBall.java` | `ffb-engine` | `src/step/bb2025/shared/step_place_ball.rs` | ✓ |
| `server/step/bb2025/shared/StepStallingPlayer.java` | `ffb-engine` | `src/step/bb2025/shared/step_stalling_player.rs` | ✓ |
| `server/step/bb2025/shared/StepSteadyFooting.java` | `ffb-engine` | `src/step/bb2025/shared/step_steady_footing.rs` | ✓ |
| `server/step/bb2025/shared/StepTakeRoot.java` | `ffb-engine` | `src/step/bb2025/shared/step_take_root.rs` | ✓ |
| `server/step/bb2025/special/StepInitBomb.java` | `ffb-engine` | `src/step/bb2025/special/step_init_bomb.rs` | ✓ |
| `server/step/bb2025/special/StepRecheckExplodeSkill.java` | `ffb-engine` | `src/step/bb2025/special/step_recheck_explode_skill.rs` | ✓ |
| `server/step/bb2025/special/StepResolveBomb.java` | `ffb-engine` | `src/step/bb2025/special/step_resolve_bomb.rs` | ✓ |
| `server/step/bb2025/special/StepSpecialEffect.java` | `ffb-engine` | `src/step/bb2025/special/step_special_effect.rs` | ✓ |
| `server/step/bb2025/start/StepBuyInducements.java` | `ffb-engine` | `src/step/bb2025/start/step_buy_inducements.rs` | ✓ |
| `server/step/bb2025/start/StepMasterChef.java` | `ffb-engine` | `src/step/bb2025/start/step_master_chef.rs` | ✓ |
| `server/step/bb2025/start/StepPrayers.java` | `ffb-engine` | `src/step/bb2025/start/step_prayers.rs` | ✓ |
| `server/step/bb2025/StepAutoGazeZoat.java` | `ffb-engine` | `src/step/bb2025/step_auto_gaze_zoat.rs` | ✓ |
| `server/step/bb2025/StepBalefulHex.java` | `ffb-engine` | `src/step/bb2025/step_baleful_hex.rs` | ✓ |
| `server/step/bb2025/StepBlackInk.java` | `ffb-engine` | `src/step/bb2025/step_black_ink.rs` | ✓ |
| `server/step/bb2025/StepCatchOfTheDay.java` | `ffb-engine` | `src/step/bb2025/step_catch_of_the_day.rs` | ✓ |
| `server/step/bb2025/StepEndFuriousOutburst.java` | `ffb-engine` | `src/step/bb2025/step_end_furious_outburst.rs` | ✓ |
| `server/step/bb2025/StepEndTurn.java` | `ffb-engine` | `src/step/bb2025/step_end_turn.rs` | ✓ |
| `server/step/bb2025/StepLookIntoMyEyes.java` | `ffb-engine` | `src/step/bb2025/step_look_into_my_eyes.rs` | ✓ |
| `server/step/bb2025/StepPrayer.java` | `ffb-engine` | `src/step/bb2025/step_prayer.rs` | ✓ |
| `server/step/bb2025/StepRaidingParty.java` | `ffb-engine` | `src/step/bb2025/step_raiding_party.rs` | ✓ |
| `server/step/bb2025/StepSelectBlitzTarget.java` | `ffb-engine` | `src/step/bb2025/step_select_blitz_target.rs` | ✓ |
| `server/step/bb2025/StepThenIStartedBlastin.java` | `ffb-engine` | `src/step/bb2025/step_then_i_started_blastin.rs` | ✓ |
| `server/step/bb2025/StepTreacherous.java` | `ffb-engine` | `src/step/bb2025/step_treacherous.rs` | ✓ |
| `server/step/bb2025/StepWisdomOfTheWhiteDwarf.java` | `ffb-engine` | `src/step/bb2025/step_wisdom_of_the_white_dwarf.rs` | ✓ |
| `server/step/bb2025/ttm/StepAlwaysHungry.java` | `ffb-engine` | `src/step/bb2025/ttm/step_always_hungry.rs` | ✓ |
| `server/step/bb2025/ttm/StepDispatchScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_dispatch_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepEndScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_end_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepEndThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_end_throw_team_mate.rs` | ✓ |
| `server/step/bb2025/ttm/StepInitScatterPlayer.java` | `ffb-engine` | `src/step/bb2025/ttm/step_init_scatter_player.rs` | ✓ |
| `server/step/bb2025/ttm/StepInitThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_init_throw_team_mate.rs` | ✓ |
| `server/step/bb2025/ttm/StepRightStuff.java` | `ffb-engine` | `src/step/bb2025/ttm/step_right_stuff.rs` | ✓ |
| `server/step/bb2025/ttm/StepSwoop.java` | `ffb-engine` | `src/step/bb2025/ttm/step_swoop.rs` | ✓ |
| `server/step/bb2025/ttm/StepThrowTeamMate.java` | `ffb-engine` | `src/step/bb2025/ttm/step_throw_team_mate.rs` | ✓ |

### server/step/game/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/game/end/StepEndGame.java` | `ffb-engine` | `src/step/game/end/step_end_game.rs` | ✓ |
| `server/step/game/start/StepInitStartGame.java` | `ffb-engine` | `src/step/game/start/step_init_start_game.rs` | ✓ |
| `server/step/game/start/StepWeather.java` | `ffb-engine` | `src/step/game/start/step_weather.rs` | ✓ |
| `server/step/game/start/UtilInducementSequence.java` | `ffb-engine` | `src/step/game/start/util_inducement_sequence.rs` | ✓ |

### server/step/generator/ (114 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/generator/AutoGazeZoat.java` | `ffb-engine` | `src/step/generator/auto_gaze_zoat.rs` | ✓ |
| `server/step/generator/BalefulHex.java` | `ffb-engine` | `src/step/generator/baleful_hex.rs` | ✓ |
| `server/step/generator/bb2016/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2016/blitz_block.rs` | ✓ |
| `server/step/generator/bb2016/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2016/blitz_move.rs` | ✓ |
| `server/step/generator/bb2016/Block.java` | `ffb-engine` | `src/step/generator/bb2016/block.rs` | ✓ |
| `server/step/generator/bb2016/Bomb.java` | `ffb-engine` | `src/step/generator/bb2016/bomb.rs` | ✓ |
| `server/step/generator/bb2016/EndGame.java` | `ffb-engine` | `src/step/generator/bb2016/end_game.rs` | ✓ |
| `server/step/generator/bb2016/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2016/end_player_action.rs` | ✓ |
| `server/step/generator/bb2016/Foul.java` | `ffb-engine` | `src/step/generator/bb2016/foul.rs` | ✓ |
| `server/step/generator/bb2016/KickTeamMate.java` | `ffb-engine` | `src/step/generator/bb2016/kick_team_mate.rs` | ✓ |
| `server/step/generator/bb2016/Move.java` | `ffb-engine` | `src/step/generator/bb2016/move.rs` | ✓ |
| `server/step/generator/bb2016/Pass.java` | `ffb-engine` | `src/step/generator/bb2016/pass.rs` | ✓ |
| `server/step/generator/bb2016/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2016/scatter_player.rs` | ✓ |
| `server/step/generator/bb2016/Select.java` | `ffb-engine` | `src/step/generator/bb2016/select.rs` | ✓ |
| `server/step/generator/bb2016/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2016/special_effect.rs` | ✓ |
| `server/step/generator/bb2016/StartGame.java` | `ffb-engine` | `src/step/generator/bb2016/start_game.rs` | ✓ |
| `server/step/generator/bb2016/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2016/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2020/BalefulHex.java` | `ffb-engine` | `src/step/generator/bb2020/baleful_hex.rs` | ✓ |
| `server/step/generator/bb2020/BlackInk.java` | `ffb-engine` | `src/step/generator/bb2020/black_ink.rs` | ✓ |
| `server/step/generator/bb2020/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2020/blitz_block.rs` | ✓ |
| `server/step/generator/bb2020/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2020/blitz_move.rs` | ✓ |
| `server/step/generator/bb2020/Block.java` | `ffb-engine` | `src/step/generator/bb2020/block.rs` | ✓ |
| `server/step/generator/bb2020/Bomb.java` | `ffb-engine` | `src/step/generator/bb2020/bomb.rs` | ✓ |
| `server/step/generator/bb2020/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/bb2020/catch_of_the_day.rs` | ✓ |
| `server/step/generator/bb2020/EndGame.java` | `ffb-engine` | `src/step/generator/bb2020/end_game.rs` | ✓ |
| `server/step/generator/bb2020/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2020/end_player_action.rs` | ✓ |
| `server/step/generator/bb2020/Foul.java` | `ffb-engine` | `src/step/generator/bb2020/foul.rs` | ✓ |
| `server/step/generator/bb2020/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/bb2020/furious_outburst.rs` | ✓ |
| `server/step/generator/bb2020/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/bb2020/look_into_my_eyes.rs` | ✓ |
| `server/step/generator/bb2020/Move.java` | `ffb-engine` | `src/step/generator/bb2020/move.rs` | ✓ |
| `server/step/generator/bb2020/MultiBlock.java` | `ffb-engine` | `src/step/generator/bb2020/multi_block.rs` | ✓ |
| `server/step/generator/bb2020/Pass.java` | `ffb-engine` | `src/step/generator/bb2020/pass.rs` | ✓ |
| `server/step/generator/bb2020/RaidingParty.java` | `ffb-engine` | `src/step/generator/bb2020/raiding_party.rs` | ✓ |
| `server/step/generator/bb2020/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2020/scatter_player.rs` | ✓ |
| `server/step/generator/bb2020/Select.java` | `ffb-engine` | `src/step/generator/bb2020/select.rs` | ✓ |
| `server/step/generator/bb2020/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/bb2020/select_blitz_target.rs` | ✓ |
| `server/step/generator/bb2020/SelectGazeTarget.java` | `ffb-engine` | `src/step/generator/bb2020/select_gaze_target.rs` | ✓ |
| `server/step/generator/bb2020/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2020/special_effect.rs` | ✓ |
| `server/step/generator/bb2020/StartGame.java` | `ffb-engine` | `src/step/generator/bb2020/start_game.rs` | ✓ |
| `server/step/generator/bb2020/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/bb2020/then_i_started_blastin.rs` | ✓ |
| `server/step/generator/bb2020/ThrowKeg.java` | `ffb-engine` | `src/step/generator/bb2020/throw_keg.rs` | ✓ |
| `server/step/generator/bb2020/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2020/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2020/Treacherous.java` | `ffb-engine` | `src/step/generator/bb2020/treacherous.rs` | ✓ |
| `server/step/generator/bb2025/ActivationSequenceBuilder.java` | `ffb-engine` | `src/step/generator/bb2025/activation_sequence_builder.rs` | ✓ |
| `server/step/generator/bb2025/AutoGazeZoat.java` | `ffb-engine` | `src/step/generator/bb2025/auto_gaze_zoat.rs` | ✓ |
| `server/step/generator/bb2025/BalefulHex.java` | `ffb-engine` | `src/step/generator/bb2025/baleful_hex.rs` | ✓ |
| `server/step/generator/bb2025/BlackInk.java` | `ffb-engine` | `src/step/generator/bb2025/black_ink.rs` | ✓ |
| `server/step/generator/bb2025/BlitzBlock.java` | `ffb-engine` | `src/step/generator/bb2025/blitz_block.rs` | ✓ |
| `server/step/generator/bb2025/BlitzMove.java` | `ffb-engine` | `src/step/generator/bb2025/blitz_move.rs` | ✓ |
| `server/step/generator/bb2025/Block.java` | `ffb-engine` | `src/step/generator/bb2025/block.rs` | ✓ |
| `server/step/generator/bb2025/Bomb.java` | `ffb-engine` | `src/step/generator/bb2025/bomb.rs` | ✓ |
| `server/step/generator/bb2025/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/bb2025/catch_of_the_day.rs` | ✓ |
| `server/step/generator/bb2025/EndGame.java` | `ffb-engine` | `src/step/generator/bb2025/end_game.rs` | ✓ |
| `server/step/generator/bb2025/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/bb2025/end_player_action.rs` | ✓ |
| `server/step/generator/bb2025/EndTurn.java` | `ffb-engine` | `src/step/generator/bb2025/end_turn.rs` | ✓ |
| `server/step/generator/bb2025/Foul.java` | `ffb-engine` | `src/step/generator/bb2025/foul.rs` | ✓ |
| `server/step/generator/bb2025/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/bb2025/furious_outburst.rs` | ✓ |
| `server/step/generator/bb2025/Kickoff.java` | `ffb-engine` | `src/step/generator/bb2025/kickoff.rs` | ✓ |
| `server/step/generator/bb2025/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/bb2025/look_into_my_eyes.rs` | ✓ |
| `server/step/generator/bb2025/Move.java` | `ffb-engine` | `src/step/generator/bb2025/move.rs` | ✓ |
| `server/step/generator/bb2025/MultiBlock.java` | `ffb-engine` | `src/step/generator/bb2025/multi_block.rs` | ✓ |
| `server/step/generator/bb2025/Pass.java` | `ffb-engine` | `src/step/generator/bb2025/pass.rs` | ✓ |
| `server/step/generator/bb2025/Punt.java` | `ffb-engine` | `src/step/generator/bb2025/punt.rs` | ✓ |
| `server/step/generator/bb2025/RaidingParty.java` | `ffb-engine` | `src/step/generator/bb2025/raiding_party.rs` | ✓ |
| `server/step/generator/bb2025/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/bb2025/scatter_player.rs` | ✓ |
| `server/step/generator/bb2025/Select.java` | `ffb-engine` | `src/step/generator/bb2025/select.rs` | ✓ |
| `server/step/generator/bb2025/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/bb2025/select_blitz_target.rs` | ✓ |
| `server/step/generator/bb2025/SpecialEffect.java` | `ffb-engine` | `src/step/generator/bb2025/special_effect.rs` | ✓ |
| `server/step/generator/bb2025/StartGame.java` | `ffb-engine` | `src/step/generator/bb2025/start_game.rs` | ✓ |
| `server/step/generator/bb2025/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/bb2025/then_i_started_blastin.rs` | ✓ |
| `server/step/generator/bb2025/ThrowARock.java` | `ffb-engine` | `src/step/generator/bb2025/throw_a_rock.rs` | ✓ |
| `server/step/generator/bb2025/ThrowKeg.java` | `ffb-engine` | `src/step/generator/bb2025/throw_keg.rs` | ✓ |
| `server/step/generator/bb2025/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/bb2025/throw_team_mate.rs` | ✓ |
| `server/step/generator/bb2025/Treacherous.java` | `ffb-engine` | `src/step/generator/bb2025/treacherous.rs` | ✓ |
| `server/step/generator/BlackInk.java` | `ffb-engine` | `src/step/generator/black_ink.rs` | ✓ |
| `server/step/generator/BlitzBlock.java` | `ffb-engine` | `src/step/generator/blitz_block.rs` | ✓ |
| `server/step/generator/BlitzMove.java` | `ffb-engine` | `src/step/generator/blitz_move.rs` | ✓ |
| `server/step/generator/Block.java` | `ffb-engine` | `src/step/generator/block.rs` | ✓ |
| `server/step/generator/CatchOfTheDay.java` | `ffb-engine` | `src/step/generator/catch_of_the_day.rs` | ✓ |
| `server/step/generator/common/Inducement.java` | `ffb-engine` | `src/step/generator/common/inducement.rs` | ✓ |
| `server/step/generator/common/RiotousRookies.java` | `ffb-engine` | `src/step/generator/common/riotous_rookies.rs` | ✓ |
| `server/step/generator/common/SpikedBallApo.java` | `ffb-engine` | `src/step/generator/common/spiked_ball_apo.rs` | ✓ |
| `server/step/generator/common/Wizard.java` | `ffb-engine` | `src/step/generator/common/wizard.rs` | ✓ |
| `server/step/generator/EndGame.java` | `ffb-engine` | `src/step/generator/end_game.rs` | ✓ |
| `server/step/generator/EndPlayerAction.java` | `ffb-engine` | `src/step/generator/end_player_action.rs` | ✓ |
| `server/step/generator/EndTurn.java` | `ffb-engine` | `src/step/generator/end_turn.rs` | ✓ |
| `server/step/generator/Foul.java` | `ffb-engine` | `src/step/generator/foul.rs` | ✓ |
| `server/step/generator/FuriousOutburst.java` | `ffb-engine` | `src/step/generator/furious_outburst.rs` | ✓ |
| `server/step/generator/Kickoff.java` | `ffb-engine` | `src/step/generator/kickoff.rs` | ✓ |
| `server/step/generator/KickTeamMate.java` | `ffb-engine` | `src/step/generator/kick_team_mate.rs` | ✓ |
| `server/step/generator/LookIntoMyEyes.java` | `ffb-engine` | `src/step/generator/look_into_my_eyes.rs` | ✓ |
| `server/step/generator/mixed/Card.java` | `ffb-engine` | `src/step/generator/mixed/card.rs` | ✓ |
| `server/step/generator/mixed/EndTurn.java` | `ffb-engine` | `src/step/generator/mixed/end_turn.rs` | ✓ |
| `server/step/generator/mixed/Kickoff.java` | `ffb-engine` | `src/step/generator/mixed/kickoff.rs` | ✓ |
| `server/step/generator/mixed/PileDriver.java` | `ffb-engine` | `src/step/generator/mixed/pile_driver.rs` | ✓ |
| `server/step/generator/mixed/QuickBite.java` | `ffb-engine` | `src/step/generator/mixed/quick_bite.rs` | ✓ |
| `server/step/generator/Move.java` | `ffb-engine` | `src/step/generator/move.rs` | ✓ |
| `server/step/generator/Pass.java` | `ffb-engine` | `src/step/generator/pass.rs` | ✓ |
| `server/step/generator/PileDriver.java` | `ffb-engine` | `src/step/generator/pile_driver.rs` | ✓ |
| `server/step/generator/Punt.java` | `ffb-engine` | `src/step/generator/punt.rs` | ✓ |
| `server/step/generator/QuickBite.java` | `ffb-engine` | `src/step/generator/quick_bite.rs` | ✓ |
| `server/step/generator/RadingParty.java` | `ffb-engine` | `src/step/generator/rading_party.rs` | ✓ |
| `server/step/generator/ScatterPlayer.java` | `ffb-engine` | `src/step/generator/scatter_player.rs` | ✓ |
| `server/step/generator/Select.java` | `ffb-engine` | `src/step/generator/select.rs` | ✓ |
| `server/step/generator/SelectBlitzTarget.java` | `ffb-engine` | `src/step/generator/select_blitz_target.rs` | ✓ |
| `server/step/generator/SelectGazeTarget.java` | `ffb-engine` | `src/step/generator/select_gaze_target.rs` | ✓ |
| `server/step/generator/Sequence.java` | `ffb-engine` | `src/step/generator/sequence.rs` | ✓ |
| `server/step/generator/SequenceGenerator.java` | `ffb-engine` | `src/step/generator/sequence_generator.rs` | ✓ |
| `server/step/generator/SpecialEffect.java` | `ffb-engine` | `src/step/generator/special_effect.rs` | ✓ |
| `server/step/generator/StartGame.java` | `ffb-engine` | `src/step/generator/start_game.rs` | ✓ |
| `server/step/generator/ThenIStartedBlastin.java` | `ffb-engine` | `src/step/generator/then_i_started_blastin.rs` | ✓ |
| `server/step/generator/ThrowKeg.java` | `ffb-engine` | `src/step/generator/throw_keg.rs` | ✓ |
| `server/step/generator/ThrowTeamMate.java` | `ffb-engine` | `src/step/generator/throw_team_mate.rs` | ✓ |
| `server/step/generator/Treacherous.java` | `ffb-engine` | `src/step/generator/treacherous.rs` | ✓ |

### server/step/mixed/ (53 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/mixed/blitz/StepRemoveTargetSelectionState.java` | `ffb-engine` | `src/step/mixed/blitz/step_remove_target_selection_state.rs` | ✓ |
| `server/step/mixed/blitz/StepSelectBlitzTargetEnd.java` | `ffb-engine` | `src/step/mixed/blitz/step_select_blitz_target_end.rs` | ✓ |
| `server/step/mixed/block/StepBlockBallAndChain.java` | `ffb-engine` | `src/step/mixed/block/step_block_ball_and_chain.rs` | ✓ |
| `server/step/mixed/block/StepBothDown.java` | `ffb-engine` | `src/step/mixed/block/step_both_down.rs` | ✓ |
| `server/step/mixed/block/StepProjectileVomit.java` | `ffb-engine` | `src/step/mixed/block/step_projectile_vomit.rs` | ✓ |
| `server/step/mixed/end/StepDedicatedFans.java` | `ffb-engine` | `src/step/mixed/end/step_dedicated_fans.rs` | ✓ |
| `server/step/mixed/end/StepPenaltyShootout.java` | `ffb-engine` | `src/step/mixed/end/step_penalty_shootout.rs` | ✓ |
| `server/step/mixed/foul/StepEjectPlayer.java` | `ffb-engine` | `src/step/mixed/foul/step_eject_player.rs` | ✓ |
| `server/step/mixed/foul/StepFoul.java` | `ffb-engine` | `src/step/mixed/foul/step_foul.rs` | ✓ |
| `server/step/mixed/foul/StepFoulChainsaw.java` | `ffb-engine` | `src/step/mixed/foul/step_foul_chainsaw.rs` | ✓ |
| `server/step/mixed/foul/StepPileDriver.java` | `ffb-engine` | `src/step/mixed/foul/step_pile_driver.rs` | ✓ |
| `server/step/mixed/inducements/StepPlayCard.java` | `ffb-engine` | `src/step/mixed/inducements/step_play_card.rs` | ✓ |
| `server/step/mixed/kickoff/StepInitKickoff.java` | `ffb-engine` | `src/step/mixed/kickoff/step_init_kickoff.rs` | ✓ |
| `server/step/mixed/kickoff/StepKickoff.java` | `ffb-engine` | `src/step/mixed/kickoff/step_kickoff.rs` | ✓ |
| `server/step/mixed/kickoff/StepSwarming.java` | `ffb-engine` | `src/step/mixed/kickoff/step_swarming.rs` | ✓ |
| `server/step/mixed/move/StepDropDivingTackler.java` | `ffb-engine` | `src/step/mixed/move/step_drop_diving_tackler.rs` | ✓ |
| `server/step/mixed/move/StepMoveBallAndChain.java` | `ffb-engine` | `src/step/mixed/move/step_move_ball_and_chain.rs` | ✓ |
| `server/step/mixed/move/StepResetFumblerooskie.java` | `ffb-engine` | `src/step/mixed/move/step_reset_fumblerooskie.rs` | ✓ |
| `server/step/mixed/move/StepTentacles.java` | `ffb-engine` | `src/step/mixed/move/step_tentacles.rs` | ✓ |
| `server/step/mixed/move/StepTrapDoor.java` | `ffb-engine` | `src/step/mixed/move/step_trap_door.rs` | ✓ |
| `server/step/mixed/multiblock/AbstractStepMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/abstract_step_multiple.rs` | ✓ |
| `server/step/mixed/multiblock/StepDauntlessMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/step_dauntless_multiple.rs` | ✓ |
| `server/step/mixed/multiblock/StepDispatchDumpOff.java` | `ffb-engine` | `src/step/mixed/multiblock/step_dispatch_dump_off.rs` | ✓ |
| `server/step/mixed/multiblock/StepDoubleStrength.java` | `ffb-engine` | `src/step/mixed/multiblock/step_double_strength.rs` | ✓ |
| `server/step/mixed/multiblock/StepFoulAppearanceMultiple.java` | `ffb-engine` | `src/step/mixed/multiblock/step_foul_appearance_multiple.rs` | ✓ |
| `server/step/mixed/pass/state/PassState.java` | `ffb-engine` | `src/step/mixed/pass/state/pass_state.rs` | ✓ |
| `server/step/mixed/pass/StepAllYouCanEat.java` | `ffb-engine` | `src/step/mixed/pass/step_all_you_can_eat.rs` | ✓ |
| `server/step/mixed/pass/StepInitPassing.java` | `ffb-engine` | `src/step/mixed/pass/step_init_passing.rs` | ✓ |
| `server/step/mixed/pass/StepPassBlock.java` | `ffb-engine` | `src/step/mixed/pass/step_pass_block.rs` | ✓ |
| `server/step/mixed/shared/StepAnimalSavagery.java` | `ffb-engine` | `src/step/mixed/shared/step_animal_savagery.rs` | ✓ |
| `server/step/mixed/shared/StepConsumeParameter.java` | `ffb-engine` | `src/step/mixed/shared/step_consume_parameter.rs` | ✓ |
| `server/step/mixed/shared/StepPickMeUp.java` | `ffb-engine` | `src/step/mixed/shared/step_pick_me_up.rs` | ✓ |
| `server/step/mixed/shared/StepSetDefender.java` | `ffb-engine` | `src/step/mixed/shared/step_set_defender.rs` | ✓ |
| `server/step/mixed/SingleReRollUseState.java` | `ffb-engine` | `src/step/mixed/single_re_roll_use_state.rs` | ✓ |
| `server/step/mixed/special/StepEndBomb.java` | `ffb-engine` | `src/step/mixed/special/step_end_bomb.rs` | ✓ |
| `server/step/mixed/start/StepPettyCash.java` | `ffb-engine` | `src/step/mixed/start/step_petty_cash.rs` | ✓ |
| `server/step/mixed/start/StepSpectators.java` | `ffb-engine` | `src/step/mixed/start/step_spectators.rs` | ✓ |
| `server/step/mixed/StepBlockDodge.java` | `ffb-engine` | `src/step/mixed/step_block_dodge.rs` | ✓ |
| `server/step/mixed/StepDropActingPlayer.java` | `ffb-engine` | `src/step/mixed/step_drop_acting_player.rs` | ✓ |
| `server/step/mixed/StepEndThenIStartedBlastin.java` | `ffb-engine` | `src/step/mixed/step_end_then_i_started_blastin.rs` | ✓ |
| `server/step/mixed/StepEndThrowKeg.java` | `ffb-engine` | `src/step/mixed/step_end_throw_keg.rs` | ✓ |
| `server/step/mixed/StepFirstMoveFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_first_move_furious_outburst.rs` | ✓ |
| `server/step/mixed/StepFoulAppearance.java` | `ffb-engine` | `src/step/mixed/step_foul_appearance.rs` | ✓ |
| `server/step/mixed/StepInitFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_init_furious_outburst.rs` | ✓ |
| `server/step/mixed/StepInitLookIntoMyEyes.java` | `ffb-engine` | `src/step/mixed/step_init_look_into_my_eyes.rs` | ✓ |
| `server/step/mixed/StepPro.java` | `ffb-engine` | `src/step/mixed/step_pro.rs` | ✓ |
| `server/step/mixed/StepQuickBite.java` | `ffb-engine` | `src/step/mixed/step_quick_bite.rs` | ✓ |
| `server/step/mixed/StepSecondMoveFuriousOutburst.java` | `ffb-engine` | `src/step/mixed/step_second_move_furious_outburst.rs` | ✓ |
| `server/step/mixed/StepThrowKeg.java` | `ffb-engine` | `src/step/mixed/step_throw_keg.rs` | ✓ |
| `server/step/mixed/StepUnchannelledFury.java` | `ffb-engine` | `src/step/mixed/step_unchannelled_fury.rs` | ✓ |
| `server/step/mixed/StepWizard.java` | `ffb-engine` | `src/step/mixed/step_wizard.rs` | ✓ |
| `server/step/mixed/ttm/StepSwoop.java` | `ffb-engine` | `src/step/mixed/ttm/step_swoop.rs` | ✓ |
| `server/step/mixed/ttm/TtmToCrowdHandler.java` | `ffb-engine` | `src/step/mixed/ttm/ttm_to_crowd_handler.rs` | ✓ |

### server/step/phase/ (7 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/step/phase/inducement/StepRiotousRookies.java` | `ffb-engine` | `src/step/phase/inducement/step_riotous_rookies.rs` | ✓ |
| `server/step/phase/kickoff/StepCoinChoice.java` | `ffb-engine` | `src/step/phase/kickoff/step_coin_choice.rs` | ✓ |
| `server/step/phase/kickoff/StepEndKickoff.java` | `ffb-engine` | `src/step/phase/kickoff/step_end_kickoff.rs` | ✓ |
| `server/step/phase/kickoff/StepKickoffAnimation.java` | `ffb-engine` | `src/step/phase/kickoff/step_kickoff_animation.rs` | ✓ |
| `server/step/phase/kickoff/StepKickoffReturn.java` | `ffb-engine` | `src/step/phase/kickoff/step_kickoff_return.rs` | ✓ |
| `server/step/phase/kickoff/StepReceiveChoice.java` | `ffb-engine` | `src/step/phase/kickoff/step_receive_choice.rs` | ✓ |
| `server/step/phase/kickoff/StepTouchback.java` | `ffb-engine` | `src/step/phase/kickoff/step_touchback.rs` | ✓ |

### server/util/ (40 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `server/util/AgilityCalc.java` | `ffb-engine` | `src/util/agility_calc.rs` | ✓ |
| `server/util/BlockDiceCalc.java` | `ffb-engine` | `src/util/block_dice_calc.rs` | ✓ |
| `server/util/BlockResultCalc.java` | `ffb-engine` | `src/util/block_result_calc.rs` | ✓ |
| `server/util/CatchCalc.java` | `ffb-engine` | `src/util/catch_calc.rs` | ✓ |
| `server/util/FoulCalc.java` | `ffb-engine` | `src/util/foul_calc.rs` | ✓ |
| `server/util/KickoffEventCalc.java` | `ffb-engine` | `src/util/kickoff_event_calc.rs` | ✓ |
| `server/util/MarkerLoadingService.java` | `ffb-server` | `src/util/marker_loading_service.rs` | ✓ |
| `server/util/MovementCalc.java` | `ffb-engine` | `src/util/movement_calc.rs` | ✓ |
| `server/util/PassCalc.java` | `ffb-engine` | `src/util/pass_calc.rs` | ✓ |
| `server/util/PassingDistanceCalc.java` | `ffb-engine` | `src/util/passing_distance_calc.rs` | ✓ |
| `server/util/PostMatchCalc.java` | `ffb-engine` | `src/util/post_match_calc.rs` | ✓ |
| `server/util/rng/EntropyPool.java` | `ffb-engine` | `src/util/rng/entropy_pool.rs` | ✓ |
| `server/util/rng/EntropyServer.java` | `ffb-engine` | `src/util/rng/entropy_server.rs` | ✓ |
| `server/util/rng/Fortuna.java` | `ffb-engine` | `src/util/rng/fortuna.rs` | ✓ |
| `server/util/rng/NetworkEntropySource.java` | `ffb-engine` | `src/util/rng/network_entropy_source.rs` | ✓ |
| `server/util/RollCalc.java` | `ffb-engine` | `src/util/roll_calc.rs` | ✓ |
| `server/util/ScatterCalc.java` | `ffb-engine` | `src/util/scatter_calc.rs` | ✓ |
| `server/util/ServerUtilBlock.java` | `ffb-engine` | `src/util/server_util_block.rs` | ✓ |
| `server/util/ServerUtilPlayer.java` | `ffb-engine` | `src/util/server_util_player.rs` | ✓ |
| `server/util/SpecialRollCalc.java` | `ffb-engine` | `src/util/special_roll_calc.rs` | ✓ |
| `server/util/StatCalc.java` | `ffb-engine` | `src/util/stat_calc.rs` | ✓ |
| `server/util/ThrowInCalc.java` | `ffb-engine` | `src/util/throw_in_calc.rs` | ✓ |
| `server/util/UtilServerCards.java` | `ffb-engine` | `src/util/util_server_cards.rs` | ✓ |
| `server/util/UtilServerCatchScatterThrowIn.java` | `ffb-engine` | `src/util/util_server_catch_scatter_throw_in.rs` | ✓ |
| `server/util/UtilServerDb.java` | `ffb-engine` | `src/util/util_server_db.rs` | ✓ |
| `server/util/UtilServerDialog.java` | `ffb-engine` | `src/util/util_server_dialog.rs` | ✓ |
| `server/util/UtilServerGame.java` | `ffb-engine` | `src/util/util_server_game.rs` | ✓ |
| `server/util/UtilServerHttpClient.java` | `ffb-engine` | `src/util/util_server_http_client.rs` | ~ |
| `server/util/UtilServerInducementUse.java` | `ffb-engine` | `src/util/util_server_inducement_use.rs` | ✓ |
| `server/util/UtilServerInjury.java` | `ffb-engine` | `src/util/util_server_injury.rs` | ✓ |
| `server/util/UtilServerPlayerMove.java` | `ffb-engine` | `src/util/util_server_player_move.rs` | ✓ |
| `server/util/UtilServerPlayerSwoop.java` | `ffb-engine` | `src/util/util_server_player_swoop.rs` | ✓ |
| `server/util/UtilServerPushback.java` | `ffb-engine` | `src/util/util_server_pushback.rs` | ✓ |
| `server/util/UtilServerReplay.java` | `ffb-server` | `src/util/server_replay.rs` | ✓ |
| `server/util/UtilServerReRoll.java` | `ffb-engine` | `src/util/util_server_re_roll.rs` | ✓ |
| `server/util/UtilServerSetup.java` | `ffb-engine` | `src/util/util_server_setup.rs` | ✓ |
| `server/util/UtilServerStartGame.java` | `ffb-engine` | `src/util/util_server_start_game.rs` (addDefaultGameOptions only) | ✓ |
| `server/util/UtilServerStartGame.java` | `ffb-server` | `src/util/server_start_game.rs` (joinGameAsPlayerAndCheckIfReadyToStart, sendServerJoin, sendUserSettings, startGame — Phase ZX.3) | ✓ |
| `server/util/UtilServerTimer.java` | `ffb-engine` | `src/util/util_server_timer.rs` | ✓ |
| `server/util/UtilSkillBehaviours.java` | `ffb-engine` | `src/util/util_skill_behaviours.rs` | ✓ |
| `server/util/WeatherCalc.java` | `ffb-engine` | `src/util/weather_calc.rs` | ✓ |

## Module: ffb-client-logic

### client/animation/ (14 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/animation/AnimationFrame.java` | `ffb-client` | `src/client/animation/AnimationFrame.rs` | — |
| `client/animation/AnimationProjector.java` | `ffb-client` | `src/client/animation/AnimationProjector.rs` | — |
| `client/animation/AnimationSequenceCard.java` | `ffb-client` | `src/client/animation/AnimationSequenceCard.rs` | — |
| `client/animation/AnimationSequenceChained.java` | `ffb-client` | `src/client/animation/AnimationSequenceChained.rs` | — |
| `client/animation/AnimationSequenceFactory.java` | `ffb-client` | `src/client/animation/AnimationSequenceFactory.rs` | — |
| `client/animation/AnimationSequenceKickoff.java` | `ffb-client` | `src/client/animation/AnimationSequenceKickoff.rs` | — |
| `client/animation/AnimationSequenceMovingEffect.java` | `ffb-client` | `src/client/animation/AnimationSequenceMovingEffect.rs` | — |
| `client/animation/AnimationSequenceSpecialEffect.java` | `ffb-client` | `src/client/animation/AnimationSequenceSpecialEffect.rs` | — |
| `client/animation/AnimationSequenceThrowing.java` | `ffb-client` | `src/client/animation/AnimationSequenceThrowing.rs` | — |
| `client/animation/CoordinateBasedSteppingStrategy.java` | `ffb-client` | `src/client/animation/CoordinateBasedSteppingStrategy.rs` | — |
| `client/animation/IAnimationListener.java` | `ffb-client` | `src/client/animation/IAnimationListener.rs` | — |
| `client/animation/IAnimationSequence.java` | `ffb-client` | `src/client/animation/IAnimationSequence.rs` | — |
| `client/animation/SteppingStrategy.java` | `ffb-client` | `src/client/animation/SteppingStrategy.rs` | — |
| `client/animation/TimerBasedSteppingStrategy.java` | `ffb-client` | `src/client/animation/TimerBasedSteppingStrategy.rs` | — |

### client/dialog/ (170 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/dialog/AbstractDialogBlock.java` | `ffb-client` | `src/client/dialog/AbstractDialogBlock.rs` | — |
| `client/dialog/AbstractDialogForTargets.java` | `ffb-client` | `src/client/dialog/AbstractDialogForTargets.rs` | — |
| `client/dialog/AbstractDialogMultiBlock.java` | `ffb-client` | `src/client/dialog/AbstractDialogMultiBlock.rs` | — |
| `client/dialog/AbstractDialogMultiBlockProperties.java` | `ffb-client` | `src/client/dialog/AbstractDialogMultiBlockProperties.rs` | — |
| `client/dialog/CommonPropertyCheckList.java` | `ffb-client` | `src/client/dialog/CommonPropertyCheckList.rs` | — |
| `client/dialog/CommonPropertyCheckListItem.java` | `ffb-client` | `src/client/dialog/CommonPropertyCheckListItem.rs` | — |
| `client/dialog/CreditEntry.java` | `ffb-client` | `src/client/dialog/CreditEntry.rs` | — |
| `client/dialog/Dialog.java` | `ffb-client` | `src/client/dialog/Dialog.rs` | — |
| `client/dialog/DialogAbout.java` | `ffb-client` | `src/client/dialog/DialogAbout.rs` | — |
| `client/dialog/DialogAboutHandler.java` | `ffb-client` | `src/client/dialog/DialogAboutHandler.rs` | — |
| `client/dialog/DialogApothecaryChoice.java` | `ffb-client` | `src/client/dialog/DialogApothecaryChoice.rs` | — |
| `client/dialog/DialogApothecaryChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogApothecaryChoiceHandler.rs` | — |
| `client/dialog/DialogArgueTheCall.java` | `ffb-client` | `src/client/dialog/DialogArgueTheCall.rs` | — |
| `client/dialog/DialogArgueTheCallHandler.java` | `ffb-client` | `src/client/dialog/DialogArgueTheCallHandler.rs` | — |
| `client/dialog/DialogAutoMarking.java` | `ffb-client` | `src/client/dialog/DialogAutoMarking.rs` | — |
| `client/dialog/DialogBlockRoll.java` | `ffb-client` | `src/client/dialog/DialogBlockRoll.rs` | — |
| `client/dialog/DialogBlockRollHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollHandler.rs` | — |
| `client/dialog/DialogBlockRollPartialReRoll.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPartialReRoll.rs` | — |
| `client/dialog/DialogBlockRollPartialReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPartialReRollHandler.rs` | — |
| `client/dialog/DialogBlockRollProperties.java` | `ffb-client` | `src/client/dialog/DialogBlockRollProperties.rs` | — |
| `client/dialog/DialogBlockRollPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogBlockRollPropertiesHandler.rs` | — |
| `client/dialog/DialogBloodlustAction.java` | `ffb-client` | `src/client/dialog/DialogBloodlustAction.rs` | — |
| `client/dialog/DialogBloodlustActionHandler.java` | `ffb-client` | `src/client/dialog/DialogBloodlustActionHandler.rs` | — |
| `client/dialog/DialogBriberyAndCorruption.java` | `ffb-client` | `src/client/dialog/DialogBriberyAndCorruption.rs` | — |
| `client/dialog/DialogBriberyAndCorruptionHandler.java` | `ffb-client` | `src/client/dialog/DialogBriberyAndCorruptionHandler.rs` | — |
| `client/dialog/DialogBribes.java` | `ffb-client` | `src/client/dialog/DialogBribes.rs` | — |
| `client/dialog/DialogBribesHandler.java` | `ffb-client` | `src/client/dialog/DialogBribesHandler.rs` | — |
| `client/dialog/DialogChangeList.java` | `ffb-client` | `src/client/dialog/DialogChangeList.rs` | — |
| `client/dialog/DialogChatCommands.java` | `ffb-client` | `src/client/dialog/DialogChatCommands.rs` | — |
| `client/dialog/DialogCoinChoice.java` | `ffb-client` | `src/client/dialog/DialogCoinChoice.rs` | — |
| `client/dialog/DialogCoinChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogCoinChoiceHandler.rs` | — |
| `client/dialog/DialogConcedeGame.java` | `ffb-client` | `src/client/dialog/DialogConcedeGame.rs` | — |
| `client/dialog/DialogConfirmEndAction.java` | `ffb-client` | `src/client/dialog/DialogConfirmEndAction.rs` | — |
| `client/dialog/DialogConfirmEndActionHandler.java` | `ffb-client` | `src/client/dialog/DialogConfirmEndActionHandler.rs` | — |
| `client/dialog/DialogCredits.java` | `ffb-client` | `src/client/dialog/DialogCredits.rs` | — |
| `client/dialog/DialogDefenderActionHandler.java` | `ffb-client` | `src/client/dialog/DialogDefenderActionHandler.rs` | — |
| `client/dialog/DialogEndTurn.java` | `ffb-client` | `src/client/dialog/DialogEndTurn.rs` | — |
| `client/dialog/DialogExtensionMascot.java` | `ffb-client` | `src/client/dialog/DialogExtensionMascot.rs` | — |
| `client/dialog/DialogFollowupChoice.java` | `ffb-client` | `src/client/dialog/DialogFollowupChoice.rs` | — |
| `client/dialog/DialogFollowupChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogFollowupChoiceHandler.rs` | — |
| `client/dialog/DialogGameChoice.java` | `ffb-client` | `src/client/dialog/DialogGameChoice.rs` | — |
| `client/dialog/DialogGameConcessionHandler.java` | `ffb-client` | `src/client/dialog/DialogGameConcessionHandler.rs` | — |
| `client/dialog/DialogGameStatistics.java` | `ffb-client` | `src/client/dialog/DialogGameStatistics.rs` | — |
| `client/dialog/DialogGameStatisticsHandler.java` | `ffb-client` | `src/client/dialog/DialogGameStatisticsHandler.rs` | — |
| `client/dialog/DialogHandler.java` | `ffb-client` | `src/client/dialog/DialogHandler.rs` | — |
| `client/dialog/DialogInformation.java` | `ffb-client` | `src/client/dialog/DialogInformation.rs` | — |
| `client/dialog/DialogInformationOkayHandler.java` | `ffb-client` | `src/client/dialog/DialogInformationOkayHandler.rs` | — |
| `client/dialog/DialogInterception.java` | `ffb-client` | `src/client/dialog/DialogInterception.rs` | — |
| `client/dialog/DialogInterceptionHandler.java` | `ffb-client` | `src/client/dialog/DialogInterceptionHandler.rs` | — |
| `client/dialog/DialogInvalidSolidDefenceHandler.java` | `ffb-client` | `src/client/dialog/DialogInvalidSolidDefenceHandler.rs` | — |
| `client/dialog/DialogJoinHandler.java` | `ffb-client` | `src/client/dialog/DialogJoinHandler.rs` | — |
| `client/dialog/DialogJourneymen.java` | `ffb-client` | `src/client/dialog/DialogJourneymen.rs` | — |
| `client/dialog/DialogJourneymenHandler.java` | `ffb-client` | `src/client/dialog/DialogJourneymenHandler.rs` | — |
| `client/dialog/DialogKeyBindings.java` | `ffb-client` | `src/client/dialog/DialogKeyBindings.rs` | — |
| `client/dialog/DialogKickOffResult.java` | `ffb-client` | `src/client/dialog/DialogKickOffResult.rs` | — |
| `client/dialog/DialogKickOffResultHandler.java` | `ffb-client` | `src/client/dialog/DialogKickOffResultHandler.rs` | — |
| `client/dialog/DialogKickoffReturnHandler.java` | `ffb-client` | `src/client/dialog/DialogKickoffReturnHandler.rs` | — |
| `client/dialog/DialogKickSkillHandler.java` | `ffb-client` | `src/client/dialog/DialogKickSkillHandler.rs` | — |
| `client/dialog/DialogLeaveGame.java` | `ffb-client` | `src/client/dialog/DialogLeaveGame.rs` | — |
| `client/dialog/DialogLicense.java` | `ffb-client` | `src/client/dialog/DialogLicense.rs` | — |
| `client/dialog/DialogLogin.java` | `ffb-client` | `src/client/dialog/DialogLogin.rs` | — |
| `client/dialog/DialogManager.java` | `ffb-client` | `src/client/dialog/DialogManager.rs` | — |
| `client/dialog/DialogOpponentBlockSelection.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelection.rs` | — |
| `client/dialog/DialogOpponentBlockSelectionHandler.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionHandler.rs` | — |
| `client/dialog/DialogOpponentBlockSelectionProperties.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionProperties.rs` | — |
| `client/dialog/DialogOpponentBlockSelectionPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogOpponentBlockSelectionPropertiesHandler.rs` | — |
| `client/dialog/DialogPassBlockHandler.java` | `ffb-client` | `src/client/dialog/DialogPassBlockHandler.rs` | — |
| `client/dialog/DialogPenaltyShootout.java` | `ffb-client` | `src/client/dialog/DialogPenaltyShootout.rs` | — |
| `client/dialog/DialogPenaltyShootoutHandler.java` | `ffb-client` | `src/client/dialog/DialogPenaltyShootoutHandler.rs` | — |
| `client/dialog/DialogPettyCash.java` | `ffb-client` | `src/client/dialog/DialogPettyCash.rs` | — |
| `client/dialog/DialogPettyCashHandler.java` | `ffb-client` | `src/client/dialog/DialogPettyCashHandler.rs` | — |
| `client/dialog/DialogPickUpChoice.java` | `ffb-client` | `src/client/dialog/DialogPickUpChoice.rs` | — |
| `client/dialog/DialogPickUpChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogPickUpChoiceHandler.rs` | — |
| `client/dialog/DialogPileDriver.java` | `ffb-client` | `src/client/dialog/DialogPileDriver.rs` | — |
| `client/dialog/DialogPileDriverHandler.java` | `ffb-client` | `src/client/dialog/DialogPileDriverHandler.rs` | — |
| `client/dialog/DialogPilingOn.java` | `ffb-client` | `src/client/dialog/DialogPilingOn.rs` | — |
| `client/dialog/DialogPilingOnHandler.java` | `ffb-client` | `src/client/dialog/DialogPilingOnHandler.rs` | — |
| `client/dialog/DialogPlayerChoice.java` | `ffb-client` | `src/client/dialog/DialogPlayerChoice.rs` | — |
| `client/dialog/DialogPlayerChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogPlayerChoiceHandler.rs` | — |
| `client/dialog/DialogProgressBar.java` | `ffb-client` | `src/client/dialog/DialogProgressBar.rs` | — |
| `client/dialog/DialogPuntToCrowd.java` | `ffb-client` | `src/client/dialog/DialogPuntToCrowd.rs` | — |
| `client/dialog/DialogPuntToCrowdHandler.java` | `ffb-client` | `src/client/dialog/DialogPuntToCrowdHandler.rs` | — |
| `client/dialog/DialogReceiveChoice.java` | `ffb-client` | `src/client/dialog/DialogReceiveChoice.rs` | — |
| `client/dialog/DialogReceiveChoiceHandler.java` | `ffb-client` | `src/client/dialog/DialogReceiveChoiceHandler.rs` | — |
| `client/dialog/DialogReplayModeChoice.java` | `ffb-client` | `src/client/dialog/DialogReplayModeChoice.rs` | — |
| `client/dialog/DialogReRoll.java` | `ffb-client` | `src/client/dialog/DialogReRoll.rs` | — |
| `client/dialog/DialogReRollBlockForTargets.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargets.rs` | — |
| `client/dialog/DialogReRollBlockForTargetsHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsHandler.rs` | — |
| `client/dialog/DialogReRollBlockForTargetsProperties.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsProperties.rs` | — |
| `client/dialog/DialogReRollBlockForTargetsPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollBlockForTargetsPropertiesHandler.rs` | — |
| `client/dialog/DialogReRollForTargets.java` | `ffb-client` | `src/client/dialog/DialogReRollForTargets.rs` | — |
| `client/dialog/DialogReRollForTargetsHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollForTargetsHandler.rs` | — |
| `client/dialog/DialogReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollHandler.rs` | — |
| `client/dialog/DialogReRollProperties.java` | `ffb-client` | `src/client/dialog/DialogReRollProperties.rs` | — |
| `client/dialog/DialogReRollPropertiesHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollPropertiesHandler.rs` | — |
| `client/dialog/DialogReRollRegenerationMultiple.java` | `ffb-client` | `src/client/dialog/DialogReRollRegenerationMultiple.rs` | — |
| `client/dialog/DialogReRollRegenerationMultipleHandler.java` | `ffb-client` | `src/client/dialog/DialogReRollRegenerationMultipleHandler.rs` | — |
| `client/dialog/DialogScalingFactor.java` | `ffb-client` | `src/client/dialog/DialogScalingFactor.rs` | — |
| `client/dialog/DialogSelectBlitzTargetHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectBlitzTargetHandler.rs` | — |
| `client/dialog/DialogSelectGazeTargetHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectGazeTargetHandler.rs` | — |
| `client/dialog/DialogSelectKeyword.java` | `ffb-client` | `src/client/dialog/DialogSelectKeyword.rs` | — |
| `client/dialog/DialogSelectKeywordHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectKeywordHandler.rs` | — |
| `client/dialog/DialogSelectLocalStoredProperties.java` | `ffb-client` | `src/client/dialog/DialogSelectLocalStoredProperties.rs` | — |
| `client/dialog/DialogSelectPosition.java` | `ffb-client` | `src/client/dialog/DialogSelectPosition.rs` | — |
| `client/dialog/DialogSelectPositionHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectPositionHandler.rs` | — |
| `client/dialog/DialogSelectSkill.java` | `ffb-client` | `src/client/dialog/DialogSelectSkill.rs` | — |
| `client/dialog/DialogSelectSkillHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectSkillHandler.rs` | — |
| `client/dialog/DialogSelectTarget.java` | `ffb-client` | `src/client/dialog/DialogSelectTarget.rs` | — |
| `client/dialog/DialogSelectWeather.java` | `ffb-client` | `src/client/dialog/DialogSelectWeather.rs` | — |
| `client/dialog/DialogSelectWeatherHandler.java` | `ffb-client` | `src/client/dialog/DialogSelectWeatherHandler.rs` | — |
| `client/dialog/DialogSetupError.java` | `ffb-client` | `src/client/dialog/DialogSetupError.rs` | — |
| `client/dialog/DialogSetupErrorHandler.java` | `ffb-client` | `src/client/dialog/DialogSetupErrorHandler.rs` | — |
| `client/dialog/DialogSkillUse.java` | `ffb-client` | `src/client/dialog/DialogSkillUse.rs` | — |
| `client/dialog/DialogSkillUseHandler.java` | `ffb-client` | `src/client/dialog/DialogSkillUseHandler.rs` | — |
| `client/dialog/DialogSoundVolume.java` | `ffb-client` | `src/client/dialog/DialogSoundVolume.rs` | — |
| `client/dialog/DialogStartGame.java` | `ffb-client` | `src/client/dialog/DialogStartGame.rs` | — |
| `client/dialog/DialogStartGameHandler.java` | `ffb-client` | `src/client/dialog/DialogStartGameHandler.rs` | — |
| `client/dialog/DialogSwarmingErrorParameterHandler.java` | `ffb-client` | `src/client/dialog/DialogSwarmingErrorParameterHandler.rs` | — |
| `client/dialog/DialogSwarmingPlayersHandler.java` | `ffb-client` | `src/client/dialog/DialogSwarmingPlayersHandler.rs` | — |
| `client/dialog/DialogTeamChoice.java` | `ffb-client` | `src/client/dialog/DialogTeamChoice.rs` | — |
| `client/dialog/DialogTeamSetup.java` | `ffb-client` | `src/client/dialog/DialogTeamSetup.rs` | — |
| `client/dialog/DialogTeamSetupHandler.java` | `ffb-client` | `src/client/dialog/DialogTeamSetupHandler.rs` | — |
| `client/dialog/DialogThreeWayChoice.java` | `ffb-client` | `src/client/dialog/DialogThreeWayChoice.rs` | — |
| `client/dialog/DialogTouchbackHandler.java` | `ffb-client` | `src/client/dialog/DialogTouchbackHandler.rs` | — |
| `client/dialog/DialogUseApothecaries.java` | `ffb-client` | `src/client/dialog/DialogUseApothecaries.rs` | — |
| `client/dialog/DialogUseApothecariesHandler.java` | `ffb-client` | `src/client/dialog/DialogUseApothecariesHandler.rs` | — |
| `client/dialog/DialogUseApothecary.java` | `ffb-client` | `src/client/dialog/DialogUseApothecary.rs` | — |
| `client/dialog/DialogUseApothecaryHandler.java` | `ffb-client` | `src/client/dialog/DialogUseApothecaryHandler.rs` | — |
| `client/dialog/DialogUseChainsaw.java` | `ffb-client` | `src/client/dialog/DialogUseChainsaw.rs` | — |
| `client/dialog/DialogUseChainsawHandler.java` | `ffb-client` | `src/client/dialog/DialogUseChainsawHandler.rs` | — |
| `client/dialog/DialogUseIgor.java` | `ffb-client` | `src/client/dialog/DialogUseIgor.rs` | — |
| `client/dialog/DialogUseIgorHandler.java` | `ffb-client` | `src/client/dialog/DialogUseIgorHandler.rs` | — |
| `client/dialog/DialogUseIgorsHandler.java` | `ffb-client` | `src/client/dialog/DialogUseIgorsHandler.rs` | — |
| `client/dialog/DialogUseMortuaryAssistant.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistant.rs` | — |
| `client/dialog/DialogUseMortuaryAssistantHandler.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistantHandler.rs` | — |
| `client/dialog/DialogUseMortuaryAssistantsHandler.java` | `ffb-client` | `src/client/dialog/DialogUseMortuaryAssistantsHandler.rs` | — |
| `client/dialog/DialogWinningsReRoll.java` | `ffb-client` | `src/client/dialog/DialogWinningsReRoll.rs` | — |
| `client/dialog/DialogWinningsReRollHandler.java` | `ffb-client` | `src/client/dialog/DialogWinningsReRollHandler.rs` | — |
| `client/dialog/DialogWizardSpell.java` | `ffb-client` | `src/client/dialog/DialogWizardSpell.rs` | — |
| `client/dialog/DialogWizardSpellHandler.java` | `ffb-client` | `src/client/dialog/DialogWizardSpellHandler.rs` | — |
| `client/dialog/IDialog.java` | `ffb-client` | `src/client/dialog/IDialog.rs` | — |
| `client/dialog/IDialogCloseListener.java` | `ffb-client` | `src/client/dialog/IDialogCloseListener.rs` | — |
| `client/dialog/inducements/AbstractBuyInducementsDialog.java` | `ffb-client` | `src/client/dialog/inducements/AbstractBuyInducementsDialog.rs` | — |
| `client/dialog/inducements/DialogBuyCards.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCards.rs` | — |
| `client/dialog/inducements/DialogBuyCardsAndInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsAndInducements.rs` | — |
| `client/dialog/inducements/DialogBuyCardsAndInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsAndInducementsHandler.rs` | — |
| `client/dialog/inducements/DialogBuyCardsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyCardsHandler.rs` | — |
| `client/dialog/inducements/DialogBuyInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyInducements.rs` | — |
| `client/dialog/inducements/DialogBuyInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyInducementsHandler.rs` | — |
| `client/dialog/inducements/DialogBuyPrayersAndInducements.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyPrayersAndInducements.rs` | — |
| `client/dialog/inducements/DialogBuyPrayersAndInducementsHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogBuyPrayersAndInducementsHandler.rs` | — |
| `client/dialog/inducements/DialogUseInducement.java` | `ffb-client` | `src/client/dialog/inducements/DialogUseInducement.rs` | — |
| `client/dialog/inducements/DialogUseInducementHandler.java` | `ffb-client` | `src/client/dialog/inducements/DialogUseInducementHandler.rs` | — |
| `client/dialog/inducements/DropDownPanel.java` | `ffb-client` | `src/client/dialog/inducements/DropDownPanel.rs` | — |
| `client/dialog/inducements/InfamousStaffTable.java` | `ffb-client` | `src/client/dialog/inducements/InfamousStaffTable.rs` | — |
| `client/dialog/inducements/InfamousStaffTableModel.java` | `ffb-client` | `src/client/dialog/inducements/InfamousStaffTableModel.rs` | — |
| `client/dialog/inducements/MercenaryTable.java` | `ffb-client` | `src/client/dialog/inducements/MercenaryTable.rs` | — |
| `client/dialog/inducements/MercenaryTableModel.java` | `ffb-client` | `src/client/dialog/inducements/MercenaryTableModel.rs` | — |
| `client/dialog/inducements/StarPlayerTable.java` | `ffb-client` | `src/client/dialog/inducements/StarPlayerTable.rs` | — |
| `client/dialog/inducements/StarPlayerTableModel.java` | `ffb-client` | `src/client/dialog/inducements/StarPlayerTableModel.rs` | — |
| `client/dialog/KeywordCheckList.java` | `ffb-client` | `src/client/dialog/KeywordCheckList.rs` | — |
| `client/dialog/KeywordCheckListItem.java` | `ffb-client` | `src/client/dialog/KeywordCheckListItem.rs` | — |
| `client/dialog/MultiReRollMnemonics.java` | `ffb-client` | `src/client/dialog/MultiReRollMnemonics.rs` | — |
| `client/dialog/PlayerCheckList.java` | `ffb-client` | `src/client/dialog/PlayerCheckList.rs` | — |
| `client/dialog/PlayerCheckListItem.java` | `ffb-client` | `src/client/dialog/PlayerCheckListItem.rs` | — |
| `client/dialog/PositionCheckList.java` | `ffb-client` | `src/client/dialog/PositionCheckList.rs` | — |
| `client/dialog/PositionCheckListItem.java` | `ffb-client` | `src/client/dialog/PositionCheckListItem.rs` | — |
| `client/dialog/PressedKeyListener.java` | `ffb-client` | `src/client/dialog/PressedKeyListener.rs` | — |
| `client/dialog/SkillCheckList.java` | `ffb-client` | `src/client/dialog/SkillCheckList.rs` | — |
| `client/dialog/SkillCheckListItem.java` | `ffb-client` | `src/client/dialog/SkillCheckListItem.rs` | — |

### client/factory/ (1 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/factory/LogicPluginFactory.java` | `ffb-client` | `src/client/factory/logic_plugin_factory.rs` | ✓ |

### client/handler/ (27 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/handler/AbstractClientCommandHandlerSketch.java` | `ffb-client` | `src/client/handler/abstract_client_command_handler_sketch.rs` | ✓ |
| `client/handler/ClientCommandHandler.java` | `ffb-client` | `src/client/handler/client_command_handler.rs` | ✓ |
| `client/handler/ClientCommandHandlerAddPlayer.java` | `ffb-client` | `src/client/handler/client_command_handler_add_player.rs` | ✓ |
| `client/handler/ClientCommandHandlerAddSketches.java` | `ffb-client` | `src/client/handler/client_command_handler_add_sketches.rs` | ✓ |
| `client/handler/ClientCommandHandlerAdminMessage.java` | `ffb-client` | `src/client/handler/client_command_handler_admin_message.rs` | ✓ |
| `client/handler/ClientCommandHandlerClearSketches.java` | `ffb-client` | `src/client/handler/client_command_handler_clear_sketches.rs` | ✓ |
| `client/handler/ClientCommandHandlerFactory.java` | `ffb-client` | `src/client/handler/client_command_handler_factory.rs` | ✓ |
| `client/handler/ClientCommandHandlerGameState.java` | `ffb-client` | `src/client/handler/client_command_handler_game_state.rs` | ✓ |
| `client/handler/ClientCommandHandlerGameTime.java` | `ffb-client` | `src/client/handler/client_command_handler_game_time.rs` | ✓ |
| `client/handler/ClientCommandHandlerJoin.java` | `ffb-client` | `src/client/handler/client_command_handler_join.rs` | ✓ |
| `client/handler/ClientCommandHandlerLeave.java` | `ffb-client` | `src/client/handler/client_command_handler_leave.rs` | ✓ |
| `client/handler/ClientCommandHandlerMode.java` | `ffb-client` | `src/client/handler/client_command_handler_mode.rs` | ✓ |
| `client/handler/ClientCommandHandlerModelSync.java` | `ffb-client` | `src/client/handler/client_command_handler_model_sync.rs` | ✓ |
| `client/handler/ClientCommandHandlerRemovePlayer.java` | `ffb-client` | `src/client/handler/client_command_handler_remove_player.rs` | ✓ |
| `client/handler/ClientCommandHandlerRemoveSketches.java` | `ffb-client` | `src/client/handler/client_command_handler_remove_sketches.rs` | ✓ |
| `client/handler/ClientCommandHandlerSetPreventSketching.java` | `ffb-client` | `src/client/handler/client_command_handler_set_prevent_sketching.rs` | ✓ |
| `client/handler/ClientCommandHandlerSketchAddCoordinate.java` | `ffb-client` | `src/client/handler/client_command_handler_sketch_add_coordinate.rs` | ✓ |
| `client/handler/ClientCommandHandlerSketchSetColor.java` | `ffb-client` | `src/client/handler/client_command_handler_sketch_set_color.rs` | ✓ |
| `client/handler/ClientCommandHandlerSketchSetLabel.java` | `ffb-client` | `src/client/handler/client_command_handler_sketch_set_label.rs` | ✓ |
| `client/handler/ClientCommandHandlerSocketClosed.java` | `ffb-client` | `src/client/handler/client_command_handler_socket_closed.rs` | ✓ |
| `client/handler/ClientCommandHandlerSound.java` | `ffb-client` | `src/client/handler/client_command_handler_sound.rs` | ✓ |
| `client/handler/ClientCommandHandlerTalk.java` | `ffb-client` | `src/client/handler/client_command_handler_talk.rs` | ✓ |
| `client/handler/ClientCommandHandlerUnzapPlayer.java` | `ffb-client` | `src/client/handler/client_command_handler_unzap_player.rs` | ✓ |
| `client/handler/ClientCommandHandlerUpdateLocalPlayerMarkers.java` | `ffb-client` | `src/client/handler/client_command_handler_update_local_player_markers.rs` | ✓ |
| `client/handler/ClientCommandHandlerUserSettings.java` | `ffb-client` | `src/client/handler/client_command_handler_user_settings.rs` | ✓ |
| `client/handler/ClientCommandHandlerZapPlayer.java` | `ffb-client` | `src/client/handler/client_command_handler_zap_player.rs` | ✓ |
| `client/handler/SubHandlerGameStateMarking.java` | `ffb-client` | `src/client/handler/sub_handler_game_state_marking.rs` | ✓ |

### client/layer/ (13 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/layer/FieldLayer.java` | `ffb-client` | `src/client/layer/FieldLayer.rs` | — |
| `client/layer/FieldLayerBloodspots.java` | `ffb-client` | `src/client/layer/FieldLayerBloodspots.rs` | — |
| `client/layer/FieldLayerEnhancements.java` | `ffb-client` | `src/client/layer/FieldLayerEnhancements.rs` | — |
| `client/layer/FieldLayerMarker.java` | `ffb-client` | `src/client/layer/FieldLayerMarker.rs` | — |
| `client/layer/FieldLayerOverPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerOverPlayers.rs` | — |
| `client/layer/FieldLayerPitch.java` | `ffb-client` | `src/client/layer/FieldLayerPitch.rs` | — |
| `client/layer/FieldLayerPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerPlayers.rs` | — |
| `client/layer/FieldLayerRangeGrid.java` | `ffb-client` | `src/client/layer/FieldLayerRangeGrid.rs` | — |
| `client/layer/FieldLayerRangeRuler.java` | `ffb-client` | `src/client/layer/FieldLayerRangeRuler.rs` | — |
| `client/layer/FieldLayerSketches.java` | `ffb-client` | `src/client/layer/FieldLayerSketches.rs` | — |
| `client/layer/FieldLayerTackleZones.java` | `ffb-client` | `src/client/layer/FieldLayerTackleZones.rs` | — |
| `client/layer/FieldLayerTeamLogo.java` | `ffb-client` | `src/client/layer/FieldLayerTeamLogo.rs` | — |
| `client/layer/FieldLayerUnderPlayers.java` | `ffb-client` | `src/client/layer/FieldLayerUnderPlayers.rs` | — |

### client/model/ (4 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/model/ChangeList.java` | `ffb-client` | `src/client/model/change_list.rs` | ✓ |
| `client/model/ControlAware.java` | `ffb-client` | `src/client/model/control_aware.rs` | ✓ |
| `client/model/OnlineAware.java` | `ffb-client` | `src/client/model/online_aware.rs` | ✓ |
| `client/model/VersionChangeList.java` | `ffb-client` | `src/client/model/version_change_list.rs` | ✓ |

### client/net/ (3 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/net/ClientCommunication.java` | `ffb-client` | `src/client/net/client_communication.rs` | ✓ |
| `client/net/ClientPingTask.java` | `ffb-client` | `src/client/net/client_ping_task.rs` | ✓ |
| `client/net/CommandEndpoint.java` | `ffb-client` | `src/client/net/command_endpoint.rs` | ✓ |

### client/overlay/ (3 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/overlay/Overlay.java` | `ffb-client` | `src/client/overlay/Overlay.rs` | — |
| `client/overlay/sketch/ClientSketchManager.java` | `ffb-client` | `src/client/overlay/sketch/ClientSketchManager.rs` | — |
| `client/overlay/sketch/TriangleCoords.java` | `ffb-client` | `src/client/overlay/sketch/TriangleCoords.rs` | — |

### client/report/ (211 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/report/AlwaysHungryMessage.java` | `ffb-client` | `src/client/report/always_hungry_message.rs` | ✓ |
| `client/report/AnimosityRollMessage.java` | `ffb-client` | `src/client/report/animosity_roll_message.rs` | ✓ |
| `client/report/ApothecaryChoiceMessage.java` | `ffb-client` | `src/client/report/apothecary_choice_message.rs` | ✓ |
| `client/report/bb2016/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2016/apothecary_roll_message.rs` | ✓ |
| `client/report/bb2016/ArgueTheCallMessage.java` | `ffb-client` | `src/client/report/bb2016/argue_the_call_message.rs` | ✓ |
| `client/report/bb2016/BlockChoiceMessage.java` | `ffb-client` | `src/client/report/bb2016/block_choice_message.rs` | ✓ |
| `client/report/bb2016/BloodLustRollMessage.java` | `ffb-client` | `src/client/report/bb2016/blood_lust_roll_message.rs` | ✓ |
| `client/report/bb2016/CardsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2016/cards_bought_message.rs` | ✓ |
| `client/report/bb2016/FanFactorRollPostMatchMessage.java` | `ffb-client` | `src/client/report/bb2016/fan_factor_roll_post_match_message.rs` | ✓ |
| `client/report/bb2016/GoForItRollMessage.java` | `ffb-client` | `src/client/report/bb2016/go_for_it_roll_message.rs` | ✓ |
| `client/report/bb2016/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2016/hypnotic_gaze_roll_message.rs` | ✓ |
| `client/report/bb2016/InducementMessage.java` | `ffb-client` | `src/client/report/bb2016/inducement_message.rs` | ✓ |
| `client/report/bb2016/InducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2016/inducements_bought_message.rs` | ✓ |
| `client/report/bb2016/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2016/injury_message.rs` | ✓ |
| `client/report/bb2016/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2016/kickoff_extra_re_roll_message.rs` | ✓ |
| `client/report/bb2016/KickoffPitchInvasionMessage.java` | `ffb-client` | `src/client/report/bb2016/kickoff_pitch_invasion_message.rs` | ✓ |
| `client/report/bb2016/KickoffRiotMessage.java` | `ffb-client` | `src/client/report/bb2016/kickoff_riot_message.rs` | ✓ |
| `client/report/bb2016/KickoffThrowARockMessage.java` | `ffb-client` | `src/client/report/bb2016/kickoff_throw_a_rock_message.rs` | ✓ |
| `client/report/bb2016/KickTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2016/kick_team_mate_roll_message.rs` | ✓ |
| `client/report/bb2016/MostValuablePlayersMessage.java` | `ffb-client` | `src/client/report/bb2016/most_valuable_players_message.rs` | ✓ |
| `client/report/bb2016/NervesOfSteelMessage.java` | `ffb-client` | `src/client/report/bb2016/nerves_of_steel_message.rs` | ✓ |
| `client/report/bb2016/NoPlayersToFieldMessage.java` | `ffb-client` | `src/client/report/bb2016/no_players_to_field_message.rs` | ✓ |
| `client/report/bb2016/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2016/pass_roll_message.rs` | ✓ |
| `client/report/bb2016/PenaltyShootoutMessage.java` | `ffb-client` | `src/client/report/bb2016/penalty_shootout_message.rs` | ✓ |
| `client/report/bb2016/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2016/raise_dead_message.rs` | ✓ |
| `client/report/bb2016/RefereeMessage.java` | `ffb-client` | `src/client/report/bb2016/referee_message.rs` | ✓ |
| `client/report/bb2016/ScatterBallMessage.java` | `ffb-client` | `src/client/report/bb2016/scatter_ball_message.rs` | ✓ |
| `client/report/bb2016/ScatterPlayerMessage.java` | `ffb-client` | `src/client/report/bb2016/scatter_player_message.rs` | ✓ |
| `client/report/bb2016/SpectatorsMessage.java` | `ffb-client` | `src/client/report/bb2016/spectators_message.rs` | ✓ |
| `client/report/bb2016/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2016/swarming_players_roll_message.rs` | ✓ |
| `client/report/bb2016/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2016/swoop_player_message.rs` | ✓ |
| `client/report/bb2016/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2016/tentacles_shadowing_message.rs` | ✓ |
| `client/report/bb2016/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2016/throw_team_mate_roll_message.rs` | ✓ |
| `client/report/bb2016/TurnEndMessage.java` | `ffb-client` | `src/client/report/bb2016/turn_end_message.rs` | ✓ |
| `client/report/bb2016/WinningsRollMessage.java` | `ffb-client` | `src/client/report/bb2016/winnings_roll_message.rs` | ✓ |
| `client/report/bb2020/AnimalSavageryMessage.java` | `ffb-client` | `src/client/report/bb2020/animal_savagery_message.rs` | ✓ |
| `client/report/bb2020/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2020/apothecary_roll_message.rs` | ✓ |
| `client/report/bb2020/BlitzRollMessage.java` | `ffb-client` | `src/client/report/bb2020/blitz_roll_message.rs` | ✓ |
| `client/report/bb2020/CardsAndInducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2020/cards_and_inducements_bought_message.rs` | ✓ |
| `client/report/bb2020/CheeringFansMessage.java` | `ffb-client` | `src/client/report/bb2020/cheering_fans_message.rs` | ✓ |
| `client/report/bb2020/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2020/hypnotic_gaze_roll_message.rs` | ✓ |
| `client/report/bb2020/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2020/injury_message.rs` | ✓ |
| `client/report/bb2020/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2020/kickoff_extra_re_roll_message.rs` | ✓ |
| `client/report/bb2020/KickoffOfficiousRefMessage.java` | `ffb-client` | `src/client/report/bb2020/kickoff_officious_ref_message.rs` | ✓ |
| `client/report/bb2020/KickTeamMateFumbleMessage.java` | `ffb-client` | `src/client/report/bb2020/kick_team_mate_fumble_message.rs` | ✓ |
| `client/report/bb2020/OfficiousRefRollMessage.java` | `ffb-client` | `src/client/report/bb2020/officious_ref_roll_message.rs` | ✓ |
| `client/report/bb2020/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2020/pass_roll_message.rs` | ✓ |
| `client/report/bb2020/PrayerAmountMessage.java` | `ffb-client` | `src/client/report/bb2020/prayer_amount_message.rs` | ✓ |
| `client/report/bb2020/PrayerRollMessage.java` | `ffb-client` | `src/client/report/bb2020/prayer_roll_message.rs` | ✓ |
| `client/report/bb2020/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2020/raise_dead_message.rs` | ✓ |
| `client/report/bb2020/SolidDefenceRollMessage.java` | `ffb-client` | `src/client/report/bb2020/solid_defence_roll_message.rs` | ✓ |
| `client/report/bb2020/StallerDetectedMessage.java` | `ffb-client` | `src/client/report/bb2020/staller_detected_message.rs` | ✓ |
| `client/report/bb2020/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2020/swarming_players_roll_message.rs` | ✓ |
| `client/report/bb2020/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2020/swoop_player_message.rs` | ✓ |
| `client/report/bb2020/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2020/tentacles_shadowing_message.rs` | ✓ |
| `client/report/bb2020/ThenIStartedBlastinMessage.java` | `ffb-client` | `src/client/report/bb2020/then_i_started_blastin_message.rs` | ✓ |
| `client/report/bb2020/ThrowAtStallingPlayerMessage.java` | `ffb-client` | `src/client/report/bb2020/throw_at_stalling_player_message.rs` | ✓ |
| `client/report/bb2020/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2020/throw_team_mate_roll_message.rs` | ✓ |
| `client/report/bb2020/TwoForOneMessage.java` | `ffb-client` | `src/client/report/bb2020/two_for_one_message.rs` | ✓ |
| `client/report/bb2020/UseFumblerooskieMessage.java` | `ffb-client` | `src/client/report/bb2020/use_fumblerooskie_message.rs` | ✓ |
| `client/report/bb2020/WeatherMageResultMessage.java` | `ffb-client` | `src/client/report/bb2020/weather_mage_result_message.rs` | ✓ |
| `client/report/bb2025/AnimalSavageryMessage.java` | `ffb-client` | `src/client/report/bb2025/animal_savagery_message.rs` | ✓ |
| `client/report/bb2025/ApothecaryRollMessage.java` | `ffb-client` | `src/client/report/bb2025/apothecary_roll_message.rs` | ✓ |
| `client/report/bb2025/BlitzRollMessage.java` | `ffb-client` | `src/client/report/bb2025/blitz_roll_message.rs` | ✓ |
| `client/report/bb2025/CheeringFansMessage.java` | `ffb-client` | `src/client/report/bb2025/cheering_fans_message.rs` | ✓ |
| `client/report/bb2025/ChompRemovedMessage.java` | `ffb-client` | `src/client/report/bb2025/chomp_removed_message.rs` | ✓ |
| `client/report/bb2025/ChompRollMessage.java` | `ffb-client` | `src/client/report/bb2025/chomp_roll_message.rs` | ✓ |
| `client/report/bb2025/DodgySnackRollMessage.java` | `ffb-client` | `src/client/report/bb2025/dodgy_snack_roll_message.rs` | ✓ |
| `client/report/bb2025/GettingEvenRollMessage.java` | `ffb-client` | `src/client/report/bb2025/getting_even_roll_message.rs` | ✓ |
| `client/report/bb2025/HypnoticGazeRollMessage.java` | `ffb-client` | `src/client/report/bb2025/hypnotic_gaze_roll_message.rs` | ✓ |
| `client/report/bb2025/InjuryMessage.java` | `ffb-client` | `src/client/report/bb2025/injury_message.rs` | ✓ |
| `client/report/bb2025/KickoffDodgySnackMessage.java` | `ffb-client` | `src/client/report/bb2025/kickoff_dodgy_snack_message.rs` | ✓ |
| `client/report/bb2025/KickoffExtraReRollMessage.java` | `ffb-client` | `src/client/report/bb2025/kickoff_extra_re_roll_message.rs` | ✓ |
| `client/report/bb2025/KickTeamMateFumbleMessage.java` | `ffb-client` | `src/client/report/bb2025/kick_team_mate_fumble_message.rs` | ✓ |
| `client/report/bb2025/MascotUsedMessage.java` | `ffb-client` | `src/client/report/bb2025/mascot_used_message.rs` | ✓ |
| `client/report/bb2025/PassRollMessage.java` | `ffb-client` | `src/client/report/bb2025/pass_roll_message.rs` | ✓ |
| `client/report/bb2025/PickUpRollMessage.java` | `ffb-client` | `src/client/report/bb2025/pick_up_roll_message.rs` | ✓ |
| `client/report/bb2025/PrayerAmountMessage.java` | `ffb-client` | `src/client/report/bb2025/prayer_amount_message.rs` | ✓ |
| `client/report/bb2025/PrayerRollMessage.java` | `ffb-client` | `src/client/report/bb2025/prayer_roll_message.rs` | ✓ |
| `client/report/bb2025/PrayersAndInducementsBoughtMessage.java` | `ffb-client` | `src/client/report/bb2025/prayers_and_inducements_bought_message.rs` | ✓ |
| `client/report/bb2025/PuntDirectionMessage.java` | `ffb-client` | `src/client/report/bb2025/punt_direction_message.rs` | ✓ |
| `client/report/bb2025/PuntDistanceMessage.java` | `ffb-client` | `src/client/report/bb2025/punt_distance_message.rs` | ✓ |
| `client/report/bb2025/PushbackMessage.java` | `ffb-client` | `src/client/report/bb2025/pushback_message.rs` | ✓ |
| `client/report/bb2025/RaiseDeadMessage.java` | `ffb-client` | `src/client/report/bb2025/raise_dead_message.rs` | ✓ |
| `client/report/bb2025/SaboteurRollMessage.java` | `ffb-client` | `src/client/report/bb2025/saboteur_roll_message.rs` | ✓ |
| `client/report/bb2025/SolidDefenceRollMessage.java` | `ffb-client` | `src/client/report/bb2025/solid_defence_roll_message.rs` | ✓ |
| `client/report/bb2025/StallerDetectedMessage.java` | `ffb-client` | `src/client/report/bb2025/staller_detected_message.rs` | ✓ |
| `client/report/bb2025/SteadyFootingRollMessage.java` | `ffb-client` | `src/client/report/bb2025/steady_footing_roll_message.rs` | ✓ |
| `client/report/bb2025/SwarmingPlayersRollMessage.java` | `ffb-client` | `src/client/report/bb2025/swarming_players_roll_message.rs` | ✓ |
| `client/report/bb2025/SwoopDirectionMessage.java` | `ffb-client` | `src/client/report/bb2025/swoop_direction_message.rs` | ✓ |
| `client/report/bb2025/SwoopPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/swoop_player_message.rs` | ✓ |
| `client/report/bb2025/TeamCaptainRollMessage.java` | `ffb-client` | `src/client/report/bb2025/team_captain_roll_message.rs` | ✓ |
| `client/report/bb2025/TeamEventMessage.java` | `ffb-client` | `src/client/report/bb2025/team_event_message.rs` | ✓ |
| `client/report/bb2025/TentaclesShadowingMessage.java` | `ffb-client` | `src/client/report/bb2025/tentacles_shadowing_message.rs` | ✓ |
| `client/report/bb2025/ThenIStartedBlastinMessage.java` | `ffb-client` | `src/client/report/bb2025/then_i_started_blastin_message.rs` | ✓ |
| `client/report/bb2025/ThrowAtPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/throw_at_player_message.rs` | ✓ |
| `client/report/bb2025/ThrowAtStallingPlayerMessage.java` | `ffb-client` | `src/client/report/bb2025/throw_at_stalling_player_message.rs` | ✓ |
| `client/report/bb2025/ThrowTeamMateRollMessage.java` | `ffb-client` | `src/client/report/bb2025/throw_team_mate_roll_message.rs` | ✓ |
| `client/report/bb2025/UseFumblerooskieMessage.java` | `ffb-client` | `src/client/report/bb2025/use_fumblerooskie_message.rs` | ✓ |
| `client/report/bb2025/WeatherMageResultMessage.java` | `ffb-client` | `src/client/report/bb2025/weather_mage_result_message.rs` | ✓ |
| `client/report/BiteSpectatorMessage.java` | `ffb-client` | `src/client/report/bite_spectator_message.rs` | ✓ |
| `client/report/BlockMessage.java` | `ffb-client` | `src/client/report/block_message.rs` | ✓ |
| `client/report/BlockRollMessage.java` | `ffb-client` | `src/client/report/block_roll_message.rs` | ✓ |
| `client/report/BombExplodesAfterCatchMessage.java` | `ffb-client` | `src/client/report/bomb_explodes_after_catch_message.rs` | ✓ |
| `client/report/BombOutOfBoundsMessage.java` | `ffb-client` | `src/client/report/bomb_out_of_bounds_message.rs` | ✓ |
| `client/report/BribesRollMessage.java` | `ffb-client` | `src/client/report/bribes_roll_message.rs` | ✓ |
| `client/report/CardDeactivatedMessage.java` | `ffb-client` | `src/client/report/card_deactivated_message.rs` | ✓ |
| `client/report/CardEffectRollMessage.java` | `ffb-client` | `src/client/report/card_effect_roll_message.rs` | ✓ |
| `client/report/CatchRollMessage.java` | `ffb-client` | `src/client/report/catch_roll_message.rs` | ✓ |
| `client/report/ChainsawRollMessage.java` | `ffb-client` | `src/client/report/chainsaw_roll_message.rs` | ✓ |
| `client/report/CoinThrowMessage.java` | `ffb-client` | `src/client/report/coin_throw_message.rs` | ✓ |
| `client/report/ConfusionRollMessage.java` | `ffb-client` | `src/client/report/confusion_roll_message.rs` | ✓ |
| `client/report/DauntlessRollMessage.java` | `ffb-client` | `src/client/report/dauntless_roll_message.rs` | ✓ |
| `client/report/DefectingPlayersMessage.java` | `ffb-client` | `src/client/report/defecting_players_message.rs` | ✓ |
| `client/report/DodgeRollMessage.java` | `ffb-client` | `src/client/report/dodge_roll_message.rs` | ✓ |
| `client/report/DoubleHiredStarPlayerMessage.java` | `ffb-client` | `src/client/report/double_hired_star_player_message.rs` | ✓ |
| `client/report/EscapeRollMessage.java` | `ffb-client` | `src/client/report/escape_roll_message.rs` | ✓ |
| `client/report/FoulAppearanceRollMessage.java` | `ffb-client` | `src/client/report/foul_appearance_roll_message.rs` | ✓ |
| `client/report/FoulMessage.java` | `ffb-client` | `src/client/report/foul_message.rs` | ✓ |
| `client/report/FumbblResultUploadMessage.java` | `ffb-client` | `src/client/report/fumbbl_result_upload_message.rs` | ✓ |
| `client/report/GameOptionsMessage.java` | `ffb-client` | `src/client/report/game_options_message.rs` | ✓ |
| `client/report/HandOverMessage.java` | `ffb-client` | `src/client/report/hand_over_message.rs` | ✓ |
| `client/report/InterceptionRollMessage.java` | `ffb-client` | `src/client/report/interception_roll_message.rs` | ✓ |
| `client/report/JumpRollMessage.java` | `ffb-client` | `src/client/report/jump_roll_message.rs` | ✓ |
| `client/report/JumpUpRollMessage.java` | `ffb-client` | `src/client/report/jump_up_roll_message.rs` | ✓ |
| `client/report/KickoffResultMessage.java` | `ffb-client` | `src/client/report/kickoff_result_message.rs` | ✓ |
| `client/report/KickoffScatterMessage.java` | `ffb-client` | `src/client/report/kickoff_scatter_message.rs` | ✓ |
| `client/report/LeaderMessage.java` | `ffb-client` | `src/client/report/leader_message.rs` | ✓ |
| `client/report/MasterChefRollMessage.java` | `ffb-client` | `src/client/report/master_chef_roll_message.rs` | ✓ |
| `client/report/mixed/AllYouCanEatMessage.java` | `ffb-client` | `src/client/report/mixed/all_you_can_eat_message.rs` | ✓ |
| `client/report/mixed/ArgueTheCallMessage.java` | `ffb-client` | `src/client/report/mixed/argue_the_call_message.rs` | ✓ |
| `client/report/mixed/BalefulHexRollMessage.java` | `ffb-client` | `src/client/report/mixed/baleful_hex_roll_message.rs` | ✓ |
| `client/report/mixed/BiasedRefMessage.java` | `ffb-client` | `src/client/report/mixed/biased_ref_message.rs` | ✓ |
| `client/report/mixed/BlockChoiceMessage.java` | `ffb-client` | `src/client/report/mixed/block_choice_message.rs` | ✓ |
| `client/report/mixed/BlockReRollMessage.java` | `ffb-client` | `src/client/report/mixed/block_re_roll_message.rs` | ✓ |
| `client/report/mixed/BloodLustRollMessage.java` | `ffb-client` | `src/client/report/mixed/blood_lust_roll_message.rs` | ✓ |
| `client/report/mixed/BreatheFireMessage.java` | `ffb-client` | `src/client/report/mixed/breathe_fire_message.rs` | ✓ |
| `client/report/mixed/BriberyAndCorruptionReRollMessage.java` | `ffb-client` | `src/client/report/mixed/bribery_and_corruption_re_roll_message.rs` | ✓ |
| `client/report/mixed/BrilliantCoachingReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/brilliant_coaching_re_rolls_lost_message.rs` | ✓ |
| `client/report/mixed/CatchOfTheDayMessage.java` | `ffb-client` | `src/client/report/mixed/catch_of_the_day_message.rs` | ✓ |
| `client/report/mixed/CloudBursterMessage.java` | `ffb-client` | `src/client/report/mixed/cloud_burster_message.rs` | ✓ |
| `client/report/mixed/DedicatedFansMessage.java` | `ffb-client` | `src/client/report/mixed/dedicated_fans_message.rs` | ✓ |
| `client/report/mixed/DoubleHiredStaffMessage.java` | `ffb-client` | `src/client/report/mixed/double_hired_staff_message.rs` | ✓ |
| `client/report/mixed/EventMessage.java` | `ffb-client` | `src/client/report/mixed/event_message.rs` | ✓ |
| `client/report/mixed/FanFactorMessage.java` | `ffb-client` | `src/client/report/mixed/fan_factor_message.rs` | ✓ |
| `client/report/mixed/FreePettyCashMessage.java` | `ffb-client` | `src/client/report/mixed/free_petty_cash_message.rs` | ✓ |
| `client/report/mixed/GoForItRollMessage.java` | `ffb-client` | `src/client/report/mixed/go_for_it_roll_message.rs` | ✓ |
| `client/report/mixed/HitAndRunMessage.java` | `ffb-client` | `src/client/report/mixed/hit_and_run_message.rs` | ✓ |
| `client/report/mixed/IndomitableMessage.java` | `ffb-client` | `src/client/report/mixed/indomitable_message.rs` | ✓ |
| `client/report/mixed/InducementMessage.java` | `ffb-client` | `src/client/report/mixed/inducement_message.rs` | ✓ |
| `client/report/mixed/KickoffPitchInvasionMessage.java` | `ffb-client` | `src/client/report/mixed/kickoff_pitch_invasion_message.rs` | ✓ |
| `client/report/mixed/KickoffSequenceActivationsCountMessage.java` | `ffb-client` | `src/client/report/mixed/kickoff_sequence_activations_count_message.rs` | ✓ |
| `client/report/mixed/KickoffSequenceActivationsExhaustedMessage.java` | `ffb-client` | `src/client/report/mixed/kickoff_sequence_activations_exhausted_message.rs` | ✓ |
| `client/report/mixed/KickoffTimeoutMessage.java` | `ffb-client` | `src/client/report/mixed/kickoff_timeout_message.rs` | ✓ |
| `client/report/mixed/LookIntoMyEyesRollMessage.java` | `ffb-client` | `src/client/report/mixed/look_into_my_eyes_roll_message.rs` | ✓ |
| `client/report/mixed/ModifiedDodgeResultSuccessfulMessage.java` | `ffb-client` | `src/client/report/mixed/modified_dodge_result_successful_message.rs` | ✓ |
| `client/report/mixed/ModifiedPassResultMessage.java` | `ffb-client` | `src/client/report/mixed/modified_pass_result_message.rs` | ✓ |
| `client/report/mixed/MostValuablePlayersMessage.java` | `ffb-client` | `src/client/report/mixed/most_valuable_players_message.rs` | ✓ |
| `client/report/mixed/NervesOfSteelMessage.java` | `ffb-client` | `src/client/report/mixed/nerves_of_steel_message.rs` | ✓ |
| `client/report/mixed/OldProMessage.java` | `ffb-client` | `src/client/report/mixed/old_pro_message.rs` | ✓ |
| `client/report/mixed/PenaltyShootoutMessage.java` | `ffb-client` | `src/client/report/mixed/penalty_shootout_message.rs` | ✓ |
| `client/report/mixed/PickMeUpMessage.java` | `ffb-client` | `src/client/report/mixed/pick_me_up_message.rs` | ✓ |
| `client/report/mixed/PickUpRollMessage.java` | `ffb-client` | `src/client/report/mixed/pick_up_roll_message.rs` | ✓ |
| `client/report/mixed/PlaceBallDirectionMessage.java` | `ffb-client` | `src/client/report/mixed/place_ball_direction_message.rs` | ✓ |
| `client/report/mixed/PlayerEventMessage.java` | `ffb-client` | `src/client/report/mixed/player_event_message.rs` | ✓ |
| `client/report/mixed/PrayerEndMessage.java` | `ffb-client` | `src/client/report/mixed/prayer_end_message.rs` | ✓ |
| `client/report/mixed/PrayerWastedMessage.java` | `ffb-client` | `src/client/report/mixed/prayer_wasted_message.rs` | ✓ |
| `client/report/mixed/ProjectileVomitMessage.java` | `ffb-client` | `src/client/report/mixed/projectile_vomit_message.rs` | ✓ |
| `client/report/mixed/PumpUpTheCrowdReRollMessage.java` | `ffb-client` | `src/client/report/mixed/pump_up_the_crowd_re_roll_message.rs` | ✓ |
| `client/report/mixed/PumpUpTheCrowdReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/pump_up_the_crowd_re_rolls_lost_message.rs` | ✓ |
| `client/report/mixed/QuickSnapRollMessage.java` | `ffb-client` | `src/client/report/mixed/quick_snap_roll_message.rs` | ✓ |
| `client/report/mixed/RaidingPartyMessage.java` | `ffb-client` | `src/client/report/mixed/raiding_party_message.rs` | ✓ |
| `client/report/mixed/RefereeMessage.java` | `ffb-client` | `src/client/report/mixed/referee_message.rs` | ✓ |
| `client/report/mixed/ScatterBallMessage.java` | `ffb-client` | `src/client/report/mixed/scatter_ball_message.rs` | ✓ |
| `client/report/mixed/ScatterPlayerMessage.java` | `ffb-client` | `src/client/report/mixed/scatter_player_message.rs` | ✓ |
| `client/report/mixed/SelectBlitzTargetMessage.java` | `ffb-client` | `src/client/report/mixed/select_blitz_target_message.rs` | ✓ |
| `client/report/mixed/SelectGazeTargetMessage.java` | `ffb-client` | `src/client/report/mixed/select_gaze_target_message.rs` | ✓ |
| `client/report/mixed/ShowStarReRollMessage.java` | `ffb-client` | `src/client/report/mixed/show_star_re_roll_message.rs` | ✓ |
| `client/report/mixed/ShowStarReRollsLostMessage.java` | `ffb-client` | `src/client/report/mixed/show_star_re_rolls_lost_message.rs` | ✓ |
| `client/report/mixed/SkillUseOtherPlayerMessage.java` | `ffb-client` | `src/client/report/mixed/skill_use_other_player_message.rs` | ✓ |
| `client/report/mixed/SkillWastedMessage.java` | `ffb-client` | `src/client/report/mixed/skill_wasted_message.rs` | ✓ |
| `client/report/mixed/ThrownKegMessage.java` | `ffb-client` | `src/client/report/mixed/thrown_keg_message.rs` | ✓ |
| `client/report/mixed/TrapDoorMessage.java` | `ffb-client` | `src/client/report/mixed/trap_door_message.rs` | ✓ |
| `client/report/mixed/TurnEndMessage.java` | `ffb-client` | `src/client/report/mixed/turn_end_message.rs` | ✓ |
| `client/report/mixed/WeatherMageRollMessage.java` | `ffb-client` | `src/client/report/mixed/weather_mage_roll_message.rs` | ✓ |
| `client/report/mixed/WinningsMessage.java` | `ffb-client` | `src/client/report/mixed/winnings_message.rs` | ✓ |
| `client/report/PassBlockMessage.java` | `ffb-client` | `src/client/report/pass_block_message.rs` | ✓ |
| `client/report/PassDeviateMessage.java` | `ffb-client` | `src/client/report/pass_deviate_message.rs` | ✓ |
| `client/report/PettyCashMessage.java` | `ffb-client` | `src/client/report/petty_cash_message.rs` | ✓ |
| `client/report/PilingOnMessage.java` | `ffb-client` | `src/client/report/piling_on_message.rs` | ✓ |
| `client/report/PlayCardMessage.java` | `ffb-client` | `src/client/report/play_card_message.rs` | ✓ |
| `client/report/PlayerActionMessage.java` | `ffb-client` | `src/client/report/player_action_message.rs` | ✓ |
| `client/report/PushbackMessage.java` | `ffb-client` | `src/client/report/pushback_message.rs` | ✓ |
| `client/report/ReceiveChoiceMessage.java` | `ffb-client` | `src/client/report/receive_choice_message.rs` | ✓ |
| `client/report/RegenerationRollMessage.java` | `ffb-client` | `src/client/report/regeneration_roll_message.rs` | ✓ |
| `client/report/ReportMessageBase.java` | `ffb-client` | `src/client/report/report_message_base.rs` | ✓ |
| `client/report/ReportMessageType.java` | `ffb-client` | `src/client/report/report_message_type.rs` | ✓ |
| `client/report/ReRollMessage.java` | `ffb-client` | `src/client/report/re_roll_message.rs` | ✓ |
| `client/report/RightStuffRollMessage.java` | `ffb-client` | `src/client/report/right_stuff_roll_message.rs` | ✓ |
| `client/report/RiotousRookiesMessage.java` | `ffb-client` | `src/client/report/riotous_rookies_message.rs` | ✓ |
| `client/report/SafeThrowRollMessage.java` | `ffb-client` | `src/client/report/safe_throw_roll_message.rs` | ✓ |
| `client/report/SecretWeaponBanMessage.java` | `ffb-client` | `src/client/report/secret_weapon_ban_message.rs` | ✓ |
| `client/report/SkillUseMessage.java` | `ffb-client` | `src/client/report/skill_use_message.rs` | ✓ |
| `client/report/SpellEffectRollMessage.java` | `ffb-client` | `src/client/report/spell_effect_roll_message.rs` | ✓ |
| `client/report/StandUpRollMessage.java` | `ffb-client` | `src/client/report/stand_up_roll_message.rs` | ✓ |
| `client/report/StartHalfMessage.java` | `ffb-client` | `src/client/report/start_half_message.rs` | ✓ |
| `client/report/ThrowInMessage.java` | `ffb-client` | `src/client/report/throw_in_message.rs` | ✓ |
| `client/report/TimeoutEnforcedMessage.java` | `ffb-client` | `src/client/report/timeout_enforced_message.rs` | ✓ |
| `client/report/WeatherMessage.java` | `ffb-client` | `src/client/report/weather_message.rs` | ✓ |
| `client/report/WeepingDaggerRollMessage.java` | `ffb-client` | `src/client/report/weeping_dagger_roll_message.rs` | ✓ |
| `client/report/WizardUseMessage.java` | `ffb-client` | `src/client/report/wizard_use_message.rs` | ✓ |

### client/root/ (31 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/ActionKey.java` | `ffb-client` | `src/client/action_key.rs` | ✓ |
| `client/ActionKeyAction.java` | `ffb-client` | `src/client/ActionKeyAction.rs` | — |
| `client/ActionKeyBindings.java` | `ffb-client` | `src/client/ActionKeyBindings.rs` | — |
| `client/ActionKeyGroup.java` | `ffb-client` | `src/client/ActionKeyGroup.rs` | — |
| `client/ActionKeyMultiAction.java` | `ffb-client` | `src/client/ActionKeyMultiAction.rs` | — |
| `client/ClientData.java` | `ffb-client` | `src/client/client_data.rs` | ✓ |
| `client/ClientLayout.java` | `ffb-client` | `src/client/client_layout.rs` | ✓ (triage correction: plain data enum, no AWT dep — see Progress Summary) |
| `client/ClientParameters.java` | `ffb-client` | `src/client/client_parameters.rs` | ✓ |
| `client/ClientReplayer.java` | `ffb-client` | `src/client/ClientReplayer.rs` | ~ (blocked: `implements ActionListener` driven by `javax.swing.Timer`, deeply calls `getUserInterface()` for playback UI/log highlighting; `createGame()`/`cloneGame()` reconstruct `Game` via `new Game(IFactorySource, FactoryManager)`, a constructor shape this project's ported `Game::new(home, away, rules)` doesn't match — see `FantasyFootballClient`'s doc note. `client/state/` calls `getReplayer()` 24×, likely only needing a small logic-only subset (`isReplaying`/`hasControl`/speed state) — real follow-up, not a narrow gap) |
| `client/Component.java` | `ffb-client` | `src/client/Component.rs` | — |
| `client/CoordinateConverter.java` | `ffb-client` | `src/client/CoordinateConverter.rs` | — (triage correction: `getFieldCoordinate` takes a Swing `MouseEvent` and calls into `UiDimensionProvider`/`PitchDimensionProvider`, both GUI-skip scale providers — not a narrow dependency, genuinely Swing-bound; see Progress Summary) |
| `client/DimensionProvider.java` | `ffb-client` | `src/client/DimensionProvider.rs` | — |
| `client/DugoutDimensionProvider.java` | `ffb-client` | `src/client/DugoutDimensionProvider.rs` | — |
| `client/FantasyFootballClient.java` | `ffb-client` | `src/client/fantasy_football_client.rs` | ✓ (promoted from GUI-skip to a real hybrid struct — see Progress Summary) |
| `client/FieldComponent.java` | `ffb-client` | `src/client/FieldComponent.rs` | — |
| `client/FontCache.java` | `ffb-client` | `src/client/FontCache.rs` | — |
| `client/GameTitle.java` | `ffb-client` | `src/client/GameTitle.rs` | — |
| `client/IconCache.java` | `ffb-client` | `src/client/IconCache.rs` | — |
| `client/IProgressListener.java` | `ffb-client` | `src/client/i_progress_listener.rs` | ✓ |
| `client/LayoutSettings.java` | `ffb-client` | `src/client/LayoutSettings.rs` | — |
| `client/ParagraphStyle.java` | `ffb-client` | `src/client/paragraph_style.rs` | ✓ (un-skipped: plain string-keyed enum, no AWT/Swing — miscategorized in the ZW.0 bulk audit alongside genuinely-Swing root files) |
| `client/PitchDimensionProvider.java` | `ffb-client` | `src/client/PitchDimensionProvider.rs` | — |
| `client/PlayerIconFactory.java` | `ffb-client` | `src/client/PlayerIconFactory.rs` | — (triage correction: every method operates on `BufferedImage`/`Graphics2D` — genuine AWT icon compositing, not narrowly Swing-touched) |
| `client/RenderContext.java` | `ffb-client` | `src/client/RenderContext.rs` | — |
| `client/ReplayControl.java` | `ffb-client` | `src/client/ReplayControl.rs` | — (triage correction: `extends JPanel implements MouseInputListener` — a real Swing widget, not plain logic) |
| `client/StatusReport.java` | `ffb-client` | `src/client/status_report.rs` | ✓ (unblocked by ZW.3: the one Swing sink, `getUserInterface().getLog().append(...)`, is now a headless `rendered_runs: Vec<RenderedRun>` capture) |
| `client/StyleProvider.java` | `ffb-client` | `src/client/StyleProvider.rs` | — |
| `client/TextStyle.java` | `ffb-client` | `src/client/text_style.rs` | ✓ (un-skipped: plain string-keyed enum, no AWT/Swing — miscategorized in the ZW.0 bulk audit alongside genuinely-Swing root files) |
| `client/UiDimensionProvider.java` | `ffb-client` | `src/client/UiDimensionProvider.rs` | — |
| `client/UserInterface.java` | `ffb-client` | `src/client/UserInterface.rs` | — |
| `client/UtilStyle.java` | `ffb-client` | `src/client/UtilStyle.rs` | — |

### client/sound/ (2 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/sound/ISoundProperty.java` | `ffb-client` | `src/client/sound/ISoundProperty.rs` | — |
| `client/sound/SoundEngine.java` | `ffb-client` | `src/client/sound/SoundEngine.rs` | — |

### client/state/ (85 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/state/ClientState.java` | `ffb-client` | `src/client/state/client_state.rs` | ✓ |
| `client/state/ClientStateFactory.java` | `ffb-client` | `src/client/state/client_state_factory.rs` | ✓ (registry shell only — `get_state_for_game`/`find_passive_state` are fully ported; see `crate::state_dispatch` for the pre-existing, deliberately coarser TurnMode-only dispatcher this file supersedes as ground truth) |
| `client/state/IPlayerPopupMenuKeys.java` | `ffb-client` | `src/client/state/i_player_popup_menu_keys.rs` | ✓ |
| `client/state/logic/AbstractBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/abstract_block_logic_module.rs` | ✓ |
| `client/state/logic/bb2016/KtmLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2016/ktm_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/GazeMoveLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/gaze_move_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/KickTeamMateLikeThrowLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/kick_team_mate_like_throw_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/SelectBlitzTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/select_blitz_target_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/SelectGazeTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/select_gaze_target_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/StabLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/stab_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/SynchronousMultiBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/synchronous_multi_block_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/ThrowKegLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/throw_keg_logic_module.rs` | ✓ |
| `client/state/logic/bb2020/TricksterLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2020/trickster_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/BlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/block_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/BombLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/bomb_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/FoulLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/foul_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/GazeLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/gaze_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/GazeMoveLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/gaze_move_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/HandOverLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/hand_over_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/PassLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/pass_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/PuntLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/punt_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/SelectBlitzTargetLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/select_blitz_target_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/SelectLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/select_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/SwarmingLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/swarming_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/SynchronousMultiBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/synchronous_multi_block_logic_module.rs` | ✓ |
| `client/state/logic/bb2025/ThrowKegLogicModule.java` | `ffb-client` | `src/client/state/logic/bb2025/throw_keg_logic_module.rs` | ✓ |
| `client/state/logic/BlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/blitz_logic_module.rs` | ✓ |
| `client/state/logic/BlockLogicExtension.java` | `ffb-client` | `src/client/state/logic/block_logic_extension.rs` | ✓ |
| `client/state/logic/ClientAction.java` | `ffb-client` | `src/client/state/logic/client_action.rs` | ✓ |
| `client/state/logic/DumpOffLogicModule.java` | `ffb-client` | `src/client/state/logic/dump_off_logic_module.rs` | ✓ |
| `client/state/logic/HighKickLogicModule.java` | `ffb-client` | `src/client/state/logic/high_kick_logic_module.rs` | ✓ |
| `client/state/logic/IllegalSubstitutionLogicModule.java` | `ffb-client` | `src/client/state/logic/illegal_substitution_logic_module.rs` | ✓ |
| `client/state/logic/Influences.java` | `ffb-client` | `src/client/state/logic/influences.rs` | ✓ |
| `client/state/logic/interaction/ActionContext.java` | `ffb-client` | `src/client/state/logic/interaction/action_context.rs` | ✓ |
| `client/state/logic/interaction/InteractionResult.java` | `ffb-client` | `src/client/state/logic/interaction/interaction_result.rs` | ✓ |
| `client/state/logic/InterceptionLogicModule.java` | `ffb-client` | `src/client/state/logic/interception_logic_module.rs` | ✓ |
| `client/state/logic/KickoffLogicModule.java` | `ffb-client` | `src/client/state/logic/kickoff_logic_module.rs` | ✓ |
| `client/state/logic/KickoffReturnLogicModule.java` | `ffb-client` | `src/client/state/logic/kickoff_return_logic_module.rs` | ✓ |
| `client/state/logic/LogicModule.java` | `ffb-client` | `src/client/state/logic/logic_module.rs` | ✓ |
| `client/state/logic/LoginLogicModule.java` | `ffb-client` | `src/client/state/logic/login_logic_module.rs` | ✓ |
| `client/state/logic/mixed/BlockKindLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/block_kind_logic_module.rs` | ✓ |
| `client/state/logic/mixed/BlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/block_logic_module.rs` | ✓ |
| `client/state/logic/mixed/BombLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/bomb_logic_module.rs` | ✓ |
| `client/state/logic/mixed/FoulLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/foul_logic_module.rs` | ✓ |
| `client/state/logic/mixed/FuriousOutburstLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/furious_outburst_logic_module.rs` | ✓ |
| `client/state/logic/mixed/GazeLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/gaze_logic_module.rs` | ✓ |
| `client/state/logic/mixed/HandOverLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/hand_over_logic_module.rs` | ✓ |
| `client/state/logic/mixed/HitAndRunLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/hit_and_run_logic_module.rs` | ✓ |
| `client/state/logic/mixed/KickEmBlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/kick_em_blitz_logic_module.rs` | ✓ |
| `client/state/logic/mixed/KickEmBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/kick_em_block_logic_module.rs` | ✓ |
| `client/state/logic/mixed/MaximumCarnageLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/maximum_carnage_logic_module.rs` | ✓ |
| `client/state/logic/mixed/PassLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/pass_logic_module.rs` | ✓ |
| `client/state/logic/mixed/PutridRegurgitationBlitzLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/putrid_regurgitation_blitz_logic_module.rs` | ✓ |
| `client/state/logic/mixed/PutridRegurgitationBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/putrid_regurgitation_block_logic_module.rs` | ✓ |
| `client/state/logic/mixed/RaidingPartyLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/raiding_party_logic_module.rs` | ✓ |
| `client/state/logic/mixed/SelectLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/select_logic_module.rs` | ✓ |
| `client/state/logic/mixed/SwarmingLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/swarming_logic_module.rs` | ✓ |
| `client/state/logic/mixed/ThenIStartedBlastinLogicModule.java` | `ffb-client` | `src/client/state/logic/mixed/then_i_started_blastin_logic_module.rs` | ✓ |
| `client/state/logic/MoveLogicModule.java` | `ffb-client` | `src/client/state/logic/move_logic_module.rs` | ✓ |
| `client/state/logic/PassBlockLogicModule.java` | `ffb-client` | `src/client/state/logic/pass_block_logic_module.rs` | ✓ |
| `client/state/logic/PlaceBallLogicModule.java` | `ffb-client` | `src/client/state/logic/place_ball_logic_module.rs` | ✓ |
| `client/state/logic/plugin/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/base_logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/bb2025/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/base_logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/bb2025/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/block_logic_extension_plugin.rs` | ✓ |
| `client/state/logic/plugin/bb2025/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/bb2025/move_logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/block_logic_extension_plugin.rs` | ✓ |
| `client/state/logic/plugin/LogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/mixed/BaseLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/base_logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/mixed/BlockLogicExtensionPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/block_logic_extension_plugin.rs` | ✓ |
| `client/state/logic/plugin/mixed/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/mixed/move_logic_plugin.rs` | ✓ |
| `client/state/logic/plugin/MoveLogicPlugin.java` | `ffb-client` | `src/client/state/logic/plugin/move_logic_plugin.rs` | ✓ |
| `client/state/logic/PushbackLogicModule.java` | `ffb-client` | `src/client/state/logic/pushback_logic_module.rs` | ✓ |
| `client/state/logic/QuickSnapLogicModule.java` | `ffb-client` | `src/client/state/logic/quick_snap_logic_module.rs` | ✓ |
| `client/state/logic/RangeGridState.java` | `ffb-client` | `src/client/state/logic/range_grid_state.rs` | ✓ |
| `client/state/logic/ReplayLogicModule.java` | `ffb-client` | `src/client/state/logic/replay_logic_module.rs` | ✓ |
| `client/state/logic/SetupLogicModule.java` | `ffb-client` | `src/client/state/logic/setup_logic_module.rs` | ✓ |
| `client/state/logic/SolidDefenceLogicModule.java` | `ffb-client` | `src/client/state/logic/solid_defence_logic_module.rs` | ✓ |
| `client/state/logic/SpectateLogicModule.java` | `ffb-client` | `src/client/state/logic/spectate_logic_module.rs` | ✓ |
| `client/state/logic/StartGameLogicModule.java` | `ffb-client` | `src/client/state/logic/start_game_logic_module.rs` | ✓ |
| `client/state/logic/SwoopLogicModule.java` | `ffb-client` | `src/client/state/logic/swoop_logic_module.rs` | ✓ |
| `client/state/logic/ThrowTeamMateLogicModule.java` | `ffb-client` | `src/client/state/logic/throw_team_mate_logic_module.rs` | ✓ |
| `client/state/logic/TouchbackLogicModule.java` | `ffb-client` | `src/client/state/logic/touchback_logic_module.rs` | ✓ |
| `client/state/logic/WaitForOpponentLogicModule.java` | `ffb-client` | `src/client/state/logic/wait_for_opponent_logic_module.rs` | ✓ |
| `client/state/logic/WaitForSetupLogicModule.java` | `ffb-client` | `src/client/state/logic/wait_for_setup_logic_module.rs` | ✓ |
| `client/state/logic/WizardLogicModule.java` | `ffb-client` | `src/client/state/logic/wizard_logic_module.rs` | ✓ |

### client/ui/ (69 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/ui/BoxButtonComponent.java` | `ffb-client` | `src/client/ui/BoxButtonComponent.rs` | — |
| `client/ui/BoxComponent.java` | `ffb-client` | `src/client/ui/BoxComponent.rs` | — |
| `client/ui/BoxSlot.java` | `ffb-client` | `src/client/ui/BoxSlot.rs` | — |
| `client/ui/chat/Autocomplete.java` | `ffb-client` | `src/client/ui/chat/Autocomplete.rs` | — |
| `client/ui/chat/AutocompleteGenerator.java` | `ffb-client` | `src/client/ui/chat/AutocompleteGenerator.rs` | — |
| `client/ui/chat/ChatSegment.java` | `ffb-client` | `src/client/ui/chat/ChatSegment.rs` | — |
| `client/ui/chat/EmojiLookup.java` | `ffb-client` | `src/client/ui/chat/EmojiLookup.rs` | — |
| `client/ui/chat/EmojiPicker.java` | `ffb-client` | `src/client/ui/chat/EmojiPicker.rs` | — |
| `client/ui/chat/MessageParser.java` | `ffb-client` | `src/client/ui/chat/MessageParser.rs` | — |
| `client/ui/ChatButtonComponent.java` | `ffb-client` | `src/client/ui/ChatButtonComponent.rs` | — |
| `client/ui/ChatComponent.java` | `ffb-client` | `src/client/ui/ChatComponent.rs` | — |
| `client/ui/ChatLogDocument.java` | `ffb-client` | `src/client/ui/ChatLogDocument.rs` | — |
| `client/ui/ChatLogScrollPane.java` | `ffb-client` | `src/client/ui/ChatLogScrollPane.rs` | — |
| `client/ui/ChatLogTextPane.java` | `ffb-client` | `src/client/ui/ChatLogTextPane.rs` | — |
| `client/ui/ColorIcon.java` | `ffb-client` | `src/client/ui/ColorIcon.rs` | — |
| `client/ui/CommandHighlightArea.java` | `ffb-client` | `src/client/ui/CommandHighlightArea.rs` | — |
| `client/ui/CommandHighlighter.java` | `ffb-client` | `src/client/ui/CommandHighlighter.rs` | — |
| `client/ui/GameTitleUpdateTask.java` | `ffb-client` | `src/client/ui/GameTitleUpdateTask.rs` | — |
| `client/ui/GraphicsEnhancer.java` | `ffb-client` | `src/client/ui/GraphicsEnhancer.rs` | — |
| `client/ui/IntegerField.java` | `ffb-client` | `src/client/ui/IntegerField.rs` | — |
| `client/ui/IReplayMouseListener.java` | `ffb-client` | `src/client/ui/IReplayMouseListener.rs` | — |
| `client/ui/LogComponent.java` | `ffb-client` | `src/client/ui/LogComponent.rs` | — |
| `client/ui/menu/CardsMenu.java` | `ffb-client` | `src/client/ui/menu/CardsMenu.rs` | — |
| `client/ui/menu/FfbMenu.java` | `ffb-client` | `src/client/ui/menu/FfbMenu.rs` | — |
| `client/ui/menu/game/GameModeMenu.java` | `ffb-client` | `src/client/ui/menu/game/GameModeMenu.rs` | — |
| `client/ui/menu/game/ReplayMenu.java` | `ffb-client` | `src/client/ui/menu/game/ReplayMenu.rs` | — |
| `client/ui/menu/game/StandardGameMenu.java` | `ffb-client` | `src/client/ui/menu/game/StandardGameMenu.rs` | — |
| `client/ui/menu/GameMenuBar.java` | `ffb-client` | `src/client/ui/menu/GameMenuBar.rs` | — |
| `client/ui/menu/HelpMenu.java` | `ffb-client` | `src/client/ui/menu/HelpMenu.rs` | — |
| `client/ui/menu/InducementsMenu.java` | `ffb-client` | `src/client/ui/menu/InducementsMenu.rs` | — |
| `client/ui/menu/MissingPlayersMenu.java` | `ffb-client` | `src/client/ui/menu/MissingPlayersMenu.rs` | — |
| `client/ui/menu/OptionsMenu.java` | `ffb-client` | `src/client/ui/menu/OptionsMenu.rs` | — |
| `client/ui/menu/PrayersMenu.java` | `ffb-client` | `src/client/ui/menu/PrayersMenu.rs` | — |
| `client/ui/menu/settings/ClientGraphicsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/ClientGraphicsMenu.rs` | — |
| `client/ui/menu/settings/ClientSettingsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/ClientSettingsMenu.rs` | — |
| `client/ui/menu/settings/GamePlayMenu.java` | `ffb-client` | `src/client/ui/menu/settings/GamePlayMenu.rs` | — |
| `client/ui/menu/settings/UserSettingsMenu.java` | `ffb-client` | `src/client/ui/menu/settings/UserSettingsMenu.rs` | — |
| `client/ui/menu/SetupMenu.java` | `ffb-client` | `src/client/ui/menu/SetupMenu.rs` | — |
| `client/ui/OffsetIcon.java` | `ffb-client` | `src/client/ui/OffsetIcon.rs` | — |
| `client/ui/PlayerDetailComponent.java` | `ffb-client` | `src/client/ui/PlayerDetailComponent.rs` | — |
| `client/ui/ResourceComponent.java` | `ffb-client` | `src/client/ui/ResourceComponent.rs` | — |
| `client/ui/ResourceSlot.java` | `ffb-client` | `src/client/ui/ResourceSlot.rs` | — |
| `client/ui/ResourceValue.java` | `ffb-client` | `src/client/ui/ResourceValue.rs` | — |
| `client/ui/ScoreBarComponent.java` | `ffb-client` | `src/client/ui/ScoreBarComponent.rs` | — |
| `client/ui/SideBarComponent.java` | `ffb-client` | `src/client/ui/SideBarComponent.rs` | — |
| `client/ui/strategies/click/ClickStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/ClickStrategy.rs` | — |
| `client/ui/strategies/click/ClickStrategyRegistry.java` | `ffb-client` | `src/client/ui/strategies/click/ClickStrategyRegistry.rs` | — |
| `client/ui/strategies/click/DoubleClickStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/DoubleClickStrategy.rs` | — |
| `client/ui/strategies/click/LeftClickAltStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickAltStrategy.rs` | — |
| `client/ui/strategies/click/LeftClickCtrlStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickCtrlStrategy.rs` | — |
| `client/ui/strategies/click/LeftClickNoModifierStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickNoModifierStrategy.rs` | — |
| `client/ui/strategies/click/LeftClickShiftStrategy.java` | `ffb-client` | `src/client/ui/strategies/click/LeftClickShiftStrategy.rs` | — |
| `client/ui/swing/JButton.java` | `ffb-client` | `src/client/ui/swing/JButton.rs` | — |
| `client/ui/swing/JCheckBox.java` | `ffb-client` | `src/client/ui/swing/JCheckBox.rs` | — |
| `client/ui/swing/JComboBox.java` | `ffb-client` | `src/client/ui/swing/JComboBox.rs` | — |
| `client/ui/swing/JLabel.java` | `ffb-client` | `src/client/ui/swing/JLabel.rs` | — |
| `client/ui/swing/JList.java` | `ffb-client` | `src/client/ui/swing/JList.rs` | — |
| `client/ui/swing/JMenu.java` | `ffb-client` | `src/client/ui/swing/JMenu.rs` | — |
| `client/ui/swing/JMenuItem.java` | `ffb-client` | `src/client/ui/swing/JMenuItem.rs` | — |
| `client/ui/swing/JPasswordField.java` | `ffb-client` | `src/client/ui/swing/JPasswordField.rs` | — |
| `client/ui/swing/JProgressBar.java` | `ffb-client` | `src/client/ui/swing/JProgressBar.rs` | — |
| `client/ui/swing/JRadioButton.java` | `ffb-client` | `src/client/ui/swing/JRadioButton.rs` | — |
| `client/ui/swing/JRadioButtonMenuItem.java` | `ffb-client` | `src/client/ui/swing/JRadioButtonMenuItem.rs` | — |
| `client/ui/swing/JTabbedPane.java` | `ffb-client` | `src/client/ui/swing/JTabbedPane.rs` | — |
| `client/ui/swing/JTable.java` | `ffb-client` | `src/client/ui/swing/JTable.rs` | — |
| `client/ui/swing/JTextField.java` | `ffb-client` | `src/client/ui/swing/JTextField.rs` | — |
| `client/ui/swing/ScaledBorderFactory.java` | `ffb-client` | `src/client/ui/swing/ScaledBorderFactory.rs` | — |
| `client/ui/swing/WrappingEditorKit.java` | `ffb-client` | `src/client/ui/swing/WrappingEditorKit.rs` | — |
| `client/ui/TurnDiceStatusComponent.java` | `ffb-client` | `src/client/ui/TurnDiceStatusComponent.rs` | — |

### client/util/ (11 files)

| Java File | Rust Crate | Rust Target | Status |
|-----------|-----------|-------------|--------|
| `client/util/MarkerService.java` | `ffb-client` | `src/client/util/MarkerService.rs` | — |
| `client/util/rng/MouseEntropySource.java` | `ffb-client` | `src/client/util/rng/MouseEntropySource.rs` | — |
| `client/util/UtilClientActionKeys.java` | `ffb-client` | `src/client/util/action_keys.rs` | ✓ |
| `client/util/UtilClientChat.java` | `ffb-client` | `src/client/util/chat.rs` | ✓ |
| `client/util/UtilClientCursor.java` | `ffb-client` | `src/client/util/UtilClientCursor.rs` | — |
| `client/util/UtilClientGraphics.java` | `ffb-client` | `src/client/util/UtilClientGraphics.rs` | — |
| `client/util/UtilClientJTable.java` | `ffb-client` | `src/client/util/UtilClientJTable.rs` | — |
| `client/util/UtilClientPlayerDrag.java` | `ffb-client` | `src/client/util/UtilClientPlayerDrag.rs` | — |
| `client/util/UtilClientReflection.java` | `ffb-client` | `src/client/util/UtilClientReflection.rs` | — |
| `client/util/UtilClientThrowTeamMate.java` | `ffb-client` | `src/client/util/UtilClientThrowTeamMate.rs` | — |
| `client/util/UtilClientTimeout.java` | `ffb-client` | `src/client/util/util_client_timeout.rs` | ✓ |

