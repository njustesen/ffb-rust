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
