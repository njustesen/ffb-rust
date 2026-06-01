# FFB Java → Rust Component Map

<!-- MAINTENANCE INSTRUCTIONS
  Update Java test counts:  grep -c "@Test" <test-file-path>
  Update Rust test counts:  grep -rc "#\[test\]" crates/ | grep -v ":0"
  Update skill status:      cross-check SESSION.md "Skills Implemented" / "Known Open Issues" each session
  Update parity row:        run cargo test -p ffb-parity after each seed batch
  Update progress.html:     run ffb-parity update_progress after each session
-->

## Testing Invariants

Two conditions must hold before any component earns **✓**:

**a) Extensive Java tests** — the Java test class covers all code paths (all editions, boundary values, error cases). "Extensive" means a reasonable future reader would consider it comprehensive, not just happy-path.

**b) Semantically equivalent Rust tests** — the Rust `#[test]` suite mirrors the Java suite scenario-for-scenario: same inputs, same expected outputs, same edge cases. The workflow is always: write Java tests first → translate to Rust → implement → iterate.

**Status legend:**
- `✓` Complete — comprehensive Java tests exist, Rust tests match count/coverage, all pass, no open issues
- `~` Partial — Rust exists but Java tests are missing or Rust test count is significantly lower
- `○` Pending — not yet translated to Rust
- `—` Not translating

**Java Tests column** = number of `@Test` methods in the named Java class(es).  
**Rust Tests column** = number of `#[test]` in the named Rust file(s).

---

## Summary

| Metric | Value |
|---|---|
| Mechanics components (with Java tests) | 17 |
| Skills — Java classes | ~290 across editions |
| Skills — unique named skills | ~165 |
| Skills with dedicated Java unit tests | ~160 (all skills covered, 4 @Test each) |
| Injury cause Java classes | 48 |
| Inducement Java classes | 28 |
| Kickoff event variants | 15 (across 3 editions) |
| Java test methods (translatable mechanics) | ~2,231 (session 25: +103 enum/model/skill @Test annotations) |
| Rust tests total (all crates) | 2,392 (822 engine, 1,190 mechanics, 349 model, 27 parity, 3 protocol, 1 client; session 30: +7 — CATCH_SCATTER modifier fix + kickoff timeout test correction) |
| Parity seeds passing | 100 / 100 (all seeds ✓; session 30: fixed BB2025 CATCH_SCATTER +1 modifier — "Inaccurate Pass or Scatter" was missing from Rust CSTI loop) |

---

## 1. Enums & Constants

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 1 | Direction | `ffb-common/.../Direction.java` | yes | `ffb-model/src/enums/direction.rs` | 14 (DirectionTest) | 13 | ✓ |
| 2 | PlayerState | `ffb-common/.../PlayerState.java` | yes | `ffb-model/src/enums/player.rs` | 11 (PlayerStateTest) | 23 | ✓ |
| 3 | PlayerGender | `ffb-common/.../PlayerGender.java` | yes | `ffb-model/src/enums/player.rs` | 10 (PlayerEnumTest) | 10 | ✓ |
| 4 | PlayerType | `ffb-common/.../PlayerType.java` | yes | `ffb-model/src/enums/player.rs` | 5 (PlayerEnumTest) | 5 | ✓ |
| 5 | GameStatus | `ffb-common/.../GameStatus.java` | yes | `ffb-model/src/enums/game.rs` | 9 (GameStatusTest) | 9 | ✓ |
| 6 | TurnMode | `ffb-common/.../TurnMode.java` | yes | `ffb-model/src/enums/turn.rs` | 16 (TurnModeTest) | 16 | ✓ |
| 7 | BlockResult | `ffb-common/.../BlockResult.java` | yes | `ffb-model/src/enums/block.rs` | 10 (EnumRoundTripTest) | 10 | ✓ |
| 8 | SeriousInjuryKind / InjuryAttribute | `ffb-common/.../injury/SeriousInjury*.java` | yes | `ffb-model/src/enums/injury.rs` | 16 (EnumRoundTripTest) | 15 | ✓ |
| 9 | SkillCategory / UsageType / DeclareCondition | `ffb-common/.../skill/Skill*.java` | yes | `ffb-model/src/enums/skill.rs` | 18 (SkillEnumTest) | 18 | ✓ |
| 10 | BoxType / TeamStatus / SendToBoxReason | `ffb-common/.../Team*.java` | yes | `ffb-model/src/enums/team.rs` | 20 (TeamEnumTest) | 20 | ✓ |
| 11 | Weather | `ffb-common/.../Weather.java` | yes | `ffb-model/src/enums/weather.rs` | 11 (EnumRoundTripTest) | 9 | ✓ |
| 12 | PassingDistance / PassResult | `ffb-common/.../PassingDistance.java` | yes | `ffb-model/src/enums/pass.rs` | 12 (EnumRoundTripTest) | 13 | ✓ |
| 13 | ApothecaryMode / Status / Type | `ffb-common/.../Apothecary*.java` | yes | `ffb-model/src/enums/apothecary.rs` | 15 (ApothecaryEnumTest) | 15 | ✓ |
| 14 | ClientStateId | `ffb-common/.../ClientStateId.java` | yes | `ffb-model/src/enums/client.rs` | 9 (ClientStateIdTest) | 9 | ✓ |
| 15 | NetCommandId / ServerStatus | `ffb-common/.../net/*.java` | yes | `ffb-model/src/enums/net.rs` | 16 (NetEnumTest) | 16 | ✓ |
| 16 | PlayerAction | `ffb-common/.../PlayerAction.java` | yes | `ffb-model/src/enums/player.rs` | 29 (PlayerActionTest) | 40 (player.rs total) | ✓ |
| 17 | ReRollProperty / Source / LeaderState | `ffb-common/.../ReRoll*.java` | yes | `ffb-model/src/enums/reroll.rs` | 9 (ReRollEnumTest) | 9 | ✓ |
| 18 | CardEffect / CardTarget | `ffb-common/.../CardEffect.java` | yes | `ffb-model/src/enums/card.rs` | 17 (CardEnumTest) | 17 | ✓ |
| 19 | KickoffResult / Rules / ModelChangeId / ReportId | `ffb-common/.../KickoffResult.java` etc. | yes | `ffb-model/src/enums/kickoff_result.rs`, `rules.rs`, `model_change.rs` | 5 (RulesTest) | 5 | ✓ |

