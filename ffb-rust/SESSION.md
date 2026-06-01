# FFB-Rust Session State

## Current Status (session 30 end, 2026-06-01)

**Test counts: 2,392 total (822 engine, 1,190 mechanics, 349 model, 27 parity, 3 protocol, 1 client)**
**Java @Test invocations: ~2,231**
**Parity: 100 / 100 seeds passing ✓**

All tests passing. Zero failures.

### Session 30 Summary

Achieved 100/100 parity. Root cause was a missing BB2025 "Inaccurate Pass or Scatter" +1 modifier in the Rust CSTI (CatchScatterThrowIn) loop for CATCH_SCATTER mode.

**BB2025 CATCH_SCATTER modifier fix (engine):**
- Java `CatchModifierCollection` adds +1 to min_roll for `CATCH_SCATTER` and `CATCH_BOMB` modes
- Rust CSTI loop used the same min_roll for first catch (CATCH_KICKOFF) and subsequent catches after bounces (CATCH_SCATTER)
- Fix: added `is_scatter: bool` to the `check_and_catch` closure; `scatter_mod = if is_scatter { 1 } else { 0 }` added to min_roll; `is_scatter = true` set after first bounce
- Effect: min_roll for lineman (agility=3, tz=0) rises from 3 to 4 for CATCH_SCATTER, matching Java exactly
- Parity advanced from 87/100 → 100/100

**Kickoff timeout test correction:**
- `kickoff_timeout_grants_team_with_fewer_turns_left_reroll` was wrong — Java BB2020 `handleTimeout()` only adjusts turn counters, never grants rerolls
- Renamed and rewrote as `kickoff_timeout_emits_event_and_no_rerolls_granted` verifying correct behavior

**COMPONENTS.md:** Parity row updated to 100/100 ✓; section 14 row 70 flipped to `100/100 ✓`.

---

### Session 29 Summary

Implemented Trickster (section 7f) — the last unimplemented skill in section 7. All section 7 skills are now ✓ or —.

**Trickster (Group 234, 4 engine tests):**
- Defender with Trickster + standing + free adjacent square + no BallAndChain → SkillUse prompt before block
- Accepted: defender moves 1 adjacent square (ball follows if carried), then block resumes
- Declined: Trickster marked used, block resumes normally
- BallAndChain cancels Trickster (no offer)
- Implementation: `PendingTrickster` struct, `pending_trickster` field on GameEngine, check in `apply_block` after DumpOff, `UseSkill { Trickster }` handler, `Action::TricksterMove { coord }` action variant, `AgentPrompt::TricksterMove { player_id, squares }` prompt variant
- Network encoder: `TricksterMove → ClientMove`

**COMPONENTS.md:** Trickster 7f `~` → `✓`. All section 7 now complete.

---

### Session 28 Summary

Implemented engine behavior for all 38 remaining ~ section 7g star player traits. All 7g skills now ✓.

**Section 7g — all 38 remaining skills implemented and tested (engine Groups 197–232):**

Group 1 — simple reroll/modifier hooks (15 skills):
- **BlindRage**: reroll source for Dauntless roll
- **BoundingLeap**: ignore leap modifier + reroll source
- **BugmansXXXXXX**: reroll 1 on KO recovery
- **HalflingLuck**: single-die reroll source
- **ThinkingMansTroll**: single-die reroll once-per-half
- **SavageBlow**: reroll block dice once-per-game
- **SavageMauling**: reroll injury roll once-per-game
- **OldPro**: armor reroll once-per-game
- **IllBeBack**: ignore first SecretWeapon ejection (via eject_secret_weapon_players)
- **SneakiestOfTheLot**: allow second foul when turn foul_used=true
- **Reliable**: fumbled TTM lands safely (no injury roll)
- **WatchOut**: ignore first BothDown per half
- **Ram**: +1 armor modifier once-per-game
- **KeenPlayer**: ejected at end of drive (via eject_secret_weapon_players)
- **UnstoppableMomentum**: reroll single Skull die on blitz

