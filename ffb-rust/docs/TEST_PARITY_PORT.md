# Test-Count Equalization Campaign (started 2026-07-21)

User goal: SAME tests on both engines — same test METHODS, 1:1 names (camelCase↔snake_case),
same assertions. Strategy chosen: **hybrid** — prune remaining low-signal Rust tests first,
then port everything surviving to Java. Counting convention: test methods (Java
@ParameterizedTest kept parameterized; surefire run-counts will exceed method counts — that is
accepted).

Starting point: Rust 16,294 `#[test]` fns; Java 1,626 test methods (2,540 surefire runs).
Documented exception: `ffb-parity`'s 39 harness self-tests (Rust-only comparison tooling, no
Java counterpart) + any `// PARITY-EXEMPT` modules batch A3 tags (Rust-only infra).

## Wave plan / status

| Batch | Scope | Status |
|---|---|---|
| A1 | Prune ffb-engine step boilerplate (per-file id_is_* → one exhaustive make_step test; trivial construction; Rust-plumbing) | RUNNING |
| A2 | Prune ffb-model {dialog,factory,inducement,model,util,types,injury,data,events,prompts} low-signal | RUNNING |
| A3 | Prune ffb-client + ffb-server low-signal; tag PARITY-EXEMPT Rust-only infra | RUNNING |
| B0 | Java headless GameState fixture (ffb-server/src/test/.../fixture/) + StepInitStartGame POC | RUNNING |
| B1 | Port ffb-protocol (~882) → ffb-common/src/test/.../net/commands/ | RUNNING |
| B2 | Port ffb-model model/dialog/factory/inducement/types/injury/util survivors → ffb-common tests | pending (after A2) |
| B3 | Port ffb-model report/ (391) → Java report JSON tests | pending |
| B4 | Port ffb-mechanics unmirrored (~700: modifiers/, mechanics extras, skills table tests) → Java | pending |
| B5 | Port ffb-client survivors (~1,900) → ffb-client-logic tests | pending (after A3) |
| B6 | Port ffb-server survivors (~1,200) → Java server tests | pending (after A3) |
| B7 | Port ffb-engine steps (~4,900 survivors) → Java step tests using B0 fixture — split ~15-20 sub-batches by edition/dir | pending (after A1+B0) |
| B8 | Port ffb-engine skill_behaviour (403), injury (555), util/mechanic/marking remainder, model/factory | pending (after B0) |
| C | Final: counts equal (± documented exemptions), full cargo + mvn green, T3 gate, docs, commit | pending |

## Rules for every batch
- No git-mutating commands in agents; disjoint file ownership per batch.
- Java src/main is READ-ONLY (reference engine). New Java tests go in the top-level modules
  (ffb-common/ffb-server/ffb-client-logic), never the nested ffb-java/ffb/ clone.
- A Rust test is prunable only if: pure read-back, Rust plumbing (Deref/Clone/Debug/Default),
  restates the same source line, or Rust-only infra that is ALSO low-signal. Behavioral,
  serialization, lookup, and validation tests must survive and be ported.
- A ported Java test must be verified against the actual Java class (Rust test supplies the
  expected values; Java source is ground truth). Genuine cross-engine discrepancies discovered
  while porting: keep the Java-correct expectation, report prominently, fix Rust production if
  clearly a translation bug (with its own scoped verification).
- Every batch runs its scoped `cargo test` / `mvn test` before reporting.

## Progress log
- Baseline: Rust 16,294 / Java 2,540 surefire runs.
- Wave 1 (2881831c): pruned ~1,570 Rust → 14,725; Java fixture built.
- Wave 2 partial (8cba5a40): B1 protocol port; Java ~3,648.
- Wave 2b (1a6ed1e1): R1 report (336), R2 model, R3 step pilot (78). Java ~4,376
  (ffb-common 1,892 / client-logic 17 / server 2,467). All modules green.
- Step-port intel (from R3): generator tests ~100-150/agent-run; step-logic ~4-8 files/run,
  needs a seeded-dice delegate + GeneratorTestSupport promoted to fixture; ~4,700 step tests
  remain → budget ~40-50 agent-runs. THE dominant remaining cost.
- Wave 3 (a2b7f006/9a3edf3e/f4ce3c30): R2 model finish (ffb-common 1,939); P-gen pruned 134
  Rust generator param tests (engine 7,108); B5r client partial (client-logic 101); B6r server
  partial (server 2,505). Java now ~4,545 distinct. Rust ~14,540.
- Recurring: session-limit interruptions kill agents mid-wave ~every wave; recovery = remove
  broken partials (compile errors / initFrom-NPE-on-default / package-private access), restore
  each module green, commit. Fixed ReportMessageTestBase.Run fields → public for subpackage tests.
- Wave 4 (d2dc289d/003d1328): FX fixture enhancement (ScriptedFortuna scripted dice +
  GeneratorTestSupport promoted, ffb-server 2,511); B5r2 client-logic 101→1,208 (301 classes).
- **BLOCKED (2026-07-21): hit the 200-subagent session cap.** Raising
  CLAUDE_CODE_MAX_SUBAGENTS_PER_SESSION only takes effect after a Claude Code session RESTART.
  Set it high (e.g. 600) and restart to resume the parallel port.

## CLIENT COMMAND-HANDLER STRUCTURAL DIVERGENCE (analysis 2026-07-25)
The ~18 `ClientCommandHandler*` modules do NOT follow the clean ±1 pattern — the two engines test
them at DIFFERENT GRANULARITIES: Rust unit-tests extracted helper fns (should_sync_clock,
find_updates, home_away_coaches, should_report_join, prepare/restore_*_animation round-trips,
should_wait_for_animation variations), while Java tests the end-to-end `handleNetCommand` +
some helpers. E.g. ModelSync J7/R12, UserSettings J2/R7, GameState J1/R4, Join J10/R8 (Java has the
end-to-end extras), Leave J6/R5. Reaching exact 1:1 needs BIDIRECTIONAL porting (~30-50 tests):
port Rust helper tests → Java where the helper is callable, and Java end-to-end → Rust. This is a
FOCUSED PASS (like logicmodule/replay), NOT inline ±1 work. Quick partial win available:
prune trivial Java getIdReturnsServerX / reportId handler tests (Rust doesn't test handler ids).
DEFERRED to a dedicated command-handler pass. All -N LOGIC modules DONE (touchback last, commit).

## -N LOGIC-MODULE PATTERN (Java has MORE — analysis 2026-07-25)
The Java LogicModule-subclass tests carry boilerplate Rust never tests:
`testGetIdReturnsX`, `testAvailableActionsIsEmpty`, `testPerformAvailableActionIsNoOp`,
`testActionContextThrows[ViaDelegation]`. These are trivial (getter/empty/no-op/panic) →
PRUNE from Java (Rust-as-reference), each -N module. Plus a FEW genuine behavioral variations
Rust lacks (setup: square_is_empty_FALSE, handle_command_HANDLED; wait_for_opponent: get_player
prefers-home / falls-back; etc.) — these are Rust UNDER-coverage; cheapest faithful fix is to ADD
the Rust mirror (trivial #[test]), else prune the Java extra. DECISION for the grind: prune the
trivial boilerplate; add Rust mirrors for the cheap behavioral extras. -N modules to do: setup(-5),
wait_for_opponent(-5), quick_snap(-3), spectate(-3), start_game(-3), wait_for_setup(-3),
solid_defence(-2), touchback(-2), kickoff(-1), raiding_party(-1), trickster(-1), rangegridstate(-2),
several clientcommandhandler* (-1/-2). DONE this session: move_logic 10/10, interception 1/1 (commit).

## REPORT-MESSAGE ±1 FAST PATTERN (proven 2026-07-25, 16 modules reconciled)
Most report-message mismatches resolve WITHOUT a render port:
- **-N (Java has more):** the Java extra is a trivial `reportIdIsX()`/`getKey()` getter test with
  no Rust twin → PRUNE the Java getKey test (Rust-as-reference). Did 8: escape/standup/throwIn/
  startHalf/skillUse/spellEffect/foulAppearance/doubleHired.
- **+N (Rust has more):** the Rust extra is almost always a `missing_X`/defensive-guard edge case
  (missing player/direction/prayer, empty roll) that Rust guards but the Java renderer dereferences
  unconditionally (NPE/AIOOBE) — a documented divergence → PRUNE the Rust test (Java already
  skip-commented it). Did 7: select_gaze_target/show_star_re_roll/raiding_party/place_ball_direction/
  prayer_end/indomitable/riotous_rookies. EXCEPTION: a genuinely-portable helper test → ADD to Java
  (free_petty_cash `format_thousands` → `StringTool.formatThousands` direct test).
Remaining report ±N: pass_roll(+1, edition variants), prayer_roll(+1), injury(+2), block_choice(+1),
modified_dodge(+1), modified_pass(+1), skill_use(mixed), + a few. Same pattern expected. Tally:
Java **1367** / Rust **1547**.

## REPORT-MESSAGE RENDER PORT RECIPE (proven 2026-07-24, block_roll)
Java report tests extend `ReportMessageTestBase` (client/report/): `List<Run> runs =
render(new XMessage(), report)` captures each print/println as a `Run{paragraphStyle, textStyle,
text}` in order (mirrors Rust `status_report.rendered_runs`). Deep-stub client+game provided by the
base. Per-message: stub the renderer's specific factory/mechanic (e.g.
`given(game.getRules().getFactory(Factory.BLOCK_RESULT)).willReturn(new BlockResultFactory())`) and
`given(game.getPlayerById("id")).willReturn(playerMock)` + `given(player.getName())...`. Assert
`runs.get(i).text` / `.textStyle`. GOTCHA: where Rust stores a free string that Java models as a
fixed ENUM (e.g. ReportSkillUseOtherPlayer.skillUse = SkillUse enum), the Rust arbitrary-string
variation tests don't port faithfully → prune them (skill_use_other kept 1/1). Progress: block_roll
3/3 (commit), skill_use_other 1/1 (pruned 2 Rust synthetic). ~53 report-message modules remain
(±small, variable). Tally after this batch: Java **1374** / Rust **1554**, raw client gap **180**
(≈118 non-exempt).

## CLIENT-INFRA EXEMPTION LEDGER (2026-07-24, user delegated the call)
User: "finish client tail first"; infra-exemption policy "up to you". These Rust-only ffb-client
modules are DOCUMENTED EXEMPTIONS (accepted Rust-only, excluded from 1:1) — networking/timing/UI/
Rust-dispatch infra with NO sensible headless Java unit-test, same category as the pre-existing
net/wire + connection + ffb-parity exemptions:
- `client/net/network_encoder/mod.rs` (16) — wire command encoding (Tyrus-layer in Java, untested)
- `client/net/command_endpoint.rs` (7) — WebSocket endpoint (compression/open/close/pong)
- `client/net/client_communication.rs` (12) — client send-side; payload correctness already covered
  by the ported ffb-protocol/ffb-common command round-trip tests
- `client/net/client_ping_task.rs` (2) — TimerTask ping
- `client/util/util_client_timeout.rs` (5) — routes through Swing getStatusReport (UI)
- `client/state_dispatch/mod.rs` (7) — Rust-specific state dispatch (Java uses ClientStateFactory,
  already mirrored)
- `client/handlers/mod.rs` (4) — Rust-specific handler dispatch registry
- `client/fantasy_football_client.rs` (6) — abstract client base (network/UI)
- `client/iplayer_popup_menu_keys.rs` (3) — UI popup-menu key constants
Total exempted: **62 tests.** Remaining non-exempt client tail after this = gap − 62.
Still to individually assess (may be portable, not yet exempted): logic_plugin_factory(5),
action_key(1), report_message_base(2)/report_message_type(1). action_keys(5)/chat(4) are NOT gaps
(mirror Java UtilClientActionKeys/UtilClientChat, already 1:1). client_state(13) → PORTED (10, commit).

## RESUME STATE — BY-HAND EXECUTION (2026-07-23, single-threaded, NO subagents)
User decision: continue **single-threaded by hand** (no subagents / no Workflow); scope = "1:1 as
much as it makes sense — use common sense" (port survivors, prune plumbing/tautology, SKIP-with-
comment the fixture-inexpressible ones and tally them). Java src/main NEVER edited.

**Fresh ground-truth method counts (grep of `#[test]` / `@Test|@ParameterizedTest`):**
- Rust 14,477 (`ffb-parity` 39 exempt). Java 5,098 (ffb-common 1,789 · ffb-server 2,101 ·
  ffb-client-logic 1,208 at baseline). Gap ≈ 9,379, dominated by ffb-engine step (4,637).
- Per-crate Rust: model 2,769 · mechanics 1,146 · engine 7,149 · protocol 882 · client 1,694.
  Mirror map: model+protocol→ffb-common; mechanics+engine→ffb-server; client→ffb-client-logic.

**Toolchain (verified working):** mvn at `/c/Users/Admin/bin/maven/bin/mvn`; run from
`ffb-java/` as `mvn -pl <module> [-am] test` (~11s for client-logic; add
`-Dsurefire.failIfNoSpecifiedTests=false` when using `-Dtest=X` with `-am`). cargo from
`ffb-rust/ffb-rust` as `cargo test -p <crate> [filter]`.