> **Note:** `EnumRoundTripTest.java` (60 tests) covers Weather, BlockResult, PassingDistance, BB2016/BB2020 SeriousInjury round-trips and boundary cases. Session 24 added dedicated Java test classes for Direction (#1), GameStatus (#5), TurnMode (#6), SkillCategory (#9), BoxType/TeamStatus/SendToBoxReason (#10), Apothecary (#13), ClientStateId (#14), NetCommandId (#15), PlayerAction (#16), ReRollProperty (#17), CardEffect (#18), PlayerGender/PlayerType (#3/#4). BlockResult (#7) and SeriousInjury (#8) Rust tests still need boosting to match their Java counterparts.

---

## 2. Value Types

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 20 | FieldCoordinate | `ffb-common/.../FieldCoordinate.java` | yes | `ffb-model/src/types/field_coordinate.rs` | 18 (FieldCoordinateTest) | 18 | ✓ |
| 21 | FieldCoordinateBounds | `ffb-common/.../FieldCoordinateBounds.java` | yes | `ffb-model/src/types/field_coordinate.rs` | 18 (FieldCoordinateTest) | 18 | ✓ |
| 22 | FantasyFootballConstants | `ffb-common/.../FantasyFootballConstants.java` | yes | `ffb-model/src/types/constants.rs` | 4 (GameConstantsTest) | 4 | ✓ |
| 23 | MoveSquare / PushbackSquare / RangeRuler | `ffb-common/.../MoveSquare.java` etc. | yes | `ffb-model/src/types/move_square.rs`, `pushback_square.rs`, `range_ruler.rs` | 6 (MoveSquareTest) | 3+2+3 | ✓ |
| 24 | BlockRoll / BlockTarget | `ffb-common/.../BlockRoll.java` | yes | `ffb-model/src/types/block_types.rs` | 5 (BlockRollTest) | 5 | ✓ |
| 25 | ReRolledActions / ReRollOptions | `ffb-common/.../ReRolledAction.java` | yes | `ffb-model/src/enums/reroll.rs` | 4 (ReRollOptionsTest) | 3 | ✓ |

---

## 3. Utilities

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 26 | GameRng (Xoshiro256\*\*) | `ffb-common/.../util/UtilRandom.java` | yes | `ffb-model/src/util/rng.rs` | — | 5 | ✓ |
| 27 | StringTool / UtilPassing | `ffb-common/.../util/StringTool.java` | yes | `ffb-model/src/util/string_tool.rs`, `passing.rs` | 5 (StringToolTest) | 5+12 | ✓ |
| 28 | state_hash (FNV-1a canonical) | (Rust-only) | n/a | `ffb-model/src/util/state_hash.rs` | — | 4 | ✓ |

---

## 4. Core Data Model

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 29 | Player | `ffb-common/.../model/Player.java` | yes | `ffb-model/src/model/player.rs` | 5 (PlayerModelTest) | 5 | ~ |
| 30 | Team | `ffb-common/.../model/Team.java` | yes | `ffb-model/src/model/team.rs` | — | 4 | ~ |
| 31 | FieldModel | `ffb-common/.../model/FieldModel.java` | yes | `ffb-model/src/model/field_model.rs` | — | 4 | ~ |
| 32 | TurnData / ActingPlayer | `ffb-common/.../model/TurnData.java` | yes | `ffb-model/src/model/turn_data.rs`, `acting_player.rs` | — | 3+3 | ~ |
| 33 | GameOptions / GameResult | `ffb-common/.../model/GameOptions.java` | yes | `ffb-model/src/model/game_options.rs`, `game_result.rs` | — | 2+2 | ~ |
| 34 | Game (central model) | `ffb-common/.../model/Game.java` | yes | `ffb-model/src/model/game.rs` | — | 4 | ~ |
| — | Roster / RosterPosition / SkillDef | `ffb-common/.../model/Roster*.java` | yes | `ffb-model/src/model/roster.rs`, `roster_position.rs`, `skill_def.rs` | — | 2+1+1 | ~ |

---

## 5. Modifier System

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 35 | Modifier contexts (Armor/Dodge/Pass/Injury…) | `ffb-server/.../mechanic/ArmorModifierValues.java` etc. | yes | `ffb-mechanics/src/modifiers/contexts.rs`, `modifiers.rs` | 7+8+6 (Armor+Injury+Weather modifier tests) | 2+25 | ✓ |

---

## 6. Mechanics

Java sources are in `ffb-server/src/main/java/com/fumbbl/ffb/server/util/` and `ffb-server/src/main/java/com/fumbbl/ffb/server/mechanic/`.  
Rust target: `ffb-mechanics/src/mechanics/`.

| ID | Mechanic | Java Test Class | Java Tests | Rust File | Rust Tests | Status | Notes |
|---|---|---|---|---|---|---|---|
| 72 | Block result (BlockResultCalc) | BlockDiceCalcTest + BlockResultCalcTest | 14+6=20 | `mechanics/block.rs` | 20 | ✓ | All methods covered including boundary values and multi-block modifiers |
| 73 | Scatter (ScatterCalc) | ScatterCalcTest | 11 | `mechanics/scatter.rs` | 11 | ✓ | All 8 directions, distance scaling, out-of-range null |
| 74 | Throw-in (ThrowInCalc) | ThrowInCalcTest | 6+8p | `mechanics/throw_in.rs` | 14 | ✓ | Java: 6 @Test + 8 @ParameterizedTest; Rust: all 4 corners, 4 sidelines, 3 editions |
| 75 | Special rolls (SpecialRollCalc) | SpecialRollCalcTest | 39 | `mechanics/special_roll.rs` | 39 | ✓ | Added isEscapeFromAlwaysHungrySuccessful test; all public methods covered |
| 76 | Post-match (PostMatchCalc) | PostMatchCalcTest | 14 | `mechanics/post_match.rs` | 20 | ✓ | 14 Java scenarios match 14 Rust; 6 extra Rust tests cover kickoff-event helpers already in KickoffEvent ✓ |
| 77 | Stat limits / lasting injury (StatCalc) | StatCalcTest | 9 | `mechanics/stat.rs` | 19 | ✓ | Java: 9 @Test + 14 @ParameterizedTest cover all 5 methods × all editions; Rust 19 cover the same |
| 78 | Weather table | WeatherCalcTest / WeatherModifierValueTest | 6 | `ffb-model/src/enums/weather.rs` | 9 | ✓ | All 11 roll values (2-12) covered in Rust; WeatherModifierValueTest 6 tests mirrored in modifiers.rs |
| 36 | Injury / armor (InjuryCalc + ArmorModifier) | InjuryCalcTest + ArmorModifierValueTest | 12+7=19 | `mechanics/injury.rs` | 42 | ✓ | Java: InjuryCalc 2 public methods (BB2016/BB2020 interpretation) + ArmorModifierValues constants all tested; Rust 42 also cover armor-breaking thresholds and casualty/SI tables |
| 37 | Pass (PassCalc + UtilPassing) | PassCalcTest | 17 | `mechanics/pass.rs` | 23 | ✓ | Java tests all 3 public PassCalc methods; Rust extras test evaluate_pass (higher-level wrapper, no Java equiv) |
| 38 | SPP (SppCalc) | SppCalcTest | 31 | `mechanics/spp.rs` | 31 | ✓ | All editions, Brawlin' Brutes, level thresholds, additional_casualty/additional_catch fields |
| — | Agility (AgilityCalc) | AgilityCalcTest | 11 | `mechanics/agility.rs` | 26 | ✓ | Added 4 @Test: catch with modifiers, interception vs catch comparison, BB2020 catch formula |
| — | Foul (FoulCalc) | FoulCalcTest | 12 | `mechanics/foul.rs` | 17 | ✓ | Java: 12 @Test + @ParameterizedTest (all doubles combinations); Rust 17 cover same + 5 parameterized mirrors |
| — | Movement (MovementCalc) | MovementCalcTest | 15 | `mechanics/movement.rs` | 20 | ✓ | Java: 15 @Test + @ParameterizedTest; Rust: 20 (5 extra cover parameterized cases) |
| — | Passing distance (PassingDistanceCalc) | PassingDistanceCalcTest | 7 | `mechanics/passing_distance.rs` | 20 | ✓ | Java: 7 @Test + 4 @ParameterizedTest covering all 4 distance buckets; Rust 20 cover same buckets |
| — | Roll (RollCalc) | RollCalcTest | 11 | `mechanics/roll.rs` | 16 | ✓ | Java: 11 @Test + @ParameterizedTest; Rust: 16 (extra tests cover parameterized cases) |
| — | Kickoff event (KickoffEventCalc) | KickoffEventCalcTest | 10 | `mechanics/kickoff_event.rs` | 10 | ✓ | All 3 methods, tie case, combined scenarios |
| 40 | Casualty table (CasualtyCalc) | CasualtyCalcTest | 19 | `ffb-mechanics/src/injury/mod.rs` | 25 | ✓ | Java: 9 @Test + 10 @ParameterizedTest; all 7 CasualtyCalc methods tested (tier BB2016/BB2020/BB2025, requiresSIRoll, seriousInjurySubType); Rust 25 exceeds Java |

> **Session 18:** Scatter, Kickoff Event, Block, Special Rolls, Throw-in, SPP all marked ✓ after verification + gap-filling. Scatter/Kickoff/Block were already comprehensive; Special Rolls added escape-from-always-hungry; Throw-in added SW/SE corner D3 tests; SPP added additional_casualty/additional_catch/per-edition tests to reach 31.
> **Session 19:** Block/Wrestle/Guard/Frenzy marked ✓ (SKILL_TABLE tests added in ffb-mechanics/src/skills/mod.rs). PostMatch, StatCalc, Weather enum, Weather table, Roll, Movement all marked ✓ (coverage already comprehensive). 5 new Java skill test classes (Catch, SureHands, Tackle, Dodge, DirtyPlayer). +25 Java @Test, +16 Rust tests.
> **Session 20:** Pass, Foul, Agility, PassingDistance all → ✓. Catch/SureHands/Tackle/Dodge/DirtyPlayer → ✓ (+20 Rust SKILL_TABLE tests). +4 Java @Test in AgilityCalcTest (catch modifiers, interception vs catch). Injury/ArmorModifier → ✓, Casualty → ✓ (all methods comprehensively tested, Java 19+19 @Test runs, Rust 42+25).
> **Session 22–23:** ~155 Java `*SkillTest` classes written (4 @Test each). SKILL_TABLE Rust tests added for all 167 skill entries (4 #[test] each, 668 total). All common skills and most edition-specific skills with engine ✓ now marked ✓.
> **Session 24:** Enum test coverage added for 13 enum groups (IDs 1, 3–10, 13–18). Java test counts updated in COMPONENTS.md for all 7a–7g skill rows. Direction, GameStatus, TurnMode, PlayerAction, ReRollProperty/LeaderState marked ✓. Total: ~2,128 Java @Test annotations, 2,049 Rust tests.
> **Session 25:** Rust enum boosts for BlockResult (#7, 2→10 ✓), SeriousInjury (#8, 3→15 ✓, BrokenNeck bug fixed), SkillCategory (#9, 15→18 ✓), BoxType/TeamStatus (#10, 13→20 ✓), Apothecary (#13, 13→15 ✓), ClientStateId (#14, 7→9 ✓), NetCommandId (#15, 10→16 ✓), CardEffect (#18, 9→17 ✓). PassingDistance (#12) marked ✓ (Rust 13 ≥ Java 12). Modifier System (#35) marked ✓. Java tests added: FieldCoordinateTest (18 existing, confirmed), GameConstantsTest (4), MoveSquareTest (6), BlockRollTest (5), ReRollOptionsTest (4), StringToolTest (5), PlayerModelTest (5). Rust boosts: field_coordinate.rs (+6), block_types.rs (+2), team.rs (+2), field_model.rs (+1), game.rs (+2). 41 missing Java skill test files written (Yoink, Drunkard, PlagueRidden, all 7g skills). Total: ~2,231 Java @Test, 2,114 Rust tests.

---

## 7. Skills

Session 19: Block, Wrestle, Guard, Frenzy are now ✓. Java tests (session 18) cover name, category, NamedProperties, and edition annotation. Rust tests (session 19) cover class_name, category, and editions from SKILL_TABLE; property behavior is covered by engine tests.

Session 22–23: Java `*SkillTest` classes written for ~155 skills (4 @Test each covering name, category, edition, NamedProperties). SKILL_TABLE Rust tests (4 #[test] each) cover all 167 entries. Skills with engine behavior ✓ are now marked ✓; those with engine behavior ~ remain ~. A handful of mixed/special skills (Drunkard, PlagueRidden, Yoink, AllYouCanEat, and ~30 other 7g entries) still have no Java test file.

Engine behavior status is sourced from `SESSION.md` "Skills Implemented" and "Known Open Issues".

### 7a. Common Skills (all editions)

Java: `ffb-common/.../skill/common/`  
Rust: `ffb-model/src/enums/skill_id.rs` (SkillId) + `ffb-mechanics/src/skills/mod.rs` (SKILL_TABLE) + engine behavior in `ffb-engine/src/engine/mod.rs`

| Skill | Java Class | SkillId in Rust | Engine Behavior | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|
| Block | common/Block.java | Block | ✓ | 5 (BlockSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Catch | common/Catch.java | Catch | ✓ | 5 (CatchSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Dauntless | common/Dauntless.java | Dauntless | ✓ | 4 (DauntlessSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| DisturbingPresence | common/DisturbingPresence.java | DisturbingPresence | ✓ | 4 (DisturbingPresenceSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| DivingCatch | common/DivingCatch.java | DivingCatch | ✓ | 4 (DivingCatchSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| DumpOff | common/DumpOff.java | DumpOff | ✓ | 4 (DumpOffSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| ExtraArms | common/ExtraArms.java | ExtraArms | ✓ | 4 (ExtraArmsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Fend | common/Fend.java | Fend | ✓ | 4 (FendSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| FoulAppearance | common/FoulAppearance.java | FoulAppearance | ✓ | 4 (FoulAppearanceSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| HailMaryPass | common/HailMaryPass.java | HailMaryPass | ~ | 4 (HailMaryPassSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ |
| Horns | common/Horns.java | Horns | ✓ | 4 (HornsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| JumpUp | common/JumpUp.java | JumpUp | ✓ | 4 (JumpUpSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| MovementIncrease | common/MovementIncrease.java | MovementIncrease | ✓ | 4 (MovementIncreaseSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Pass | common/Pass.java | Pass (n/a — built-in) | ✓ | 4 (PassSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Sprint | common/Sprint.java | Sprint | ✓ | 4 (SprintSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| StandFirm | common/StandFirm.java | StandFirm | ✓ | 4 (StandFirmSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| StripBall | common/StripBall.java | StripBall | ✓ | 4 (StripBallSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| SureHands | common/SureHands.java | SureHands | ✓ | 5 (SureHandsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Tackle | common/Tackle.java | Tackle | ✓ | 5 (TackleSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Tentacles | common/Tentacles.java | Tentacles | ✓ | 4 (TentaclesSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| ThickSkull | common/ThickSkull.java | ThickSkull | ✓ | 4 (ThickSkullSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| TwoHeads | common/TwoHeads.java | TwoHeads | ✓ | 4 (TwoHeadsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |
| Wrestle | common/Wrestle.java | Wrestle | ✓ | 5 (WrestleSkillTest) | 4 (SKILL_TABLE) + engine | ✓ |

### 7b. BB2016-Only Skills

Java: `ffb-common/.../skill/bb2016/`

| Skill | Java Class | SkillId in Rust | Engine Behavior | Java Tests | Rust Tests | Status | Notes |
|---|---|---|---|---|---|---|---|
| Accurate | bb2016/Accurate.java | Accurate | ✓ | 4 (AccurateSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also in mixed/ |
| AlwaysHungry | bb2016/AlwaysHungry.java | AlwaysHungry | ✓ | 4 (AlwaysHungrySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also in mixed/ |
| Animosity | bb2016/Animosity.java | Animosity | ✓ | 4 (AnimositySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025, mixed |
| ArmourIncrease | bb2016/ArmourIncrease.java | ArmourIncrease | ✓ | 4 (ArmourIncreaseSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BallAndChain | bb2016/BallAndChain.java | BallAndChain | ✓ | 4 (BallAndChainSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Groups 158+170: scatter movement, forced blocks, crowd surf |
| BloodLust | bb2016/BloodLust.java | BloodLust | ✓ | 4 (BloodLustSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Bombardier | bb2016/Bombardier.java | Bombardier | ✓ | 4 (BombardierSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| BoneHead | bb2016/BoneHead.java | BoneHead | ✓ | 4 (BoneHeadSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| BreakTackle | bb2016/BreakTackle.java | BreakTackle | ✓ | 4 (BreakTackleSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Chainsaw | bb2016/Chainsaw.java | Chainsaw | ✓ | 4 (ChainsawSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Claw | bb2016/Claw.java | Claws | ✓ | 4 (ClawSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Mixed: Claws |
| Decay | bb2016/Decay.java | Decay | ✓ | 4 (DecaySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| DirtyPlayer | bb2016/DirtyPlayer.java | DirtyPlayer | ✓ | 5 (DirtyPlayerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025; Rust table has editions=[Bb2020,Bb2025] |
| Disposable | bb2016/Disposable.java | Disposable | — | 4 (DisposableSkillTest) | 4 (SKILL_TABLE) | — | post-match roster flag only |
| DivingTackle | bb2016/DivingTackle.java | DivingTackle | ✓ | 4 (DivingTackleSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| FanFavourite | bb2016/FanFavourite.java | FanFavourite | ✓ | 4 (FanFavouriteSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 166: count_fan_favourites_on_pitch |
| Frenzy | bb2016/Frenzy.java | Frenzy | ✓ | 5 (FrenzySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Grab | bb2016/Grab.java | Grab | ✓ | 4 (GrabSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Guard | bb2016/Guard.java | Guard | ✓ | 5 (GuardSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| HypnoticGaze | bb2016/HypnoticGaze.java | HypnoticGaze | ✓ | 4 (HypnoticGazeSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| KickOffReturn | bb2016/KickOffReturn.java | KickOffReturn | ✓ | 4 (KickOffReturnSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| KickTeamMate | bb2016/KickTeamMate.java | KickTeamMate | ✓ | 4 (KickTeamMateSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Leap | bb2016/Leap.java | Leap | ✓ | 4 (LeapSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Loner | bb2016/Loner.java | Loner | ✓ | 4 (LonerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| MightyBlow | bb2016/MightyBlow.java | MightyBlow | ✓ | 4 (MightyBlowSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| MonstrousMouth | bb2016/MonstrousMouth.java | MonstrousMouth | ✓ | 4 (MonstrousMouthSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| MultipleBlock | bb2016/MultipleBlock.java | MultipleBlock | ✓ | 4 (MultipleBlockSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| NervesOfSteel | bb2016/NervesOfSteel.java | NervesOfSteel | ✓ | 4 (NervesOfSteelSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| NoHands | bb2016/NoHands.java | NoHands | ✓ | 4 (NoHandsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| NurglesRot | bb2016/NurglesRot.java | NurglesRot | — | 4 (NurglesRotSkillTest) | 4 (SKILL_TABLE) | — | Post-game only, not relevant to real-time engine |
| PassBlock | bb2016/PassBlock.java | PassBlock | ✓ | 4 (PassBlockSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| PilingOn | bb2016/PilingOn.java | PilingOn | ✓ | 4 (PilingOnSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| PrehensileTail | bb2016/PrehensileTail.java | PrehensileTail | ✓ | 4 (PrehensileTailSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| ReallyStupid | bb2016/ReallyStupid.java | ReallyStupid | ✓ | 4 (ReallyStupidSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Regeneration | bb2016/Regeneration.java | Regeneration | ✓ | 4 (RegenerationSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| RightStuff | bb2016/RightStuff.java | RightStuff | ✓ | 4 (RightStuffSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| SafeThrow | bb2016/SafeThrow.java | SafeThrow | ✓ | 4 (SafeThrowSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| SecretWeapon | bb2016/SecretWeapon.java | SecretWeapon | ✓ | 4 (SecretWeaponSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Shadowing | bb2016/Shadowing.java | Shadowing | ✓ | 4 (ShadowingSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| SideStep | bb2016/SideStep.java | SideStep | ✓ | 4 (SidestepSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| SneakyGit | bb2016/SneakyGit.java | SneakyGit | ✓ | 4 (SneakyGitSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Stab | bb2016/Stab.java | Stab | ✓ | 4 (StabSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Stakes | bb2016/Stakes.java | Stakes | ✓ | 4 (StakesSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| StrengthIncrease | bb2016/StrengthIncrease.java | StrengthIncrease | ✓ | 4 (StrengthIncreaseSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| StrongArm | bb2016/StrongArm.java | StrongArm | ✓ | 4 (StrongArmSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Stunty | bb2016/Stunty.java | Stunty | ✓ | 4 (StuntySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| SureFeet | bb2016/SureFeet.java | SureFeet | ✓ | 4 (SureFeetSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| Swarming | bb2016/Swarming.java | Swarming | ✓ | 4 (SwarmingSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020 |
| Swoop | bb2016/Swoop.java | Swoop | ✓ | 4 (SwoopSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| TakeRoot | bb2016/TakeRoot.java | TakeRoot | ✓ | 4 (TakeRootSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| ThrowTeamMate | bb2016/ThrowTeamMate.java | ThrowTeamMate | ✓ | 4 (ThrowTeamMateSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Timmmber | bb2016/Timmmber.java | Timmmber | ✓ | 4 (TimmmberSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Titchy | bb2016/Titchy.java | Titchy | ✓ | 4 (TitchySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| VeryLongLegs | bb2016/VeryLongLegs.java | VeryLongLegs | ✓ | 4 (VeryLongLegsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2020, bb2025 |
| WeepingDagger | bb2016/WeepingDagger.java | WeepingDagger | ✓ | 4 (WeepingDaggerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| WildAnimal | bb2016/WildAnimal.java | WildAnimal | ✓ | 4 (WildAnimalSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |

### 7c. BB2020-Only Skills (not in BB2016)

Java: `ffb-common/.../skill/bb2020/`

| Skill | Java Class | SkillId in Rust | Engine Behavior | Java Tests | Status | Notes |
|---|---|---|---|---|---|---|
| Brawler | bb2020/Brawler.java | Brawler | ✓ | 4 (BrawlerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2025 |
| BreatheFire | bb2020/BreatheFire.java | BreatheFire | ✓ | 4 (BreatheFireSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2025 — session 12 |
| CloudBurster | bb2020/CloudBurster.java | CloudBurster | ✓ | 4 (CloudBursterSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 147: forces re-roll on Long Pass interception |
| Defensive | bb2020/Defensive.java | Defensive | ✓ | 4 (DefensiveSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also bb2025 |
| Fumblerooskie | bb2020/Fumblerooskie.java | Fumblerooskie | ✓ | 4 (FumblerooskieSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 159: intentional ball drop, ball stays at square |
| HitAndRun | bb2020/HitAndRun.java | HitAndRun | ✓ | 4 (HitAndRunSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 142: move 1 square after block resolution |
| PassingIncrease | bb2020/PassingIncrease.java | PassingIncrease | ✓ | 4 (PassingIncreaseSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | PA decrements by 1; skill added to player list after selection |
| PileDriver | bb2020/PileDriver.java | PileDriver | ✓ | 4 (PileDriverSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 139: fouls after POW, blocked when foul already used |
| PogoStick | bb2020/PogoStick.java | PogoStick | ✓ | 4 (PogoStickSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Leap variant — session 12 |
| ProjectileVomit | bb2020/ProjectileVomit.java | ProjectileVomit | ✓ | 4 (ProjectileVomitSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 121: success injures defender, failure injures attacker |
| RunningPass | bb2020/RunningPass.java | RunningPass | ✓ | 4 (RunningPassSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 143: allows move after quick pass; without skill thrower is done |

### 7d. BB2025-Only Skills (not in BB2016/BB2020)

Java: `ffb-common/.../skill/bb2025/`

| Skill | Java Class | SkillId in Rust | Engine Behavior | Java Tests | Status | Notes |
|---|---|---|---|---|---|---|
| AgilityIncrease | bb2025/AgilityIncrease.java | AgilityIncrease | ✓ | 4 (AgilityIncreaseSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| BigHand | bb2025/BigHand.java | BigHand | ✓ | 4 (BigHandSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 51: suppresses DisturbingPresence on pickup; without BigHand DP applies |
| Bullseye | bb2025/Bullseye.java | Bullseye | ✓ | 4 (BullseyeSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 137: accurate TTM lands at target, no scatter |
| Dodge | bb2025/Dodge.java | Dodge | ✓ | 5 (DodgeSkillTest — tests mixed/Dodge BB2016+BB2020) | 4 (SKILL_TABLE) + engine | ✓ | BB2025 adds ONCE_PER_TURN; Rust table editions=[Bb2025]; also mixed/ |
| EyeGouge | bb2025/EyeGouge.java | EyeGouge | ✓ | 4 (EyeGougeSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 114: removes defender TZ after push-back |
| Fumblerooski | bb2025/Fumblerooski.java | Fumblerooskie | ✓ | 4 (FumblerooskiSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | BB2025 variant; same SkillId as BB2020 Fumblerooskie |
| GiveAndGo | bb2025/GiveAndGo.java | GiveAndGo | ✓ | 4 (GiveAndGoSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 136: extra move after HandOff, acting player stays as thrower |
| Hatred | bb2025/Hatred.java | Hatred | ✓ | 4 (HatredSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 133: rerolls Skull result; no fire without flag or in BB2020 |
| Insignificant | bb2025/Insignificant.java | — | — | 4 (InsignificantSkillTest) | 4 (SKILL_TABLE) | — | Roster-only rule (max 50% of team); no engine behavior to translate |
| Juggernaut | bb2025/Juggernaut.java | Juggernaut | ✓ | 4 (JuggernautSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Kick | bb2025/Kick.java | Kick | ✓ | 4 (KickSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Leader | bb2025/Leader.java | Leader | ✓ | 4 (LeaderSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| LethalFlight | bb2025/LethalFlight.java | LethalFlight | ✓ | 4 (LethalFlightSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 138: increases injury when landing on opponent; no trigger without skill |
| LoneFouler | bb2025/LoneFouler.java | LoneFouler | ✓ | 4 (LoneFoulerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| NoBall | bb2025/NoBall.java | NoBall | ✓ | 4 (NoBallSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 135: prevents pass/pickup/catch in legal actions |
| Pogo | bb2025/Pogo.java | Pogo | ✓ | 4 (PogoSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Leap variant — session 12 |
| Pro | bb2025/Pro.java | Pro | ✓ | 4 (ProSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also mixed/ |
| Punt | bb2025/Punt.java | Punt | ✓ | 4 (PuntSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| PutTheBootIn | bb2025/PutTheBootIn.java | PutTheBootIn | ✓ | 4 (PutTheBootInSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 112: allows assist with counter-assist and in tackle zones |
| QuickFoul | bb2025/QuickFoul.java | QuickFoul | ✓ | 4 (QuickFoulSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 144: move after foul in BB2025, not in BB2016 |
| Saboteur | bb2025/Saboteur.java | Saboteur | ✓ | 4 (SaboteurSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 146: KD prompts D6 roll; 4+ knocks attacker down |
| Sidestep | bb2025/Sidestep.java | Sidestep | ✓ | 4 (SidestepSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | BB2025 name for SideStep |
| SteadyFooting | bb2025/SteadyFooting.java | SteadyFooting | ✓ | 4 (SteadyFootingSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 145: roll D6 on fall, 6 avoids knockdown |
| Taunt | bb2025/Taunt.java | Taunt | ✓ | 4 (TauntSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 151: offers SkillUse on pushback; forces/rejects follow-up |
| Unsteady | bb2025/Unsteady.java | Unsteady | ✓ | 4 (UnsteadySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | SecureTheBall excluded for Unsteady; normal player can use it |
| ViolentInnovator | bb2025/ViolentInnovator.java | ViolentInnovator | ✓ | 4 (ViolentInnovatorSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 160: grants SPP for Stab CAS in BB2025; without skill no SPP |

### 7e. Special Skills (Star Player Abilities)

Java: `ffb-common/.../skill/bb2020/special/`, `bb2025/special/`, `mixed/special/`  
Many of these are unique to specific star players and appear in limited game scenarios.

| Skill | Package | SkillId in Rust | Engine Behavior | Java Tests | Rust Tests | Status | Notes |
|---|---|---|---|---|---|---|---|
| ASneakyPair | bb2020+bb2025/special | ASneakyPair | ✓ | 4 (ASneakyPairSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BlastIt | bb2020+bb2025/special | BlastIt | ✓ | 4 (BlastItSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BlastinSolvesEverything | bb2025/special | BlastinSolvesEverything | ✓ | 4 (BlastinSolvesEverythingSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BrutalBlock | bb2020/special | BrutalBlock | ✓ | 4 (BrutalBlockSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BurstOfSpeed | bb2020/special | BurstOfSpeed | ✓ | 4 (BurstOfSpeedSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| ConsummateProfessional | bb2020/special | ConsummateProfessional | ✓ | 4 (ConsummateProfessionalSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| DwarfenScourge / DwarvenScourge | bb2020+bb2025/special | DwarfenScourge | ✓ | 4 (DwarfenScourgeSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| ExcuseMeAreYouAZoat | bb2020+bb2025/special | ExcuseMeAreYouAZoat | ✓ | 4 (ExcuseMeAreYouAZoatSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| FrenziedRush | bb2020+bb2025/special | FrenziedRush | ✓ | 4 (FrenziedRushSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| GhostlyFlames | bb2020/special | GhostlyFlames | ✓ | 4 (GhostlyFlamesSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Group 148: blitz chainsaw +4 armor modifier |
| Incorporeal | bb2020+bb2025/special | Incorporeal | ✓ | 4 (IncorporealSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| KrumpAndSmash | bb2025/special | KrumpAndSmash | ✓ | 4 (KrumpAndSmashSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| LordOfChaos | bb2020+bb2025/special | LordOfChaos | ✓ | 4 (LordOfChaosSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| MasterAssassin | bb2020+bb2025/special | MasterAssassin | ✓ | 4 (MasterAssassinSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| MesmerizingDance / MesmerisingDance | bb2020+bb2025/special | MesmerizingDance | ✓ | 4 (MesmerizingDanceSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| PumpUpTheCrowd | bb2020+bb2025/special | PumpUpTheCrowd | ✓ | 4 (PumpUpTheCrowdSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| PutridRegurgitation | bb2020+bb2025/special | PutridRegurgitation | ✓ | 4 (PutridRegurgitationSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| SlashingNails | bb2025/special | SlashingNails | ✓ | 4 (SlashingNailsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| TeamCaptain | bb2025/special | TeamCaptain | ✓ | 4 (TeamCaptainSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| TheBallista | bb2020+bb2025/special | TheBallista | ✓ | 4 (TheBallistaSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| ThenIStartedBlastin | bb2020/special | ThenIStartedBlastin | ✓ | 4 (ThenIStartedBlastinSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | Also an injury cause |
| TwoForOne | bb2020/special | TwoForOne | ✓ | 4 (TwoForOneSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| WhirlingDervish | bb2020+bb2025/special | WhirlingDervish | ✓ | 4 (WhirlingDervishSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| WisdomOfTheWhiteDwarf | bb2020+bb2025/special | WisdomOfTheWhiteDwarf | ✓ | 4 (WisdomOfTheWhiteDwarfSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| WoodlandFury | bb2025/special | WoodlandFury | ✓ | 4 (WoodlandFurySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| WorkingInTandem | bb2025/special | WorkingInTandem | ✓ | 4 (WorkingInTandemSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Yoink | bb2020/special | Yoink | ✓ | 4 (YoinkSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |

### 7f. Mixed/Multi-Edition Skills

Java: `ffb-common/.../skill/mixed/` (skills shared across multiple editions in a single class)

| Skill | Engine | Java Tests | Rust Tests | Status | Notes |
|---|---|---|---|---|---|
| AnimalSavagery | ✓ | 4 (AnimalSavagerySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| ArmBar | ✓ | 4 (ArmBarSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Cannoneer | ✓ | 4 (CannoneerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| IronHardSkin | ✓ | 4 (IronHardSkinSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| MyBall | ✓ | 4 (MyBallSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| OnTheBall | ✓ | 4 (OnTheBallSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| PickMeUp | ✓ | 4 (PickMeUpSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| SafePairOfHands | ✓ | 4 (SafePairOfHandsSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| SafePass | ✓ | 4 (SafePassSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Trickster | ✓ | 4 (TricksterSkillTest) | 4 (SKILL_TABLE) + 4 (engine, Group 234) | ✓ | canMoveBeforeBeingBlocked: defender moves 1 square before block resolves; BallAndChain cancels; once per game |
| UnchannelledFury | ✓ | 4 (UnchannelledFurySkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Drunkard | ~ | 4 (DrunkardSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | +1 GFI modifier implemented |
| PlagueRidden | — | 4 (PlagueRiddenSkillTest) | 4 (SKILL_TABLE) | — | post-match roster flag only (allowsRaisingLineman) |

### 7g. Mixed/Special Skills (star player abilities in mixed package)

Java: `ffb-common/.../skill/mixed/special/`

| Skill | Engine | Java Tests | Rust Tests | Status | Notes |
|---|---|---|---|---|---|
| AllYouCanEat | ✓ | 4 (AllYouCanEatSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | second bomb in same activation |
| BalefulHex | ✓ | 4 (BalefulHexSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| BeerBarrelBash | ✓ | 4 (BeerBarrelBashSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ThrowKeg action, bomb-like arc |
| BlackInk | ✓ | 4 (BlackInkSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | auto-succeed HypnoticGaze once-per-game |
| BlindRage | ✓ | 4 (BlindRageSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | reroll source for Dauntless |
| BoundingLeap | ✓ | 4 (BoundingLeapSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ignore leap modifier + reroll |
| BugmansXXXXXX | ✓ | 4 (BugmansXXXXXXSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | reroll 1 on KO recovery |
| CatchOfTheDay | ✓ | 4 (CatchOfTheDaySkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | D6≥3 to pick up ball at activation |
| CrushingBlow | ✓ | 4 (CrushingBlowSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | +1 armor modifier once-per-game |
| FuriousOutburst | ✓ | 4 (FuriousOutburstSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ArmourRollAttack action instead of block |
| FuryOfTheBloodGod | ✓ | 4 (FuryOfTheBloodGodSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | 2 extra blocks after failed Frenzy |
| GoredByTheBull | ✓ | 4 (GoredByTheBullSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | +1 block die on blitz |
| HalflingLuck | ✓ | 4 (HalflingLuckSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | single-die reroll source |
| IllBeBack | ✓ | 4 (IllBeBackSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ignore first SecretWeapon ejection |
| Indomitable | ✓ | 4 (IndomitableSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | double ST after Dauntless success |
| Kaboom | ✓ | 4 (KaboomSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | force bomb explode at player's square |
| KeenPlayer | ✓ | 4 (KeenPlayerSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ejected at end of drive |
| KickEmWhileTheyReDown | ✓ | 4 (KickEmWhileTheyReDownSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | chainsaw can target prone/KO opponents |
| LookIntoMyEyes | ✓ | 4 (LookIntoMyEyesSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| MaximumCarnage | ✓ | 4 (MaximumCarnageSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | second chainsaw attack in same activation |
| OldPro | ✓ | 4 (OldProSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | armor reroll once-per-game |
| PrimalSavagery | ✓ | 4 (PrimalSavagerySkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | LashOut action: D6+ST vs D6+AV |
| QuickBite | ✓ | 4 (QuickBiteSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | bite adjacent opponent after catch |
| RaidingParty | ✓ | 4 (RaidingPartySkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | move adjacent open teammate 1 square |
| Ram | ✓ | 4 (RamSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | +1 armor+injury modifier once-per-game |
| Reliable | ✓ | 4 (ReliableSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | fumbled TTM lands safely |
| SavageBlow | ✓ | 4 (SavageBlowSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | reroll block dice once-per-game |
| SavageMauling | ✓ | 4 (SavageMaulingSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | reroll injury roll once-per-game |
| ShotToNothing | ✓ | 4 (ShotToNothingSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | gain HailMaryPass temporarily once-per-game |
| Slayer | ✓ | 4 (SlayerSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| SneakiestOfTheLot | ✓ | 4 (SneakiestOfTheLotSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | allow second foul when foul_used |
| StarOfTheShow | ✓ | 4 (StarOfTheShowSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | grant team reroll after TD |
| StrongPassingGame | ✓ | 4 (StrongPassingGameSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | add player ST to pass roll |
| SwiftAsTheBreeze | ✓ | 4 (SwiftAsTheBreezeSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ignore GFI/dodge modifier once-per-game |
| TastyMorsel | ✓ | 4 (TastyMorselSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | Bite action: armor roll with +1 injury |
| TheFlashingBlade | ✓ | 4 (TheFlashingBladeSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ArmourRollAttack instead of block |
| ThinkingMansTroll | ✓ | 4 (ThinkingMansTrollSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | single-die reroll once-per-half |
| ToxinConnoisseur | ✓ | 4 (ToxinConnoisseurSkillTest) | 4 (SKILL_TABLE) + engine | ✓ | |
| Treacherous | ✓ | 4 (TreacherousSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | stab teammate for ball |
| UnstoppableMomentum | ✓ | 4 (UnstoppableMomentumSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | reroll Skull on blitz block die |
| ViciousVines | ✓ | 4 (ViciousVinesSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | block range 2 squares |
| WatchOut | ✓ | 4 (WatchOutSkillTest) | 4 (SKILL_TABLE) + 2 engine | ✓ | ignore first BothDown per half |

---

## 8. Injury Causes

Java: `ffb-common/src/main/java/com/fumbbl/ffb/injury/` (48 classes)  
Rust: InjuryCause enum + injury resolution paths in `ffb-engine/src/engine/mod.rs` + `ffb-mechanics/src/injury/mod.rs`  
Java tests: `InjuryCalcTest` (12), `InjuryTypeBlockBB2025Test` (3)

Most injury causes map to a Rust engine path rather than a 1:1 class. The Rust side handles them through `apply_injury(cause, ...)` dispatching.

| Java Injury Class | Translate? | Rust Equivalent | Status | Notes |
|---|---|---|---|---|
| injury/Block.java | yes | engine: block injury path | ✓ | |
| injury/BlockProne.java | yes | engine: block prone path | ✓ | |
| injury/BlockProneForSpp.java | yes | engine: block prone + SPP | ✓ | |
| injury/BlockStunned.java | yes | engine: block stunned | ✓ | |
| injury/BlockStunnedForSpp.java | yes | engine: block stunned + SPP | ✓ | |
| injury/DropDodge.java | yes | engine: `apply_fall_injury` | ✓ | Session 12 |
| injury/DropDodgeForSpp.java | yes | engine: dodge fall + SPP | ✓ | Session 12 |
| injury/DropGFI.java | yes | engine: `apply_fall_injury` | ✓ | Session 12 |
| injury/DropJump.java | yes | engine: leap/pogo fall | ✓ | Session 12 |
| injury/CrowdPush.java | yes | engine: crowd surf | ✓ | |
| injury/CrowdPushForSpp.java | yes | engine: crowd surf + SPP | ✓ | |
| injury/Foul.java | yes | engine: foul injury | ✓ | |
| injury/FoulForSpp.java | yes | engine: foul + SPP | ✓ | |
| injury/FoulWithChainsaw.java | yes | engine: chainsaw foul | ✓ | |
| injury/FoulForSppWithChainsaw.java | yes | engine: chainsaw foul + SPP | ✓ | |
| injury/Chainsaw.java | yes | engine: chainsaw attack | ✓ | |
| injury/ChainsawForSpp.java | yes | engine: chainsaw + SPP | ✓ | |
| injury/Stab.java | yes | engine: stab | ✓ | Session 12 |
| injury/StabForSpp.java | yes | engine: stab + SPP | ✓ | Session 12 |
| injury/Fireball.java | yes | engine: wizard fireball | ✓ | Session 12 |
| injury/Lightning.java | yes | engine: wizard lightning | ✓ | Session 12 |
| injury/Bomb.java | yes | engine: bomb | ✓ | |
| injury/BombForSpp.java | yes | engine: bomb + SPP | ✓ | |
| injury/BallAndChain.java | yes | engine: BallAndChain injury | ~ | Implemented with Group 158 |
| injury/BreatheFire.java | yes | engine: BreatheFire injury | ✓ | Session 12 |
| injury/BreatheFireForSpp.java | yes | engine: BreatheFire + SPP | ✓ | Session 12 |
| injury/Bitten.java | yes | engine: Bitten (BloodLust) | ✓ | |
| injury/EatPlayer.java | yes | engine: MonstrousMouth | ✓ | |
| injury/ProjectileVomit.java | yes | engine: ProjectileVomit | ~ | |
| injury/QuickBite.java | yes | engine: QuickBite | ~ | |
| injury/KegHit.java | yes | engine: KegHit (Bloodweiser) | ~ | |
| injury/KTMInjury.java | yes | engine: TTM hit | ✓ | |
| injury/KTMHitPlayer.java (→TTMHitPlayer) | yes | engine: TTM hit player | ✓ | |
| injury/KTMHitPlayerForSpp.java | yes | engine: TTM + SPP | ✓ | |
| injury/KTMCrowd.java | yes | engine: TTM crowd | ✓ | |
| injury/KTMFumbleInjury.java | yes | engine: TTM fumble | ✓ | |
| injury/KTMFumbleApoKoInjury.java | yes | engine: TTM fumble APO KO | ✓ | |
| injury/TTMLanding.java | yes | engine: TTM landing | ✓ | |
| injury/PilingOnArmour.java | yes | engine: PilingOn armor roll | ✓ | |
| injury/PilingOnInjury.java | yes | engine: PilingOn injury | ✓ | |
| injury/PilingOnKnockedOut.java | yes | engine: PilingOn KO | ✓ | |
| injury/Sabotaged.java | yes | engine: Saboteur target | ~ | Implemented with Saboteur Group 146 |
| injury/Saboteur.java | yes | engine: Saboteur trigger | ~ | Implemented with Saboteur Group 146 |
| injury/TrapDoorFall.java | yes | engine: trapdoor fall | ~ | |
| injury/TrapDoorFallForSpp.java | yes | engine: trapdoor + SPP | ~ | |
| injury/ThenIStartedBlastin.java | yes | engine: ThenIStartedBlastin | ✓ | |
| injury/ThrowARock.java | yes | engine: ThrowARock kickoff | ~ | |
| context/InjuryContext.java | yes | `ffb-mechanics/src/modifiers/contexts.rs` | ✓ | |
| context/ModifiedInjuryContext.java | yes | `ffb-mechanics/src/modifiers/contexts.rs` | ✓ | |

---

## 9. Inducements, Cards & Prayers

### 9a. Inducement Infrastructure

Java: `ffb-common/src/main/java/com/fumbbl/ffb/inducement/`

| Component | Java Class | Translate? | Rust Location | Status |
|---|---|---|---|---|
| Inducement base | inducement/Inducement.java | yes | `ffb-mechanics/src/inducement/mod.rs` | ✓ |
| InducementCollection | inducement/InducementCollection.java | yes | `ffb-mechanics/src/inducement/mod.rs` | ✓ |
| InducementType | inducement/InducementType.java | yes | JSON data + InducementDef | ✓ |
| InducementDuration | inducement/InducementDuration.java | yes | InducementDef | ~ |
| InducementPhase | inducement/InducementPhase.java | yes | InducementDef | ~ |
| Prayer (base) | inducement/Prayer.java | yes | `ffb-mechanics/src/inducement/mod.rs` | ✓ |
| Card (base) | inducement/Card.java | yes | pending | ○ |
| CardType | inducement/CardType.java | yes | `ffb-model/src/enums/card.rs` | ~ |
| CardEffect | inducement/CardEffect.java | yes | `ffb-model/src/enums/card.rs` | ~ |
| BriberyAndCorruptionAction | inducement/BriberyAndCorruptionAction.java | yes | engine: ArgueTheCall | ✓ |

### 9b. Inducement Implementations (by edition)

| Inducement | Java Source | Implemented in Rust | Status | Notes |
|---|---|---|---|---|
| Bribes | bb2020/InducementCollection | yes | ✓ | |
| Argue the Call | bb2020/InducementCollection | yes | ✓ | |
| Wizard (Fireball) | bb2020/InducementCollection | yes | ✓ | |
| Wizard (Lightning) | bb2020/InducementCollection | yes | ✓ | |
| Master Chef (BB2020) | bb2020/InducementCollection | yes | ✓ | |
| Prayers to Nuffle | bb2020/Prayer.java + Prayers.java | yes | ✓ basic | ~ |
| Bloodweiser Kegs | bb2020/InducementCollection | yes | ✓ | |
| Bugman's XXXXXX | bb2020/InducementCollection | yes | ~ | |
| Halfling Master Chef (BB2016) | bb2016/InducementCollection | yes | ~ | |
| Riotous Rookies | bb2020/InducementCollection | yes | ~ | Engine emits RiotousRookies event at kickoff; 4 engine tests (Groups 49–50) |
| Magic Item Cards (BB2016) | bb2016/Cards.java + CardType | yes | ○ | Card system pending |
| Dirty Trick Cards (BB2016) | bb2016/Cards.java + CardType | yes | ○ | Card system pending |
| Infamous Staff | bb2020/InducementCollection (infamousStaff) | yes | ○ | Player-based inducement; complex roster interaction |
| Star Players | bb2020/InducementCollection | yes | ~ | Roster-loaded |
| BB2025 Prayers | bb2025/Prayer.java + Prayers.java | yes | ~ | Prayer table now edition-aware: BB2025 uses blessing_of_nuffle / dazzling_catching; 2 new tests |

---

## 10. Kickoff Events

Java: `ffb-common/src/main/java/com/fumbbl/ffb/bb2016/KickoffResult.java` etc.  
Rust: `ffb-model/src/kickoff/mod.rs` (tables) + `ffb-mechanics/src/kickoff/mod.rs` (helpers) + engine processing  
Java tests: `KickoffEventCalcTest` (10 tests)  
Rust tests: `ffb-mechanics/src/mechanics/kickoff_event.rs` (10) + `ffb-model/src/kickoff/mod.rs` (8) + `ffb-mechanics/src/kickoff/mod.rs` (6) = 24

| Event | BB2016 | BB2020 | BB2025 | Rust Implemented | Status |
|---|---|---|---|---|---|
| Get the Ref | ✓ | ✓ | ✓ | ✓ | ✓ |
| Riot / Time-out | Riot | Time-out | Time-out | ✓ | ✓ |
| Perfect Defence / Solid Defence | Perfect | Solid | Solid | ✓ | ✓ |
| High Kick | ✓ | ✓ | ✓ | ✓ | ✓ |
| Cheering Fans | ✓ | ✓ | ✓ (prayer/assist variant) | ✓ | ✓ |
| Weather Change | ✓ | ✓ | ✓ | ✓ | ✓ |
| Brilliant Coaching | ✓ | ✓ | ✓ | ✓ | ✓ |
| Quick Snap | ✓ | ✓ | ✓ | ✓ | ✓ |
| Blitz / Charge | Blitz | Blitz | Charge | ✓ | ✓ |
| Throw a Rock / Officious Ref / Dodgy Snack | Rock | OficiousRef | DodgySnack | ~ | ✓ |
| Pitch Invasion | ✓ | ✓ | ✓ | ✓ | ✓ |

---

## 11. Game Engine Phases

Java source: `ffb-server/src/main/java/com/fumbbl/ffb/server/step/` (~730 files, **not translating** the step structure).  
Rust: unified `ffb-engine/src/engine/mod.rs` (33,000+ LOC, 719 tests).

| Phase | Representative Java Steps | Rust Handler | Status | Notes |
|---|---|---|---|---|
| Coin toss | StepInitGame | `handle_coin_choice` | ✓ | |
| Kickoff setup / receive choice | StepKickoff* | `handle_receive_choice` | ✓ | |
| Player setup | StepSetup* | Setup state | ✓ | |
| Player activation | StepActivate* | `activate_player` | ✓ | |
| Movement / GFI / dodge | StepMove*, StepGfi*, StepDodge* | `move_player`, `go_for_it`, `dodge` | ✓ | |
| Ball pickup | StepPickup* | `pickup_ball` | ✓ | |
| Block / blitz | StepBlock* | `block`, `blitz` | ✓ | |
| Pushback / follow-up | StepPushback* | `resolve_pushback` | ✓ | |
| Pass | StepPass* | `pass_ball` | ✓ | |
| Catch / interception | StepCatch*, StepIntercept* | `catch_ball` | ✓ | |
| Hand-off | StepHandOff* | `hand_off` | ✓ | |
| Foul | StepFoul* | `foul` | ✓ | |
| Injury / KO resolution | StepInjury* | `apply_injury`, `apply_fall_injury` | ~ | Session 12: partial wiring |
| Touchdown | StepTD* | `score_touchdown` | ✓ | |
| Kickoff events | StepKickoffEvent* | `process_kickoff_event` | ✓ | |
| Half-time / end-game | StepHalfTime*, StepEndGame* | `handle_half_time`, `end_game` | ~ | |
| Re-roll offer / consume | StepReRoll* | `offer_reroll`, `consume_reroll` | ✓ | |
| Apothecary | StepApothecary* | `apply_apothecary` | ✓ | |
| SPP awards | StepSPP* | wired into injury/TD paths | ✓ | |
| Throw Team Mate | StepTTM* | TTM action chain | ✓ | |
| Bomb | StepBomb* | Bomb action | ✓ | |
| BreatheFire | (bb2020/bb2025) | `breathe_fire` | ✓ | Session 12 |
| BallAndChain movement | (bb2016) | `handle_ball_and_chain_move` | ~ | Group 158: 3 tests covering scatter, collision, crowd surf |

---

## 12. Agent / Legal Actions

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 44 | AgentPrompt / AgentResponse | (Rust-only) | n/a | `ffb-model/src/prompts/agent_prompt.rs` | — | 2 | ~ |
| 49 | Action enum + PlayerActionChoice | (Rust-only) | n/a | `ffb-engine/src/action/mod.rs` | — | 2 | ~ |
| 50 | legal_actions (activate, move, block…) | (Rust-only) | n/a | `ffb-engine/src/legal_actions/mod.rs` | — | 21 | ~ |
| 57 | Agent trait + RandomAgent | external: ffb-ai/ | yes | `ffb-engine/src/agent/mod.rs` | — | 4 | ~ |
| 58 | run_game loop | (Rust-only) | n/a | `ffb-engine/src/engine/mod.rs` | — | engine | ~ |
| 59 | PathProbabilityFinder | ffb-ai/PathProbabilityFinder.java | yes | pending | — | — | ○ |
| 60 | MoveDecisionEngine (full legal action enum) | ffb-ai/ | yes | partial in legal_actions | — | 21 | ○ |

---

## 13. Network Protocol

| ID | Component | Java Source | Translate? | Rust Location | Java Tests | Rust Tests | Status |
|---|---|---|---|---|---|---|---|
| 45 | NetCommand trait + parse/serialize | `ffb-common/.../net/` | yes | `ffb-protocol/src/commands/mod.rs` | — | 1 | ~ |
| 46 | Client commands (~35 structs) | `ffb-common/.../command/` | yes | `ffb-protocol/src/client_commands/mod.rs` | — | 1 | ~ |
| 47 | Server commands (~18 structs) | `ffb-common/.../report/` | yes | `ffb-protocol/src/server_commands/mod.rs` | — | 1 | ~ |
| 61 | ClientConnection (WebSocket) | `ffb-client/.../net/` | yes | `ffb-client/src/connection/` | — | — | ~ |
| 62 | ServerCommand handlers | (client logic) | yes | `ffb-client/src/handlers/` | — | — | ~ |
| 63 | ClientStateDispatch | (client logic) | yes | `ffb-client/src/state_dispatch/mod.rs` | — | 1 | ~ |
| 64 | NetworkActionEncoder | (client logic) | yes | `ffb-client/src/network_encoder/` | — | — | ~ |

---

## 14. Parity Infrastructure

| ID | Component | Java Source | Translate? | Rust Location | Rust Tests | Status |
|---|---|---|---|---|---|---|
| 65 | JSONL log format | (Rust-only) | n/a | `ffb-parity/src/log_format.rs` | 3 | ~ |
| 66 | run_java_headless (ParityRunner.java) | `ffb-server/ParityRunner.java` | n/a | `ffb-parity/src/runner.rs` | — | ~ |
| 67 | run_rust_headless (Xoshiro RNG + hashes) | (Rust-only) | n/a | `ffb-parity/src/runner.rs` | — | ~ |
| 68 | compare_logs (step-level diff) | (Rust-only) | n/a | `ffb-parity/src/comparator.rs` | — | ~ |
| 69 | progress.html dashboard | (Rust-only) | n/a | `ffb-parity/src/update_progress.rs` | — | ~ |
| 70 | Parity runner 100/100 seeds | (goal) | n/a | `ffb-parity/src/main.rs` | — | **100/100** ✓ |
| 71 | Network integration test | (goal) | n/a | `ffb-parity/src/network_test.rs` | — | ○ |

---

## 15. NOT Translating

These Java modules are explicitly excluded from the Rust translation:

| Module | Location | Reason |
|---|---|---|
| Step/Stack orchestration (~730 files) | `ffb-server/src/main/java/.../server/step/` | Replaced by unified GameEngine state machine |
| Database layer | `ffb-server/src/main/java/.../server/db/` | Server-only persistence (MySQL/MariaDB) |
| Jetty WebSocket server | `ffb-server/src/main/java/.../server/net/` | Server infrastructure |
| Request routing / handlers | `ffb-server/src/main/java/.../server/handler/` | Server infrastructure |
| Admin tools | `ffb-server/src/main/java/.../server/admin/` | Server infrastructure |
| Injury step handlers | `ffb-server/src/main/java/.../server/injury/` | Orchestration, not computation |
| Inducement step handlers | `ffb-server/src/main/java/.../server/inducements/` | Orchestration |
| Skill behaviour handlers | `ffb-server/src/main/java/.../server/skillbehaviour/` | Orchestration |
| Sequence generators | `ffb-server/src/main/java/.../step/generator/` | Java step-stack specific |
| AWT/Swing client GUI | `ffb-client/src/main/java/` | GUI — Rust has no GUI |
| Client state machine | `ffb-client-logic/src/main/java/.../client/state/` | GUI orchestration |
| Assets / sounds / icons | `ffb-resources/` | Assets |
| Build tools | `ffb-tools/` | Build-time Java utilities |
| TalkHandler tests | `ffb-server/src/main/java/.../handler/talk/*Test.java` | Server integration tests |

---

> **Session 23 (2026-05-31):** Added 4 Rust `#[test]` functions per skill for all 149 previously-untested SKILL_TABLE entries in `crates/ffb-mechanics/src/skills/mod.rs`. Tests cover: class_name string, category enum, edition membership, and from_class_name round-trip. Mechanics test count: 430 → 1,026. Total workspace tests: 1,332 → 1,946, all passing.

> **Session 22 (2026-05-31):** Massive Java skill test expansion — wrote 4-test Java classes for ~150 additional skills across all editions (bb2016, bb2020, bb2025, common, mixed, mixed/special, bb2020/special, bb2025/special). Java total: 379 → 1,941 tests. Pattern: name, category, one NamedProperty assertion (or class name for mixed multi-edition skills), and @RulesCollection edition annotation. All 1,941 tests pass.

*Last updated: 2026-06-01 (session 25)*
