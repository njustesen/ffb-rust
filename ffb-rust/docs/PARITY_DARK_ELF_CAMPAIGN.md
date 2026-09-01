# Parity campaign — HeuristicAgent on dark_elf vs dark_elf (all three rulesets)

Started 2026-09-01, immediately after chaos_pact closed (nine gates + randoms 100/100 and pushed).
Same goal shape, same three-loop procedure (`.claude/commands/chaos-iter.md` with MATCHUP=dark_elf).

## Goal

Nine `PARITY: 100/100` — dark_elf v dark_elf, tier 3, seeds 1-100, heuristic `--heur-classes all`,
bb2016/bb2020/bb2025 × scales 0/1.0/1e6, plus coverage recorded.

New surface vs the chaos family: the STAR-heavy team specs — bb2020 fields THREE stars (Helmut
Wulf: Chainsaw/Old Pro/Stand Firm/Secret Weapon; nr 1 = star 54496 with **Black Ink**; Horkon) and
bb2025 fields the Zoat (star 39558: **"Excuse Me, Are You a Zoat?"** auto-gaze). Witch Elves
(Frenzy/Dodge/Jump Up), Assassins (Stab/Shadowing), Runners (Dump-Off + bb2025 **PUNT**).
Baseline @1.0: bb2016 **20/20**, bb2020 1/20, bb2025 0/20 with 4 RUST PANICS
(`throw_in_mechanic.rs:40 Unable to determine throwInDirection`).

## ITER1 — the gaze stars: BLACK_INK / AUTO_GAZE_ZOAT declarations were dropped by the harness, and the gaze never retired the gazer

