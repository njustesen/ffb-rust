# Cross-Engine Test Parity (2026-07-20)

**Final state: Rust `cargo test --workspace` 16,256 passed / 0 failed / 5 documented ignores;
Java `mvn -pl ffb-common,ffb-server,ffb-client-logic test` 2,522 passed / 0 failed
(was 17,984 Rust / ~1,422 Java before this effort: ~2,150 non-sensible Rust tests pruned,
~1,100 mirrored tests added across both engines).**

Goal (user): confidence in behavioral and structural parity between the Java engine
(`ffb-java/`, reference) and the Rust translation (`ffb-rust/`), with the **same tests, same
function names (camelCase↔snake_case), same assertions** on both engines wherever the two
codebases have comparable surface — plus removal of tests that added no value.

## The mirrored core suite

These domains now have 1:1 test parity — every Java test method has a Rust `#[test]` with the
snake_case of its name and the same inputs/assertions, and vice versa:

| Domain | Java location | Rust location |
|---|---|---|
| Skill definitions (297 skills) | `ffb-server/src/test/.../skill/*SkillTest.java` (202 files) | `crates/ffb-model/src/skill/**` `#[cfg(test)]` |
| Calc utilities (17 classes) | `.../util/*CalcTest.java`, `StringToolTest` | `crates/ffb-engine/src/util/*.rs` |
| Mechanic calcs | `.../mechanic/{Injury,Casualty,Spp}CalcTest.java` | `crates/ffb-engine/src/mechanic/*.rs` |
| Auto-marking | `.../marking/MarkerGeneratorTest.java` (47) | `crates/ffb-engine/src/marking/marker_generator.rs` |
| Injury types | `.../injury/injuryType/InjuryTypeBlockBB2025Test.java` | `crates/ffb-engine/src/injury/injuryType/injury_type_block.rs` |
| Model enums | `.../model/*EnumTest.java`, `EnumRoundTripTest` | `crates/ffb-model/src/enums/*.rs` |
| Model types | `.../model/{FieldCoordinate,MoveSquare,BlockRoll,GameConstants}Test.java` | `crates/ffb-model/src/types/*.rs` |
| Properties | `ffb-common/.../CommonPropertyTest.java`, `RulesTest` | `crates/ffb-model/src/model/common_property.rs`, `enums/rules.rs` |
| Client handlers | `ffb-client-logic/.../SubHandlerGameStateMarkingTest.java` | `crates/ffb-client/.../sub_handler_game_state_marking.rs` |
| Server misc | `.../commandline/InifileParamFilterTest`, `.../net/SessionTimeoutTaskTest`, `ServerUrlPropertyTest` | `crates/ffb-server/...`, `crates/ffb-engine/src/server_url_property.rs` |

Conventions used by the mirror:
- Java `name_is_Dodge` → Rust `name_is_dodge`; illegal identifiers mapped
  (`_2016IsNotEligibleFor2020` → `bb2016_is_not_eligible_for_2020`).
- Java `@ParameterizedTest`/`@EnumSource`/`@CsvSource` → one Rust `#[test]` looping the same rows.
- Java `hasSkillProperty(NamedProperties.x)` (after `postConstruct()`) → Rust
  `SkillId::X.properties().contains(&"x")` — the **live** property table in
  `crates/ffb-model/src/enums/skill_id.rs` (per-skill-struct registration methods are dead code).
- Java `CancelSkillProperty(x)` → property string `"cancelsX"` in the same table.
- Java `assertThrows(IllegalArgumentException)` ↔ Rust `Option::None` returns, cross-referenced
  in comments where the error-handling idiom differs (e.g. ThrowInCalc).

## Deliberate asymmetries (documented, not bugs)

**Rust-only tests (kept):** the Java engine has no unit tests for whole subsystems that the Rust
suite covers heavily — engine steps (~5,000 tests, `crates/ffb-engine/src/step/**`),
skill behaviours (419, `skill_behaviour/**`), protocol serialization round-trips (~880,
`ffb-protocol`), report wire-format round-trips (391, `ffb-model/src/report/**`), client
handlers, factories, serde round-trips. These pin the Rust translation's internals and the wire
contract; porting them to Java would mean building a Mockito harness for every step — out of
scope. Behavioral equivalence for the step machine is instead enforced by the **ffb-parity
harness** (`crates/ffb-parity`, driven by `scripts/parity_run.py` / `run_t3_*.ps1`), which runs
both engines headless on the same seed and diffs JSONL logs, plus the T3 coverage checklist.

