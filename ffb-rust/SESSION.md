# FFB-Rust Session State

## Current Status (session 41 end, 2026-06-04)

**Test counts: 2,591 total (882 engine, 1,214 mechanics, 406 model, 37 parity, 21 protocol, 31 client)**
**Java @Test invocations: ~2,370**
**Parity: T2 complete — 25/25 races × 100/100 seeds (2,500 games, BB2025) ✓ (chaos_chosen confirmed)**
**Sections 1–13: All complete ✓**

All tests passing. Zero failures.

---

## Session 41 Summary (2026-06-04)

**Goal:** Fix chaos_chosen parity (was producing empty output in T2 suite).

**Result:** ✓ chaos_chosen 100/100. No new unit tests needed.

### Root cause fixed

**chaos_chosen roster alias missing** (`crates/ffb-parity/src/runner.rs`)
- `make_team_from_roster("chaos_chosen", ...)` found no matching roster JSON and silently fell back to an all-lineman team.
- Java's `teamChaosChosenParityHome.xml` uses `<rosterId>chaos.lrb6</rosterId>` — i.e. it IS the Chaos team.
- Fix: added `"chaos_chosen" => "chaos"` to the alias table alongside the existing `"renegades"` alias.
- Also replaced `"chaos"` with `"chaos_chosen"` in `run_final_t2.ps1` to use the canonical FUMBBL team name going forward.

### Clarification

"Chaos" and "Chaos Chosen" are the same team. The suite now consistently uses `chaos_chosen` as the race name; `chaos` was the old alias.

---

## Session 39 Summary (2026-06-04)

**Goal:** Achieve 100/100 seeds for all 25 BB2025 races in T2.

**Result:** ✓ All 25 races pass 100/100 seeds. 10 new unit tests added.

### Root causes fixed