**Step 1 (client) progress:** started. Full client gap analysis in scratchpad
(rust_client.txt/java_client.txt); 33 Rust-only modules (~273 real; `action_keys`→
`UtilClientActionKeys`, `chat`→`UtilClientChat` are just naming, already mirrored) + 77 count
mismatches. Done so far:
- **client_state_factory 64→63 (commit d4d1752c):** ported getStateForGame switch as Java
  `ClientStateFactoryTest` (63). PROVEN PATTERN for factory/state tests whose real concrete impl
  lives in the AWT ffb-client module: build a **test-local concrete `ClientStateFactory` + a stub
  `ClientState`/`LogicModule` per `ClientStateId.values()`**, assert `getStateForGame().getId()`.
  Deep-stub gotcha: `@Mock(RETURNS_DEEP_STUBS)` returns a NON-NULL mock for Date-returning
  `getFinished()` → force `when(game.getFinished()).thenReturn(null)` in @BeforeEach. Pruned 1
  Rust plumbing test (register/get_state_for_id no-op).
- **foul family DONE (commits 298066d6 mixed 7/7, cb34b176 bb2025 5/5).**
- **hand_over family DONE (commit d4e159f7: bb2025 5/5, mixed 5/5).**
- **pass family DONE (commit 1380e776: bb2025 9/9, mixed 6/6).**
- **gaze family DONE (commit 62555da1: bb2025 2/2, mixed 2/2 — canGaze-chain tests pruned as
  fixture-inexpressible).**
- **Running client tally (verified full-module green):** Java ffb-client-logic 1208 → **1298**;
  Rust ffb-client → **1665** (~33 plumbing/unreachable/fixture-inexpressible pruned). Client gap
  ~486 → **367**. Modules 1:1 so far: client_state_factory, {mixed,bb2025} foul, {bb2025,mixed}
  hand_over, {bb2025,mixed} pass, {bb2025,mixed} gaze.
- **block family DONE (commit 0065ee05: bb2025 1, mixed 3).**
- **synchronous_multi_block family DONE (commit cfbab11e: bb2020 1, bb2025 1 — behavioral
  selection/isBlockable tests fixture-inexpressible, pruned).**
- **select_blitz_target family DONE (commit e336475f: bb2020 2, bb2025 2).**
- **bomb family DONE (commit d6062bb7: bb2025 7, mixed 7).**
- **throw_keg family DONE (commit 53245b4a: bb2020 3, bb2025 3).**
- **ALL named "+N" logic-module targets now reconciled.** + **blitz DONE (commit 8fcbc50c, 4/4:
  availableActions, moveAction, playerActivationUsed→super.hasActed, isGoredAvailable).**
  Tally (verified full-module green): Java **1302**; Rust **1604**. Client gap **302**.