**Java-only tests (kept):** reflection-based tests with no Rust analogue
(`@RulesCollection` edition-annotation checks per skill — covered in Rust at table level by
`crates/ffb-mechanics/src/skills/mod.rs` exhaustive tests incl. pinned per-edition counts;
`assertFieldLength` reflection in CommonPropertyValueTest/IClientPropertyValueTest;
null-argument tests inexpressible on Rust slices; Mockito transient-marker assertions where the
Rust FieldModel keeps no transient lists). `DialogBlockRollPropertiesTest` /
`DialogReRollPropertiesTest` contain zero `@Test` methods (manual Swing launchers) — not tests.

**Skill edition variants:** Java has one `XSkillTest` per skill *simple name* (199+3 created);
where a skill class exists in several edition packages, the Rust file for each edition mirrors
the same-name Java test but re-verified against **that edition's** `postConstruct()` (documented
per-file). `SkillId::properties()` is a cross-edition union, so Java per-edition
`does_not_have_*` assertions are mirrored only when the property is absent from the union.

## Non-sensible tests removed

Rust (~2,150 removed):
- 344 `debug_format_nonempty` / `clone_does_not_panic` / assertion-free construction tests
  (ffb-protocol) + 82 more across the other crates.
- ~796 self-restating table tests in `ffb-mechanics/src/skills/mod.rs` → replaced by 6
  exhaustive data-driven tests (unique ids/class-names, cross-check vs `SkillId::class_name()`,
  lookup round-trips, pinned edition counts 27 common / 58 BB2016 / 135 BB2020 / 157 BB2025,
  pinned category distribution).
- 921 trivial getter read-backs in `ffb-model/src/report/**` (every report's wire id remains
  pinned by `to_json_value_has_report_id` + `serialization_round_trip`).
- Dedup in `ffb-mechanics` where engine-side mirror modules carry the Java-aligned suite
  (foul/kickoff/movement/scatter/post_match/roll/special_roll/stat trimmed to
  mechanics-crate-specific cases with pointer comments).

Java (removed/fixed):
- `{Armor,Injury,Weather}ModifierValue{s,Test}.java` deleted — orphan constant classes with zero
  production callers, created purely as test scaffolding (verified absent upstream in
  christerk/ffb); Rust twins `crates/ffb-engine/src/mechanic/*_modifier_values.rs` deleted to match.
- Misnamed tests renamed (assertions untouched): `attackerExactlyDoubleDefender_returnsThreeDice`
  → `..._returnsTwoDice` etc. in BlockDiceCalcTest.
- The 17 `TalkHandler*Test.java` under `src/main/.../handler/talk/` are **production** test-game
  commands, not tests — untouched, excluded from parity accounting.

## Real parity bugs found & fixed during mirroring

Property table (`skill_id.rs`) — 56 Java-registered properties were missing (live gameplay
effects silently absent in Rust): Timmmber, SafeThrow, BreatheFire, JumpUp, StripBall, SureHands,
ArmBar, BigHand, BloodLust, DivingTackle, Leader, MyBall, PickMeUp, PlagueRidden,
UnchannelledFury, PrimalSavagery, RaidingParty, Reliable, SavageBlow, SneakiestOfTheLot,
StarOfTheShow, SwiftAsTheBreeze, TastyMorsel, TheFlashingBlade, ThinkingMansTroll,
UnstoppableMomentum, ViciousVines, Bullseye, Hatred, NoBall, Taunt, Unsteady, Fumblerooski,
MonstrousMouth, VeryLongLegs, ASneakyPair, BlastinSolvesEverything, LordOfChaos, TeamCaptain,
WoodlandFury, WorkingInTandem — all verified against Java `postConstruct()` line-by-line.
Also: `SkillId::Yoink` had no `SKILL_TABLE` row (added: Trait, OncePerGame, BB2020+BB2025);
duplicate unreachable `Swarming` arm removed; `SPP_TABLE_BB2020.additional_catch` 0 → 1
(Java `SppCalc.additionalSpp` grants it to all non-BB2016 editions).