Three stacked mechanisms, all exposed by the same two seeds (bb2020 seed 1 i=6 Kiroth's Black Ink,
bb2025 seed 1 i=9 the Zoat's gaze):

1. **Harness (`sendStarSpecialDeclaration`)**: the heuristic declares star specials as a COMMAND
   PAIR (ActingPlayer(MOVE) + UseSkill), but its switch was missing BLACK_INK and AUTO_GAZE_ZOAT —
   the bare `ActingPlayer(pid, BLACK_INK)` declaration reached phase 2, whose sendConcreteAction
   switch has no gaze arm, and the `default:` DESELECTED it: on the Java side the whole activation
   was a silent no-op (zero dice, zero draws, player still eligible) while Rust gazed and moved.
   Proved by FFB_GAZE probes in the stock steps that NEVER FIRED (probe-never-fires = dead file),
   then arm probes that also never fired — the gaze arms at ParityRunner:784/842 belong to the
   RANDOM path. Fix: the two cases in sendStarSpecialDeclaration (properties canGazeAutomatically
   / canGazeAutomaticallyThreeSquaresAway).
2. **Engine (three more copies of the Estelle/Baleful-Hex fault)**: `Game::mark_skill_used` and
   both `step_black_ink` twins wrote only `Player.used_skills`; Java's
   `actingPlayer.markSkillUsed` is a term of `hasActed()`, which is what retires the gazer
   STANDING+inactive at the deselect. Java's H2 ended `1:i`, Rust's `1:a` — one bit, whole game.
   Fix: `util_server_steps::mark_skill_used` at all three sites.
3. The zoat/BlackInk PLAYER_CHOICE answers (coord-sort + actionRng / min-x,y) already matched the
   dead-step-era contracts on both sides.

## ITER2 — Stand Firm must PROMPT under the heuristic (bb2020 seed 1 i=11)

Helmut Wulf's Stand Firm against a blitz: Java's PUSHBACK shows DialogSkillUseParameter and the
heuristic driver answers through its useSkill sampler (2 draws); the Rust bb2020
StandFirmBehaviour auto-ACCEPTED inline (written for the RANDOM contract's free always-use) — two
draws short, streams split. Fix: park `pending_skill_use` exactly like the bb2025 Sidestep
behaviour, and route the answer back into `standing_firm` (the step's UseSkill arm filed EVERY
answer into `side_stepping`). bb2020 seed 1 GREEN.

## ITER3 — drive PUNT end-to-end (bb2025 seeds 1/6/7/10; the throwInDirection panic)

- **Harness**: `isHandledActingAction` was missing PUNT (forceDispatch — the bare declaration
  pushes the punt sequence; INIT_PUNT's square wait was already driven by sendPuntTarget), so Java
  deselected every heuristic punt (`UNHANDLED_ACTING_ACTION_AT_PICK: PUNT`) while Rust executed it.
  Both sides had picked the SAME candidate at the same draw totals (RSUM/JSUM n=2051, RCAND/JCAND
  weight-identical) — the divergence was entirely the harness gate.
- **Panic root cause**: `StepInitPunt`'s `Action::Punt` arm consumed the away coach's MIRRORED
  coordinate raw (Java un-transforms: `checkCommandIsFromHomePlayer ? c : c.transform()`), handing
  StepPuntDirection `transform((19,5)) = (6,5)` — a 13-square diagonal whose direction template
  panicked the throw-in mechanic. Punt targets are strictly ORTHOGONAL (`findPuntSquares`), so a
  faithful coordinate can never be diagonal.
- **PuntToCrowd**: a boundary punter gets DialogPuntToCrowdParameter; unhandled, it fell to the
  NON-SEEDED RandomStrategy (`sendPuntToCrowd(RANDOM.nextBoolean())`). Contract: deterministic
  DECLINE, zero draws, on both sides (ParityRunner case + Rust auto-decline).
- **Own goal, caught the same hour**: the PUNT_TO_CROWD case was first inserted BETWEEN
  `case PLAYER_CHOICE:` and `case KICK_SKILL:` — a fall-through pair — so every PlayerChoice
  dialog answered with a punt command and looped (the zoat dialog re-fired 8×, seed 1 "regressed"
  to i=9). Moved above the pair.

Seed 6's punt now byte-identical (ball b23,1 both, rng 34=34); bb2025 seed 1 + seeds 6-10 GREEN.

## ITER4 — the punter must retire: acting-player skill mark, fourth copy (bb2025 seeds 40/63)

Java `StepPuntDirection`: `actingPlayer.markSkillUsed(NamedProperties.canPunt)` — the mark is a
`hasActed()` term, retiring the punter STANDING+inactive when the activation ends. Rust used the
player-only `Game::mark_skill_used`, so the punter stayed ACTIVE (J `a07 …,0` vs R `…,1` at the
post-punt record) and every later pick diverged. Also caught 6 of 6 reds in the bb2025 RANDOM
control (94→re-running) — the control's first catch this campaign. Fix:
`util_server_steps::mark_skill_used`. **Audit note**: `game.mark_skill_used(` still has ~11
callers (animosity, missed_pass ×2, init_feeding ×2, init_bomb, drop_falling_players/PilingOn,
raiding_party, pump_up ×2); each needs checking against its Java site for the actingPlayer-vs-
player distinction before converting — only verified sites were converted here.

**Measured @1.0**: bb2016 100/100, bb2020 100/100, bb2025 98→100 (40, 63 green single-seed);
standing amazon ×3 + lineman ×3 100/100; randoms bb2016/bb2020 100/100.

## ITER5 — the random agent mirrors the PUNT declaration (bb2025 random control 94→100)

All six random-control reds were `J Activate(*,PUNT)` vs a Rust deselect: `is_handled_acting_action`
(the ParityRunner `isHandledActingAction` mirror) never gained PUNT when the harness did. The
control caught a contract asymmetry the heuristic gates could not.

## ITER6 — a 0.0 wUse pin is not deterministic; and the batched JVM poisons on a stock NPE (@1e6)

bb2025 @1e6 ran 37/100 with seeds 38-100 red CONSECUTIVELY — the signature of a mid-batch event,
not 63 game bugs. Seed 38 alone: both drivers answered USE on a pinned-0.0 DumpOff (`JSKILL/RSKILL
idx=0`) — the softmax at scale 1e6 flattens [0,1] to a coin flip (parity HELD; the flip was
mirrored) and drove both engines into the undriveable DUMP_OFF pass: stock Java NPE
(StepBlockStatistics, the dark_elf-seed-55 family) whose exception then POISONED the shared JVM
for seeds 39-100, and a Rust `goto unknown label ''` in the dump-off chain (real engine bug, now
unreachable again; BACKLOG candidate). Fix: the four undriveable pins (DumpOff/PrimalSavagery/
SafePairOfHands/Swoop) still spend the sampler draws but the ANSWER is overridden to DECLINE in
both drivers. Lesson: **consecutive seed ranges of reds = batch-state poisoning, test the seeds
solo**; a "pin" through a temperature-scaled sampler is not a pin.

## ITER7 — MB defender dialog + the Frenzy blitz Foul-Appearance edition split (@1e6 stragglers)

- bb2020 seed 24: the heuristic's MULTIPLE_BLOCK reached `DialogOpponentBlockSelectionParameter`
  (a not-own-choice roll: the DEFENDER picks the die) — no ParityRunner case, RandomStrategy has
  none either → dialog re-fired to the sameDialog abort. Case added (+ the bb2025 PROPERTIES
  twin): first `needsSelection()` target, die index 0, injected for the DIALOG'S team — mirroring
  Rust's headless auto-select-0. (Own-goal avoided this time: placed ABOVE `case PLAYER_CHOICE:`.)
- bb2020 seed 48: Java `FoulAppearanceBehaviour.handleFailure` publishes END_PLAYER_ACTION for
  `GAZE || isBlockAction() || isBlitzing()` in bb2025 but WITHOUT `isBlitzing()` in bb2020 — a
  bb2020 Witch Elf whose FRENZY second-block FA failed must cancel only the second block and
  CONTINUE THE BLITZ MOVE. The shared Rust step ran the bb2025 shape for every edition; now
  rules-dispatched. Test `failed_blitz_fa_ends_action_only_in_bb2025`. En route: the bb2020
  `step_end_blocking.rs` twin is DEAD CODE (make_step dispatches the bb2025 twin for all
  editions) — a probe in it never fired; the live twin's identical text hid nothing this time.

Probes kept (FFB_TRACE-gated): RPUNTDIR (punt direction inputs), REB (EndBlocking continue gate).