Group 2 — moderate extensions (10 skills):
- **GoredByTheBull**: +1 block die on blitz activation
- **CrushingBlow**: +1 armor modifier once-per-game
- **ShotToNothing**: grant HailMaryPass temporarily once-per-game
- **StarOfTheShow**: grant team reroll after TD scored
- **SwiftAsTheBreeze**: ignore GFI/dodge modifier after fail once-per-game
- **Treacherous**: stab teammate for ball (via apply_stab path)
- **QuickBite**: bite adjacent opponent after successful catch
- **RaidingParty**: move adjacent open teammate 1 square (UseSkill handler)
- **Indomitable**: double ST after Dauntless success
- **StrongPassingGame**: add player ST as negative pass modifier

Group 3 — complex/new action types (13 skills):
- **AllYouCanEat**: second ThrowBomb in same activation (don't set has_acted)
- **BeerBarrelBash**: ThrowKeg action, bomb-like arc with injury
- **BlackInk**: auto-succeed HypnoticGaze once-per-game
- **CatchOfTheDay**: D6≥3 at activation to pick up ball within 3 squares
- **FuriousOutburst**: ArmourRollAttack action instead of block
- **FuryOfTheBloodGod**: 2 extra block actions after failed Frenzy second block
- **Kaboom**: force bomb to explode at player's square
- **KickEmWhileTheyReDown**: chainsaw legal targets include prone/KO opponents
- **MaximumCarnage**: second chainsaw attack in same activation
- **PrimalSavagery**: LashOut action, D6+ST vs D6+AV
- **TastyMorsel**: Bite action, armor roll with +1 injury modifier
- **TheFlashingBlade**: ArmourRollAttack action instead of block
- **ViciousVines**: block range extended to 2 squares

**New Action variants added:**
- `Action::LashOut { target_id }` — PrimalSavagery
- `Action::Bite { target_id }` — TastyMorsel
- `Action::ArmourRollAttack { target_id }` — FuriousOutburst / TheFlashingBlade
- `Action::ThrowKeg { coord }` — BeerBarrelBash
- `Action::CatchOfTheDay` — CatchOfTheDay

**New ActingPlayer field:** `fury_of_blood_god_blocks: u8` — tracks extra blocks remaining

**Section 7 status:** All 7a–7g skills ✓ (except Trickster 7f, PlagueRidden 7g — marked —)

**Phase C — Kickoff Events (11 events → ✓):**
Added 11 Rust engine tests (Group 233) for all kickoff events. All 11 events now have engine-level test coverage. Events tested:
GetTheRef, Riot (BB2016), PerfectDefence (BB2016), HighKick, CheeringFans, WeatherChange, BrilliantCoaching, QuickSnap, Blitz, ThrowARock (BB2016), PitchInvasion

**Phase D — MoveSquare (ID 23 → ✓):**
Verified 8 Rust tests (3 move_square + 2 pushback_square + 3 range_ruler) cover all 6 Java scenarios.

**Phase E — Utilities (IDs 26, 27, 28 → ✓):**
- GameRng (5 Rust, no Java equivalent) → ✓
- StringTool (5 Java / 5 Rust — all 5 scenarios covered) → ✓
- state_hash (4 Rust, Rust-only) → ✓

**Remaining `~`:** Trickster (canMoveBeforeBeingBlocked needs pre-block movement state)

### Session 27 Summary

Closed all remaining section 1–4 test gaps, resolved miscategorized skills, implemented Drunkard GFI modifier in the engine, and added all 41 missing section 7g star player traits to the Rust codebase.

**Section 1–4 gaps closed (all → ✓):**
- ID 2 PlayerState: already ≥ Java (23 Rust vs 11 Java) — verified ✓
- ID 3 PlayerGender: +2 tests (genitive round-trip, serde round-trip) → ✓
- ID 4 PlayerType: +1 test (serde round-trip) → ✓
- ID 19 KickoffResult/Rules: verified ≥ Java — ✓
- ID 20–21 FieldCoordinate: +3 tests (is_on_pitch, step, neighbours) → 18 total ✓
- ID 25 ReRollOptions: added `ReRollOptions` struct + 3 tests → ✓

**Skill classification fixes:**
- HailMaryPass: `~ → ✓` (2 engine tests confirmed)
- Disposable: `~ → —` (post-match roster flag, no engine behavior)
- PlagueRidden: `~ → —` (allowsRaisingLineman post-match only)

**Drunkard engine implementation:**
- Added `has_drunkard` flag to movement GFI path in engine
- `gfi_mod += 1` when player has Drunkard, raising target from 2 → 3
- Added 2 engine tests (Group 196): target=3 with Drunkard, target=2 without

**Section 7g — all 41 star player traits added to Rust:**
- Added 41 SkillId variants: AllYouCanEat, BeerBarrelBash, BlackInk, BlindRage, BoundingLeap, BugmansXXXXXX, CatchOfTheDay, CrushingBlow, Drunkard, FuriousOutburst, FuryOfTheBloodGod, GoredByTheBull, HalflingLuck, IllBeBack, Indomitable, Kaboom, KeenPlayer, KickEmWhileTheyReDown, MaximumCarnage, OldPro, PlagueRidden, PrimalSavagery, QuickBite, RaidingParty, Ram, Reliable, SavageBlow, SavageMauling, ShotToNothing, SneakiestOfTheLot, StarOfTheShow, StrongPassingGame, SwiftAsTheBreeze, TastyMorsel, TheFlashingBlade, ThinkingMansTroll, Treacherous, Trickster, UnstoppableMomentum, ViciousVines, WatchOut
- Added 41 SKILL_TABLE entries (category=Trait, editions=[Bb2020, Bb2025])
- Added 164 mechanics tests (4 per skill: class_name, category, editions, lookup_by_class_name)
- Drunkard: marked ✓ (SKILL_TABLE + 2 engine tests)
- PlagueRidden: marked — (post-match only)
- Trickster: remains ~ (canMoveBeforeBeingBlocked not yet in engine)
- Remaining 35 7g skills: ~ (SKILL_TABLE + 4 tests, engine behavior pending)

**Remaining work:**
- Trickster engine behavior (canMoveBeforeBeingBlocked)
- Engine behavior for 35 remaining 7g star player skills

### Session 26 Summary

Audited all `~` skills in COMPONENTS.md to determine actual engine test coverage. All `~` skills were already implemented in `ffb-engine/src/engine/mod.rs`; they remained `~` only because COMPONENTS.md had not been updated.

**Thin-coverage skills boosted** (wrote 1 complementary negative/edition test per skill):
- BurstOfSpeed: +1 → 2 tests ✓
- SafePass: +1 → 2 tests ✓  
- MyBall: +1 → 2 tests ✓
- LordOfChaos: +1 → 2 tests ✓
- PumpUpTheCrowd: +1 → 2 tests ✓
- BlastinSolvesEverything: +1 → 2 tests ✓
- FanFavourite: +1 → 2 tests ✓
- KickTeamMate: +1 → 2 tests ✓
- Timmmber: +1 → 2 tests ✓
- Cannoneer: +1 → 2 tests ✓

**Marked ✓ in COMPONENTS.md** (all had ≥2 comprehensive engine tests):
- Section 7b: BallAndChain, FanFavourite, KickTeamMate, SecretWeapon, Stakes, Timmmber
- Section 7c: CloudBurster, Fumblerooskie, HitAndRun, PassingIncrease, PileDriver, ProjectileVomit, RunningPass
- Section 7d: BigHand, Bullseye, EyeGouge, Fumblerooski, GiveAndGo, Hatred, LethalFlight, NoBall, PutTheBootIn, QuickFoul, Saboteur, SteadyFooting, Taunt, Unsteady, ViolentInnovator
- Section 7e: ASneakyPair, BlastIt, BlastinSolvesEverything, BurstOfSpeed, ConsummateProfessional, ExcuseMeAreYouAZoat, FrenziedRush, GhostlyFlames, Incorporeal, LordOfChaos, MesmerizingDance, PumpUpTheCrowd, PutridRegurgitation, SlashingNails, TeamCaptain, TwoForOne, WhirlingDervish, WisdomOfTheWhiteDwarf, WoodlandFury, WorkingInTandem, Yoink
- Section 7f: ArmBar, Cannoneer, IronHardSkin, MyBall, PickMeUp, SafePass
- Section 7g: BalefulHex, LookIntoMyEyes

**Remaining `~` skills** (no engine tests yet or not implemented):
- Disposable: no engine behavior (TV calculation only)
- Trickster: not yet implemented in engine (TurnMode::Trickster exists in model)
- Drunkard: no engine tests
- PlagueRidden: no engine tests
- 7g mixed/special: AllYouCanEat, BeerBarrelBash, BlackInk, BlindRage, BoundingLeap, BugmansXXXXXX, CatchOfTheDay, CrushingBlow, FuriousOutburst, FuryOfTheBloodGod, GoredByTheBull, HalflingLuck, IllBeBack, Indomitable, Kaboom, KeenPlayer, KickEmWhileTheyReDown, MaximumCarnage, OldPro, PrimalSavagery, QuickBite, RaidingParty, Ram, Reliable, SavageBlow, SavageMauling, ShotToNothing, SneakiestOfTheLot, StarOfTheShow, StrongPassingGame, SwiftAsTheBreeze, TastyMorsel, TheFlashingBlade, ThinkingMansTroll, Treacherous, UnstoppableMomentum, ViciousVines, WatchOut

### Session 25 Summary

Completed Phases A–G of the cross-repo parity plan: value types, utilities, data model, and enum boosts.

**Rust enum boosts** (matching Java scenarios):
- `block.rs`: +8 tests → 10 total; ALL CAPS names confirmed; ID 7 → ✓
- `injury.rs`: +12 tests → 15 total; BrokenNeck → Ag bug fixed; ID 8 → ✓
- `skill.rs`: +3 tests → 18 total; ID 9 → ✓
- `team.rs`: +7 tests → 20 total; ID 10 → ✓
- `apothecary.rs`: +2 tests → 15 total; ID 13 → ✓
- `client.rs`: +2 tests → 9 total; ID 14 → ✓
- `net.rs`: +6 tests → 16 total; ID 15 → ✓
- `card.rs`: +8 tests → 17 total; ID 18 → ✓
- Total model crate: 276 → 340 tests (+64)

**Rust value type / model boosts:**
- `field_coordinate.rs`: +6 tests → 15 total
- `constants.rs`: +4 tests → 4 total (new test module)
- `block_types.rs`: +2 tests → 5 total
- `team.rs` (model): +2 tests → 4 total
- `field_model.rs`: +1 test → 4 total
- `game.rs`: +2 tests → 4 total

**Java test classes written** (in `ffb-server/src/test/java/com/fumbbl/ffb/server/`):
- `model/GameConstantsTest.java` — 4 tests (field dimensions, endzone bounds)
- `model/MoveSquareTest.java` — 6 tests (MoveSquare, PushbackSquare, RangeRuler)
- `model/BlockRollTest.java` — 5 tests
- `model/ReRollOptionsTest.java` — 4 tests
- `model/PlayerModelTest.java` — 5 tests (uses RosterPlayer concrete subclass)
- `util/StringToolTest.java` — 5 tests
- `skill/YoinkSkillTest.java` — 4 tests
- `skill/DrunkardSkillTest.java` — 4 tests
- `skill/PlagueRiddenSkillTest.java` — 4 tests
- 38 × `skill/*SkillTest.java` for 7g mixed/special skills — 4 tests each (152 tests total)

**Marked ✓:** IDs 7, 8, 9, 10, 12 (PassingDistance, already ≥ Java), 13, 14, 15, 18, 22, 24, 35 (Modifier System)

**Remaining work:**
- Section 2: IDs 20–21 (FieldCoordinate — Rust 15 vs Java 18, need 3 more), 23 (MoveSquare — need Rust tests for Java scenarios), 25 (ReRollOptions — Rust 3 vs Java 4)
- Section 3: ID 26 (GameRng — ffb-ai not accessible from test classpath; skip for now)
- Section 4: IDs 30–34 model classes (no Java test due to factory constructor requirements)
- Parity seeds 57–100 (blocked by Sweltering Heat halftime RNG issue)

### Session 24 Summary

Added comprehensive enum test coverage across both repos (Phase 1 of the cross-repo parity plan):

**Java test classes written/verified** (in `ffb-server/src/test/java/com/fumbbl/ffb/server/model/`):
- `DirectionTest.java` — 14 tests
- `GameStatusTest.java` — 9 tests
- `TurnModeTest.java` — 16 tests
- `SkillEnumTest.java` — 18 tests (SkillCategory, SkillUsageType, DeclareCondition)
- `TeamEnumTest.java` — 20 tests (BoxType, SendToBoxReason, TeamStatus)
- `ApothecaryEnumTest.java` — 15 tests (ApothecaryMode, ApothecaryStatus, ApothecaryType)
- `ClientStateIdTest.java` — 9 tests
- `NetEnumTest.java` — 16 tests (NetCommandId, ServerStatus, LeaderState)
- `PlayerEnumTest.java` — 15 tests (PlayerGender, PlayerType)
- `PlayerActionTest.java` — 29 tests (new file)
- `ReRollEnumTest.java` — 9 tests (ReRollProperty)
- `CardEnumTest.java` — 17 tests (CardEffect, CardTarget)

**Rust enum tests added** (matching Java scenarios):
- `direction.rs`: +7 tests → 13 total
- `game.rs`: +7 tests → 9 total
- `turn.rs`: +13 tests → 16 total
- `skill.rs`: +12 tests → 15 total
- `team.rs`: +11 tests → 13 total
- `apothecary.rs`: +11 tests → 13 total
- `client.rs`: +5 tests → 7 total
- `net.rs`: +8 tests → 10 total
- `player.rs`: +17 tests → 40 total
- `reroll.rs`: +6 tests → 9 total
- `card.rs`: +7 tests → 9 total
- Total model crate: 172 → 276 tests (+104)

**COMPONENTS.md documentation debt resolved:**
- All skill rows (7a–7g) updated with correct Java test counts (0 → 4 per skill)
- Skills with engine ✓ + Java tests + SKILL_TABLE Rust tests now marked ✓
- Enum rows (IDs 1–18) updated with new test counts and status
- Summary section updated: 2,049 Rust / ~2,128 Java @Test annotations

**Remaining work:**
- Phase 2: Value type tests (IDs 20–25) — FieldCoordinate, constants, block types, reroll options
- Phase 3: Utility tests (IDs 26–27) — GameRng, StringTool/UtilPassing
- Phase 4: Data model tests (IDs 29–34) — Player, Team, FieldModel, Game, etc.
- Phase 5: Modifier system parity audit (21 Java vs 25 Rust)
- Missing Java skill tests: Drunkard, PlagueRidden, Yoink + ~30 7g skills
- Parity seeds 57–100 (deferred — blocked by Sweltering Heat halftime RNG issue)

### Session 23 Summary

Added 4 Rust `#[test]` entries per skill for all 149 previously-untested skills in `crates/ffb-mechanics/src/skills/mod.rs`. Tests cover: `class_name`, `category`, edition membership, and `from_class_name` round-trip. Total new tests: 596 (mechanics went from ~430 → 1,026). All 1,946 workspace tests pass.

**Remaining work:**
- Parity seeds 57–100 (deferred — seed 57 blocked by Sweltering Heat halftime RNG issue)

### Session 22 Summary

Wrote Java unit tests (4 @Test each) for ~150 additional skills across all editions. Pattern: `getName()`, `getCategory()`, `hasSkillProperty(NamedProperties.X)` or `getSkillProperties() != null`, and `@RulesCollection` annotation (or `getClass().getSimpleName()` for mixed multi-edition skills).


### Session 21 Summary

**Parity: 56/100 seeds passing** (up from 0/100).

Root cause of seed 57 failure (and all higher seeds that roll SwelteringHeat):
- Java `StepEndTurn.getFaintingCount()` (bb2025) consumes 3 extra game-RNG dice at halftime when weather = `SWELTERING_HEAT` (2d6=2):
  - `d3` = fainting count
  - `d(on_pitch_size)` × fainting_count for home team
  - `d(on_pitch_size)` × fainting_count for away team
- This creates a 3-die RNG offset that shifts all subsequent dice rolls (H2 kickoff scatter, kickoff result, CSTI bounce).
- Fix applied to `ffb-engine/src/engine/mod.rs` halftime block: consume matching dice in the Rust engine when `self.game.weather == Weather::SwelteringHeat`.
- Seeds 1–56 confirmed passing (none rolled SwelteringHeat). Seed 57 still fails under investigation — fix may have a stale-build issue or secondary divergence source.

Full dice sequence for seed 57 (confirmed from Java DICE_TRACE):
- pos 1-2: d3 fans (StepSpectators)
- pos 3-4: d6 weather → sum=2=SwelteringHeat
- pos 5: d2 coin (StepCoinChoice)
- pos 6-7: d8+d6 H1 kickoff scatter (SW, dist 3)
- pos 8-9: 2×d6 H1 kickoff result (sum=10=Charge)
- pos 10: d3 Charge roll (StepApplyKickoffResult)
- pos 11: d8 H1 CSTI bounce (WEST: 22,13→21,13)
- pos 12: d3=1 HALFTIME fainting count
- pos 13: d11=8 HALFTIME home player select
- pos 14: d11=3 HALFTIME away player select
- pos 15-16: d8+d6 H2 kickoff scatter
- pos 17-18: 2×d6 H2 kickoff result
- pos 19: d8 H2 CSTI bounce

## Session 12 Additions

### New Behaviors Implemented

**Leap/Pogo/PogoStick fall injury** — Failed leap now calls `apply_fall_injury` (was incorrectly just setting player to PRONE without any armor/injury roll). Both the immediate-fail path and the reroll-declined path are fixed.

**Dodge fail injury** — Failed dodge (both immediate fail with no reroll and reroll-declined) now calls `apply_fall_injury`.

**GFI fail injury** — Failed GFI (both immediate fail with no reroll and reroll-declined) now calls `apply_fall_injury`.

**`apply_fall_injury` helper** — New centralized method used by all fall paths. Handles: armor roll (with Stunty modifier), injury roll (with Niggling), ThickSkull downgrade, Decay upgrade, Regeneration, serious injury roll, SPP for attacker (when applicable), apothecary eligibility.

**When armor holds during a fall** — `apply_fall_injury` now sets player state to PRONE (stunned) even when armor isn't broken, emitting an `Injury` event with only the armor roll.

**BreatheFire (BB2020+)** — Full implementation:
- Roll 6: KNOCK_DOWN — defender takes full armor+injury roll
- Roll 1 or effective 1: FAILURE — attacker burns themselves (armor+injury), turnover
- Effective roll < 4: NO_EFFECT
- Effective roll 4-5: PRONE — defender placed prone, no armor roll
- Defender with ST > 4: effective roll = roll - 1
- `BreatheFireRoll` event added to `GameEvent` enum
- `Action::BreatheFire { target_id }` added to Action enum
- `PlayerActionChoice::BreatheFire` added
- Wired in `ActivatePlayer` handler and standalone `Action::BreatheFire` handler

**Pogo/PogoStick as Leap variants** — `has_leap` check extended to include `SkillId::PogoStick` and `SkillId::Pogo`.

### Bug Fixes / Correctness

- Fireball injury path: now correctly uses PS_BADLY_HURT for CAS (not PS_KNOCKED_OUT), applies ThickSkull/Decay/Regeneration/SI properly
- Lightning injury path: same improvements as fireball; +1 to armor roll (not injury)
- Crowd surf ThickSkull: ThickSkull check was missing when crowd-surfed player would be KO'd
- Stab injury path: now applies ThickSkull/Decay/Regeneration/SI/SPP properly; was missing all of these

### New Tests Added (session 12, groups 119-120)

- `pogo_allows_move_into_occupied_square`, `pogo_stick_allows_move_into_occupied_square`
- `fireball_applies_decay_and_cas_correctly`, `lightning_applies_serious_injury_on_cas`
- `stab_produces_serious_injury_on_cas`, `stab_decay_player_becomes_cas`
- `leap_into_occupied_square_fails_player_goes_prone` (updated to check Injury event + not-standing state)
- `dodge_fail_triggers_armor_roll`
- `gfi_fail_triggers_armor_roll`
- `breathe_fire_roll_6_knocks_down_defender`
- `breathe_fire_prone_result_places_defender_prone`
- `breathe_fire_failure_injures_attacker`

## Architecture

The Java server uses a ~730-file Step/Stack pattern. The Rust port uses a DIFFERENT architecture: a unified `GameEngine` state machine in `ffb-engine/src/engine/mod.rs` (~18,400+ lines). There is no 1:1 file mapping.

**Crate structure:**
- `ffb-model` — enums, types, data model structs
- `ffb-mechanics` — pure computation functions
- `ffb-engine` — GameEngine state machine, Action enum, RandomAgent
- `ffb-protocol` — JSON serialization
- `ffb-client` — WebSocket connection
- `ffb-parity` — Java vs Rust comparison binary

## Events Emitted by Engine

**Emitted:** Most game events including: BlockRoll, DodgeRoll, GoForItRoll, CatchRoll, PassRoll, InterceptionRoll, PlayerFellDown, PlayerMoved, BallPickedUp, BallScattered, Touchdown, Injury, Pushback, ScatterBall, ScatterPlayer, ThrowIn, KickoffScatter, ReRoll, SkillUse, PlayerAction, StartHalf, ReceiveChoice, WinningsRoll, ApothecaryChoice, WizardUse, SwarmingPlayersRoll, WeepingDaggerRoll, AnimalSavagery, SafeThrowRoll, SwoopPlayer, ThrowTeamMateRoll, BombExplodesAfterCatch, BombOutOfBounds, BreatheFireRoll

**Not yet emitted:** RiotousRookies, ThenIStartedBlastin, CardDeactivated, CardEffectRoll, DefectingPlayers, PassBlock (event, not action), PettyCash, DoubleHiredStarPlayer, GameOptions, TimeoutEnforced

## Mechanics Layer — COMPLETE (session 7)

All 15 mechanic files in `ffb-mechanics/src/mechanics/` implemented + tested with 349 tests.

## Skills Implemented

Tier 1 movement: TwoHeads ✓, BreakTackle ✓, Leap ✓, HypnoticGaze ✓, Frenzy ✓, Juggernaut ✓, Tentacles ✓, Shadowing ✓, DivingTackle ✓, SureFeet ✓, Sprint ✓, Titchy ✓, PogoStick/Pogo ✓ (Leap variants)

Tier 2 block: Wrestle ✓, Sidestep ✓, StandFirm ✓, Grab ✓, PilingOn ✓, DirtyPlayer ✓, SneakyGit ✓, Horns ✓, Dauntless ✓, Claws ✓, MultipleBlock ✓, Brawler ✓, BrutalBlock ✓, DwarfenScourge/DwarvenScourge ✓

Tier 3 special: PassBlock ✓, Kick ✓, SafePairOfHands ✓, Deflect ✓, Accurate ✓, StrongArm ✓, OnTheBall ✓, Loner ✓, Pro ✓, Leader ✓, Animosity ✓, BloodLust ✓, BoneHead ✓, ReallyStupid ✓, WildAnimal ✓, Confusion ✓, AlwaysHungry ✓

Other: Block ✓, Dodge ✓, Catch ✓, SureHands ✓, Tackle ✓, MightyBlow ✓, StripBall ✓, Guard ✓, Regeneration ✓, ThickSkull ✓, Decay ✓, NigglingInjuries modifier ✓, TakeRoot ✓, Stab ✓, DumpOff ✓, Chainsaw ✓, KickOffReturn ✓, AnimalSavagery ✓, SafeThrow ✓, Swoop ✓, WeepingDagger ✓, Swarming ✓, FoulAppearance ✓, DivingCatch ✓, VeryLongLegs ✓, Bombardier ✓, ThrowTeamMate ✓, RightStuff ✓, Punt ✓, MasterAssassin ✓, MonstrousMouth ✓, TheBallista ✓, BreatheFire ✓, LoneFouler ✓, KrumpAndSmash ✓, Slayer ✓, ToxinConnoisseur ✓, UnchannelledFury ✓, JumpUp ✓, Fend ✓, Defensive ✓, DisturbingPresence ✓, NervesOfSteel ✓, ExtraArms ✓, NoHands ✓, PrehensileTail ✓

## Inducements Implemented

Bribes ✓, ArgueTheCall ✓, Wizard (Fireball/Lightning) ✓, MasterChef ✓, PrayersToNuffle ✓, BloodweiserKegs ✓, KickoffReturn ✓

## Known Open Issues

- Roster JSON loader: star_players and bb2020_rosters tests fail (pre-existing format mismatch)
- `cargo` must run from PowerShell or `~/.cargo/bin/cargo` in Git Bash (not on PATH in Bash)
- Parity test: 56/100 seeds passing; seed 57 blocked by Sweltering Heat halftime fix (applied to mod.rs, still failing — possible stale binary or secondary divergence)
- CloudBurster (force interception re-roll) not yet implemented (complex post-interception prompt)
- GhostlyFlames (chainsaw armor modifier removal) not yet implemented (complex modifier interaction)
- EyeGouge (remove pushed opponent's assists) not yet implemented (requires per-player TZ flag)
- Saboteur (knockdown sabotage) not yet implemented (complex interactive prompt)
- Bullseye (TTM re-roll for superb throw) not yet implemented (complex multi-step)
- HitAndRun (move after block) not yet implemented (complex multi-step interactive)
- SteadyFooting (avoid falling, 6+ roll, BB2025) not yet implemented
- BallAndChain (special movement type) not yet implemented
- NurglesRot (post-game mechanic) not relevant to real-time engine
- Fumblerooskie/Fumblerooski (intentional fumble) not yet implemented
- SESSION.md note: RendingClaws does not exist in Java — was listed in error

## Runtime Notes

- `cargo` requires PowerShell (not Git Bash)
- Run tests: `cargo test --workspace` from `C:\Users\Admin\niels\ffb-rust`
- Or: `/c/Users/Admin/.cargo/bin/cargo test --workspace --manifest-path /c/Users/Admin/niels/ffb-rust/Cargo.toml`
- Java source: `C:\Users\Admin\niels\ffb\ffb-server\src\main\java\com\fumbbl\ffb\server\`