Production divergences fixed to match Java (batch F2): StringTool `bind()` unmatched-placeholder
handling, RangeRuler `minimum_roll_display` roll-1 formatting, `TurnMode::from_name` case
sensitivity, ServerUrlProperty `url()` missing full-URL early return, InifileParamFilter
trailing-flag handling, CommonProperty `is_stored_remote` defaults, `ClientStateId` Display,
`CardEffect::skills()` implemented, and `PlayerState::has_tacklezones()` no longer consults
`is_eye_gouged()` (Java checks eye gouge only at assist-counting time in ServerUtilPlayer,
which Rust already mirrors separately).

## Remaining known divergences (pinned by `#[ignore = "PARITY: ..."]` tests)

- `injury_type_block.rs::all_modes_should_be_present` — Rust `BlockMode` has 7 variants (3
  engine-internal: UseMightyBlow/UseClaws/UseClawsAndMightyBlow) vs Java's 4. This is the ONLY
  remaining ignored parity test.

Windows note: two Java skill-test classes carry non-obvious names to avoid case-insensitive
filesystem collisions — `SidestepBb2025SkillTest` (vs bb2016 `SideStepSkillTest`) and
`BloodlustMixedSkillTest` (inside `BloodLustSkillTest.java`); class FILES collide on NTFS, not
just filenames.

## Follow-up session (2026-07-21): reroll sources + remaining gaps closed

- **Reroll-source system ported.** Java's `Skill.registerRerollSource(ReRolledAction,
  ReRollSource)` mechanism (26 skills, ~37 pairs, consumed via
  `UtilCards.getUnusedRerollSource` min-priority selection) now lives in Rust as the static
  `SkillId::reroll_sources()` table (skill_id.rs, action strings in the Rust engine's
  vocabulary: PICK_UP→"PICKUP", GO_FOR_IT/RUSH→"GFI"), consumed by the single chokepoint
  `ffb-engine/src/step/abstract_step_with_re_roll.rs::find_skill_reroll_source` (replacing a
  hardcoded 5-action property map + WhirlingDervish special case that left ~25 skills' rerolls
  silently unavailable). Priority mirrors Java `ReRollSources` (all 1 except THE_BALLISTA = 2);
  ties break by SkillId order. Tests: exhaustive table test, 31 Rust per-skill mirrors, 18 new
  Java per-skill `getRerollSource` assertions, 6 chokepoint behavior tests (priority, tie-break,
  used-skill fallthrough). The per-skill-struct `register_reroll_source` map remains a faithful
  but inert translation, like the per-struct property registration.
- **usage_type tables reconciled** — 26 `SKILL_TABLE` rows + 15 `SkillId::usage_type()` arms
  fixed against Java constructors (latest-edition-wins flattening, e.g. WisdomOfTheWhiteDwarf
  bb2025 OncePerGame beats bb2020 OncePerTurnByTeamMate); pinned by
  `skills::tests::usage_type_agrees_with_skill_id_table`.
- **Game date fields added** (`scheduled`/`started`/`finished`, Java Game.java:37-39);
  `StepInitStartGame` sets `started` where Java does; the client marking handler derives
  `reconnecting = started.is_some()` per Java; the 4 reconnect tests run un-ignored.
- **Insignificant** now uses the plain two-arg constructor (bb2025 Java: not a negative trait).
- Playability gate re-run after the engine changes: see SESSION.md.

## Known non-test gaps (need engine-level design decisions)

- **Modifier registrations** (NervesOfSteel/Titchy/StrongArm/Accurate pass modifiers,
  CrushingBlow) — Java registers on the skill; Rust models these in `ffb-mechanics` modifier
  collections; asserted there, not per-skill.

(Resolved 2026-07-21: reroll-source table ported; usage_type tables reconciled; Insignificant
constructor fixed — see the follow-up section below.)

## How to run everything

```bash
# Rust
cd ffb-rust && cargo test --workspace

# Java (all mirrored suites)
cd ffb-java && /c/Users/Admin/bin/maven/bin/mvn -pl ffb-common,ffb-server,ffb-client-logic test

# Behavioral parity harness (both engines headless, JSONL diff)
python ffb-rust/scripts/parity_run.py
```
