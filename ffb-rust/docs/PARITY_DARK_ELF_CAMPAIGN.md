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