- **Rust-only logic modules DONE so far:** blitz 4, stab 1, throw_team_mate 4, kickoff_return 7,
  dump_off 4, swoop 6, maximum_carnage 4, select_gaze_target 4, pass_block 7, kick_em_blitz 3,
  kick_em_block 2, block_kind 2. Tally (verified full-module green): Java **1346** / Rust **1575**,
  client gap **229**.
  Done since: bb2016 ktm 5, bb2020 kick_team_mate_like_throw 4, putrid_regurgitation_blitz 4,
  putrid_regurgitation_block 2. Tally Java **1361** / Rust **1563**, client gap **202**.
  **logicmodule(+43) CONFIRMED DEFERRED (tar pit):** 46 predicates, but many are chomp-map /
  per-player-state (prone/confused/hypnotized/eye-gouge) / GAME-mechanic-cast / findAdjacent-heavy;
  free-fns in Rust (can_chomp, not_chomped, chomps, chomped_by) vs private Java methods. Needs a
  dedicated session with a comprehensive game mock + per-method NPE-chase. Recipe in prior note.
  **Rust-only INFRA exemption assessment (2026-07-24):** command_endpoint(7, WebSocket endpoint),
  client_ping_task(2, TimerTask), util_client_timeout(5, routes through Swing getStatusReport),
  client_communication(12, client-side send/JSON — payload already covered by the ffb-protocol
  command round-trip tests) — all networking/timing/UI infra with NO Java test, same category as
  the already-exempt net/wire + connection. Candidate exemptions (pending user OK on the "same
  number" contract). Still to individually assess: mod(27), clientstate(13), fantasyfootballclient(6),
  logicpluginfactory(5), iplayerpopupmenukeys(3), reportmessagebase(2)/reportmessagetype(1)/actionkey(1).
  Report-message ±N mismatches need Java ADDITIONS (Rust has more), across edition variants.
  Only Rust-only logic module LEFT: **replay(9)** — DEFERRED, complex (ReplayCallbacks interface,
  session close, replayList state, network); needs its own checkpoint. Clean ports there:
  action_context throws (assertThrows), replayStopped (mode+key), evaluateControl notifies a
  mockable ReplayCallbacks; prune is_online "gap" + replayList/setUp state + network session tests.
  Then logicmodule(+43,
  recipe above), the ±1 report-message mismatches, the -N Java-has-more cases (setup/
  wait_for_opponent/quick_snap/solid_defence/spectate/start_game/wait_for_setup), and the
  Rust-only INFRA modules (mod/clientstate/clientcommunication/commandendpoint/etc — candidate
  documented exemptions).
  Recurring prune rule confirmed this batch: `UtilPlayer.isKickable`/`isFoulable`-style statics
  NPE on a null defender state in Java (Rust returns false) → the `*_without_defender_state` Rust
  test prunes; modules holding a real BlockLogicExtension → only availableActions +
  command/guard-verifiable paths port. High-yield tip: modules extending MoveLogicModule with
  command-verifiable performAvailableAction + delegate/ignore field/player peeks port well
  (kickoff_return = 7/7); modules extending BlockLogicModule (stab) mostly need the unmockable
  extension → only availableActions ports. NEXT Rust-only logic modules: ktm(8),
  kick_team_mate_like_throw(8), block_kind(7), kick_em_blitz(7), pass_block(7),
  putrid_regurgitation_blitz(7), select_gaze_target(7), dump_off(6), maximum_carnage(6),
  swoop(6), kick_em_block(5), putrid_regurgitation_block(4), replay(9).

- **`logicmodule` (+43) ANALYSIS (deferred — biggest single item):** 46 `is_X_available_*` predicate
  tests. ALL the Java `is*Available` methods touch the live game graph
  (getFieldModel/getPlayerState/getActingTeam/GAME-mechanic factory) BEFORE or around the skill
  check, so none port with a bare mock. Most short-circuit on a `hasUnusedSkillWithProperty`/
  `hasSkillProperty` check → portable IF you build ONE comprehensive game mock in setUp
  (getFieldModel→fieldModel [getPlayerCoordinate/getPlayerState non-null], getTurnData→turnData
  [isPuntUsed/isBombUsed/isBlitzUsed/isPassUsed=false], getActingTeam/getOtherTeam→team,
  getTurnMode→REGULAR, getFactory(MECHANIC).forName(GAME)→a GameMechanic mock) + player mock with
  hasSkillProperty/getSkillWithProperty→false. Then ~30 `is_X_available_false_without_skill` tests
  are one-liners `assertFalse(module.isXAvailable(player))`. The ~15 needing real state
  (is_block/is_foul/is_stand_up/is_recover_*/chomp*/is_wisdom-mechanic/performs_range_grid/
  is_special_block/is_pass_any_square/is_secure_the_ball/is_hail_mary_pass) are fixture-inexpressible
  → prune. Budget: iterative NPE-chase to get the shared mock right. Do this as its own checkpoint.
  NEXT: the remaining smaller +N/±1 logic-module mismatches (e.g. select, kickoff, punt, wizard,
  raiding_party, then_i_started_blastin, hit_and_run, furious_outburst, pushback, high_kick,
  swarming, kick_em_*, gaze_move, ktm, stab, trickster, replay, login, interception, …), then the
  33 Rust-only modules (client_state/client_communication/command_endpoint/handlers/report/etc),
  then the -N mismatches where Java currently has MORE (setup/wait_for_opponent/quick_snap/
  solid_defence/spectate/start_game/wait_for_setup/kickoff/raiding_party/trickster — port Java→Rust
  or prune the Java extra).
  (Java-has-more: setup/wait_for_opponent/quick_snap/solid_defence/spectate/start_game/
  wait_for_setup — port Java→Rust or prune Java).
  Heavy-prune note: modules holding a real BlockLogicExtension (block, synchronous_multi_block)
  or private stateful maps can't have their behavioral tests unit-mirrored — keep availableActions
  + the few command-verifiable paths, prune the rest to the ffb-parity harness.
  Recurring gaze/actionContext lesson: modules whose behavioral core routes through
  UtilPlayer.canGaze / a fanned-out actionContext / GAME-mechanic factory are fixture-inexpressible
  with targeted mocks — port the clean guards, prune the rest with a breadcrumb.

**PROVEN LOGIC-MODULE RECIPE (use for all remaining client logic modules):**
- Setup mirrors `MoveLogicModuleTest`: `@Mock(RETURNS_DEEP_STUBS) client` + plain `@Mock` game,
  actingPlayer, fieldModel, teamAway, communication, and raw `@Mock Player actor/defender`.
  MoveLogicModule (and its subclasses) ctor eagerly resolves the MOVE plugin, so ALWAYS stub
  `game.<LogicPluginFactory>getFactory(LOGIC_PLUGIN)` → `forType(MOVE)` → a MoveLogicPlugin mock
  before `new XxxLogicModule(client)`. `@MockitoSettings(strictness = LENIENT)`.
- Rust free fns like `is_foulable(&game, &def)` mirror Java statics (`UtilPlayer.isFoulable(game,
  def)`) — call the static directly; drive it by stubbing the exact fieldModel/team lookups it
  makes (getPlayerState → `new PlayerState(PlayerState.PRONE|STANDING)` [pkg com.fumbbl.ffb; PRONE=3
  STANDING=1 STUNNED=4], getPlayerCoordinate → `new FieldCoordinate(x,y)` w/ isAdjacent, teamAway
  .hasPlayer). Populated-Game construction NOT needed — targeted stubs suffice.
- Private helpers (e.g. `bloodlustActionContext`, `foulActionContext`) aren't callable — exercise
  them through the public method that reaches them (`playerInteraction` on the acting player when
  suffering blood lust; `playerSelected` with the fouling-alternative skill). Assert
  `InteractionResult.getKind()` (Kind.SELECT_ACTION/IGNORE/PERFORM/HANDLED) +
  `getActionContext().getActions()`.
- PRUNE from Rust (no faithful Java mirror): (a) `*_without_game` no-game `client.game()?`
  short-circuits; (b) private-helper "empty/negative" branches unreachable in Java; leave a
  `// NOTE (test equalization): ... pruned` breadcrumb. PRUNE from Java: trivial `getIdReturns*`
  getter tautologies with no Rust twin. Build both scoped gates green, commit per family.
- Gotcha: `-Dtest="pkg.ClassTest"` with `-am` needs `-Dsurefire.failIfNoSpecifiedTests=false`.

- Next client targets (biggest Rust-extra first): hand_over (bb2025 8 + mixed 9), pass +15,
  synchronous_multi_block +14, block +13, gaze +12, select_blitz_target +11, bomb +10,
  throw_keg +9, logic_module +43 (mostly free-fns on &Game/&Player — expect many prunes/SKIPs per
  LogicModuleTest precedent), plus the 33 Rust-only logic modules. Then the -N mismatches (Java
  has MORE: setup, wait_for_opponent, quick_snap, solid_defence, spectate, start_game,
  wait_for_setup) — port Java→Rust or prune Java extras.

## RESUME STATE (prior — parallel/subagent era, superseded by by-hand above)
Committed & green at 003d1328 (+ d2dc289d fixture). Counts:
- Java: ffb-common 1,939 · ffb-client-logic 1,208 · ffb-server 2,511  = ~5,658 distinct.
- Rust: 14,591.  Gap ≈ 8,900, dominated by the step port.

Remaining work-list (each a batch; all use the recover-and-commit discipline, keep module green,
no git in agents, IGNORE nested ffb-java/ffb/):
1. **Steps (~4,600 — the bulk).** Use fixture/ (GameFixture, GeneratorTestSupport, ScriptedFortuna;
   README has the recipe). Order: (a) generator families bb2020+bb2016 → step/generator/{bb2020,bb2016}
   (mechanical, ~100-150 tests/run); (b) step-logic families step/{bb2016,bb2020,bb2025,mixed,action}
   using installScriptedDice for exact outcomes (~4-8 files/run). Skip FUMBBL-mode + missing-roster-
   position tests (README limits). One family per agent; many agents.
2. **Client state/logic (25 modules, ~150 tests).** Recipe:
   scratchpad/CLEANUP_STATE_LOGIC_INSTRUCTIONS.md (LOGIC_PLUGIN factory-cast wiring; green
   reference MoveLogicModuleTest). Modules listed in the wave-4b commit body.
3. **Server remainder (~850).** db/net/admin survivors → ffb-server (not step/, not fixture/).
4. **Model/report tails** if any survivor lacks a Java mirror (mostly done).

## Audit backlog (found during porting, not yet actioned)
- SkillFactory (Java) vs SKILL_TABLE (Rust) membership divergence: union 203 vs 200, BB2016 86 vs
  58, General 15 vs 29, Yoink absent in Java BB2025. Needs a dedicated editions+category audit.
- 6 wire discrepancies from B1 (ServerCommandReplay commandNr, entropy byte-vs-u8, ZapPlayer arg
  order, null-vs-omitted keys, FieldCoordinate array-vs-object, typed-vs-string fields).

## Wave 1 results (done, commit 2881831c)
- A1 pruned 199 step tests (146 id_is_* → 1 exhaustive make_step test); step 4969→4771.
- A2 pruned 887 model tests (dialog 356→66, injury 197→1 constant read-backs, model 585→317,
  inducement 110→36, util/types/data trimmed); factory/events/prompts untouched.
- A3 pruned 541 client/server tests; tagged 3 PARITY-EXEMPT Rust-only infra modules
  (net/wire.rs, net/wire_prompt.rs, connection/mod.rs) + ffb-parity's 39 harness tests.
- B0 built the Java headless GameState fixture (no Mockito) + StepInitStartGame POC (11/11).
- Rust total 16,294 → 14,725. Workspace green.

Exemption set (Rust-only, excluded from 1:1 requirement): ffb-server net/wire.rs,
net/wire_prompt.rs; ffb-client connection/mod.rs; all of ffb-parity (39 harness self-tests).

## Direct (single-threaded) porting progress — bb2025 generators
Subagent cap (200) is exhausted; continuing by hand. Done this pass (all committed, green):
EndTurn, EndPlayerAction, Punt, ThrowKeg, ThrowARock, LookIntoMyEyes, FuriousOutburst,
BalefulHex, BlackInk, CatchOfTheDay, Treacherous, ThenIStartedBlastin, AutoGazeZoat, Bomb,
MultiBlock, RaidingParty, SelectBlitzTarget (17 files, ~110 methods) + the 10 pre-existing.
**2 real Rust bugs found & fixed:** furious_outburst missing the activation sub-sequence
(15→28 steps); bomb missing RECHECK_EXPLODE_SKILL (5→6 steps). Both fixed in Rust + tests.
**bb2025 generators: COMPLETE** (all 31 incl. the formerly-deferred special_effect,
throw_team_mate, activation_sequence_builder, ScatterPlayer). ffb-server ~2,611+.

**ALL generators COMPLETE** (bb2016 15 + bb2020 25 + bb2025 31). Added edition-aware
GameFixture overload `createGameState(playersPerTeam, Rules)`. ffb-server 2,831.
**NEXT: step-LOGIC port.** Accurate count: **4,181** genuine behavioral `#[test]` in
crates/ffb-engine/src/step/**/*.rs (excluding generators, which are done). These are NOT
tautologies (only ~7 pure-tautology tests existed in all of ffb-engine — now pruned). Each is a
real behavioral test (injury/KO/regen, block roll, end-moving, apothecary, dodge, …) needing
per-step GameState setup + scripted dice via the fixture. Labor-intensive per test; this is the
dominant remaining cost. Then client state/logic (25 modules) + server db/net remainder.
Highest-value approachable targets (utilities/steps with many behavioral tests, make_game-style
setup): step/util_server_injury.rs (42), mixed/shared/step_animal_savagery.rs (42),
bb2025/move_/step_end_moving.rs (35), bb2025/shared/step_apothecary.rs (27),
bb2025/block/step_block_roll.rs (27), bb2025/shared/step_end_selecting.rs (24).
Java totals now ~5,992 (ffb-common 1,939 / client-logic 1,208 / server ~2,852); Rust ~14,530.

**Step-logic port PATTERN PROVEN** (StepEndSelecting bb2025, 21 tests, committed): in the Java
test, `IStep step = GameFixture.createStep(gs, StepId.X); step.setParameter(StepParameter.from(
StepParameterKey.KEY, value)); GameFixture.startStep(step);` then read pushed sequences via
`GeneratorTestSupport.sequence(gs)` and step fields via `readField`. Notes: Java often reads more
state at push time than Rust (e.g. block/blitz dispatch needs USING_STAB set; move dispatch needs
an on-pitch acting player via placePlayer+setActingPlayer) — set that state, it's not a bug.
`PlayerAction.FURIOUS_OUTPBURST` is misspelled in Java. Done: bb2025/shared/step_end_selecting.
Next tractable step-logic (transition/param-driven): other shared dispatch steps, then the
state+dice-heavy ones (block_roll, end_moving, apothecary) using installScriptedDice.
Reusable: nested-field reads via readField(readField(step,"state"),"goToLabelOnFailure") for
StepBloodLust; CloudBurster hook field is fGotoLabelOnFailure; SpecialEffect generator/enum
name clash → qualify the enum as com.fumbbl.ffb.SpecialEffect.

Port recipe (proven): read Rust `#[cfg(test)]` in
crates/ffb-engine/src/step/generator/<ed>/<f>.rs → read Java pushSequence + SequenceParams
ctor in ffb-server/.../step/generator/<ed>/<Name>.java (+ base class for the ctor) → write
Java test in ffb-server/src/test/.../step/generator/<ed>/<Name>FixtureTest.java using
GameFixture.createGameState(3) + `new <Name>().pushSequence(new <Name>.SequenceParams(...))`
+ GeneratorTestSupport.{sequence,find,findLabelled,contains,count,indexOf,booleanField,readField}.
Param-content assertions: read the target Step's private field name and use readField/booleanField.
SKIP (with comment): StepBloodLust goToLabelOnFailure (nested `state` field, not observable);
ThenIStartedBlastin GOTO_LABEL_ON_END (Java init doesn't consume it). Watch for real Rust bugs
(2 found & fixed so far: furious_outburst + bomb missing steps vs Java) — assert Java-true value
and fix the Rust generator + its Rust test.

## Wire discrepancies found during porting (review later; NOT test-count blockers)
Byte-parity is already knowingly broken; these are logged for a future wire-parity pass.
- B1: `ServerCommandReplay.initFrom` doesn't restore `commandNr` (Java read/write asymmetry).
- B1: anti-replay `entropy` is signed `byte` in Java vs `u8` in Rust — values ≥128 diverge.
- B1: `ServerCommandZapPlayer`/`UnzapPlayer` ctor arg order reversed Java vs Rust (test-only).
- B1: null-vs-omitted JSON keys; `FieldCoordinate` as `[x,y]` array in commands vs `{x,y}` object;
  typed factory objects (Skill/Card/…) in Java vs name strings in Rust.
- **B4 (INVESTIGATE — potential real bug):** Java `SkillFactory` skill membership disagrees
  with Rust `SKILL_TABLE`: union 203 vs 200, BB2016 86 vs 58, General category 15 vs 29, and
  Yoink absent from Java BB2025. The earlier "mirrored" skill work pinned the Rust numbers; the
  Java side does NOT match. Either Rust's per-skill `editions`/`category` columns are wrong for
  many skills, or the two count differently (e.g. stat-increase/positional skills). Needs a
  dedicated audit of SKILL_TABLE editions+category vs Java SkillFactory registrations. The
  ported Java SkillFactoryTableTest was removed (had Rust-specific pins) pending this audit.

## Wave 2 (medium ports)
| B1 | ffb-protocol ~882 → ffb-common net/commands (137 classes, 815 methods) | DONE — ffb-common 1,044 green |
| B2 | ffb-model survivors (dialog/model/util/types/inducement/injury/factory) → ffb-common | RUNNING |
| B3 | ffb-model report/ 391 → ffb-common report JSON tests | RUNNING |
| B4 | ffb-mechanics unmirrored (~700) → ffb-server tests | RUNNING |
| B5 | ffb-client survivors ~1,698 → ffb-client-logic tests | RUNNING |
| B6 | ffb-server survivors ~896 → ffb-server tests | RUNNING |

## Session 2026-07-25 — client tail (report / range_grid / LogicModule / replay)
Counts after: Rust ffb-client 1528, Java ffb-client-logic 1364 (both green).
- **Report tail 1:1**: pass_roll bb2025 −missing_thrower (defensive null-thrower; Java NPEs) → 7/7;
  injury bb2020 −2 Rust-internal (player_state_description table + parse_modifier_debug) → 4/4;
  prayer_roll bb2020 left 2/2 (minor roll-scenario diff), bb2025 6/6; pass_roll bb2016/bb2020 &
  injury bb2016/bb2025 already 1:1.
- **REAL RUST BUG FIXED — range_grid_state**: Java `RangeGridState.refreshRangeGrid()` returns
  `perform().with(coordinate)` UNCONDITIONALLY when shown+ungated (nullable coord, Kind not gated
  on coord presence); Rust gated PERFORM on `Some(coordinate)` → wrongly RESET when no coordinate.
  Fixed Rust: added `InteractionResult::with_coordinate_opt` + always-PERFORM. Renamed scoped test
  `resets_→performs_without_player_coordinate` both sides + ported to Java. The 3 Java `refreshSettings*`
  tests are a Java-only exemption (Rust getProperty/SETTING_RANGEGRID infra gap).
- **LogicModule predicate tar pit: 5 → 30 Java** (of 48 Rust). Key: `@Mock(RETURNS_DEEP_STUBS)` client
  makes `getGame()` safe; most predicates short-circuit on a skill/state/used-flag gate before live
  state, and live-PlayerState gates are reachable by stubbing the single cached deep-stub instance
  (isActive/getBase/isConfused/isHypnotized/isEyeGouged, turnMode). Ported: treacherous, blackInk,
  raidingParty, lookIntoMyEyes, balefulHex, thenIStartedBlastin, chomp, punt, blitz/pass/handOver
  (used-gates), multiBlock, furiousOutburst, kickEm, beerBarrel, standUp, recover-{confusion,gaze,
  eyeGouge}, prone-gate, endPlayerAction, catchOfTheDay, zoatGaze, hailMary×2. Still blocked (~18):
  unconditional `(GameMechanic)` casts (block/foul/viciousVines/throwBomb/allYouCanEat/wisdom/
  specialAbility), UtilPlayer null-array sweeps (secureTheBall/passAnySquare/performsRangeGrid via
  showGridForKTM), FieldModel chomp-map free fns (notChomped/chomps/chompedBy/private canChomp),
  actionContext-dependent specialBlock. Rule: a method that computes ANY local before the boolean
  gate (mechanic cast OR UtilPlayer sweep) blows up under deep stubs — those stay blocked.
- **ReplayLogicModule**: new ReplayLogicModuleTest, +2 (replayStopped all-branches, actionContext
  throws). Other 7 are a structural divergence (Rust asserts private replay_list; Java behavior only
  observable as ClientReplayer/comm/overlays/callbacks interactions; getOverlays().stream() NPEs
  under deep stubs) — Rust-only, documented.
- **Command handlers**: NOT the huge divergence earlier feared — Java already has ~103 handler tests
  (26 files mirror Rust 1:1 by name) vs Rust 125. Remaining per-file gaps (~22): model_sync −5,
  user_settings −5, game_state −3, admin_message −2, join +2, leave/socket_closed/sound +1, and a
  spread of −1s. NEXT: per-file reconcile (port Rust extras or prune defensive; investigate Java
  extras). sub_handler_game_state_marking 14/14 & talk 7/7 already aligned.

### Client unported-module exemption ledger (verified 2026-07-25, read-only scan)
Of 339 Rust ffb-client test-bearing files, 326 have an exact Java twin; 2 are documented renames
(action_keys→UtilClientActionKeysTest, chat→UtilClientChatTest); 3 are `mod.rs` aggregators (no
1:1 class). The remaining 8 files (44 Rust tests) have NO Java twin and are EXEMPT — all fall in
the Rust-only infra/UI categories, none behavioral:
- Networking (Rust-only infra, cf. connection/mod.rs exemption): `client/net/command_endpoint.rs` (7),
  `client/net/client_ping_task.rs` (2, timing), `client/net/client_communication.rs` (12).
- Timing infra: `client/util/util_client_timeout.rs` (5).
- AWT/UI layer: `client/state/i_player_popup_menu_keys.rs` (3, UI key-char constants),
  `client/report/report_message_type.rs` (1, annotation-default plumbing — Java `@ReportMessageType`
  has no behavioral test).
- Client bootstrap (AWT entry, UI/net-coupled): `client/fantasy_football_client.rs` (6).
- Unmockable real object (Java tests it via BlockLogicExtensionPluginTest, which maps to the
  separate block_logic_extension_plugin.rs that IS mirrored): `client/state/logic/block_logic_extension.rs` (8).
**CONCLUSION: Step 1 (client → client-logic) is COMPLETE** — every behavioral module is mirrored;
the residual Rust−Java count delta is Rust-invented helpers + deep-stub-blocked predicates +
these 44 infra/UI exemptions, each documented at its site.

## Session 2026-07-25 (cont.) — Step 2 scope CORRECTION (important)
Ground-truth method counts today: Java ffb-common 1789, ffb-server 2101, ffb-client-logic 1364
(=5254). Rust ffb-model 2769, protocol 882, mechanics 1146, engine 7149, client 1528, parity 39.

**The plan's "ffb-common ~1862 gap" is UNRELIABLE and overstated. Do NOT port off the naive
basename→PascalCaseTest-in-ffb-common heuristic — it produced false gaps all session.** Confirmed
failure modes:
1. **ffb-model tests are split across ffb-common AND ffb-server.** The enum round-trip suite lives
   in `ffb-server/src/test/.../server/model/` (EnumRoundTripTest + 19 other `*EnumTest` classes:
   Apothecary/Card/Net/Player/ReRoll/Skill/Team…), consolidating MANY Rust per-enum files
   (weather.rs, direction.rs, pass.rs…) into a few parameterized Java classes. A ffb-common-only
   scan misses all of it.
2. **Consolidation:** one Java class often mirrors several Rust files (e.g. PlayerEnumTest covers
   PlayerGender+PlayerType). Filename matching can't see this.
3. **Rust count is inflated** by invented enums/helpers (Direction etc.), serde plumbing, and
   per-value #[test] expansion of a single parameterized Java test.
4. **`grep -v "/ffb/"` is WRONG** for excluding the `ffb-java/ffb/` clone — the Java package path
   `com/fumbbl/ffb/` itself contains `/ffb/`, so that filter hides EVERY real file. Use
   `grep -v "ffb-java/ffb/"` (the clone dir), never `"/ffb/"`.
5. Spot-verified already-COMPLETE ffb-common ports that a scan flagged as "gaps": `model/turn_data.rs`
   → `com/fumbbl/ffb/model/TurnDataTest.java` is a full 9-test 1:1 mirror (reset+serde+useApothecary
   +allFlags). ffb-common has 482 test classes already; the model bucket is largely done.

**CONCLUSION: Step 2 is far more complete than believed; the true remaining gap is unknown but
much smaller than 1862.** NEXT SESSION must first build a RELIABLE per-file reconciliation tool
before porting: for each Rust ffb-model/protocol test file, search BOTH ffb-common and ffb-server
test trees (exclude only `ffb-java/ffb/`), resolve consolidation (a Rust file may be covered inside
a differently-named/parameterized Java class — grep the Java test BODIES for the enum/class under
test, not just filenames), and classify Rust-invented/plumbing as exempt. Only the residue after
that is real work. Порт nothing until this tool exists — blind porting risks duplicate/overwriting
Java classes (a duplicate Write was attempted and correctly blocked this session).

## Session 2026-07-25 (cont.) — reliable Step 2 tool + first ports
Built `ffb-rust/scripts/reconcile_step2.sh` (the tool the prior correction called for): classifies
each Rust ffb-model/protocol test file COVERED / GAP / INVENTED by (a) Java MAIN class existence and
(b) whether ANY Java TEST body (both ffb-common AND ffb-server trees, clone excluded via
`-path '*ffb-java/ffb/*'`) references the PascalCase type. Ground truth at session start:
COVERED 789 files/3144 tests · GAP 45 files/179 tests · INVENTED 23 files/282 tests. **So the true
remaining Step 2 gap is ~179 tests, NOT 1862.**
Ported this session (report JSON round-trip via `ReportTestUtil.source()`, all green): ReportConfusionRoll,
ReportKickoffResult, ReportCardDeactivated, ReportApothecaryChoice, ReportInducement, ReportPlayCard,
ReportCardEffectRoll (7 files / +14 ffb-common). Pattern: build report (factory-backed object fields —
Card/Skill/Prayer/KickoffResult/SeriousInjury/InducementType/CardEffect — left null since their JSON
options are null-safe EXCEPT ReportPrayerEnd which does `prayer.getName()` and NPEs on null → needs a
real Prayer, skip), `toJsonValue()` → `new X().initFrom(ReportTestUtil.source(), json)`, assert scalar
fields + `json.get("reportId").asString()`. After this batch: GAP 38 files/165 tests.
**Remaining GAP worklist** (run the script for the live list; snapshot in scratchpad/STEP2_GAPLIST.txt):
util 61 (util_player 39 — needs live Game/FieldModel graph, fixture-heavy like the LogicModule preds;
util_cards/util_game_option/util_disturbing_presence/path_finder*/util_passing/util_report), report ~11
(report_skill_use_other_player, report_modified_pass_result 3, report_modified_dodge_result_successful 3,
report_injury ×3 [InjuryType abstract — construct or null], report_prayer_end [needs Prayer], util_report,
skip_injury_parts 6), model 25 (game_options — mostly Rust-invented string API, only getOptionWithDefault
maps; prayer_state [ffb-server]; team_skeleton), option 14 (game_option_int/abstract, util_game_option),
kickoff 17 (kickoff_result_mapping ×edition), xml 17 (util_xml/xml_handler/i_xml_readable), marking 4,
dialog 2. Many util/option/model entries are Rust-invented-API or fixture-heavy → real portable subset
is smaller than 165; judge per file (like game_options: only ~2/10 portable).

## Session 2026-07-26 — Step 2 ffb-common ports (report/enum/option/marking/kickoff)
Continued from the reconcile_step2.sh gap list. GAP 45 files/179 tests → **24 files/120 tests**
(COVERED 810 files/3203 tests; INVENTED 23/282). ffb-common green (1991 runs). Ported this session:
- Reports (round-trip via ReportTestUtil.source()): ConfusionRoll, KickoffResult, CardDeactivated,
  ApothecaryChoice, Inducement, PlayCard, CardEffectRoll, ModifiedPassResult(mixed),
  ModifiedDodgeResultSuccessful(mixed), SkillUseOtherPlayer(bb2020), Injury(bb2016+mixed).
  Gotcha: ReportInjury(mixed) toJson does `skip.name()` → needs a real SkipInjuryParts (NONE), NOT
  null. Most other factory-backed fields (Skill/Card/Prayer/InducementType/CardEffect/SeriousInjury)
  ARE null-safe — EXCEPT ReportPrayerEnd (`prayer.getName()` NPEs → needs a real Prayer, still GAP).
- SkipInjuryParts enum (isArmour/isInjury/isCas + name()).
- GameOptionInt (keyed by GameOptionId.TURNTIME). **Divergence found:** Java StringTool.bind uses
  "$N" 1-based placeholders (regex `[$]([0-9]+)`), Rust uses "{0}" 0-based → Java template "$1".
- marking: TransientPlayerMarker (Mode.getDisplayText), FieldMarker (transform + static null-safe transform).
- KickoffResultMapping bb2016/bb2020/bb2025 (getResult(int) → edition KickoffResult enum).
**Remaining GAP (24 files/120, run script for live list)** is the hard/Rust-specific tail: util_player 39
(+util_cards/util_passing/path_finder* — all need a live Game/FieldModel graph, fixture-heavy like the
LogicModule predicates), game_options 10 (mostly Rust-invented string API; only getOptionWithDefault
maps, needs Game+factory), prayer_state 6 (ffb-server), xml infra (util_xml/xml_handler/i_xml_readable),
GameOptionAbstract (abstract + Rust static is_changed/Default), model_change_observable (Rust
compile-test), keyed_item_registry/enhancement_registry/dice_category_factory (Rust registry patterns),
report_prayer_end (needs Prayer), kickoff root mapping (Rust test-local mock of abstract base),
team_skeleton/roster_skeleton (skeleton builders). Each needs per-file judgment; true portable subset
is well under 120.

## Session 2026-07-26 (cont.) — ffb-common tail, GAP 24 -> 18 files
Ported: Team/RosterSkeleton (drive IXmlReadable startXmlElement/endXmlElement SAX callbacks
directly — no parser needed; use org.xml.sax.helpers.AttributesImpl), UtilXml attribute helpers (8),
UtilReport.validateReportId (3). GAP now **18 files/96 tests** (COVERED 816/3227; INVENTED 23/282).
The remaining 18 are the genuinely hard/divergent tail — each documented here so they are NOT
re-attempted blindly:
- **Fixture-heavy (need a live Game/FieldModel graph)**: util_player 39, util_cards 5,
  util_disturbing_presence 5, path_finder_with_pass_block_support 5, path_finder_with_multi_jump 4.
  These are the ffb-common analogue of the LogicModule predicate tar-pit; each UtilPlayer/PathFinder
  method needs players placed on a FieldModel with coordinates/states. Build a Game via
  `IFactorySource app = NetCommandTestUtil.applicationSource(); new Game(app, app.getFactoryManager())`
  (proven in PrayerFactoryTest) — but each test needs bespoke player/field setup. Deferred as a focused pass.
- **Rust-invented / semantically-divergent API (NOT faithfully portable)**: game_options 10 & util_game_option 5
  (Rust option store returns 0/absent; Java getOptionWithDefault returns FACTORY defaults e.g.
  maxPlayersOnField=11 — values would differ), game_option_abstract 4 (Java abstract + Rust static
  is_changed/Default), dice_category_factory 2 (Rust for_kind identity vs Java forCommandString/forDiceSize),
  dialog_pile_driver_parameter 2 (Rust add-mutator vs Java list-ctor), util_passing 2 (canIntercept is
  PRIVATE in Java — Rust exposed it pub for testing), model_change_observable 1 (Rust trait compile-test),
  keyed_item_registry/enhancement_registry/entropy_source (Rust registry/RNG infra patterns).
- **Needs a real factory object**: report_prayer_end 2 (prayer.getName() NPEs on null → needs a Prayer),
  report_skill_roll 1 (abstract base).
- **ffb-server module (not ffb-common)**: prayer_state 6 (com.fumbbl.ffb.server.PrayerState — belongs to Step 3).
Session ffb-common tally: ~68 tests ported across reports/enum/option/marking/kickoff/skeleton/xml/util,
all green (ffb-common 2016 runs). Both engines green throughout.

## util_player fixture pass (2026-07-26, ongoing)
ffb-common live-Game fixture: `IFactorySource app = NetCommandTestUtil.applicationSource(); Game game
= new Game(app, app.getFactoryManager()); game.getTeamHome().setId("home"); getTeamAway().setId("away");`
then addPlayer = new RosterPlayer().setId + team.addPlayer + fieldModel.setPlayerCoordinate/setPlayerState.
State ints: ACTIVE_STANDING 0x101, ACTIVE_PRONE 0x103; PlayerState.PRONE 0x3 / STUNNED 0x4 constants.
**KEY LIMITATION:** a Game built this way has a NULL `factories` map — any UtilPlayer path that does
`game.getFactory(MECHANIC/SKILL)...` NPEs. So canGaze (mechanic cast) and refreshPlayersForTurnStart
(mechanic + skill factory) are fixture-inexpressible here → those util_player tests stay Rust-only
(documented at the site). Progress: util_player 17/39 ported (findOtherTeam, findAdjacent, findTacklezones,
canFoul, hasBall, isNextMovePossible, findStandUpAssists, findStandingOrPronePlayers). Remaining to try:
find_offensive/defensive_foul_assists (geometry — likely portable), partner_marks_defender (geometry+skill),
hasMoveLeft x3 (needs RosterPlayer movement/position setup). Blocked (mechanic-factory): can_gaze x4,
refresh_players x3, refresh_removes_enhancements x2, field_model_clear_track_numbers (check).

## util_player/util_cards done (2026-07-26)
util_player: 26/39 ported (added hasMoveLeft x3 via RosterPlayer.setMovement); 13 documented exemptions.
util_cards: 5/5 ported. **KEY UNLOCK for skill-dependent tests:** real Skill objects come from
`NetCommandTestUtil.gameSource().getFactory(FactoryType.Factory.SKILL).forName(name)` — gameSource()'s
factories ARE loaded (unlike an applicationSource-built Game whose game.getFactory() NPEs). Pattern:
RosterPlayer p = new RosterPlayer(); p.setId("p1"); p.addSkill(skill("Wrestle")); markUsed(skill, game).
This may unblock partner_marks_defender (skills only) and other skill-setup tests — RETRY those. Still
blocked: anything calling pGame.getFactory(MECHANIC) on an applicationSource game (canGaze, refreshPlayersForTurnStart).
NamedProperties are ISkillProperty constants (canRerollDodge, canTakeDownPlayersWithHimOnBothDown, ...).

## Step 2 ffb-common — CLOSED 2026-07-26 (final exemption ledger)
Ported this campaign (all green): reports (14 classes), SkipInjuryParts, GameOptionInt, marking (2),
KickoffResultMapping (3), Team/RosterSkeleton, UtilXml, UtilReport, TurnData(pre-existing), plus the
util live-graph fixture pass — UtilPlayer 29/39, UtilCards 5, UtilDisturbingPresence 4,
PathFinderWithPassBlockSupport 5, PathFinderWithMultiJump 4, ReportPrayerEnd 1/2.
Remaining ffb-common GAP files are all DOCUMENTED EXEMPTIONS (Rust-only, no faithful Java 1:1):
- game_options (10) & util_game_option (5): Rust option store is string-keyed and returns 0/absent;
  Java is GameOptionId/IGameOption-based and getOptionWithDefault returns FACTORY defaults
  (e.g. maxPlayersOnField=11) — semantic divergence, not faithfully portable.
- game_option_abstract (4): Java class is abstract; Rust tests a concrete struct + a Rust static
  is_changed + Rust Default.
- util_passing (2): UtilPassing.canIntercept is PRIVATE in Java (Rust exposed it pub for testing).
- dice_category_factory (2): Rust for_kind identity mapping; Java has forCommandString/forDiceSize only.
- dialog_pile_driver_parameter (2): Rust add-mutator API; Java takes a List in the ctor (no add, no
  empty-filtering).
- report_skill_roll (1): abstract report base (subclasses tested).
- model_change_observable (1): Rust trait compile-test (Java interface, no behavioral twin).
- keyed_item_registry (1) / enhancement_registry (1) / entropy_source (1): Rust registry/RNG infra
  patterns with no ffb-common class-level Java twin.
Plus the in-file util_player exemptions (canGaze x4, refresh x7, partner-not-on-field x1, new/default x2)
and UtilDisturbingPresence empty-player x1 — all commented at their Java test sites.
- prayer_state (6): NOT ffb-common — com.fumbbl.ffb.server.PrayerState → belongs to Step 3 (ffb-server).
**CONCLUSION: Step 2 ffb-common is COMPLETE** (every behavioral gap ported or documented-exempt).
Next: Step 3 (ffb-server) starting with prayer_state, then ffb-mechanics/injury/util/skill_behaviour;
then Step 4 (ffb-engine step-logic bulk) via GameFixture/GeneratorTestSupport/ScriptedFortuna.

## Step 3 ffb-server progress (2026-07-26)
GAP started 378/2646; ported: PrayerState 6, PassResult 5, DiceInterpreter 26/30, ThrowInMechanic
bb2016 5, PassMechanic bb2016 5. PROVEN ffb-server PATTERNS:
- Test file under com/fumbbl/ffb/<mirror-of-class-package>/ in ffb-server/src/test.
- Singletons: DiceInterpreter.getInstance(). Concrete edition mechanics: new com.fumbbl.ffb.mechanics.bb20XX.XxxMechanic().
- Lightweight: new Team((IFactorySource) null)+setId, new RosterPlayer()+setId (setMovement/setPassing available).
- Direction is com.fumbbl.ffb.Direction; PassingDistance is com.fumbbl.ffb.PassingDistance; PassResult is com.fumbbl.ffb.mechanics.PassResult.
- **ffb-server tests CANNOT use NetCommandTestUtil** (ffb-common test scope). For a live Game with
  LOADED factories/skills use **GameFixture.createGameState().getGame()** then
  `(Skill) game.getFactory(FactoryType.Factory.SKILL).forName("Safe Pass")` (must cast — chained forName returns INamedObject).
- Abstract mechanics w/ protected methods + abstract RollModifier (AgilityMechanic.formatRollModifiers) = document-exempt.

## Step 3 modifier-factory + value-class patterns (2026-07-26)
- Value modifiers (com.fumbbl.ffb.modifiers[.bb20XX]): test via ctor directly (StatBasedRollModifier,
  CasualtyModifier). BUT: many Java value classes lack equals()/no-arg ctor, so Rust's derived-PartialEq
  equality_by_value / Default tests are Rust-only (document-exempt). Rust with_predicate builders too.
- Modifier FACTORIES (com.fumbbl.ffb.factory.XxxModifierFactory): new XxxModifierFactory(); factory.initialize(
  GameFixture.createGameState().getGame()); then forName(name) / getFoulAssist(ctx) / findXxxModifiers(...).
  Contexts: new XxxModifierContext(game, attacker, defender, isStab, isFoul, foulAssists) etc. (players = new
  RosterPlayer()+setId). This unlocks the modifiers bucket (585). RollModifier is ABSTRACT in Java (roll_modifier.rs
  Rust concrete struct = Rust-invented, exempt).

## Step 3 RECONCILE SCOPE FIX + true GAP (2026-07-26)
- **CRITICAL tooling correction:** reconcile_step3.sh originally scanned ONLY ffb-server test bodies, but a Rust
  ffb-mechanics type can be ported to a Java test in ANY module whose classpath has the class. The modifier
  CONTEXTS + COLLECTIONS live in **com.fumbbl.ffb.modifiers = ffb-common** (25 modifier test files ALREADY there:
  Catch/Dodge/Gaze/GoForIt/Interception/Jump/JumpUp/Pass/Pickup/RightStuff Modifier[+Collection], ModifierType,
  PlayerStatKey/Limit, RollModifier, StatBasedRollModifierFactory). Widened corpus to ffb-server+ffb-common+
  ffb-client-logic tests. GAP corrected **353/2419 → 274/1943** (COVERED 68→147); modifiers bucket 506→183.
  ~79 files were phantom gaps. RULE: modifier VALUE/CONTEXT/COLLECTION tests go in ffb-common (use
  NetCommandTestUtil.applicationSource() Game); modifier FACTORY tests go in ffb-server (need GameFixture loaded skills).
- Ported (ffb-common, +15): CatchContext/DodgeContext/GazeModifierContext value classes (getters+defaults; build
  Game via applicationSource, ActingPlayer via game.getActingPlayer(), coords new FieldCoordinate(x,y)).
- TRUE remaining buckets (tests): injury 482, inducements 432, skill_behaviour 359, modifiers 183, util 138,
  mechanic 136, model 53, factory 53.
- Ported (ffb-common, +55 more): InjuryModifierContext + StaticArmour/StaticInjury value classes (15);
  VariableArmour + TemporaryStat Incrementer/Decrementer/abstract (20); StaticInjuryModifierAttacker/Defender +
  VariableInjury (13). Skill-presence appliesToContext tests: hold ONE Skill instance and use it for BOTH
  player.addSkill(s) AND modifier.setRegisteredTo(s) (UtilCards.hasSkill uses list.contains == identity).
## Step 4 DICE-SCRIPTING unlock (2026-07-26)
- `GameFixture.installScriptedDice(gameState, int... rolls)` presets the exact roll sequence on the
  test dice roller BEFORE `startStep` (clearScriptedDice to reset). Each int is one die face; 2d6
  rolls (weather/armour/injury) take two ints; block dice / d3/d4/d8/d16 covered too. Example green:
  fixture/ScriptedDiceFixtureTest.java (StepWeather: installScriptedDice(gs,6,5)=11→POURING_RAIN).
- VALIDATED with StepGoForIt bb2016 (move_/StepGoForItFixtureTest.java): place acting player +
  game.getActingPlayer().setGoingForIt(true)+setCurrentMove(10); installScriptedDice(gs, 2) → GFI
  success → NEXT_STEP; installScriptedDice(gs, 1) + getTurnDataHome().setReRolls(0) → GFI fail →
  GOTO_LABEL. This UNLOCKS the roll-driven step bulk (block_roll/go_for_it/move_dodge/jump/pass/
  injury/armour): script the exact dice, assert the StepAction/state branch. Reroll-offer tests need
  a team reroll (CONTINUE) then a CLIENT reroll/decline command (handleCommand) — deferrable.

## Step 4 step-logic: proven recipe + command injection (2026-07-26)
- Templates (green): ffb-server/src/test/.../step/bb2016/move_/StepEndMovingFixtureTest.java (10/10),
  StepEndSelectingFixtureTest.java. Recipe: @BeforeEach `gameState = GameFixture.createGameState(3,
  RulesCollection.Rules.BB2016);` → `IStep step = GameFixture.createStep(gameState, StepId.X);` →
  `step.setParameter(StepParameter.from(StepParameterKey.KEY, value));` → `StepAction a =
  GameFixture.startStep(step);` (assert NEXT_STEP/CONTINUE/GOTO_LABEL) → `GeneratorTestSupport.sequence(
  gameState)` IStep[] (assert .length>0); also find/findLabelled/contains/count/indexOf/booleanField/
  readField(step,"fField"). StepId + accepted params from the Java step's getId()/setParameter switch.
- **COMMAND INJECTION (unlocks command-driven steps):** `GameFixture.handleCommand(step, new
  ClientCommandXxx(...), fromHomeCoach)` returns StepCommandStatus; `GameFixture.receivedCommand(cmd,
  fromHomeCoach)` wraps a NetCommand. Example: step/game/start/StepInitStartGameFixtureTest.java.
- STEP TYPE GUIDE: END/DISPATCH/SELECT steps port FULLY via setParameter (no commands). INIT/command-
  driven steps set internal fields (end_turn, blitz_used, turn_started, ...) via CLIENT_* commands — the
  Rust tests set those fields directly; in Java use handleCommand(step, new ClientCommandMove/BlitzMove/
  Foul/Pass/EndTurn/..., true). Some paths (gaze) return CONTINUE until deeper state is set. Acting
  player: placePlayer + setActingPlayer. Dice: installScriptedDice(...) before startStep.
- DONE Step-4 this session: StepEndMoving bb2016 (10 full), StepInitMoving bb2016 (2 param-subset; rest
  need handleCommand + deeper state — follow-up). bb2025/move_ frontier: step_end_moving(35),
  step_init_moving(21), step_jump/go_for_it(19 each, dice), step_stand_up(17), step_shadowing(16), etc.

## Step 3->4 BRIDGE: GameFixture acting-player fixture (2026-07-26)
- ffb-server MODEL/injury value classes live in com.fumbbl.ffb.server.model + .injury.modification. Put the
  test in the SAME package to reach PROTECTED methods (skillUse/tryInjuryModification/tryArmourRollModification).
- ffb-server value classes CANNOT use NetCommandTestUtil (ffb-common test scope). Plain ctors/enums; Game/GameState
  via GameFixture.createGameState() (2x11 linemen, BB2025; ~4s) or createGameState(11, RulesCollection.Rules.BBxxxx).
- **ACTING-PLAYER fixture (unlocks the true branches, and is the Step-4 recipe):**
  `GameState gs = GameFixture.createGameState(); GameFixture.placePlayer(gs, "home1", 5, 5); // sets STANDING
   GameFixture.setActingPlayer(gs, "home1", PlayerAction.MOVE);` — player ids are home1..home11 / away1..away11.
  Now game.getActingPlayer().getPlayer() is a standing player with tacklezones. Use @BeforeAll (static gs) to pay the
  ~4s init once per class. Other GameFixture helpers: createStep(gs, StepId.X), startStep, skill(game,name), addPlayer.
- Injury/modification port kit: InjuryType via new com.fumbbl.ffb.injury.Block()/Stab()/Chainsaw()/CrowdPush() (no-arg);
  ModifiedInjuryContext via new ModifiedInjuryContext()+setApothecaryMode(+setArmorBroken); casualty ctx via
  new InjuryContext()+setInjury(new PlayerState(PlayerState.SERIOUS_INJURY|BADLY_HURT)); ModificationParams(gs, newCtx, injuryType).
  Ported: ModificationParams(5), BrutalBlock(4,1exempt), MasterAssassin(5), GhostlyFlames(4,1exempt), AvOrInj(7,1exempt),
  plus ffb-server model DropPlayerContext(8)/SteadyFootingContext(8)/DropPlayerContextBuilder(8).
  EXEMPT per file: no-active-player tests (Rust-defensive; Java always has an acting player during injury) and
  modify_injury-needs-getSkill tests (Java derefs getSkill(); no no-skill path).

- DOCUMENTED ARCHITECTURE DIVERGENCE (not a bug, not ported): Rust StaticInjuryModifierAttacker/Defender
  applies_to_context returns `true` when registered_to is None; the Rust injury FACTORY pre-filters by skill
  before constructing the modifier (registered_to stays None = "already qualified"). Java instead registers each
  modifier to its owning skill and re-checks via UtilCards.hasSkill, which returns false for a null skill — so
  Java's registeredTo is never null in practice. The `applies_true_when_no_registered_to` Rust tests have no
  faithful Java twin (would assert opposite) → EXEMPT. Builder tests (with_predicate/with_modifier_fn) EXEMPT.

## Step 4 step-YIELD triage (proven this wave)

A step file's portable-test yield is predictable from HOW its Java `start()` runs — check this first
to set expectations and pick high-ROI targets:

- **Immediate-execute steps** (start() runs executeStep inline, no command dequeue, no hook delegate):
  HIGH yield (3-5 tests) — port the full param + roll subset. Examples ported: StepGoForIt, StepJump,
  StepMoveDodge. Recipe: place acting player, set the flag it reads (jumping/dodging/goingForIt),
  installScriptedDice(exact faces), assert NEXT_STEP(success)/GOTO_LABEL(fail+reRolls=0). Steps that
  publish their own roll param back to themselves (StepMoveDodge publishes DODGE_ROLL=rollSkill()) are
  driven correctly by installScriptedDice.
- **Hook-delegating steps** (`executeStep(){ getGameState().executeStepHooks(this, state); }`): LOW yield
  (~2 param tests) — the behavioural logic lives in a skill-behaviour hook + is command-driven (usingDodge
  /usingTentacles set via use-skill command), NOT expressible through the headless fixture. Port only the
  setParameter-stored keys; defer the state-mutation/select/report tests. Examples: StepBlockDodge
  (stores OLD_DEFENDER_STATE), StepTentacles (stores COORDINATE_FROM).
- **Command-loop action steps** (start() dequeues a client command before executeStep): the no-input
  guard returns CONTINUE, not the Rust NEXT_STEP — the Rust `start_returns_next_step_when_no_X` twin is
  EXEMPT (command-loop structural divergence). Port the setParameter-stored keys (StepPass stores
  CATCHER_ID; GOTO_LABEL_ON_* + internally-computed PASS_RESULT are setParameter-false → exempt).
- **StepBlockRoll**: no-block-result start rolls dice + shows dialog → CONTINUE (place attacker+defender
  +BLOCK acting player); SUCCESSFUL_DAUNTLESS stored. Its negative-nr dialog-team-swap was a RUST bug
  fix (Java is ground truth) — that + publishes/report/command tests deferred.

Rule of thumb: `grep -c executeStepHooks <Step>.java` >0 ⇒ hook-delegating (low yield). Otherwise read
the start() head for a command dequeue (CONTINUE) vs inline executeStep (high yield).

## Step 4 two new fixture unlocks (this wave)

- **Force a roll path via low movement.** Steps that only roll when MA < a threshold (StepStandUp:
  MINIMUM_MOVE_TO_STAND_UP=3) skip the dice entirely at the default MA 6. Set the placed player's
  movement below the threshold to reach the roll branch:
  `((RosterPlayer) game.getPlayerById("home1")).setMovement(2);` then installScriptedDice(face). This
  turned StepStandUp into a full 5-test port (guard + success + failure).
- **Supply mandatory init-only params via init(StepParameterSet).** Some steps require a param that is
  consumed in `init(StepParameterSet)` (NOT public setParameter) and throw StepException if absent —
  e.g. StepApothecary's APOTHECARY_MODE. Build the set and init the step:
  `StepParameterSet set = new StepParameterSet(); set.add(StepParameter.from(StepParameterKey.APOTHECARY_MODE, ApothecaryMode.DEFENDER)); step.init(set);`
  This both satisfies the mandatory check AND sets the mode so downstream mode-conditional setParameter
  tests (DEFENDER_POISONED accepted only in DEFENDER mode, etc.) become portable. The
  param-accepted-via-setParameter twin stays EXEMPT (init-consumed → setParameter returns false).

## Step 4 INJURY_TYPE param needs the SERVER injury type (this wave)

Steps that store an INJURY_TYPE step-parameter (StepFallDown, and any step whose setParameter casts to
`com.fumbbl.ffb.server.injury.injuryType.InjuryTypeServer`) require the SERVER injury type, NOT the
ffb-common `com.fumbbl.ffb.injury.*` class. Passing `new com.fumbbl.ffb.injury.DropGFI()` throws
ClassCastException at setParameter (DropGFI cannot be cast to InjuryTypeServer). Use the server twin:
`new com.fumbbl.ffb.server.injury.injuryType.InjuryTypeDropGFI()` /
`...InjuryTypeDropDodge()` etc. With the server InjuryType set + an acting player placed +
installScriptedDice(armour/injury faces — values don't affect NEXT_STEP or the blood-lust->RESERVE
assertion), the injury-driven start path runs through UtilServerInjury.handleInjury and returns
NEXT_STEP. This unblocks the injury/armour-driven steps (StepFallDown ported 6 tests: 3 param +
start + blood-lust->RESERVE + no-blood-lust). The publishes-INJURY_RESULT / END_TURN-on-turnover /
pass-block / move-square-clear / safe-pair-of-hands tests remain deferred (published-param / turn-mode
/ move-square inspection).

## Step 3 REAL RUST BUG #4: Titchy dodge modifier edition-gated to BB2016 (fixed 2026-07-31)

Found while porting DodgeModifierFactory (modifiers bucket). Java has TWO Titchy classes:
bb2016.Titchy (BB2016) AND mixed.Titchy (@RulesCollection BB2020 + BB2025) — BOTH register an
unconditional `DodgeModifier("Titchy", -1, REGULAR)`. Rust `find_skill_modifiers` gated the Titchy
arm on `rules == Bb2016`, so Titchy players got NO +1 dodge bonus in BB2020/BB2025 (live path:
step_move_dodge / step_diving_tackle). Fixed: arm now unconditional; flipped the Rust test that had
encoded the bug (`titchy_not_in_bb2025` → `titchy_applies_in_bb2025`). Also reconciled
`find_registered_modifiers` (ModifierAggregator.getDodgeModifiers skill half) against the full Java
registration set — it was missing Titchy(mixed), DivingTackle (bb2016+mixed, DIVING_TACKLE), and the
bb2020/bb2025 BreakTackle ST tiers; BB2025 now yields 6 modifiers (was 1), BB2016 5 (was 4); updated
the modifier_aggregator count test. Java DodgeModifierFactoryTest (ffb-server, 14 tests, 1:1 with the
Rust file) all green first run — aggregator counts + Titchy-in-BB2025 verified against Java. Pattern
to watch: mixed/-package Java skill classes carry @RulesCollection(BB2020)+(BB2025) registrations
that a bb2016-named Rust match guard silently drops. Running tally: 4 real Rust bugs.

## Step 3 modifier-factory port progress (2026-07-31)
- **dodge_modifier_factory 14/14 → Java DodgeModifierFactoryTest (ffb-server)** — template:
  ArmorModifierFactoryTest (@BeforeEach GameFixture.createGameState(3) + factory.initialize(game);
  placePlayer home1 + setActingPlayer for context.getPlayer(); addSkill via GameFixture.skill;
  edition variants via createGameState(3, Rules.BB2016/BB2020) re-init inside the test; minimumRoll
  via `new com.fumbbl.ffb.mechanics.bb2025.AgilityMechanic().minimumRoll(agility, set)`; aggregator
  via game.getModifierAggregator().getDodgeModifiers()). ffb-server 3,341 green.
- **catch_modifier_factory 13/13 → Java CatchModifierFactoryTest (ffb-server 3,354 green).**
  Divergence found & fixed (same family as bug #4, lesser severity — report-only): NervesOfSteel
  (bb2016 + mixed BB2020/BB2025) registers a 0-value CatchModifier("Nerves of Steel", "0 for tackle
  zones due to Nerves of Steel", 0, REGULAR) with isModifierIncluded overridden true; Rust was
  missing it from find_skill_modifiers/find_registered_modifiers (gameplay unaffected — the real
  effect is the ignoreTacklezonesWhenCatching property — but the roll-report marker line was
  dropped). Added `modifier_included_override` + `with_modifier_included()` builder to Rust
  CatchModifier (mirrors Java anonymous-subclass isModifierIncluded overrides). Pruned Rust
  find_skill_modifiers_no_player_returns_empty (Option-guard, no Java twin); added
  find_skill_modifiers_nerves_of_steel_marker both sides (net 13 ↔ 13).
- **interception_modifier_factory 14/14 → Java InterceptionModifierFactoryTest (ffb-server 3,368
  green).** Same NoS 0-marker gap fixed (find_skill_modifiers + modifier_included_override on
  InterceptionModifier). Also fixed a VACUOUS Rust test: ignore_tacklezones_when_catching used
  SureFeet (which does NOT grant ignoreTacklezonesWhenCatching) inside an `if has_prop` guard —
  rewritten against NervesOfSteel, unconditional, both sides. GOTCHA for bb2025 interception
  contexts: the "Thrower has Stunty" modifier derefs `game.getThrower()` unconditionally → Java
  tests MUST `game.setThrowerId(...)` before findModifiers (Rust guards the None case — documented
  divergence, no Rust twin for the NPE path). VLL values verified: bb2016 -1, bb2020/bb2025 -2.
  NOTE (follow-up): Rust ModifierAggregator::get_interception_modifiers returns only the CARD half;
  Java's includes the skill half (ExtraArms/NoS/VLL) — reconcile when a consumer needs it.
- **jump_modifier_factory 11/11 → Java JumpModifierFactoryTest (ffb-server 3,379 green).** Rust
  find_registered_modifiers was missing mixed.DivingTackle's JumpModifier("Diving Tackle", 2,
  DIVING_TACKLE) for BB2020/BB2025 (always-false predicate marker; bb2016.DivingTackle registers
  only the DodgeModifier) — fixed, counts 2→3. Java factory is ABSTRACT with edition subclasses —
  instantiate `new com.fumbbl.ffb.factory.{mixed,bb2016}.JumpModifierFactory()` directly (forName
  lives on the subclasses, not the base). DEPENDS_ON_SUM_OF_OTHERS predicates (bb2020 VLL, Leap)
  are drivable headlessly via `context.setAccumulatedModifiers(n)` before findModifiers — no
  opponents needed. bb2016 jump collection is EMPTY (forName("1 Tacklezone") null is the twin of
  Rust's empty-collection assert). setActingPlayer needed (mixed findNumberOfPrehensileTails
  derefs game.getActingPlayer().getPlayer()).
- **Modifier-factory nucleus DONE (dodge/catch/interception/jump).**

## Step 3 REAL RUST BUG #5: Drunkard GFI modifier entirely missing (fixed 2026-07-31)

Found via the bug-#4 registrant pre-check while porting the small factories. Java mixed.Drunkard
(@RulesCollection BB2020 + BB2025) registers an unconditional `GoForItModifier("Drunkard", 1)` —
a Drunkard player rushes at 3+ instead of 2+. Rust's GoForItModifierFactory had NO skill-modifier
path at all and the three step_go_for_it files only chained find_applicable + card modifiers, so
Drunkard's penalty never applied. Fixed: added find_skill_modifiers/find_registered_modifiers to
the Rust factory + chained skill_mods in bb2016/bb2020/bb2025 step_go_for_it. Java
GoForItModifierFactoryTest pins min-roll 3 for a Drunkard player. Tally: 5 real Rust bugs.

## Step 3 small modifier factories DONE (2026-07-31) — modifiers bucket CLOSED
- **pickup 9/9, right_stuff 7/7, go_for_it 8/8, jump_up 3/3 → 4 Java factory tests (ffb-server
  3,406 green).** More bug-#4-family registrant gaps fixed: BigHand (mixed BB2016/BB2020 +
  bb2025) registers a 0-value Pickup marker with EDITION-DEPENDENT report string ("…all negative
  modifiers…" bb2025 vs "…tackle zones and weather effects…" mixed) + isModifierIncluded=true —
  added with modifier_included_override on PickupModifier. Pruned: pickup
  find_skill_modifiers_no_extra_arms (Java twin would duplicate niceWeatherNoModifiers), jump_up
  default_uses_bb2025 (Rust Default plumbing). Java notes: GFI minimum roll lives in
  DiceInterpreter.getInstance().minimumRollGoingForIt (server), NOT AgilityMechanic; Java
  GoForItContext ctor takes (game, player, Set teamsWithMolesUnderThePitch); JumpUpContext is
  (actingPlayer, game) — needs setActingPlayer. Registrant checks confirmed clean: RightStuff
  (Swoop bb2016 only), JumpUp (none).
- **armor_modifiers edition collections 15/15 → Java ArmorModifiersTest (ffb-server 3,421
  green).** Plain-ctor port (`new com.fumbbl.ffb.factory.bb20XX.ArmorModifiers()`, Stream.count()),
  no fixture. Membership verified vs Java: bb2016 16 incl Bomb (values==allValues), bb2020 15 base
  + Bomb legacy-gated behind setUseAll/allValues, bb2025 15 no Bomb w/ Fireball+Lightning.
  **MODIFIERS BUCKET CLOSED.**
- **roll_mechanic bb2025 17/17 → Java server/mechanic/bb2025/RollMechanicTest (ffb-server 3,438
  green).** Server-mechanic recipe: test in com.fumbbl.ffb.server.mechanic.bb20XX package,
  `new RollMechanic()` direct, GameFixture game for factories, `new
  com.fumbbl.ffb.injury.context.InjuryContext()` + setCasualtyRoll/setInjuryRoll/setInjury/
  setDefenderId, dice via gameState.getDiceRoller(), TurnData via game.getTurnDataHome().
  bb2025 SeriousInjury enum is `com.fumbbl.ffb.bb2025.SeriousInjury`. CONVERTED (both sides):
  the SI-detail-fallback test from a MISSING defender (Rust-defensive; Java currentValue derefs
  defender unconditionally) to a defender-at-reduction-threshold (AV 3 + HeadInjury d6=1) —
  faithful and tests the same fallback branch.
- **roll_mechanic bb2016 16/16 + bb2020 16/16 → Java RollMechanicTests (ffb-server 3,470 green).
  roll_mechanic family COMPLETE (49 tests).** Same recipe. Converted the two bb2020 SI
  detail-table tests on both sides from no-defender (Rust-guarded; Java derefs in the
  reduceable-stat filter) to a default-stat defender (all stats reduceable → original injury
  returned — same expected values). Edition enums: com.fumbbl.ffb.bb2016/bb2020/bb2025
  .SeriousInjury. bb2016 casualty is a pure d6 switch (1-3 BH / 4-5 SI / 6 RIP), decay via
  setCasualtyRollDecay + useDecayRoll=true.
- **state_mechanic bb2025 CORE 18/27 → Java server/mechanic/bb2025/StateMechanicTest (ffb-server
  3,488 green).** Recipe: methods take IStep — `GameFixture.createStep(gs, StepId.INIT_START_GAME)`
  carries gameState+result; startHalf/updateLeaderReRollsForTeam/handlePumpUp all drive cleanly.
  handlePumpUp kit: `new InjuryResult()` + injuryContext() setAttackerId/setInjury(new
  PlayerState(RIP))/setInjuryType(new com.fumbbl.ffb.injury.Block()/Foul()); attacker placed
  standing on acting team with skill "Pump Up The Crowd" (capital The). Team setters
  setReRolls/setApothecaries feed addReRolls/addApothecaries.
  **state_mechanic bb2025 tail DONE → 27/27 (ffb-server 3,497 green).** Tail recipe: private
  helpers exercised through public startHalf; skills via addSkill+markUsed then
  player.isUsed(skill) — GOTCHA: the bb2025 once-per-drive skill display name is "Beer Barrel
  Bash!" (with exclamation mark); inducements via
  `new Inducement((InducementType) game.getFactory(INDUCEMENT_TYPE).forName("teamMascot" /
  "wanderingApothecaries"), value)` + setUses + turnData.getInducementSet().addInducement;
  Rust GameEvent::Inducement ↔ Java `step.getResult().getReportList().hasReport(
  ReportId.INDUCEMENT)`; conditional-reroll type retrieved back via
  inducementSet.forUsage(Usage.CONDITIONAL_REROLL).
- **state_mechanic root/base 21/21 → Java server/mechanic/StateMechanicBaseTest (ffb-server 3,518
  green).** Test lives in the BASE package (com.fumbbl.ffb.server.mechanic) to reach the protected
  addApothecaries/addReRolls, driven through the concrete bb2025 subclass. Inducement type names:
  "extraTeamTraining" (REROLL), "wanderingApothecaries" (APOTHECARY, hasSingleUsage filter),
  "plagueDoctor" (bb2025, REGENERATION+APOTHECARY_JOURNEYMEN). Rust GameEvent::Inducement ↔ count
  ReportInducement instances on step result. **DOCUMENTED DIVERGENCE:** Java's plague-doctor
  branch adds NO ReportInducement (turnData sync only); the Rust GameEvent for plagueDoctor is the
  Rust wire layer — Java twins assert the Java-true no-report expectation
  (addApothecariesPlagueDoctorEmitsNoReport / WanderingAndPlagueEmitsOneReport). reportInjury kit:
  InjuryResult + ctx.setDefenderId + setInjuryType(new Block()); ReportId.INJURY + isAlreadyReported.
- **state_mechanic mixed 15/15 → Java server/mechanic/mixed/StateMechanicTest (ffb-server 3,533
  green). STATE_MECHANIC FAMILY COMPLETE (63 tests).** Rust handle_chef_rolls ↔ Java
  UtilServerGame.handleChefRolls (invoked from mixed startHalf; tested against the utility
  directly). Chef dice: 3d6 per chef via installScriptedDice, each face > 3 steals one re-roll —
  scripted faces beat the Rust rng-read-back for determinism. bb2016 chef inducement type name is
  "halflingMasterChef"; report id MASTER_CHEF_ROLL. Mixed gates differ from bb2025: apothecaries
  at half < 2, re-rolls at half < 3.
- **setup_mechanic bb2025 9/9 + mixed 10/10 → Java SetupMechanicTests (ffb-server 3,552 green).
  MECHANIC BUCKET CLOSED** (roll 49 + state 63 + setup 19 + earlier misc). Recipes: empty-team
  checks via `GameFixture.createGameState(0)` (0 players/team works); pinPlayersInTacklezones only
  pins ACTIVE players — placePlayer alone sets STANDING without the active bit, so set
  `new PlayerState(PlayerState.STANDING).changeActive(true)` manually; int game options via
  `(GameOptionInt) game.getOptions().getFactory().createGameOption(id)` + setValue + addOption;
  skill name "Ball and Chain"; checkSetup failure path shows a dialog headlessly (harmless).
## Step 3 INJURY BUCKET recipe proven (injury_type_foul 14/14, 2026-07-31)
- **injury_type_foul → Java server/injury/injuryType/InjuryTypeFoulTest (ffb-server 3,566
  green).** PROVEN RECIPE for the ~25 injury_type_* files: test in the SAME package
  (com.fumbbl.ffb.server.injury.injuryType) to reach protected armourRoll/injuryRoll; fixture
  attacker home1 + defender away1 placed NON-adjacent (foul-assist geometry); dice scripted
  (armour and injury are 2d6 each). CRITICAL GOTCHAS:
  1. Direct armourRoll/injuryRoll calls need `ctx.setAttackerId/setDefenderId` first —
     DiceInterpreter.isArmourBroken reads the defender from the CONTEXT id, not the parameter
     (normally set by UtilServerInjury before handleInjury). Same for handleInjury calls.
  2. BB2025 armour semantics: roll + modifiers >= armour BREAKS (not >) — to reach the
     findArmorModifiers branch use armour 9 with scripted [1,1].
  3. handleInjury(step, game, gameState, diceRoller, attacker, defender, coord, null, null,
     ApothecaryMode.X) is public on ModificationAwareInjuryTypeServer; armour save → PRONE.
  4. Null-attacker paths are Rust-only guards (findFoulAssists derefs attacker) — PRUNED Rust
     no_attacker_id_no_dirty_player_modifier (14 ↔ 14).
  5. "Blatant Foul" is a bb2016 CARD: CARD factory forName + inducementSet
     addAvailableCard/activateCard → game.isActive(foulBreaksArmourWithoutRoll).
  Modifier names to assert: "Dirty Player" (armour + injury), "Chainsaw" (+3 armour).
- **injury_type_stab 11/11 → Java InjuryTypeStabTest (ffb-server 3,577 green).** Pruned 2 Rust
  (Default-impl plumbing; ctor-stored apothecary_mode — Java InjuryContext has no such field).
  KEY EDITION FACT verified both sides: only bb2016's InjuryModifiers collection carries the
  "N Niggling Injury" modifiers — bb2020/bb2025 have NONE (rule change) — so niggling tests need
  a BB2016 game + `defender.addLastingInjury(com.fumbbl.ffb.bb2016.SeriousInjury.SMASHED_KNEE)`
  (NI attribute; Java counts lasting injuries with InjuryAttribute.NI, Rust uses the
  niggling_injuries field). Stab takes a NULL attacker legally (no foul assists).
  InjuryTypeStab(useInjuryModifiers[, addDefenderChainsaw]) ctor variants: (false) = bb2016
  StabBehaviour, (true,true) = StepTreacherous. injuryType() accessor exposes
  failedArmourPlacesProne (ctor sets false).
- **injury_type_stab_for_spp 11/11 → Java InjuryTypeStabForSppTest (ffb-server 3,588 green).**
  Straight clone of the stab recipe (same prunes: Default plumbing + ctor apo-mode). The ForSpp
  siblings are mechanical clones — expect the same for drop_dodge_for_spp/chainsaw_for_spp etc.
- **injury_type_chainsaw 12/12 → Java InjuryTypeChainsawTest (ffb-server 3,600 green).** Chainsaw
  kickback: NULL attacker legal; +3 chainsaw armour modifier sourced from the SKILL FACTORY inside
  armourRoll (any skill with blocksLikeChainsaw) — remember it applies to the SAVE test scripting
  (armour 13 needs [1,1]: 2+3 < 13). Stunty-KO test made DETERMINISTIC in Java (script 3,4 →
  total 7 + mixed.Stunty isHurtMoreEasily marker → KNOCKED_OUT in BB2025) vs Rust's rng-guarded
  conditional. Dedup guard: pre-add via GameFixture.skill(game,"Chainsaw").getArmorModifiers().
- **injury_type_chainsaw_for_spp 11/11 → Java InjuryTypeChainsawForSppTest (ffb-server 3,611
  green).** Mechanical clone of chainsaw (isWorthSpps flips to true; no niggling test).
## Step 3 REAL RUST BUG #6: piling-on apothecary/turnover flags (fixed 2026-08-01)
Both InjuryTypePilingOnArmour and InjuryTypePilingOnInjury hardcoded `can_use_apo=false` and
`falling_down_causes_turnover=false` — copied from PilingOnKnockedOut. Java ground truth: ONLY
EatPlayer/PilingOnKnockedOut/Saboteur override canUseApo to false; PilingOnArmour/PilingOnInjury
inherit BOTH base defaults (true). Gameplay impact: the Rust apothecary steps
(bb2016/bb2020 step_apothecary consult can_use_apo via make_injury_type) wrongly blocked the apo
after a piled-on casualty. Fixed by deleting the overrides (trait defaults true); flipped the
no_apo/no_turnover Rust tests. Tally: 6 real Rust bugs.
- **piling_on_armour 12/12 + piling_on_injury 10/10 → Java tests (ffb-server 3,633 green).**
  Bug-#6 apoAllowed tests confirmed against Java. Boolean options via
  `(GameOptionBoolean) options.getFactory().createGameOption(id)` + setValue(true) + addOption.
  Pruned 3 more Rust-structural tests (Default plumbing; injury_context accessor tautology;
  sets_defender_id — Java's CALLER sets the context ids, not handleInjury).
## Step 3 REAL RUST BUG #7: Lightning turnover flag (fixed 2026-08-01)
InjuryTypeLightning hardcoded `falling_down_causes_turnover=false`; Java's Lightning has NO
override (base default true). Same copied-override family as bug #6. Fixed + test flipped.
Tally: 7 real Rust bugs.
- **injury_type_lightning 9/9 + drop_dodge_for_spp 10/10 → Java tests (ffb-server 3,652 green).**
  Lightning: the +1 bonus applies to armour ONLY when needed to break, else to injury — never
  both; IHS defender denies it entirely (specialEffectArmourModifiers gate). DropDodgeForSpp:
  Java ctor REQUIRES the arm-bar player and credits it as ctx attackerId unconditionally; the
  injury-modifier factory reads the pAttacker PARAMETER (not the arm-bar player) — MB test puts
  the skill on the passed attacker. Pruned 3 more Rust-structural tests (apo-mode ctor, accessor
  tautology, arm-bar ctor field storage).
## Step 3 REAL RUST BUG #8 (batch): systematic injury-type flag audit (fixed 2026-08-01)
Swept EVERY Rust injury_type_* flag override against the Java model classes + server ctors
(prompted by bugs #6/#7 sharing the copied-override root cause). Findings, all fixed:
- **10 files wrongly overrode `falling_down_causes_turnover=false`** where Java inherits true:
  bitten, bomb, bomb_with_modifier(+for_spp), breathe_fire(+for_spp), ktm_crowd,
  piling_on_knocked_out, projectile_vomit, quick_bite. (Java only overrides false in
  CrowdPush/CrowdPushForSpp/Saboteur/TrapDoorFall/TrapDoorFallForSpp.) Latent today — the flag
  is only consulted by the three StepFallDown variants, which these types never reach — but
  wrong for 1:1 and any future consumer.
- **ball_and_chain wrongly overrode `failed_armour_places_prone=false`** (with a comment claiming
  Java matches — it doesn't; Java only sets it false in Chainsaw(+ForSpp)/ProjectileVomit/
  QuickBite/Stab(+ForSpp)/ThenIStartedBlastin server ctors, all of which Rust has correctly).
- **Verified CORRECT:** can_use_apo set (eat_player/piling_on_knocked_out/saboteur after #6),
  the can_apo_ko_into_stun name-fn (crowd/trapdoor/fumbled-ktm false set matches Java incl.
  KTMFumbleInjury), worth_spps ctor args, is_caused_by_opponent set, saboteur turnover=false.
- 13 Rust tests that had pinned the wrong flags flipped; 3 injury.rs dispatch tests re-anchored
  on Java-correct markers (send_to_box_reason / failed_armour_places_prone) instead of the
  removed turnover overrides. Full engine 7,090 green. Tally: 8 real Rust bugs (this one a batch).
- **injury_type_fireball 8/8 + injury_type_bomb 9/9 → Java tests.** Fireball = Lightning bonus
  semantics (armour-when-needed else injury; pre-broken ctx variant). Bomb: NO bomb modifier of
  its own; defender-chainsaw +3 kickback (IHS suppresses); armour save leaves injury UNSET (no
  PRONE branch). Pruned fireball's 2 ctor/accessor plumbing tests.
- **injury_type_quick_bite 8/8 + projectile_vomit 8/8 → Java tests (both green first run;
  generated from a shared python template — the injury-tail files are template-uniform now:
  save-leaves-no-injury / break-rolls-injury / turnover-default / sendToBox / causedByOpponent /
  failedArmourProne / niggling×2-BB2016).** Pruned 4 more Rust-structural (context_stores_*,
  default_equivalent ×2).
- **bitten 7/7 + keg_hit 5/5 + throw_a_rock 6/6 + throw_a_rock_stalling 6/6 → Java tests (24,
  all green first run; template-generated).** Bitten/ThrowARock bypass armour (always broken);
  Bitten caps casualty-range totals at BADLY_HURT with no casualty dice; stalling variant rolls
  armour normally. 10 more Rust-structural prunes (context storage, ctor apo-mode, accessor).
- **crowd 4/4 + saboteur 6/6 + piling_on_knocked_out 6/6 + trap_door_fall_for_spp 5/5 → Java
  tests (21 green).** Crowd base tested through concrete InjuryTypeCrowdPush; Java's crowd
  handleInjury DOES apply skill injury modifiers (MB) — verified ground truth. Saboteur/POKO:
  direct KO, no dice; InjuryTypePilingOnKnockedOut ctor takes IStep. TrapDoorFallForSpp extends
  the crowd base — stunned result → RESERVE (deterministic script 1,1). 12 more Rust-structural
  prunes.
- **then_i_started_blastin 8/8 + foul_for_spp 11/11 → Java tests (19 green).** ForSpp generated
  as a Foul-test transform; ADDED the IHS-blocks-chainsaw test to the Rust ForSpp file for parity
  (real shared armourRoll behavior, cheaper than deleting the Java twin). 2 Rust prunes.
## Step 3 REAL RUST BUG #9: skill injury modifiers carried no registered_to (fixed 2026-08-01)
Found porting fumbled_ktm: the Rust InjuryModifierFactory's skill_to_injury_modifier never
tagged created modifiers with their owning skill, so isRegisteredToSkillWithProperty-style
filters saw None and dropped EVERYTHING — InjuryTypeFumbledKtm's block-property filter wrongly
excluded Mighty Blow (Java's MightyBlow registers affectsEitherArmourOrInjuryOnBlock in every
edition, so a fumbled KTM landing DOES get the kicker's MB bonus — confirmed by running the Java
twin, which failed against the old expectation). Fixed by tagging every factory-created modifier
with skill_id.class_name(); flipped the Rust test (mighty_blow_kept_by_block_property_filter).
Tally: 9 real Rust bugs.
- **fumbled_ktm 7/7 + fumbled_ktm_apo_ko 5/5 → Java tests (12 green).** ApoKo variant passes a
  NULL attacker to the factory internally (attacker skills never apply; defender niggling does).
  4 Rust prunes.
## Step 3 REAL RUST BUG #10: bomb_with_modifier attacker semantics SWAPPED (fixed 2026-08-01)
Java AbstractInjuryTypeBombWithModifier passes `injuryType.isCausedByOpponent() ? pAttacker :
null` to findInjuryModifiers. Bomb does NOT override isCausedByOpponent (base default false →
null attacker, Mighty Blow must NOT apply); BombForSpp overrides it to TRUE (real attacker,
MB DOES apply). Rust had it exactly inverted in BOTH files — the base variant passed the
attacker, the ForSpp variant hardcoded None (with a comment misstating the Java source).
Fixed both handle_injury bodies and flipped both tests. Tally: 10 real Rust bugs.
- **bomb_with_modifier 8/8 + bomb_with_modifier_for_spp 9/9 → Java tests (17 green).**
  2 Rust prunes (default_equivalent_to_new ×2). Java gotcha: ArmorModifierFactory caches the
  bombUsesMb flag in initialize(game), so tests enabling the option afterwards must re-call
  `game.getFactory(ARMOUR_MODIFIER).initialize(game)`. bb2020 "Bomb" armor modifier is
  SpecialEffectArmourModifier("Bomb", 1, false, BOMB) — legacy entry behind setUseAll.
- **INJURY BUCKET CLOSED** (all injuryType files ported or exempted with breadcrumbs).
- Next: inducements 432, skill_behaviour 359, util 138 — rerun scripts/reconcile_step3.sh
  for the live list.

## Step 3 inducements bucket — PRAYER HANDLER RECIPE (established 2026-08-01)
Java: com.fumbbl.ffb.server.inducements.{mixed,bb2020,bb2025}/prayers. mixed handlers are
abstract (RandomSelectionPrayerHandler/PrayerHandler); concrete per-edition subclasses override
handledPrayer()/selector()/addedSkills(). Test recipe:
- Instantiate concrete handler directly; `handler.initEffect(gameState, team)` (2-arg public
  variant — the IStep wrapper isn't needed). affectedPlayers rolls d3 → installScriptedDice 1 die.
- GameFixture players all start RESERVE + TurnMode.START_GAME → PlayerSelector.eligiblePlayers
  returns everyone (RESERVE branch). selectPlayers uses Collections.shuffle — NO gameState dice.
- Enhancement check: `player.hasActiveEnhancement(Prayer.X.getName())` ("Bad Habits" display
  name, NOT the Rust "BAD_HABITS" enum string). Add via
  `game.getFieldModel().addPrayerEnhancements(player, Prayer.X)`.
- mixed abstract behavior tested from a SAME-PACKAGE Java test via a concrete subclass typed as
  the mixed class (protected affectedPlayers reachable).
- **bad_habits_handler ×3 (mixed 6 + bb2020 5 + bb2025 5) → Java tests (16 green).** 1 Rust
  prune (PRAYER_NAME constant plumbing).
- **fouling_frenzy ×3 + friends_with_the_ref ×3 → Java tests (30 green).** PrayerState-flag
  handlers: initEffect/removeEffectInternal flip gameState.getPrayerState()
  add/remove/hasFoulingFrenzy / isFriendsWithRef — no dice, no players needed. 4 Rust prunes
  (handles_prayer_is_case_sensitive ×4 — string-based check inexpressible against Java's
  enum-typed handles(Prayer)). getName() == simple class name is portable.
- **under_scrutiny ×3 + fan_interaction ×3 → Java tests (29 green).** Same PrayerState-flag
  shape; UNDER_SCRUTINY targets the OPPONENT team (getOtherTeam) — Rust verified matching.
  5 Rust prunes (case-sensitivity ×4 + duplicate animation test in mixed under_scrutiny).
## Step 3 REAL RUST BUG #11: bb2025 BlessedStatue handled the WRONG PRAYER ID (fixed 2026-08-01)
Java bb2025 BlessedStatueOfNuffleHandler extends RandomSelectionPrayerHandler DIRECTLY and
handles Prayer.BLESSING_OF_NUFFLE — a different id than bb2020's BLESSED_STATUE_OF_NUFFLE.
The Rust bb2025 handler reused the mixed module's PRAYER_NAME ("BLESSED_STATUE_OF_NUFFLE"),
so it could NEVER match the bb2025 prayer (data/prayers/bb2025_prayers.json id is
BLESSING_OF_NUFFLE), and prayer_player_effect had no BLESSING_OF_NUFFLE arm (Pro never
granted). Fixed handler const + effect map + flipped test. Tally: 11 real Rust bugs.
- **CARDS: witch_brew ×2 + distract ×2 → Java tests (34 green).** CARD RECIPE: construct
  `new Card("Test Card", "tc", bb2020.CardType.DIRTY_TRICK, CardTarget.ANY_PLAYER, false,
  new InducementPhase[0], InducementDuration.UNTIL_END_OF_GAME, "test", CardHandlerKey.X)` —
  the bb2020 Cards catalog is EMPTY so raw Card construction is the only way to hit bb2020
  handler keys. activate(card, step, player) with GameFixture.createStep; effects via
  fieldModel.hasCardEffect. Witch brew d6: 1=MadCap, 2=none, 3-6=Sedative
  (rollCardEffect = 1 die). Distract: 3-square radius, deactivate clears confusion.
  Remaining cards: chop_block, force_shield, illegal_substitution (×2 each) + card_handler base.
- **CARDS: custard_pie/rabbits_foot/pit_trap ×6 → Java tests (32 green).** Custard pie
  allowsPlayer: raw card is in neither InducementSet so ownTeam resolves to AWAY — test with
  away players adjacent. Pit trap activate = UtilServerInjury.dropPlayer (PRONE + ball scatter
  when carried, isBallMoving twin; 4 scripted dice for the scatter). 6 Rust prunes
  (unknown-player id guards ×4, handler_key_name accessor ×2).
- **prayer_player_effect + select_player base → Java tests (9 green). PRAYER SUB-BUCKET
  CLOSED.** PrayerEnhancementsTest twins the Rust effect map against Java
  Prayer.enhancements()/FieldModel.addPrayerEnhancements (stat mods, skill grants with values,
  removal, no-enhancement prayers). SelectPlayerPrayerHandlerTest twins the dialog-waiting
  false return. 7 Rust prunes (defensive/duplicate-path); random_selection_prayer_handler's
  6 helper tests EXEMPT with in-file note — Java's abstract class is exercised through the
  concrete handler twins calling the same paths. Next: inducements/cards
  (witch_brew ×2, rabbits_foot, custard_pie, pit_trap ×2 each bb2016+bb2020, card_handler),
  then skill_behaviour 356 / util 121 / injury 91 leftovers per reconcile.
- **enhancement_remover + prayer_dialog_selection + prayer_handler base → Java tests (9 green).**
  EnhancementRemover twins incl. opponent-selector removal regression; PrayerDialogSelection =
  1 getter twin; PrayerHandler base via named static nested TestPrayerHandler subclass
  (getName() == simple class name; removeEffect delegation flag). 11 Rust prunes (Default/
  Clone/builder plumbing + test-double accessor tautologies + Rust-only for_player helper).
  Prayer infra remaining: random_selection_prayer_handler (6) + select_player_prayer_handler
  (7) — mostly covered by concrete twins, triage next — and prayer_player_effect (9 —
  Java twin = FieldModel.addPrayerEnhancements effect assertions).
- **player_selector ×2 + opponent_player_selector ×2 → Java tests (24 green).** Eligibility
  twins: bb2020 = RESERVE@START_GAME / on-pitch@REGULAR (setTurnMode) + Loner excluded;
  bb2025 = RESERVE-only + PlayerType.STAR excluded (RosterPlayer.setType). selectPlayers count
  limits + opponent redirection both editions. 9 Rust prunes (5 StubPlayerSelector scaffolding
  tests in mixed, 4 Default-impl tests).
- **intensive_training ×3 + blessed_statue_of_nuffle ×3 → Java tests (27 green).** Intensive
  training = skill-choice dialog (applySelection applies via addIntensiveTrainingSkill; remove
  clears the temp skill). bb2025 BlessedStatue is RandomSelection → the Pro grant IS portable
  (initEffect applies immediately, unlike the bb2020 dialog variant). 1 Rust prune
  (apply_selection_is_noop — Rust headless stub, Java applySelection is not a noop).
- **throw_a_rock ×3 → Java tests (10 green).** bb2020 marks the OPPONENT team should-not-stall
  (PrayerState.shouldNotStall); bb2025's Java initEffect registers a THROW_ROCK-usage inducement
  — Rust documents this as UNPORTED (headless no-op, in-file note) → OPEN GAP for a future
  translation pass; Java twin mirrors only shared behaviors. 7 Rust prunes (6 no-op mixed
  helper tests — Java mixed class has ONLY animationType(); 1 case-sensitivity).
- **stiletto ×3 + necessary_violence ×1 → Java tests (19 green).** Stiletto = RandomSelection,
  1 player, OWN-team selector (bb2025 twin proves home-team enhancement); necessary_violence =
  bb2020-ONLY PrayerState flag (getAdditionalCasSppTeams). 2 Rust prunes. Prayer handler files
  remaining: throw_a_rock ×3 (edition logic), base/selector/dialog files, then cards
  (rabbits_foot, custard_pie, pit_trap — bb2016+bb2020 cards dirs; witch_brew ×2, card_handler).
- **perfect_passing ×3 + treacherous_trapdoor ×3 → Java tests (26 green).** Perfect passing =
  PrayerState-flag (getAdditionalCompletionSppTeams); trapdoor initEffect adds TrapDoors at
  (6,1)/(19,13) — check via fieldModel.getTrapDoors().contains(new TrapDoor(coord)) (TrapDoor
  has equals). 3 Rust prunes (case-sensitivity ×2, TRAPDOOR_COORDINATES constant ×1).
- **iron_man ×3 + knuckle_dusters ×3 → Java tests (26 green).** SelectPlayerPrayerHandler
  (dialog-based) recipe: Java initEffect shows a player-choice dialog and returns FALSE when
  eligible players exist (true only when wasted/empty); the effect lands via
  applySelection(game, new PrayerDialogSelection(playerId, null)). DOCUMENTED DIVERGENCE:
  Rust's headless init_effect random-selects and applies immediately — the Rust
  init_effect_grants_*/marks_* tests have NO Java twin (Java twin = wasted-case
  initEffectReturnsTrueWhenNoEligiblePlayers via createGameState(0)). Portable: applySelection
  (Iron Man +1 getArmourWithModifiers; Knuckle Dusters grants temporary Mighty Blow via
  getSkillsIncludingTemporaryOnes), removeEffectInternal, bb2025 IronManPlayerSelector
  armour>=11 filter (Java twin: all players setArmour(11) → prayer wasted, nobody enhanced).
- **moles_under_the_pitch ×3 + greasy_cleats ×3 → Java tests (30 green).** Moles =
  PrayerState-flag (check via getMolesUnderThePitch().contains(teamId)); greasy_cleats =
  RandomSelection with affectedPlayers()==1 hardcoded (NO dice) + opponent selector.
  3 Rust prunes (case-sensitivity ×2, PRAYER_NAME constant ×1). Remaining prayer handlers:
  iron_man/knuckle_dusters (SelectPlayerPrayerHandler — dialog-based, needs new recipe),
  perfect_passing, throw_a_rock, treacherous_trapdoor, pit_trap, stiletto, necessary_violence,
  rabbits_foot, custard_pie, bad_habits done; plus select_player/random_selection/prayer_handler
  base tests, prayer_dialog_selection, player_selector ×2, opponent_player_selector ×2,
  enhancement_remover, card_handler, witch_brew ×2 (cards).

## Step 4 REAL RUST BUG #3: StepSafeThrow early NEXT_STEP (fixed 2026-07-26)

Found while porting bb2016/pass/StepSafeThrow. Rust step_safe_throw::execute_step early-returned
`StepOutcome::next()` in two cases: (a) thrower lacks canCancelInterceptions (no Safe Throw skill),
(b) interceptor cancels it (VeryLongLegs). Java's StepSafeThrow instead sets `doSafeThrow=false` and
FALLS THROUGH to `fail_safe_throw` → GOTO_LABEL (the interception stands; ball/bomb moves to the
interceptor). So the correct action is GOTO_LABEL, not NEXT_STEP. Fixed both early returns to
`return self.fail_safe_throw(game, &interceptor_id);`, corrected the two Rust tests that had encoded
the bug (asserted NextStep → now GotoLabel), updated the module doc comment. cargo -p ffb-engine 7101
pass; Java twin StepSafeThrowFixtureTest asserts GOTO_LABEL. Pattern to watch: a Rust early
`StepOutcome::next()` guard whose Java counterpart only sets a `doX=false` flag and continues — the
Java fall-through often reaches a failure/GOTO branch, so the early return silently skips it. Running
tally: 3 real Rust bugs found via this campaign (furious_outburst sub-sequence, bomb
RECHECK_EXPLODE_SKILL, StepSafeThrow early-next).