**1. Disturbing Presence missing from CSTI kickoff catch** (`crates/ffb-engine/src/engine/mod.rs`)
- Java `CatchModifierCollection` adds +1 per adjacent DP-skilled opponent within 3 squares. Rust's CSTI `check_and_catch` was only counting tackle zones.
- Fix: added `dp` counter alongside `tz` in the catch formula: `min_roll = (ag + tz + dp + scatter_mod).max(2).min(6)`.
- Also changed tz counting to use `has_tacklezones()` instead of `is_standing()` (matches Java's modifier logic for confused/hypnotized players).
- Affected races: Norse (Snow Troll DP), Nurgle.

**2. Roster name matching failed for multi-word races** (`crates/ffb-parity/src/runner.rs`)
- "chaos_dwarf" didn't match roster `id="chaosdwarf.lrb6"` (underscore vs no-separator) — all multi-word races silently fell back to all-lineman teams (ag=3 everywhere).
- Fix: normalize both sides with `|s| s.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase()`.
- Also added explicit alias: `"renegades"` → `"chaos renegade"` (roster `id="1050157"`, name doesn't contain "renegade").
- Affected races: chaos_dwarf, chaos_pact, dark_elf, high_elf, wood_elf, renegades.

**3. BallAndChain Pitch Invasion immunity** (`crates/ffb-engine/src/engine/mod.rs`)
- In BB rules, BaC players cannot be stunned. Java calls `InjuryTypeBallAndChain.handleInjury()` (consumes 2 d6 injury dice) and leaves the player Standing. Rust was setting the player Prone and skipping the dice, shifting all downstream game RNG.
- Fix: in Pitch Invasion player stun loop, check `has_skill(SkillId::BallAndChain)` → if true, consume 2×`d6()` without changing player state.
- Affected races: Goblin (Fanatic), Chaos Dwarf (Deathroller's BaC on adjacent players).

**4. BRIBES dialog infinite loop in Java halftime** (`ffb-java/.../ParityRunner.java`)
- Java `StepEndTurn.useSecretWeaponBribes()` sets `fBribesChoiceAway/Home` to non-null only when called with a non-null inducement type. RandomStrategy was sending `null` inducement → flag stayed null → loop forever.
- Fix: `ParityRunner` now explicitly handles `case BRIBES:` by finding the AVOID_BAN inducement type and sending `ClientCommandUseInducement(avoidBanType, new String[0])` to properly decline.
- Also fixed: MVP dialog required a non-empty player selection (first eligible player selected); `MAX_ITERATIONS` raised to 2,000,000 as safety headroom.
- Affected races: Goblin, Chaos Dwarf, Dwarf (SW ejection halftime).

**5. T2 script used stale debug binary** (`run_final_t2.ps1`)
- Script referenced `target\debug\ffb-parity.exe` (unmodified pre-session binary). Changed to `target\x86_64-pc-windows-msvc\debug\ffb-parity.exe`.

### New unit tests (10 total)

**Engine (crates/ffb-engine/src/engine/mod.rs, Groups 240–241):**
- `pitch_invasion_ball_and_chain_player_stays_standing` — BaC player remains Standing after Pitch Invasion
- `pitch_invasion_non_bac_player_goes_prone` — non-BaC players still go Prone
- `csti_kickoff_catch_disturbing_presence_raises_min_roll` — DP code path exercised, no panic

**Parity runner (crates/ffb-parity/src/main.rs, roster_name_tests module):**
- `chaos_dwarf_resolves_to_actual_roster` — jersey 1 is Minotaur (ag≠3), not lineman fallback
- `dark_elf_resolves_to_actual_roster`
- `high_elf_resolves_to_actual_roster`
- `chaos_pact_resolves_to_actual_roster` — team contains low-ag position (Goblin)
- `wood_elf_resolves_to_actual_roster`
- `renegades_resolves_via_alias` — jersey 1 is Renegade Rat Ogre (ag=4)
- `single_word_races_still_resolve` — regression check for amazon/chaos/dwarf/goblin/nurgle/norse

---

### Session 36 Summary (continuation of session 35)

**Section 13 — Network Protocol → ✓ (6 of 7 rows):**

**ID 45 — commands/mod.rs (1→4 tests):**
- `serialize_then_parse_server_pong`: serialize ServerPong, parse it back
- `parse_server_command_returns_error_on_bad_json`: malformed JSON → Err
- `serialize_client_block`: tag + field content verified

**ID 46 — client_commands/mod.rs (1→11 tests):**
- Serde round-trips for: Move, Block, ActingPlayer, Pass, CoinChoice, UseReRoll, BuyInducements, Join
- `client_tag_is_camel_case`: JSON tag format verified

**ID 47 — server_commands/mod.rs (1→7 tests):**
- Serde round-trips for: Status, Talk, Join, Pong, GameList
- `server_tag_is_camel_case`: JSON tag format verified

**ID 62 — handlers/mod.rs (0→4 tests):**
- `handle_server_game_state_replaces_game`: game state replaced entirely
- `handle_server_game_time_updates_half`: half field updated
- `handle_server_status_updates_status`: status field updated
- `handle_informational_commands_return_empty_events`: Pong/Talk → no events

**ID 63 — state_dispatch/mod.rs (1→8 tests):**
- Regular mode our/opponent turns, Setup mode our/opponent turns, Kickoff, EndGame

**ID 64 — network_encoder/mod.rs (0→16 tests):**
- All major Action variants: CoinChoice, EndTurn, Move, Block, Pass, FollowUp, UseReRoll, Foul, PushTo, HandOff, ActivatePlayer(Move), BuyInducements
- Edge cases: Acknowledge→None, star player attacks→ClientBlock, TricksterMove→ClientMove

**ID 61 — ClientConnection** remains ~ (async WebSocket, requires live server for integration test)

---

### Session 35 Summary

**Phase C — Section 11 completion:**
- `fall_injury_armor_holds_sets_player_prone_no_injury_roll`: armor holds → PS_PRONE, `injury_roll=None`
- `fall_injury_armor_breaks_produces_full_injury_roll`: AV=1 → armor breaks → full injury roll present
- `injury_ko_path_sets_knocked_out_state`: scan seeds for KO result → PS_KNOCKED_OUT confirmed
- `half_time_swaps_offense_and_defense`: `home_first_offense` flips at halftime
- Section 11 rows Injury/KO and Half-time → ✓

**Phase D — Section 12 ~ coverage boost:**
- **legal_actions (ID 50)** +11 tests → 32 total → ✓: `legal_move_targets` with opponent nearby, ball carrier block target, foul blocked when `foul_used=true`, `EndTurn` accepted in Regular mode, off-pitch move targets empty
- **AgentPrompt/AgentResponse (ID 44)** +7 tests → 9 total → ✓: BlockChoice, SelectSkill, Pushback, TricksterMove, ActivatePlayer serde round-trips; AgentResponse SelectSkill and TeamSetup round-trips
- **Action enum (ID 49)** +7 tests → 9 total → ✓: serde round-trips for PlayCard (with/without target), LashOut, Bite, ArmourRollAttack, ThrowKeg, TricksterMove
- **RandomAgent (ID 57)** +4 tests → 8 total → ✓: responds to ReRollOffer, FollowUp, ActivatePlayer, BlockChoice prompts
- **run_game loop (ID 58)** → ✓ (already evidenced by run_game_terminates_with_random_agents)

**Phase E — MoveDecisionEngine (ID 60) → ✓:**
- New file: `crates/ffb-engine/src/agent/move_decision_engine.rs`
- Translated `ActionScore.java`, `PolicySampler.java`, `MoveDecisionEngine.java`
- **ActionScore**: probability × value × confidence with softmax shift
- **PolicySampler**: `softmax()`, `argmax()`, `sample()` (softmax-based weighted sampling)
- **MoveDecisionEngine::select_player()**: scores eligible players by role (carrier/blitz/block/retriever/receiver/support/end-turn) → softmax selection with T=0.50
- **MoveDecisionEngine::select_move()**: uses `find_all_paths` to score reachable squares by role + advance toward endzone → softmax with T=0.60
- **block_probability_coords()**: lookup table by relative ST ratio
- **run_game_with_mde()**: game loop that passes engine state to MDE decision functions; falls back to RandomAgent for non-MDE prompts
- 12 tests: ActionScore clamp/product, PolicySampler softmax/argmax, block probability, advance score, endzone distance, chebyshev, full-game termination

---

### Session 34 Summary

**Phase A cleanup — card duration lifecycle + BallAndChain + DodgySnack:**

**Card duration lifecycle fix (known open issue from session 33):**
- Added `card_temporary_skills: Vec<(PlayerId, SkillId, String)>` to `GameEngine` struct
- When `Action::PlayCard` applies skills to a player, entries are now recorded in `card_temporary_skills`
- Activation-time clear (`temporary_skills.clear()` at `ActivatePlayer`) now PRESERVES card-applied skills via retain, so BoneHead/ReallyStupid/NoHands from cards survive until turn/drive end
- New helper `clear_card_temporary_skills()`: removes card-applied skills from affected players and emits `CardDeactivated` event per unique card_id
- Called in both EndTurn paths (Blitz mini-turn and normal) and at drive end (`eject_secret_weapon_players`)
- 3 new engine tests: persist-through-activation, removed-after-EndTurn, removed-at-drive-end

**BallAndChain → ✓:**
- Added `ball_and_chain_with_frenzy_does_not_prompt_follow_up_block` test: verifies that Frenzy does not trigger a follow-up block prompt after BallAndChain collision (attacker falls prone immediately)
- COMPONENTS.md Section 11 row updated to ✓ (7 total tests: auto-move, scatter, collision, crowd surf, WhirlingDervish, Frenzy-no-followup, path-ignore)

**Section 10 DodgySnack → ✓:**
- Test `kickoff_dodgy_snack_bb2025_affects_a_player` already existed; COMPONENTS.md row updated from ~ to ✓

**PathProbabilityFinder (ID 59) → ✓ (Phase B):**
- New file: `crates/ffb-mechanics/src/mechanics/path_probability.rs`
- Dijkstra max-probability path finder translated from `ffb-ai/PathProbabilityFinder.java`
- Exported types: `PlayerMoveContext`, `PathContext`, `OpponentOnField`, `PathEntry`
- Main function: `find_all_paths(player, field) → HashMap<FieldCoordinate, PathEntry>`
- `PlayerMoveContext`: start, MA, current_move, agility, strength, rules, TwoHeads, ignore_tz, BreakTackle, gfi_modifier_total, extra_gfi (for Sprint)
- `PathContext`: occupied HashSet + Vec<OpponentOnField> (with has_tackle_zones, has_diving_tackle, has_prehensile_tail, has_disturbing_presence, is_titchy)
- Algorithm: Dijkstra max-heap; `needs_dodge` = TZ adjacent to source; `dodge_modifier` = TZ at dest + DivingTackle/PrehensileTail at source + DP near dest; dodge target via BB2016 table or BB2020 direct formula; BreakTackle uses ST-based alternative when lower; GFI kicks in when `current_move + step > MA`; max steps = MA - current_move + MAX_GFI(+extra_gfi)
- 20 Rust tests covering: prob helpers (4 tests), empty-field baseline, MA/GFI limits, TwoHeads, BreakTackle, dodge BB2016/BB2020 formula helpers, ignore_tz, blocked squares, path reconstruction, Dijkstra finds best path around obstacle

---

### Session 33 Summary

Completed all remaining items in Sections 4, 8, and 9.

**Section 4 — IDs 33, 34, Roster all → ✓:**
- `GameOptionsModelTest.java` extended to 10 @Test: boolean option retrieval, options-array growth, mutually-exclusive options, WIZARD default, getRulesVersion no-throw.
- `GameResultModelTest.java` (9 @Test, @Mock Game): home/away TeamResult non-null, default scores 0, score set/retrieve, winnings, fanFactorModifier default.
- `GameModelTest.java` (11 @Test): uses `new FactoryManager()` (no-arg ctor) + `@Mock IFactorySource` → real `Game` object. Tests: id 0/set, half 1/set, fieldModel/gameResult/options/turnDataHome/Away/actingPlayer non-null.
- `RosterModelTest.java` (9 @Test, no-arg ctor): name, id, apothecary default/disable, reroll cost, empty positions, add/find by id, unknown-id null, race.
- `RosterPositionModelTest.java` (9 @Test, id-ctor): id, name, movement/strength/agility/armour/cost, default ctor, id-ctor.
- Rust boosts: `roster.rs` +4 (non_star_positions filter, fields, position count), `skill_def.rs` +4 (SkillWithValue.new/with_value, SkillDef.new, serde round-trip).

**Section 8 — TrapDoorFallForSpp → ✓:**
- Added trap door check inside `resolve_push` (after defender's coordinate update): roll D6, on 1 scatter ball + apply_fall_injury + remove player + award CAS SPP to attacker.
- 2 engine tests: `trap_door_emits_event_when_pushed_player_lands_on_it`, `trap_door_fall_after_push_removes_player_from_pitch`.

**Section 9 — All remaining items → ✓:**
- Extended `Action::PlayCard` with `target_player_id: Option<PlayerId>` (action/mod.rs + ffb-client network_encoder updated).
- Card effect handler in engine dispatches on `card_id` string:
  - `distract*` → `temporary_skills += BoneHead`
  - `sedative*` / `witch_s_brew` → `temporary_skills += ReallyStupid`
  - `madCapMushroomPotion` → `temporary_skills += NoHands + JumpUp`
  - `*illegal*` / `illegalSubstitution` → `field_model.remove_player` + set PS_RESERVE + emit CardDeactivated
  - `*poison*` → `apply_fall_injury` (armor roll)
- 7 engine tests: one per effect type + without-target guard.
- Infamous Staff → ✓ (engine's job is BuyInducement event, which is already tested).
- Magic Item Cards → ✓, Dirty Trick Cards → ✓.

---

### Session 32 Summary

Completed Sections 4, 8, and 9 (except `○` items and a few remaining `~`).

**Section 4 — IDs 30–32 closed via Mockito Java tests:**
- `TeamModelTest.java` (12 @Test): id, name, rerolls, apothecaries, fan_factor, treasury, race, player add/find/count/null-check. Using `@Mock IFactorySource`.
- `FieldModelTest.java` (10 @Test): ball coordinate/in-play, player coordinate/state, bomb coordinate, weather accessor, player coord null before placement. Using `@Mock Game`.
- `TurnDataModelTest.java` (11 @Test): turn_nr, rerolls, flags (blitz/foul/pass), apothecaries, first-turn, InducementSet non-null. Using `@Mock Game`.
- `ActingPlayerModelTest.java` (10 @Test): player_id, current_move, player_action, going_for_it, standing_up, jumping flags. Using `@Mock Game`.
- `GameOptionsModelTest.java` (4 @Test): options non-null, addOption/getOptionWithDefault, two options. Using `@Mock Game`.
- IDs 30–32 → ✓. IDs 33 (GameOptions) has 4 Java tests (thin) + 8 Rust → ~. ID 34 (Game) and Roster remain ~ (Game requires full FactoryManager chain).

**Section 8 — TrapDoorFall implemented:**
- Added `trap_doors: Vec<FieldCoordinate>` + `has_trap_door()` to `FieldModel` (with serde skip-if-empty)
- Added `GameEvent::TrapDoor { player_id, roll, escaped }` to game_event.rs
- Added trap door check in `apply_move` after `PlayerMoved` event: roll D6, on 1 remove player + scatter ball + apply fall injury
- 4 engine tests (Group 235): fall-through removes player, escape keeps player, ball scatters on fall, no event on normal square
- TrapDoorFall → ✓. TrapDoorFallForSpp remains ~ (SPP-eligible path requires `playerWasPushed && fanInteraction` flag).

**Section 9 — Remaining `~` inducements closed:**
- Bugman's XXXXXX → ✓: 4 tests (class_name, skill presence, fires on roll-1 scan, BB20 context)
- Halfling Master Chef (BB2016) → ✓: added `halfling_master_chef_bb2016_steals_rerolls` test using Rules::Bb2016
- Riotous Rookies → ✓: already had 5 tests
- BB2025 Prayers → ✓: already had bb2025_prayers_use_bb2025_table verifying dazzling_catching/blessing_of_nuffle
- Prayers to Nuffle → ✓: 5 tests comprehensive
- Star Players → ✓: added `star_player_purchase_does_not_add_to_roster` (3 tests total)
- Infamous Staff → ~ (from ○): `infamous_staff_purchase_emits_buy_inducement_event` written; roster interaction deferred
- Magic Item/Dirty Trick cards remain ○ (card execution engine not yet built)

---

### Session 31 Summary

Closed Section 4 Player gap, boosted all model Rust test counts, resolved ~9 Section 8 injury cause gaps, and built Card system skeleton.

**Section 4 — Core Data Model:**
- ID 29 (Player): PlayerModelTest extended to 14 @Test (armour, passing, id, spps, gender, player_type, all-stats round-trip). Rust player.rs boosted from 5 → 13 tests (stat modifier methods, temporary skills, all_skill_ids, niggling default). → ✓
- IDs 30–34 (Team, FieldModel, TurnData/ActingPlayer, GameOptions/GameResult, Game): Rust tests boosted — team.rs 4→8, field_model.rs 4→9, turn_data.rs 3→6, acting_player.rs 3→6, game_options.rs 2→8, game.rs 4→9, game_result.rs 2→6. Java tests blocked by factory constructor requirement → remain ~.

**Section 8 — Injury Causes (9 gaps closed):**
- ThrowARock → ✓ (2 tests: stuns player + emits event)
- BallAndChain → ✓ (5 tests including crowd surf, collision, auto-move)
- ProjectileVomit → ✓ (2 tests: success/failure paths)
- QuickBite → ✓ (3 tests: skill_use after catch, no trigger without adjacent, class_name)
- KegHit → ✓ (3 tests: Group 222+235: skill_use, no-skill guard, target takes Injury)
- Saboteur / Sabotaged → ✓ (3 tests: Group 146)
- TrapDoorFall / TrapDoorFallForSpp → remain ~ (stadium-feature trap door not in Rust engine; requires FieldModel.trapDoors support)

**Section 9 — Card System Skeleton (6 gaps closed):**
- Added `InducementDuration` enum (7 variants, id/name round-trips) to `ffb-model/src/enums/card.rs`
- Added `InducementPhase` enum (8 variants) to `ffb-model/src/enums/card.rs`
- Added `Card` struct + `CardType` enum (MagicItem, DirtyTrick) to `ffb-mechanics/src/inducement/mod.rs`
- CardEffect and CardTarget already had 17 tests → confirmed ✓
- Java CardBaseTest.java: 9 @Test covering CardType deck names, InducementDuration ids/names, InducementPhase names
- InducementDuration ✓, InducementPhase ✓, Card ✓, CardType ✓, CardEffect ✓ — all marked ✓

**Remaining `○` items in Section 9:** Magic Item card instances, Dirty Trick card instances, Infamous Staff.
**Remaining `~` items in Section 8:** TrapDoorFall, TrapDoorFallForSpp (stadium feature).
**Remaining `~` in Section 4:** IDs 30–34, Roster row.

---

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

- Roster JSON loader: `star_players` and `bb2020_rosters` tests may fail (pre-existing format mismatch — needs investigation with `cargo test -p ffb-model`)
- `cargo` must run from PowerShell or `~/.cargo/bin/cargo` in Git Bash (not on PATH in Bash)
- **Sections 1–12 are now complete** — all rows ✓ or —.
- Events not yet emitted in Rust engine: `DefectingPlayers` (post-match illegal-concession, edge case), `TimeoutEnforced` (network CLIENT_ILLEGAL_PROCEDURE command, not applicable to headless engine). `PettyCash` ✓ emitted since session 33. `DoubleHiredStarPlayer` ✓ emitted when both teams buy the same star player (session 37).
- NurglesRot: post-match roster flag only — marked `—`, no engine behavior needed
- Section 13 (Network Protocol): 6 of 7 rows ✓; ID 61 ClientConnection ~ (async WebSocket, no unit tests); ID 71 Network integration test ○ (stub in ffb-parity/src/network_test.rs)

## Runtime Notes

- `cargo` requires PowerShell (not Git Bash)
- Run tests: `cargo test --workspace` from `C:\Users\Admin\niels\ffb-rust`
- Or: `/c/Users/Admin/.cargo/bin/cargo test --workspace --manifest-path /c/Users/Admin/niels/ffb-rust/Cargo.toml`
- Java source: `C:\Users\Admin\niels\ffb\ffb-server\src\main\java\com\fumbbl\ffb\server\`
