package com.fumbbl.ffb.ai.parity;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.FieldCoordinateBounds;
import com.fumbbl.ffb.IDialogParameter;
import com.fumbbl.ffb.PlayerAction;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.TurnMode;
import com.fumbbl.ffb.ai.simulation.CapturingClientCommunication;
import com.fumbbl.ffb.ai.simulation.HeadlessFantasyFootballServer;
import com.fumbbl.ffb.ai.simulation.HeadlessGameSetup;
import com.fumbbl.ffb.ai.simulation.MatchRunner;
import com.fumbbl.ffb.ai.strategy.RandomStrategy;
import com.fumbbl.ffb.model.ActingPlayer;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.model.TurnData;
import com.fumbbl.ffb.PlayerChoiceMode;
import com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter;
import com.fumbbl.ffb.dialog.DialogReceiveChoiceParameter;
import com.fumbbl.ffb.net.commands.ClientCommandActingPlayer;
import com.fumbbl.ffb.net.commands.ClientCommandBlock;
import com.fumbbl.ffb.net.commands.ClientCommandFoul;
import com.fumbbl.ffb.net.commands.ClientCommandHandOver;
import com.fumbbl.ffb.ai.parity.heuristic.ClassMask;
import com.fumbbl.ffb.ai.parity.heuristic.HeuristicDriver;
import com.fumbbl.ffb.net.commands.ClientCommandMove;
import com.fumbbl.ffb.net.commands.ClientCommandPass;
import com.fumbbl.ffb.model.Team;
import com.fumbbl.ffb.net.commands.ClientCommandCoinChoice;
import com.fumbbl.ffb.net.commands.ClientCommandPlayerChoice;
import com.fumbbl.ffb.net.commands.ClientCommandEndTurn;
import com.fumbbl.ffb.net.commands.ClientCommandKickoff;
import com.fumbbl.ffb.net.commands.ClientCommandReceiveChoice;
import com.fumbbl.ffb.net.commands.ClientCommandStartGame;
import com.fumbbl.ffb.net.commands.ClientCommandTouchback;
import com.fumbbl.ffb.server.GameState;
import com.fumbbl.ffb.server.step.IStep;
import com.fumbbl.ffb.server.step.StepId;
import com.fumbbl.ffb.server.util.UtilServerSetup;
import com.fumbbl.ffb.util.UtilBox;
import com.fumbbl.ffb.util.UtilPassing;
import com.fumbbl.ffb.net.commands.ClientCommandInterceptorChoice;

import java.io.File;
import java.io.FileOutputStream;
import java.io.PrintWriter;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashSet;
import java.util.Set;
import java.util.List;
import java.util.logging.Level;
import java.util.logging.Logger;

/**
 * Runs a single seeded game and emits a full JSONL decision log.
 *
 * <p>Usage: ParityRunner [serverDir] homeTeamId awayTeamId seed [output.jsonl]
 *
 * <p>The output matches the format emitted by ffb-rust's parity_runner binary.
 * Log lines: game_start, one step per INIT_SELECTING phase-1 decision, game_end.
 *
 * <p>Uses {@link Xoshiro256StarStar} seeded from the given seed so dice rolls
 * are deterministic and can be compared against the Rust engine.
 *
 * <p>ALL random decisions use decisionRng (seeded) to match Rust's ParityAgent exactly.
 * Every dialog type that previously used Java's non-seeded RANDOM now uses decisionRng
 * with the same number of nextLong() calls as the Rust parity agent.
 */
public class ParityRunner {

    private static final int MAX_ITERATIONS = 2_000_000;

    /** Verbose stderr diagnostics, enabled with -Dffb.parityDebug=true. */
    private static final boolean DEBUG = Boolean.getBoolean("ffb.parityDebug");
    private static final boolean DEBUG_ACT = System.getenv("FFB_ACT_TRACE") != null;

    /**
     * Parity tier (see ffb-rust AGENT_CONTRACT.md):
     *   2 — T2 regression behavior: one decisionRng pick per turn, then immediate
     *       deselect + EndTurn (no concrete actions). This is the 26-race suite.
     *   3 — T3 Phase 2: real activations; after each pick the acting player performs
     *       a concrete action (move, later block/blitz/pass/hand-over/foul).
     */
    private int tier = 2;

    /**
     * Non-null when `--agent heuristic` is in force. Answers only the prompt classes in its
     * ClassMask; everything else falls through to the random policy below, whose RNG consumption
     * is byte-matched against Rust's RandomAgent.
     */
    private HeuristicDriver heuristic;

    /**
     * `--multimove N` (default 0 = off). Submit a planned path of up to N one-step squares in a
     * single CLIENT_MOVE instead of one. Mirrors RandomAgent::multimove; see
     * docs/PARITY_HEURISTIC_CAMPAIGN.md -- it exists to test multi-square move-stack consumption in
     * both engines without first porting the heuristic agent's scorer.
     */
    private int multimove = 0;

    private final PrintWriter out;
    private final CapturingClientCommunication comm = new CapturingClientCommunication();
    private final List<PendingStep> pending = new ArrayList<>();
    private int stepIndex = 1;
    // Decision RNG: seeded with game seed ^ 0xDEADBEEFCAFE0001 to match Rust.
    // Used for: CoinChoice, ReceiveChoice, KickBall, player selection at activation.
    private Xoshiro256StarStar decisionRng;
    private int decisionRngAdvances = 0;

    // Action diversity RNG: seeded with game seed ^ 0xC0FFEE_ACE0_0001 to match Rust's action_rng.
    // Used for: move target selection (adjacent square pick).
    private Xoshiro256StarStar actionRng;
    private long currentSeed = 0;

    // Per-turn activation tracking (matches Rust's eligible_this_turn and used_this_turn).
    private List<Object[]> eligibleThisTurn = new ArrayList<>();
    private Set<String> usedThisTurn = new HashSet<>();
    private String lastTurnKey = "";

    // True when the previous INIT_SELECTING phase 2 sent a deselect (non-Regular mode).
    private boolean justDeselected = false;

    // True once the blitz block command was sent for the current activation —
    // prevents re-sending CLIENT_BLOCK when INIT_SELECTING re-enters after the block.
    private boolean blitzBlockSent = false;
    // BB2016 blitz is a 3-command dance: CLIENT_ACTING_PLAYER(BLITZ_MOVE) → CLIENT_BLITZ_MOVE(path)
    // → CLIENT_BLOCK(target). This tracks whether the CLIENT_BLITZ_MOVE half has been sent for the
    // current activation (reset at each new phase-1 pick, alongside blitzBlockSent).
    private boolean bb2016BlitzMoveSent = false;

    private static final class PendingStep {
        int i;
        int turn;
        int half;
        String active;
        String stateHash;
        String chosen;
        String postHash = "";

        PendingStep(int i, int turn, int half, String active, String stateHash, String chosen) {
            this.i = i;
            this.turn = turn;
            this.half = half;
            this.active = active;
            this.stateHash = stateHash;
            this.chosen = chosen;
        }
    }

    public ParityRunner(PrintWriter out) {
        this.out = out;
    }

    // ── Entry point ───────────────────────────────────────────────────────────

    public static void main(String[] args) throws Exception {
        Logger.getLogger("").setLevel(Level.WARNING);
        Logger.getLogger("org.eclipse.jetty").setLevel(Level.OFF);

        // Extract --tier N and --seed-end M (anywhere in the arg list); remaining args stay positional.
        // --seed-end enables BATCH mode: run [seed .. seedEnd] in this single JVM, amortizing JVM
        // start-up, fat-jar class-loading and server construction across all seeds. The output path
        // is then a template containing "{seed}", substituted per seed.
        int tierArg = 2;
        long seedEndArg = -1;
        String rulesetArg = null;   // e.g. "BB2016"; null keeps the BB2025 default
        // Heuristic-agent ladder (AGENT_CONTRACT_HEURISTIC.md, docs/PARITY_HEURISTIC_CAMPAIGN.md).
        // `--agent heuristic` swaps the driver for the prompt classes named by `--heur-classes`;
        // every other class keeps the random contract untouched, so tier 2 and tier 3 stay
        // byte-identical to their historical behaviour when the flag is absent.
        String agentArg = "random";
        int multimoveArg = 0;
        float heurScaleArg = 0.0f;
        String heurClassesArg = "none";
        List<String> positional = new ArrayList<>();
        for (int i = 0; i < args.length; i++) {
            if ("--tier".equals(args[i]) && i + 1 < args.length) {
                tierArg = Integer.parseInt(args[++i]);
            } else if ("--seed-end".equals(args[i]) && i + 1 < args.length) {
                seedEndArg = Long.parseUnsignedLong(args[++i]);
            } else if ("--ruleset".equals(args[i]) && i + 1 < args.length) {
                rulesetArg = args[++i];
            } else if ("--agent".equals(args[i]) && i + 1 < args.length) {
                agentArg = args[++i];
            } else if ("--multimove".equals(args[i]) && i + 1 < args.length) {
                multimoveArg = Integer.parseInt(args[++i]);
            } else if ("--heur-scale".equals(args[i]) && i + 1 < args.length) {
                heurScaleArg = Float.parseFloat(args[++i]);
            } else if ("--heur-classes".equals(args[i]) && i + 1 < args.length) {
                heurClassesArg = args[++i];
            } else {
                positional.add(args[i]);
            }
        }
        args = positional.toArray(new String[0]);

        if (args.length < 3) {
            System.err.println("Usage: ParityRunner [serverDir] homeTeamId awayTeamId seed [output.jsonl] [--tier N]");
            System.exit(1);
        }

        File serverDir;
        String homeTeamId, awayTeamId;
        long seed;
        String outputPath = null;

        File possibleDir = new File(args[0]);
        if (args.length >= 4 && possibleDir.isDirectory()) {
            serverDir = possibleDir;
            homeTeamId = resolveTeamId(args[1]);
            awayTeamId = resolveTeamId(args[2]);
            seed = Long.parseUnsignedLong(args[3]);
            if (args.length > 4) outputPath = args[4];
        } else {
            File cwd = new File(System.getProperty("user.dir"));
            File candidate = new File(cwd.getParentFile(), "ffb-server");
            serverDir = candidate.exists() ? candidate : new File(cwd, "ffb-server");
            homeTeamId = resolveTeamId(args[0]);
            awayTeamId = resolveTeamId(args[1]);
            seed = Long.parseUnsignedLong(args[2]);
            if (args.length > 3) outputPath = args[3];
        }

        long seedEnd = (seedEndArg >= 0) ? seedEndArg : seed;

        // Build the server (and, with it, load classes / warm the JVM) ONCE, then reuse it for
        // every seed — this is what amortizes JVM start-up + fat-jar class-loading + server
        // construction across the whole batch. Each seed still gets a fresh GameState via
        // HeadlessGameSetup.create and re-seeds its own RNGs in run().
        HeadlessFantasyFootballServer server = new HeadlessFantasyFootballServer();
        Xoshiro256StarStar.traceEnabled = Boolean.getBoolean("ffb.diceTrace");

        for (long s = seed; s <= seedEnd; s++) {
            GameState gameState = HeadlessGameSetup.create(server, homeTeamId, awayTeamId, serverDir, rulesetArg);
            Xoshiro256StarStar rng = new Xoshiro256StarStar(s);
            server.getFortuna().setDelegate(rng);

            String path = outputPath;
            PrintWriter out;
            if (path != null) {
                if (path.contains("{seed}")) path = path.replace("{seed}", Long.toUnsignedString(s));
                out = new PrintWriter(new FileOutputStream(path), true);
            } else {
                out = new PrintWriter(new java.io.BufferedWriter(
                    new java.io.OutputStreamWriter(System.out, StandardCharsets.UTF_8)), true);
            }

            ParityRunner runner = new ParityRunner(out);
            runner.tier = tierArg;
            runner.multimove = multimoveArg;
            if ("heuristic".equals(agentArg)) {
                // Seeded from the game seed, exactly as the Rust side does: the heuristic stream is
                // `seed ^ "HEURISTI"`, independent of the game dice and of decisionRng/actionRng.
                runner.heuristic = new HeuristicDriver(
                    s, heurScaleArg, ClassMask.parse(heurClassesArg));
            } else if (!"random".equals(agentArg)) {
                System.err.println("--agent must be 'random' or 'heuristic', got: " + agentArg);
                System.exit(2);
            }
            runner.run(gameState, homeTeamId, awayTeamId, s);

            out.flush();
            if (path != null) out.close();
        }
    }

    // ── Game loop ─────────────────────────────────────────────────────────────

    /**
     * `java.util.Collections.shuffle(List)` (the ONE-ARG overload) draws from a private static
     * `Random` inside `java.util.Collections`, seeded from system entropy — so it is
     * non-deterministic WITHIN JAVA: the same parity seed can produce a different result on two
     * runs, and no Rust port could mirror it.
     *
     * The engine reaches it in BB2020 via
     * `StepApplyKickoffResult.handleCheeringFans` -> `Collections.shuffle(availablePrayerRolls)`,
     * which picks which Prayer to Nuffle the winning team receives. That path fires in a MIRROR
     * match because it compares two D6 rolls, not team values. (Same call in bb2020/bb2025
     * `StepPrayers`, `bb2025/StepThrowARock`, `bb2020/StepAssignTouchdowns` — currently
     * unreachable here, but the same hazard.)
     *
     * Rather than disable the feature, make Java reproducible: seed that shared field per game
     * from the parity seed. The engine is NOT modified — this is reflection from the harness, and
     * needs `--add-opens java.base/java.util=ALL-UNNAMED` (added by ffb-parity's runner.rs).
     * Rust reproduces the identical permutation with
     * `ffb_model::util::java_random::{JavaRandom, collections_shuffle}`, which are 1:1 ports of
     * `java.util.Random` and `Collections.shuffle` pinned in tests against real JVM output.
     *
     * NOTE: this is a SHARED stream for the whole game, like the dice stream — every shuffle call
     * draws from it in order, so Rust must shuffle at the same points and in the same sequence.
     */
    private static void seedCollectionsShuffleRng(long seed) {
        try {
            java.lang.reflect.Field f = java.util.Collections.class.getDeclaredField("r");
            f.setAccessible(true);
            f.set(null, new java.util.Random(seed ^ 0x5EEDC0113C7104L));
        } catch (ReflectiveOperationException | RuntimeException e) {
            // Loud, not silent: without this the run is non-deterministic and any "green" result
            // for a matchup that reaches a shuffle site is meaningless.
            System.err.println("FATAL: could not seed java.util.Collections' shuffle RNG: " + e
                + " — run the JVM with --add-opens java.base/java.util=ALL-UNNAMED");
            throw new IllegalStateException("Collections shuffle RNG not seeded", e);
        }
    }

    public void run(GameState gameState, String homeTeamId, String awayTeamId, long seed) {
        Game game = gameState.getGame();

        this.currentSeed = seed;
        this.decisionRng = new Xoshiro256StarStar(seed ^ 0xDEADBEEFCAFE0001L);
        this.actionRng = new Xoshiro256StarStar(seed ^ 0xC0FFEE_ACE0_0001L);
        seedCollectionsShuffleRng(seed);
        String initialHash = stateHash(game);
        out.println(String.format(
            "{\"i\":0,\"type\":\"game_start\",\"home\":\"%s\",\"away\":\"%s\",\"seed\":%d,\"state_hash\":\"%s\"}",
            escJson(homeTeamId), escJson(awayTeamId), seed, initialHash));

        MatchRunner.injectForTeam(gameState, new ClientCommandStartGame(), true);
        MatchRunner.injectForTeam(gameState, new ClientCommandStartGame(), false);

        int iter = 0;
        // Track same-dialog repetitions to detect and break infinite loops
        // (e.g., a dialog re-fires after its handler fails to properly advance the state)
        com.fumbbl.ffb.dialog.DialogId lastDialogId = null;
        int sameDialogCount = 0;
        // Track a step that stays current without advancing (no dialog) — an UNHANDLED_STEP whose
        // default EndTurn can't advance it (e.g. ANIMAL_SAVAGERY) otherwise spins to MAX_ITERATIONS
        // (2,000,000), which reads as a hang. Break out fast so the run yields a definitive result
        // instead of stalling the whole parity report.
        StepId lastStepId = null;
        int sameStepNoDialogCount = 0;
        String endReason = "finished";
        while (game.getFinished() == null && ++iter < MAX_ITERATIONS) {
            IStep step = gameState.getCurrentStep();
            if (step == null) { endReason = "step_stack_empty"; break; }

            IDialogParameter dialog = game.getDialogParameter();
            StepId stepId = step.getId();

            if (iter > 100000 && iter % 100000 < 8) {
                System.err.println("SPIN: iter=" + iter + " step=" + stepId
                    + " dialog=" + (dialog == null ? "null" : dialog.getId())
                    + " mode=" + game.getTurnMode() + " homePlaying=" + game.isHomePlaying());
            }

            // Safety: if the same dialog fires > 500 times consecutively, clear it
            if (dialog != null) {
                if (dialog.getId() == lastDialogId) {
                    if (++sameDialogCount > 500) {
                        System.err.println("DIALOG_LOOP_CLEARED: " + dialog.getId()
                            + " at step " + stepId + " half=" + game.getHalf());
                        game.setDialogParameter(null);
                        sameDialogCount = 0; lastDialogId = null; continue;
                    }
                } else { lastDialogId = dialog.getId(); sameDialogCount = 1; }
            } else { lastDialogId = null; sameDialogCount = 0; }

            // Safety: a no-dialog step that never advances = stuck (unhandled step). Break early.
            if (dialog == null && stepId == lastStepId) {
                if (++sameStepNoDialogCount > 500) {
                    System.err.println("STUCK_STEP: " + stepId + " unadvanced for "
                        + sameStepNoDialogCount + " iters — ending game to avoid a MAX_ITERATIONS spin");
                    break;
                }
            } else {
                lastStepId = (dialog == null) ? stepId : null;
                sameStepNoDialogCount = (dialog == null) ? 1 : 0;
            }

            if (dialog != null && stepId != StepId.INIT_SELECTING) {
                handleDialog(dialog, game, gameState);
            } else {
                handleStep(stepId, game, gameState);
            }
        }

        if (iter >= MAX_ITERATIONS) endReason = "max_iterations";
        System.err.println("END_REASON: " + endReason + " iter=" + iter
            + " half=" + game.getHalf() + " turnHome=" + game.getTurnDataHome().getTurnNr()
            + " turnAway=" + game.getTurnDataAway().getTurnNr());

        // Finalize: fill post_hashes and flush all pending steps
        String endHash = stateHash(game);
        for (int i = 0; i < pending.size(); i++) {
            pending.get(i).postHash = (i + 1 < pending.size())
                ? pending.get(i + 1).stateHash
                : endHash;
        }
        for (PendingStep s : pending) {
            out.println(String.format(
                "{\"i\":%d,\"type\":\"step\",\"turn\":%d,\"half\":%d,\"active\":\"%s\","
                + "\"dialog\":\"None\",\"state_hash\":\"%s\","
                + "\"actions\":[\"EndTurn\"],\"chosen\":\"%s\","
                + "\"dice\":[],\"post_hash\":\"%s\"}",
                s.i, s.turn, s.half, s.active, s.stateHash, s.chosen, s.postHash));
        }

        int scoreHome = game.getGameResult().getScoreHome();
        int scoreAway = game.getGameResult().getScoreAway();
        if (System.getenv("FFB_TRACE") != null) { System.err.println("JAVA_END state=" + stateString(game)); }
        out.println(String.format(
            "{\"i\":%d,\"type\":\"game_end\",\"home_score\":%d,\"away_score\":%d,\"state_hash\":\"%s\"}",
            stepIndex, scoreHome, scoreAway, endHash));
    }

    // ── Step handling ─────────────────────────────────────────────────────────

    private void handleStep(StepId stepId, Game game, GameState gameState) {
        switch (stepId) {

            case SETUP:
                resetCurrentTeam(game);
                placeReserves(game, gameState);
                MatchRunner.inject(gameState, new ClientCommandEndTurn(TurnMode.SETUP, null));
                break;

            case KICKOFF: {
                // Deterministic random kick coord — matches Rust ParityAgent.
                // Home kicks to away's half (x 13..25), away kicks to home's half (x 0..12).
                boolean home = game.isHomePlaying();
                // The heuristic agent SCORES every square of the receiving half (Rust
                // `AgentPrompt::KickBall`, T = 0.10) when the `kick` class is on, and spends NO
                // decisionRng doing it — one sampler draw replaces the two uniform draws below.
                if (heuristic != null
                    && heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.KICK_BALL)) {
                    FieldCoordinate hk = heuristic.kickBall(game);
                    MatchRunner.inject(gameState,
                        new ClientCommandKickoff(home ? hk : hk.transform()));
                    break;
                }
                decisionRngAdvances++;
                int xRaw = (int) Long.remainderUnsigned(decisionRng.nextLong(), 13L);
                decisionRngAdvances++;
                int yRaw = (int) Long.remainderUnsigned(decisionRng.nextLong(), 13L);
                int x = home ? xRaw + 13 : xRaw;
                int y = yRaw + 1;
                FieldCoordinate kickCoord = new FieldCoordinate(x, y);
                MatchRunner.inject(gameState, new ClientCommandKickoff(home ? kickCoord : kickCoord.transform()));
                break;
            }

            case APPLY_KICKOFF_RESULT:
                MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                break;

            case SWOOP:
                sendSwoopTarget(game, gameState);
                break;

            case INIT_PUNT:
                sendPuntTarget(game, gameState);
                break;

            // Bomb re-throw window (TurnMode BOMB_HOME / BOMB_AWAY): after a bomb is caught the
            // catcher becomes the acting player with THROW_BOMB and StepInitPassing PARKS with a
            // null thrower, waiting for a client command naming the new target. NOTHING DECLARED
            // this action -- the engine set it -- so the phase-1/phase-2 activation loop is never
            // entered for it, and the step spun forever as UNHANDLED_STEP. That is the whole reason
            // "phase 2 is never reached for THROW_BOMB": the re-throw has no declaration at all.
            case INIT_PASSING: {
                // StepInitPassing.executeStep RETURNS IMMEDIATELY while the thrower is unset, so a
                // decline is impossible here: neither CLIENT_END_TURN nor a deselect can advance the
                // step (both set their flag and are then swallowed by that early return). The only
                // command that advances it is CLIENT_PASS, which sets thrower = acting player. So the
                // bomb must actually be thrown, using the SAME candidate rule and the SAME single
                // actionRng draw as an ordinary pass -- Rust's agent mirrors it on the BombRethrow
                // prompt via legal_pass_receivers.
                // Gate on thrower==null: that is the RE-THROW park (no CLIENT_PASS seen yet).
                // A park with the thrower SET is an OUT-OF-RANGE declared pass/bomb --
                // InitPassing's CLIENT_PASS/TARGET_COORDINATE handlers set the thrower before the
                // range check refuses to advance -- and the established contract for that park is
                // END_TURN (zero dice), which Rust's InitPassing refusal also produces (underworld
                // seed 72). Redrawing unconditionally made Java re-pick an out-of-range target
                // until it landed in range (goblin bb2016 seed 21: two rejected picks, then a
                // full bomb chain Rust never ran).
                ActingPlayer bombAp = game.getActingPlayer();
                if (game.getThrower() == null && bombAp != null && bombAp.getPlayerId() != null) {
                    sendPassAction(game, gameState, bombAp.getPlayerId());
                } else {
                    MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                }
                break;
            }

            case HIT_AND_RUN:
                sendHitAndRunTarget(game, gameState);
                break;

            case THEN_I_STARTED_BLASTIN:
                // Turn-mode wait, no dialog: the coach answers with CLIENT_TARGET_SELECTED.
                // Fires for BOTH waits — the initial pick (source = the acting star,
                // opponents only) and the roll-2 replacement pick (source = the ORIGINAL
                // target, either team, star excluded; the opposing coach chooses). Candidate
                // rule mirrors the client's ThenIStartedBlastinLogicModule.isValidTarget;
                // answer contract mirrors the Rust agent: coordinate-sorted, single
                // actionRng pick, EndTurn when empty.
                sendBlastinTarget(game, gameState);
                break;

            case RAIDING_PARTY:
                // Square wait (no dialog): StepRaidingParty published the eligible squares as
                // MoveSquares and waits for CLIENT_FIELD_COORDINATE. Same contract as
                // sendPuntTarget/sendHitAndRunTarget: coordinate-sorted, single actionRng pick,
                // mirrored for the away coach (the server un-mirrors).
                sendRaidingPartyTarget(game, gameState);
                break;

            case FIRST_MOVE_FURIOUS_OUTBURST:
            case SECOND_MOVE_FURIOUS_OUTBURST:
                // Square waits (no dialog), exactly like RAIDING_PARTY above:
                // StepFirstMoveFuriousOutburst publishes the empty squares adjacent to the stab
                // target as MoveSquares and waits for CLIENT_FIELD_COORDINATE; the second move
                // publishes the squares within 3 of the new position and waits again. Same
                // contract: coordinate-sorted, single actionRng pick, mirrored for the away coach.
                sendFuriousOutburstSquare(game, gameState);
                break;

            case INIT_SELECTING: {
                ActingPlayer ap = game.getActingPlayer();
                if (ap == null || ap.getPlayerId() == null) {
                    boolean homePlaying = game.isHomePlaying();
                    int turn = homePlaying ? game.getTurnDataHome().getTurnNr()
                                          : game.getTurnDataAway().getTurnNr();
                    if (turn < 1) {
                        MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                        break;
                    }
                    // Build turn key to detect new turns (same as Rust's last_turn_key).
                    String turnKey = game.getHalf() + ":" + turn + ":" + homePlaying;
                    if (!turnKey.equals(lastTurnKey)) {
                        // New turn: compute and save full eligible list in roster order.
                        lastTurnKey = turnKey;
                        eligibleThisTurn = computeEligiblePlayers(game);
                        usedThisTurn.clear();
                    }
                    // Non-Regular modes (kickoff Blitz!, QuickSnap): one activation then EndTurn.
                    if (game.getTurnMode() != TurnMode.REGULAR && !usedThisTurn.isEmpty()) {
                        justDeselected = true;
                        MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                        break;
                    }
                    // Build remaining eligible (roster order, same as Rust's eligible_this_turn Vec).
                    List<Object[]> remaining = new ArrayList<>();
                    for (Object[] entry : eligibleThisTurn) {
                        String pid = (String) entry[0];
                        if (!usedThisTurn.contains(pid)) {
                            remaining.add(entry);
                        }
                    }
                    // Pick with inactive-skip loop (AGENT_CONTRACT.md §2.4): a pick that
                    // lands on a just-unstunned player (PlayerState.isActive() == false)
                    // consumes its decisionRng call but is rejected; re-pick from the
                    // shrunk remaining list. Tier 2 never rejects (no action is ever
                    // started, so the server-side SKIP never triggers) — the loop runs
                    // once there, preserving the historical T2 RNG pattern.
                    while (true) {
                        if (remaining.isEmpty() || justDeselected) {
                            // All players used for this turn, or non-Regular mode reset: EndTurn.
                            justDeselected = false;
                            usedThisTurn.clear();
                            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                            break;
                        }
                        // Pick random player using decisionRng (matches Rust's decision_rng.pick()).
                        int pi = (int) Long.remainderUnsigned(decisionRng.nextLong(), remaining.size());
                        decisionRngAdvances++;
                        Object[] entry = remaining.remove(pi);
                        String playerId = (String) entry[0];
                        if (DEBUG_ACT) System.err.println("JAVA_PPICK pid=" + playerId + " pick=" + pi
                            + " N=" + (remaining.size() + 1));
                        PlayerAction[] actions = (PlayerAction[]) entry[1];
                        usedThisTurn.add(playerId);
                        if (tier >= 3) {
                            Player<?> picked = game.getPlayerById(playerId);
                            PlayerState pickedState = (picked != null)
                                ? game.getFieldModel().getPlayerState(picked) : null;
                            if (pickedState != null && !pickedState.isActive()) {
                                if (DEBUG) System.err.println("SKIP_INACTIVE pid=" + playerId);
                                continue; // rejected pick — decisionRng call consumed, no step logged
                            }
                        }
                        // Action pick (AGENT_CONTRACT.md §5): tier 3 first drops snapshot
                        // actions consumed earlier this turn (the eligible list is captured
                        // at turn start), then consumes exactly 1 actionRng call over the
                        // remaining list (even when length 1). Tier 2 keeps actions[0]
                        // (historical T2 RNG pattern).
                        PlayerAction action;
                        if (tier >= 3) {
                            PlayerAction[] live = filterStaleActions(game, actions);
                            int ai = (int) Long.remainderUnsigned(actionRng.nextLong(), live.length);
                            action = live[ai];
                            if (DEBUG_ACT) {
                                StringBuilder sb = new StringBuilder();
                                for (PlayerAction a : live) { if (sb.length() > 0) sb.append(','); sb.append(a); }
                                System.err.println("JAVA_ACT_PICK pid=" + playerId + " N=" + live.length
                                    + " idx=" + ai + " action=" + action + " live=[" + sb + "]"
                                    + " snapshot=" + actions.length);
                            }
                        } else {
                            action = actions[0];
                        }
                        // sendConcreteAction handles only MOVE/STAND_UP/BLOCK/BLITZ*/FOUL*/PASS*/
                        // HAND_OVER*/THROW_TEAM_MATE*; everything else hits its `default:` arm and is
                        // immediately DESELECTED without touching the game state. Recording a step for
                        // such an activation logged a phantom no-op step (identical pre/post hash) that
                        // the Rust agent, which re-picks inside its own loop, never produces -- shifting
                        // every later step index by one (goblin bb2016 seed 1 i=2: the Bombardier's
                        // THROW_BOMB). Skip it the same way an inactive pick is skipped above: the
                        // decisionRng/actionRng calls are already consumed and the player is already in
                        // usedThisTurn, so the next iteration simply picks someone else.
                        if (!isHandledActingAction(action)) {
                            System.err.println("UNHANDLED_ACTING_ACTION_AT_PICK: " + action + " pid=" + playerId
                                + " -- deselecting, no step logged");
                            continue;
                        }
                        String chosen = "Activate(" + playerId + "," + action.toString() + ")";
                        recordStep(game, chosen, gameState.getDiceRoller().getCallCount());
                        blitzBlockSent = false;
                        bb2016BlitzMoveSent = false;
                        // BLITZ is declared as BLITZ_MOVE: StepInitSelecting dispatches it to
                        // BLITZ_SELECT (target selection sets blitzUsed), then the block is
                        // sent from phase 2 once the target selection state exists.
                        // TREACHEROUS declares as the client's command pair: the acting player
                        // is set with PASS_MOVE (a ball action Treacherous itself adds to the
                        // menu), then CLIENT_USE_SKILL(treacherous) — StepInitSelecting's skill
                        // chain turns that into fDispatchPlayerAction=TREACHEROUS+forceGoto.
                        // BLACK_INK declares as ActingPlayer(MOVE) + UseSkill(blackInk); the
                        // engine dispatches BLACK_INK via the InitSelecting skill chain, and the
                        // player continues the MOVE afterwards (phase 2 sendMoveAction).
                        if (action == PlayerAction.BLACK_INK) {
                            Player<?> bPlayer = game.getPlayerById(playerId);
                            com.fumbbl.ffb.model.skill.Skill blackInk =
                                bPlayer.getSkillWithProperty(com.fumbbl.ffb.model.property.NamedProperties.canGazeAutomatically);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                blackInk, true, playerId, null, false));
                            break;
                        }
                        if (action == PlayerAction.THEN_I_STARTED_BLASTIN) {
                            Player<?> zPlayer = game.getPlayerById(playerId);
                            java.util.Optional<com.fumbbl.ffb.model.skill.Skill> blastin =
                                com.fumbbl.ffb.util.UtilCards.getUnusedSkillWithProperty(zPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canBlastRemotePlayer);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            if (blastin.isPresent()) {
                                MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                    blastin.get(), true, playerId, null, false));
                            }
                            break;
                        }
                        if (action == PlayerAction.CATCH_OF_THE_DAY) {
                            Player<?> cPlayer = game.getPlayerById(playerId);
                            java.util.Optional<com.fumbbl.ffb.model.skill.Skill> cotd =
                                com.fumbbl.ffb.util.UtilCards.getUnusedSkillWithProperty(cPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canGetBallOnGround);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            if (cotd.isPresent()) {
                                MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                    cotd.get(), true, playerId, null, false));
                            }
                            break;
                        }
                        if (action == PlayerAction.BALEFUL_HEX) {
                            Player<?> bPlayer = game.getPlayerById(playerId);
                            java.util.Optional<com.fumbbl.ffb.model.skill.Skill> hex =
                                com.fumbbl.ffb.util.UtilCards.getUnusedSkillWithProperty(bPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canMakeOpponentMissTurn);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            if (hex.isPresent()) {
                                MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                    hex.get(), true, playerId, null, false));
                            }
                            break;
                        }
                        if (action == PlayerAction.LOOK_INTO_MY_EYES) {
                            Player<?> lPlayer = game.getPlayerById(playerId);
                            // getUnusedSkillWithProperty(Player,..) returns Optional — never null-check it.
                            java.util.Optional<com.fumbbl.ffb.model.skill.Skill> lime =
                                com.fumbbl.ffb.util.UtilCards.getUnusedSkillWithProperty(lPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canStealBallFromOpponent);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            if (lime.isPresent()) {
                                MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                    lime.get(), true, playerId, null, false));
                            }
                            break;
                        }
                        if (action == PlayerAction.AUTO_GAZE_ZOAT) {
                            // Declared exactly like BLACK_INK: ActingPlayer(MOVE) then
                            // ClientCommandUseSkill(zoat). Java's CLIENT_USE_SKILL chain keys on
                            // canGazeAutomaticallyThreeSquaresAway and dispatches AUTO_GAZE_ZOAT
                            // with forceGotoOnDispatch, never calling changeActingPlayer.
                            Player<?> zPlayer = game.getPlayerById(playerId);
                            java.util.Optional<com.fumbbl.ffb.model.skill.Skill> zoat =
                                com.fumbbl.ffb.util.UtilCards.getUnusedSkillWithProperty(zPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canGazeAutomaticallyThreeSquaresAway);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            if (zoat.isPresent()) {
                                MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                    zoat.get(), true, playerId, null, false));
                            }
                            break;
                        }
                        if (action == PlayerAction.WISDOM_OF_THE_WHITE_DWARF) {
                            // The client offers WISDOM from the ordinary action modules and sends
                            // sendUseWisdom() -> ClientCommandUseTeamMatesWisdom. Java's
                            // StepInitSelecting handles it by setting ONLY the dispatch action
                            // (fDispatchPlayerAction = WISDOM_OF_THE_WHITE_DWARF,
                            // forceGotoOnDispatch = true) and never calls changeActingPlayer, so
                            // the declared action stays MOVE. Same command pair as BLACK_INK;
                            // Rust folds it into one ActivatePlayer and bridges identically.
                            MatchRunner.inject(gameState,
                                new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            MatchRunner.inject(gameState,
                                new com.fumbbl.ffb.net.commands.ClientCommandUseTeamMatesWisdom());
                            break;
                        }
                        if (action == PlayerAction.THROW_KEG) {
                            // The client declares a keg in TWO commands: sendActingPlayer(player,
                            // THROW_KEG) puts it in ClientStateId.THROW_KEG, then the coach clicks a
                            // target and sendThrowKeg(target) follows. Both are handled by
                            // StepInitSelecting, the second publishing TARGET_PLAYER_ID.
                            // Targets are ThrowKegLogicModule.isValidTarget: distance <= 3, base
                            // STANDING, opposing team. Coordinate-sorted, single actionRng pick —
                            // Rust's agent folds the same pair into one ActivatePlayer carrying the
                            // target and draws identically.
                            FieldModel kegFm = game.getFieldModel();
                            FieldCoordinate kegCoord = playerCoordinate(game, playerId);
                            Team kegOpponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
                            List<Player<?>> kegTargets = new ArrayList<>();
                            if (kegCoord != null) {
                                for (Player<?> op : kegOpponent.getPlayers()) {
                                    FieldCoordinate oc = kegFm.getPlayerCoordinate(op);
                                    PlayerState ops = kegFm.getPlayerState(op);
                                    if (oc == null || ops == null) continue;
                                    if (oc.distanceInSteps(kegCoord) <= 3
                                            && ops.getBase() == PlayerState.STANDING) {
                                        kegTargets.add(op);
                                    }
                                }
                            }
                            sortPlayersByCoordinate(kegTargets, kegFm);
                            if (kegTargets.isEmpty()) {
                                // No square the coach could click — the declaration never completes.
                                if (DEBUG) System.err.println("JAVA_KEG_DESELECT pid=" + playerId
                                    + " (no valid keg target)");
                                MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                                break;
                            }
                            int kegIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), kegTargets.size());
                            Player<?> kegTarget = kegTargets.get(kegIdx);
                            if (DEBUG) System.err.println("JAVA_KEG_PICK pid=" + playerId + " N="
                                + kegTargets.size() + " idx=" + kegIdx + " target=" + kegTarget.getId());
                            MatchRunner.inject(gameState,
                                new ClientCommandActingPlayer(playerId, PlayerAction.THROW_KEG, false));
                            MatchRunner.inject(gameState,
                                new com.fumbbl.ffb.net.commands.ClientCommandThrowKeg(kegTarget.getId()));
                            break;
                        }
                        if (action == PlayerAction.FURIOUS_OUTPBURST) {
                            // The turn-start eligible snapshot goes STALE: the team's blitz can be
                            // spent, or the star's targets can walk out of range, between the
                            // snapshot and this declaration. Declaring it anyway makes the STOCK
                            // ENGINE THROW — every abort path in the bb2025 FuriousOutburst
                            // sequence jumps to the `END` label, which IS StepEndFuriousOutburst,
                            // and that step dereferences
                            // fieldModel.getTargetSelectionState().getSelectedPlayerId()
                            // unconditionally (NPE at StepEndFuriousOutburst:71; it killed the
                            // batched JVM at wood_elf bb2025 seed 65 i=190, state `f1000,0000`).
                            // The real client never gets there because SelectLogicModule
                            // re-evaluates isFuriousOutburstAvailable at click time. Re-check it
                            // here and DESELECT when it no longer holds — the same treatment
                            // sendFoulAction gives a foul whose victim has moved. Rust's agent
                            // mirrors this in random_agent's 'reselect loop.
                            Player<?> foPlayer = game.getPlayerById(playerId);
                            FieldModel foFm = game.getFieldModel();
                            PlayerState foPs = (foPlayer == null) ? null : foFm.getPlayerState(foPlayer);
                            FieldCoordinate foCoord = (foPlayer == null) ? null : foFm.getPlayerCoordinate(foPlayer);
                            Team foOpponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
                            boolean stillAvailable = foPlayer != null && foPs != null && foCoord != null
                                && foPs.isActive() && foPs.getBase() == PlayerState.STANDING
                                && !game.getTurnData().isBlitzUsed()
                                && com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(foPlayer,
                                    com.fumbbl.ffb.model.property.NamedProperties.canTeleportBeforeAndAfterAvRollAttack)
                                && com.fumbbl.ffb.util.ArrayTool.isProvided(
                                    com.fumbbl.ffb.util.UtilPlayer.findBlockablePlayers(game, foOpponent, foCoord, 3));
                            if (!stillAvailable) {
                                if (DEBUG) System.err.println("JAVA_FO_DESELECT pid=" + playerId
                                    + " (stale turn-start offer)");
                                MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                                break;
                            }
                            MatchRunner.inject(gameState,
                                new ClientCommandActingPlayer(playerId, PlayerAction.FURIOUS_OUTPBURST, false));
                            break;
                        }
                        if (action == PlayerAction.RAIDING_PARTY) {
                            Player<?> rPlayer = game.getPlayerById(playerId);
                            com.fumbbl.ffb.model.skill.Skill raiding =
                                rPlayer.getSkillWithProperty(com.fumbbl.ffb.model.property.NamedProperties.canMoveOpenTeamMate);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.MOVE, false));
                            MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                raiding, true, playerId, null, false));
                            break;
                        }
                        if (action == PlayerAction.TREACHEROUS) {
                            Player<?> tPlayer = game.getPlayerById(playerId);
                            com.fumbbl.ffb.model.skill.Skill treacherous =
                                tPlayer.getSkillWithProperty(com.fumbbl.ffb.model.property.NamedProperties.canStabTeamMateForBall);
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, PlayerAction.PASS_MOVE, false));
                            MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandUseSkill(
                                treacherous, true, playerId, null, false));
                            break;
                        }
                        PlayerAction declared = (action == PlayerAction.BLITZ) ? PlayerAction.BLITZ_MOVE : action;
                        MatchRunner.inject(gameState, new ClientCommandActingPlayer(playerId, declared, false));
                        break;
                    }
                } else {
                    // Player is activated (phase 2).
                    // Tier 2 and non-Regular modes (Blitz!, QuickSnap): immediately deselect;
                    // justDeselected makes the next phase-1 visit EndTurn (one pick per turn).
                    if (tier <= 2 || game.getTurnMode() != TurnMode.REGULAR) {
                        justDeselected = true;
                        MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                    } else {
                        sendConcreteAction(game, gameState);
                    }
                }
                break;
            }

            case KICKOFF_RETURN:
                MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                break;

            case INIT_MOVING: {
                // After StepInitSelecting dispatched CLIENT_MOVE, the move sequence stops here
                // waiting for the end-of-move signal. Send deselect to end the player's
                // activation and return INIT_SELECTING to the next player (same team turn).
                ActingPlayer imAp = game.getActingPlayer();
                String imPid = (imAp != null) ? imAp.getPlayerId() : "null";
                if (DEBUG) System.err.println("JAVA_IM pid=" + imPid + " si=" + stepIndex);
                MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                break;
            }

            case PUSHBACK:
                sendPushback(game, gameState);
                break;

            case SELECT_BLITZ_TARGET:
                sendBlitzTargetSelection(game, gameState);
                break;

            case END_TURN:
                // Send deselect for StepEndTurn — matches the original default handler
                // behavior that T1/T2 seeds 1-10 were verified against. Do NOT send EndTurn
                // here (EndTurn causes goblin SW ejection to loop), and do NOT omit the
                // injection (omitting causes T1b halftime divergence).
                MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                break;

            case INIT_THROW_TEAM_MATE: {
                // The thrown player was picked up (phase 2 sent it at INIT_SELECTING) and the step is
                // now waiting for the throw target — send the deterministic target square.
                ActingPlayer ttmAp = game.getActingPlayer();
                String ttmPid = (ttmAp != null) ? ttmAp.getPlayerId() : null;
                if (ttmPid == null) {
                    MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                } else {
                    sendThrowTeamMateTarget(game, gameState, ttmPid);
                }
                break;
            }

            default:
                System.err.println("UNHANDLED_STEP: " + stepId + " turnMode=" + game.getTurnMode());
                MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
                break;
        }
    }

    // ── Dialog handling ───────────────────────────────────────────────────────

    private void handleDialog(IDialogParameter dialog, Game game, GameState gameState) {
        // The heuristic agent gets first refusal on every dialog. It answers only the classes in
        // its mask and returns false otherwise, so an unported class keeps the random contract --
        // which is what makes the ladder gateable one class at a time.
        if (heuristic != null && heuristic.tryDialog(dialog, game, gameState)) {
            return;
        }
        if (System.getenv("FFB_DLG_TRACE") != null) System.err.println("JAVA_DLG " + dialog.getId()
            + " mode=" + game.getTurnMode());
        switch (dialog.getId()) {

            // ── Informational / clear-only dialogs ──────────────────────────
            // INFORMATION_OKAY (e.g. Look Into My Eyes failure notice) is purely
            // informational — the step has already NEXT_STEPped. Rust's agent
            // acknowledges it without an RNG draw; clearing mirrors that exactly.
            case INFORMATION_OKAY:
            case KICKOFF_RETURN:
            case SETUP_ERROR:
            case SWARMING_ERROR:
            case INVALID_SOLID_DEFENCE:
                game.setDialogParameter(null);
                break;

            // ── Blitz target selection dialog: same handling as the step ─────
            // (the server shows a dialog while StepSelectBlitzTarget waits; falling
            // to RandomStrategy here would be non-seeded nondeterminism)
            case SELECT_BLITZ_TARGET:
                sendBlitzTargetSelection(game, gameState);
                break;

            // ── Seeded coin / receive (original parity behavior) ────────────
            case COIN_CHOICE:
                decisionRngAdvances++;
                MatchRunner.inject(gameState, new ClientCommandCoinChoice(
                    Long.remainderUnsigned(decisionRng.nextLong(), 2L) == 0));
                break;

            case RECEIVE_CHOICE: {
                decisionRngAdvances++;
                boolean receive = Long.remainderUnsigned(decisionRng.nextLong(), 2L) == 0;
                MatchRunner.inject(gameState, new ClientCommandReceiveChoice(receive));
                break;
            }

            // -- Interception: pick a candidate, coordinate-sorted, 1 actionRng pick --
            // Mirrors Rust's AgentPrompt::Interception arm exactly. The candidate set comes from the
            // ENGINE's own UtilPassing.findInterceptors - the same call StepIntercept makes - so the
            // harness cannot drift from the engine it is testing. Sorted by COORDINATE, never by id:
            // the two engines' player ids differ.
            case INTERCEPTION: {
                sendInterceptorChoice(game, gameState);
                break;
            }

            // ── Touchback: nearest player to kick-from (13,8) ───────────────
            case TOUCHBACK: {
                boolean homeReceives = !game.isHomePlaying();
                // The heuristic agent SCORES the receiver (Rust `AgentPrompt::Touchback`,
                // T = 0.20) when the `touchback` class is on. Note it enumerates a DIFFERENT
                // eligible set than the nearest-to-kick-from rule below -- on-pitch players WITH
                // TACKLE ZONES, which is what StepTouchback builds -- so it must not share this
                // block's candidate loop.
                if (heuristic != null
                    && heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.TOUCHBACK)) {
                    Player<?> chosen = heuristic.touchback(game);
                    if (chosen == null) {
                        game.setDialogParameter(null);
                    } else {
                        FieldCoordinate hc = game.getFieldModel().getPlayerCoordinate(chosen);
                        MatchRunner.injectForTeam(gameState,
                            new ClientCommandTouchback(homeReceives ? hc : hc.transform()),
                            homeReceives);
                    }
                    break;
                }
                Team recvTeam = homeReceives ? game.getTeamHome() : game.getTeamAway();
                FieldCoordinate kickFrom = new FieldCoordinate(13, 8);
                FieldCoordinate bestCoord = null;
                int bestDist = Integer.MAX_VALUE;
                for (Player<?> p : recvTeam.getPlayers()) {
                    PlayerState ps = game.getFieldModel().getPlayerState(p);
                    FieldCoordinate coord = game.getFieldModel().getPlayerCoordinate(p);
                    boolean onPitch = coord != null && coord.getX() >= 0 && coord.getX() <= 25
                                                     && coord.getY() >= 0 && coord.getY() <= 14;
                    if (ps != null && ps.isStanding() && onPitch) {
                        int dx = coord.getX() - kickFrom.getX();
                        int dy = coord.getY() - kickFrom.getY();
                        int dist = dx * dx + dy * dy;
                        if (dist < bestDist) { bestDist = dist; bestCoord = coord; }
                    }
                }
                if (bestCoord != null) {
                    FieldCoordinate cmdCoord = homeReceives ? bestCoord : bestCoord.transform();
                    MatchRunner.injectForTeam(gameState, new ClientCommandTouchback(cmdCoord), homeReceives);
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            // ── All in-game choice dialogs: deterministic, no decisionRng ──────
            // T1/T2 parity requires zero decisionRng consumption for any event that
            // fires only on some seeds (skill-use, rerolls, block dice, apothecary).
            // One mismatched call shifts every subsequent decision, causing failure on
            // ~8-16% of seeds depending on team composition.
            //
            // Rules: always decline rerolls/skills/apothecary; always pick die 0;
            // always no follow-up; always argue with first available player.
            // These choices are consistent with Rust's deterministic Confirm behavior.
            //
            // For T3 (full game play) these would become seeded-random decisions once
            // both engines handle the complete in-turn sequence identically.

            case BLOCK_ROLL: {
                // The heuristic agent SCORES the block dice (Rust `AgentPrompt::BlockChoice`,
                // T = 0.12) when the `blockchoice` class is on; otherwise die index 0, which is
                // what the byte-matched random contract picks.
                com.fumbbl.ffb.dialog.DialogBlockRollParameter br =
                    (com.fumbbl.ffb.dialog.DialogBlockRollParameter) dialog;
                int brIdx = heuristicBlockChoice(game, br.getNrOfDice(), br.getBlockRoll());
                comm.clearCaptured();
                comm.sendBlockChoice(brIdx);
                injectCaptured(dialog, game, gameState);
                break;
            }

            case BLOCK_ROLL_PARTIAL_RE_ROLL: {
                // This is BB2020's block-roll dialog (bb2020 StepBlockRoll shows
                // DialogBlockRollPartialReRollParameter where bb2016 shows DialogBlockRollParameter
                // and bb2025 shows DialogBlockRollPropertiesParameter). All three map to the SAME
                // Rust prompt -- the shared step's `AgentPrompt::BlockChoice` -- so all three take
                // the die index from the heuristic agent, or 0 when the class is off.
                com.fumbbl.ffb.dialog.DialogBlockRollPartialReRollParameter bpr =
                    (com.fumbbl.ffb.dialog.DialogBlockRollPartialReRollParameter) dialog;
                int bprIdx = heuristicBlockChoice(game, bpr.getNrOfDice(), bpr.getBlockRoll());
                comm.clearCaptured();
                comm.sendBlockChoice(bprIdx);
                injectCaptured(dialog, game, gameState);
                break;
            }

            case BLOCK_ROLL_PROPERTIES: {
                // BB2025 block roll: the step waits for CLIENT_BLOCK_CHOICE. Never use a reroll
                // here (AGENT_CONTRACT.md §7) — a reroll-decline would just re-show the dialog
                // forever. The die index comes from the heuristic agent when `blockchoice` is on,
                // and is 0 otherwise. Rust raises the SAME `AgentPrompt::BlockChoice` for this
                // dialog as for BLOCK_ROLL; there is no separate "properties" prompt.
                com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter brp =
                    (com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter) dialog;
                int brpIdx = heuristicBlockChoice(game, brp.getNrOfDice(), brp.getBlockRoll());
                comm.clearCaptured();
                comm.sendBlockChoice(brpIdx);
                injectCaptured(dialog, game, gameState);
                break;
            }

            case FOLLOWUP_CHOICE: {
                // Always decline follow-up — deterministic, matches Rust UseReRoll(false).
                comm.clearCaptured();
                comm.sendFollowupChoice(false);
                injectCaptured(dialog, game, gameState);
                break;
            }

            case BLOODLUST_ACTION: {
                // Vampire failed Blood Lust. Deterministic parity: keep the declared action
                // (change=false), matching the Rust agent. FFB's RandomStrategy would answer this
                // with an UNSEEDED Random (RANDOM.nextBoolean()) — not reproducible — so drive it
                // explicitly here instead of falling through to the default dialog handler.
                String bltTeamId = getDialogTeamId(dialog);
                com.fumbbl.ffb.net.commands.ClientCommandBloodlustAction bltCmd =
                    new com.fumbbl.ffb.net.commands.ClientCommandBloodlustAction(false);
                if (bltTeamId != null) {
                    MatchRunner.injectForTeam(gameState, bltCmd, bltTeamId.equals(game.getTeamHome().getId()));
                } else {
                    MatchRunner.inject(gameState, bltCmd);
                }
                break;
            }

            case PETTY_CASH: {
                // bb2016 StepPettyCash blocks on a ClientCommandPettyCash whenever a team's
                // treasury is >= 50k and FORCE_TREASURY_TO_PETTY_CASH is off (goblin 80k,
                // halfling 180k). Transfer 0 — deterministic, no RNG, and matches
                // RandomStrategy.respondToDialog(PETTY_CASH). The step raises the dialog once
                // per team, so it MUST be injected for the team named in the dialog parameter,
                // otherwise UtilServerSteps.checkCommandIsFromHomePlayer keeps crediting the
                // home team and the other team's dialog re-fires forever.
                String pcTeamId = getDialogTeamId(dialog);
                com.fumbbl.ffb.net.commands.ClientCommandPettyCash pcCmd =
                    new com.fumbbl.ffb.net.commands.ClientCommandPettyCash(0);
                if (pcTeamId != null) {
                    MatchRunner.injectForTeam(gameState, pcCmd, pcTeamId.equals(game.getTeamHome().getId()));
                } else {
                    MatchRunner.inject(gameState, pcCmd);
                }
                break;
            }

            case RE_ROLL_BLOCK_FOR_TARGETS: {
                // Multi-block die selection (DialogReRollBlockForTargetsParameter, shown by
                // StepBlockRollMultiple): pick DIE INDEX 0 for the first still-unselected
                // target, no pro (-1), no reroll — mirrors Rust's headless auto-select-0.
                // The Dauntless/FoulAppearance variant (DialogReRollForTargetsParameter)
                // declines with a null-source UseReRollForTarget — mirrors Rust's
                // ReRollForTargets arm.
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsParameter) {
                    com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsParameter bt =
                        (com.fumbbl.ffb.dialog.DialogReRollBlockForTargetsParameter) dialog;
                    com.fumbbl.ffb.model.BlockRoll pick = null;
                    for (com.fumbbl.ffb.model.BlockRoll roll : bt.getBlockRolls()) {
                        if (roll.needsSelection()) { pick = roll; break; }
                    }
                    if (pick != null) {
                        MatchRunner.inject(gameState,
                            new com.fumbbl.ffb.net.commands.ClientCommandBlockOrReRollChoiceForTarget(
                                pick.getTargetId(), 0, -1, null));
                    } else {
                        game.setDialogParameter(null);
                    }
                } else if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollForTargetsParameter) {
                    com.fumbbl.ffb.dialog.DialogReRollForTargetsParameter rt =
                        (com.fumbbl.ffb.dialog.DialogReRollForTargetsParameter) dialog;
                    MatchRunner.inject(gameState,
                        new com.fumbbl.ffb.net.commands.ClientCommandUseReRollForTarget(
                            rt.getReRolledAction(), null, null));
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }
            case RE_ROLL:
            case RE_ROLL_PROPERTIES: {
                // Always decline — deterministic. No game RNG consumed for the declined roll.
                comm.clearCaptured();
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollParameter) {
                    com.fumbbl.ffb.dialog.DialogReRollParameter rr =
                        (com.fumbbl.ffb.dialog.DialogReRollParameter) dialog;
                    comm.sendUseReRoll(rr.getReRolledAction(),
                        reRollSourceFor(game, rr.getReRolledAction(), rr.isTeamReRollOption()));
                } else if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter) {
                    com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter rp =
                        (com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter) dialog;
                    comm.sendUseReRoll(rp.getReRolledAction(),
                        reRollSourceFor(game, rp.getReRolledAction(), true));
                } else {
                    game.setDialogParameter(null);
                    break;
                }
                injectCaptured(dialog, game, gameState);
                // Clear the answered dialog ONLY if it is still the same object: a step that
                // stays on CONTINUE after the decline (StepThenIStartedBlastin.fail() roll=2
                // -> flip + CONTINUE) never hides it, so the stale RE_ROLL_PROPERTIES re-fired
                // 500x (chaos_dwarf bb2025 seed 6 i=90). But the injection can synchronously
                // run the WHOLE turn end and show a NEW dialog (the half-boundary
                // ARGUE_THE_CALL) - clearing unconditionally wiped that and stuck END_TURN
                // for 501 iters (seed 2). Same lesson as RAIDING_PARTY, plus the same-object
                // guard.
                if (game.getDialogParameter() == dialog) {
                    game.setDialogParameter(null);
                }
                break;
            }

            case SKILL_USE: {
                // Always USE the skill — matches Rust engine which auto-uses Sure Hands/Catch
                // during catch/pickup attempts without emitting a SkillUse prompt.
                // Declining caused divergence: Java skips the reroll die, Rust consumes it.
                // Using on both sides: both consume the extra die → states match.
                // EXCEPTION: Dump Off is optional and DECLINED (the blocked ball-carrier does NOT
                // throw a Quick Pass). Using it enters TurnMode.DUMP_OFF → DEFENDER_ACTION dialog →
                // INIT_PASSING, which the ParityRunner cannot drive → the stock engine NPEs in
                // StepBlockStatistics (dark_elf seed 55: java=None). The Rust agent likewise declines
                // the DumpOff SkillUse prompt, so both engines keep the ball and let the block proceed.
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogSkillUseParameter) {
                    com.fumbbl.ffb.dialog.DialogSkillUseParameter su =
                        (com.fumbbl.ffb.dialog.DialogSkillUseParameter) dialog;
                    // PrimalSavagery = Animal Savagery's optional "lash out at an adjacent OPPONENT"
                    // offer. Decline it (like DumpOff) so both engines fall through to the mandatory
                    // lash-out-at-a-team-mate PlayerChoice, keeping the block target identical.
                    // SafePairOfHands: optional "place the ball in an adjacent square instead of it
                    // scattering" when the carrier is knocked down. Using it enters TurnMode.
                    // SAFE_PAIR_OF_HANDS → a PLACE_BALL coach dialog the ParityRunner cannot drive
                    // (renegades seed 81: STUCK_STEP PLACE_BALL → java=None). Rust's StepPlaceBall
                    // auto-declines it (dialog not ported), so decline here too → the ball scatters
                    // in both engines and no PLACE_BALL dialog is entered.
                    // Swoop: optional TTM deflection. Using it enters a CLIENT_SWOOP target dialog
                    // the ParityRunner cannot drive → the SWOOP step gets STUCK and the game
                    // force-ends (goblin seed 3 i=194: a thrown Doom Diver). Decline (like
                    // SafePairOfHands) so the thrown player lands normally in both engines.
                    String skillName = (su.getSkill() == null) ? null : su.getSkill().getClass().getSimpleName();
                    boolean useSkill = (skillName == null)
                        || (!"DumpOff".equals(skillName) && !"PrimalSavagery".equals(skillName)
                            && !"SafePairOfHands".equals(skillName) && !"Swoop".equals(skillName));
                    comm.clearCaptured();
                    comm.sendUseSkill(su.getSkill(), useSkill, su.getPlayerId());
                    injectCaptured(dialog, game, gameState);
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            case SELECT_SKILL: {
                // DialogSelectSkillParameter — shared by the Intensive Training prayer and
                // Wisdom of the White Dwarf. WITHOUT this arm the dialog fell through to the
                // default, i.e. the NON-SEEDED RandomStrategy: silent nondeterminism for parity.
                // Both Java call sites hand over a FLAT list already sorted by skill name
                // (Comparator.comparing(Skill::getName)), so answer the lowest name with ZERO rng
                // — the same contract both Rust agents use (min-by-name over the prompt's ids).
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogSelectSkillParameter) {
                    com.fumbbl.ffb.dialog.DialogSelectSkillParameter ssp =
                        (com.fumbbl.ffb.dialog.DialogSelectSkillParameter) dialog;
                    com.fumbbl.ffb.model.skill.Skill best = null;
                    if (ssp.getSkills() != null) {
                        for (com.fumbbl.ffb.model.skill.Skill s : ssp.getSkills()) {
                            if (s == null) continue;
                            if (best == null || s.getName().compareTo(best.getName()) < 0) {
                                best = s;
                            }
                        }
                    }
                    if (best == null) {
                        game.setDialogParameter(null);
                        break;
                    }
                    if (DEBUG) System.err.println("JAVA_SELECT_SKILL pick=" + best.getName());
                    comm.clearCaptured();
                    comm.sendSkillSelection(ssp.getPlayerId(), best);
                    injectCaptured(dialog, game, gameState);
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            case APOTHECARY_CHOICE: {
                // Always keep old injury state — deterministic, no RNG.
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogApothecaryChoiceParameter) {
                    com.fumbbl.ffb.dialog.DialogApothecaryChoiceParameter ac =
                        (com.fumbbl.ffb.dialog.DialogApothecaryChoiceParameter) dialog;
                    comm.clearCaptured();
                    comm.sendApothecaryChoice(ac.getPlayerId(), ac.getPlayerStateOld(),
                        ac.getSeriousInjuryOld(), ac.getPlayerStateOld());
                    injectCaptured(dialog, game, gameState);
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            case USE_APOTHECARY: {
                // Always decline — deterministic, no RNG.
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogUseApothecaryParameter) {
                    com.fumbbl.ffb.dialog.DialogUseApothecaryParameter apo =
                        (com.fumbbl.ffb.dialog.DialogUseApothecaryParameter) dialog;
                    List<com.fumbbl.ffb.ApothecaryType> types = apo.getApothecaryTypes();
                    com.fumbbl.ffb.ApothecaryType apoType = (types != null && !types.isEmpty())
                        ? types.get(0) : com.fumbbl.ffb.ApothecaryType.TEAM;
                    comm.clearCaptured();
                    comm.sendUseApothecary(apo.getPlayerId(), false, apoType, apo.getSeriousInjury());
                    injectCaptured(dialog, game, gameState);
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            case ARGUE_THE_CALL: {
                // Always argue with first eligible player (original behavior).
                // Declining by clearing the dialog causes an infinite END_TURN loop for
                // goblin SW teams. Arguing properly advances StepEndTurn.
                // Note: the argue roll (d6) consumes game RNG in Java; Rust declines (0 dice).
                // This causes halftime divergence for SW races — addressed separately.
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogArgueTheCallParameter) {
                    com.fumbbl.ffb.dialog.DialogArgueTheCallParameter argueParam =
                        (com.fumbbl.ffb.dialog.DialogArgueTheCallParameter) dialog;
                    String[] playerIds = argueParam.getPlayerIds();
                    String teamId = getDialogTeamId(dialog);
                    String firstPlayer = (playerIds != null && playerIds.length > 0) ? playerIds[0] : null;
                    if (DEBUG) System.err.println("JAVA_ARGUE_DIALOG team=" + teamId
                        + " ids=" + java.util.Arrays.toString(playerIds));
                    com.fumbbl.ffb.net.commands.ClientCommandArgueTheCall argueCmd =
                        firstPlayer != null
                            ? new com.fumbbl.ffb.net.commands.ClientCommandArgueTheCall(firstPlayer)
                            : new com.fumbbl.ffb.net.commands.ClientCommandArgueTheCall();
                    try {
                        if (teamId != null) {
                            MatchRunner.injectForTeam(gameState, argueCmd,
                                teamId.equals(game.getTeamHome().getId()));
                        } else {
                            MatchRunner.inject(gameState, argueCmd);
                        }
                    } catch (RuntimeException e) {
                        game.setDialogParameter(null);
                    }
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            // ── Player choice: always decline (empty selection) ──────────────
            // All PlayerChoice modes are declined deterministically — no player is
            // selected for High Kick, Solid Defence, Diving Catch, Kick Skill, etc.
            // This matches Rust's AgentResponse::PlayerChoice { player_id: "" } for all
            // PlayerChoice prompts. RandomStrategy used non-seeded RANDOM here, causing
            // 1-6% divergence per race when optional player selections were made.
            case PLAYER_CHOICE:
            case KICK_SKILL: {
                // Most PlayerChoice modes: decline with empty selection.
                // Exception: MVP must have a non-empty selection (server loops otherwise).
                if (dialog instanceof DialogPlayerChoiceParameter) {
                    DialogPlayerChoiceParameter pcp = (DialogPlayerChoiceParameter) dialog;
                    PlayerChoiceMode mode = pcp.getPlayerChoiceMode();
                    String teamId = getDialogTeamId(dialog);
                    Player[] selection;
                    String[] pids = pcp.getPlayerIds();
                    if (mode == PlayerChoiceMode.MVP && pids != null && pids.length > 0) {
                        // Must pick at least one player for MVP — pick the first available player object.
                        Player<?> mvpPlayer = game.getPlayerById(pids[0]);
                        selection = (mvpPlayer != null) ? new Player[]{ mvpPlayer } : new Player[0];
                    } else if ((mode == PlayerChoiceMode.ANIMAL_SAVAGERY
                            || mode == PlayerChoiceMode.BLACK_INK) && pids != null && pids.length > 0) {
                        // Animal Savagery is MANDATORY (min=1, max=1): a confused player with ≥2
                        // adjacent team-mates MUST lash out at exactly one. Declining with an empty
                        // selection re-fires the dialog → STUCK_STEP. Pick the team-mate with the MIN
                        // (x,y) board coordinate — identical rule to the Rust agent's ANIMAL_SAVAGERY
                        // arm; board coords are engine-agnostic so both engines lash out at the SAME
                        // team-mate and their shared block dice align.
                        Player<?> best = null;
                        FieldCoordinate bestCoord = null;
                        for (String pid : pids) {
                            Player<?> cand = game.getPlayerById(pid);
                            if (cand == null) continue;
                            FieldCoordinate cc = game.getFieldModel().getPlayerCoordinate(cand);
                            if (cc == null) continue;
                            if (bestCoord == null
                                    || cc.getX() < bestCoord.getX()
                                    || (cc.getX() == bestCoord.getX() && cc.getY() < bestCoord.getY())) {
                                best = cand;
                                bestCoord = cc;
                            }
                        }
                        selection = (best != null) ? new Player[]{ best } : new Player[0];
                    } else if (mode == PlayerChoiceMode.BALEFUL_HEX && pids != null && pids.length > 0) {
                        // Baleful Hex target choice: single actionRng pick over the dialog's
                        // list in its given order (the step's findPlayers = opponent team nr
                        // order) — identical contract to the Rust agent's BALEFUL_HEX arm.
                        int bhIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), pids.length);
                        Player<?> bhPick = game.getPlayerById(pids[bhIdx]);
                        selection = (bhPick != null) ? new Player[]{ bhPick } : new Player[0];
                    } else if (mode == PlayerChoiceMode.RAIDING_PARTY && pids != null && pids.length > 0) {
                        // Raiding Party team-mate choice: single actionRng pick over the dialog's
                        // list in its given order (the step's findPlayers = team nr order) —
                        // identical contract to the Rust agent's RAIDING_PARTY PlayerChoice arm.
                        // The step's square phase sets NO new dialog (it publishes MoveSquares and
                        // CONTINUEs), so the answered PLAYER_CHOICE stays set server-side; clear it
                        // locally or the runner re-answers the stale dialog forever.
                        int rpIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), pids.length);
                        Player<?> rpPick = game.getPlayerById(pids[rpIdx]);
                        if (DEBUG) System.err.println("JAVA_RP_PICK N=" + pids.length + " idx=" + rpIdx
                            + " pid=" + pids[rpIdx]);
                        Player[] rpSelection = (rpPick != null) ? new Player[]{ rpPick } : new Player[0];
                        ClientCommandPlayerChoice rpCmd = new ClientCommandPlayerChoice(mode, rpSelection);
                        try {
                            if (teamId != null) {
                                MatchRunner.injectForTeam(gameState, rpCmd,
                                    teamId.equals(game.getTeamHome().getId()));
                            } else {
                                MatchRunner.inject(gameState, rpCmd);
                            }
                        } catch (RuntimeException e) {
                            // fall through to clearing the dialog below
                        }
                        game.setDialogParameter(null);
                        break;
                    } else if (mode == PlayerChoiceMode.AUTO_GAZE_ZOAT && pids != null && pids.length > 0) {
                        // Zoat gaze-target choice. Without an arm this fell to the default, i.e.
                        // the NON-SEEDED RandomStrategy. Coordinate-sort then a single actionRng
                        // pick — identical contract to the Rust agent's AUTO_GAZE_ZOAT arm.
                        List<Player<?>> zCands = new ArrayList<>();
                        for (String pid : pids) {
                            Player<?> cand = game.getPlayerById(pid);
                            if (cand != null) zCands.add(cand);
                        }
                        sortPlayersByCoordinate(zCands, game.getFieldModel());
                        Player[] zSelection = new Player[0];
                        if (!zCands.isEmpty()) {
                            int zIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), zCands.size());
                            if (DEBUG) System.err.println("JAVA_ZOAT_PICK N=" + zCands.size()
                                + " idx=" + zIdx + " pid=" + zCands.get(zIdx).getId());
                            zSelection = new Player[]{ zCands.get(zIdx) };
                        }
                        ClientCommandPlayerChoice zCmd = new ClientCommandPlayerChoice(mode, zSelection);
                        try {
                            if (teamId != null) {
                                MatchRunner.injectForTeam(gameState, zCmd,
                                    teamId.equals(game.getTeamHome().getId()));
                            } else {
                                MatchRunner.inject(gameState, zCmd);
                            }
                        } catch (RuntimeException e) {
                            game.setDialogParameter(null);
                        }
                        break;
                    } else if (mode == PlayerChoiceMode.WISDOM && pids != null && pids.length > 0) {
                        // Wisdom of the White Dwarf team-mate choice (minSelects = 1, so an empty
                        // selection re-fires the dialog forever). StepWisdomOfTheWhiteDwarf builds
                        // wisePlayers from UtilPlayer.findStandingOrPronePlayers, whose order is
                        // not a documented contract, so COORDINATE-SORT before the single
                        // actionRng pick — board coordinates are engine-agnostic, and Rust's
                        // WISDOM arm sorts identically.
                        List<Player<?>> wiseCands = new ArrayList<>();
                        for (String pid : pids) {
                            Player<?> cand = game.getPlayerById(pid);
                            if (cand != null) wiseCands.add(cand);
                        }
                        sortPlayersByCoordinate(wiseCands, game.getFieldModel());
                        Player[] wiseSelection = new Player[0];
                        if (!wiseCands.isEmpty()) {
                            int wIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), wiseCands.size());
                            if (DEBUG) System.err.println("JAVA_WISDOM_PICK N=" + wiseCands.size()
                                + " idx=" + wIdx + " pid=" + wiseCands.get(wIdx).getId());
                            wiseSelection = new Player[]{ wiseCands.get(wIdx) };
                        }
                        ClientCommandPlayerChoice wCmd = new ClientCommandPlayerChoice(mode, wiseSelection);
                        try {
                            if (teamId != null) {
                                MatchRunner.injectForTeam(gameState, wCmd,
                                    teamId.equals(game.getTeamHome().getId()));
                            } else {
                                MatchRunner.inject(gameState, wCmd);
                            }
                        } catch (RuntimeException e) {
                            game.setDialogParameter(null);
                        }
                        break;
                    } else if (mode == PlayerChoiceMode.FURIOUS_OUTBURST && pids != null && pids.length > 0) {
                        // Furious Outburst stab-target choice. UNLIKE the other star dialogs the
                        // list order here is NOT a contract: StepInitFuriousOutburst builds
                        // `eligiblePlayers` as a HashSet and hands `foundPlayers.toArray(..)` to the
                        // dialog, so the order is identity-hash order. COORDINATE-SORT first, then a
                        // single actionRng pick — board coordinates are engine-agnostic, so Rust's
                        // FURIOUS_OUTBURST arm lands on the same target. Like RAIDING_PARTY, the
                        // step's square phase sets NO new dialog (it publishes MoveSquares and
                        // CONTINUEs), so the answered PLAYER_CHOICE stays set server-side; clear it
                        // locally or the runner re-answers the stale dialog forever.
                        List<Player<?>> foCands = new ArrayList<>();
                        for (String pid : pids) {
                            Player<?> cand = game.getPlayerById(pid);
                            if (cand != null) foCands.add(cand);
                        }
                        sortPlayersByCoordinate(foCands, game.getFieldModel());
                        Player[] foSelection = new Player[0];
                        if (!foCands.isEmpty()) {
                            int foIdx = (int) Long.remainderUnsigned(actionRng.nextLong(), foCands.size());
                            if (DEBUG) System.err.println("JAVA_FO_PICK N=" + foCands.size() + " idx=" + foIdx
                                + " pid=" + foCands.get(foIdx).getId());
                            foSelection = new Player[]{ foCands.get(foIdx) };
                        }
                        ClientCommandPlayerChoice foCmd = new ClientCommandPlayerChoice(mode, foSelection);
                        try {
                            if (teamId != null) {
                                MatchRunner.injectForTeam(gameState, foCmd,
                                    teamId.equals(game.getTeamHome().getId()));
                            } else {
                                MatchRunner.inject(gameState, foCmd);
                            }
                        } catch (RuntimeException e) {
                            // fall through to clearing the dialog below
                        }
                        game.setDialogParameter(null);
                        break;
                    } else if ((mode == PlayerChoiceMode.IRON_MAN
                            || mode == PlayerChoiceMode.KNUCKLE_DUSTERS
                            || mode == PlayerChoiceMode.BLESSED_STATUE_OF_NUFFLE)
                            && pids != null && pids.length > 0) {
                        // Prayers to Nuffle that pick a player (Iron Man, Knuckle Dusters, Blessed
                        // Statue of Nuffle) show a MANDATORY DialogPlayerChoiceParameter (minSelects=1):
                        // declining with an empty selection re-fires the dialog forever, so the step
                        // never completes and every loop top reports UNHANDLED_STEP: PRAYER until the
                        // 500-iteration cap - the Java game is then garbage and the comparison fails at
                        // step 0 (lineman bb2020 seed 26, and the same seed in human/ogre).
                        // Pick the LOWEST PLAYER NUMBER: these prayers choose among RESERVES, which have
                        // no board coordinates, so the min-(x,y) rule used for Animal Savagery cannot
                        // apply. Player numbers are engine-agnostic (both teams are built from the same
                        // spec), so Rust's agent picks the same player.
                        Player<?> lowest = null;
                        for (String pid : pids) {
                            Player<?> p = game.getPlayerById(pid);
                            if (p == null) continue;
                            if (lowest == null || p.getNr() < lowest.getNr()) {
                                lowest = p;
                            }
                        }
                        selection = (lowest != null) ? new Player[]{ lowest } : new Player[0];
                    } else {
                        selection = new Player[0];
                    }
                    ClientCommandPlayerChoice cmd = new ClientCommandPlayerChoice(mode, selection);
                    try {
                        if (teamId != null) {
                            MatchRunner.injectForTeam(gameState, cmd,
                                teamId.equals(game.getTeamHome().getId()));
                        } else {
                            MatchRunner.inject(gameState, cmd);
                        }
                    } catch (RuntimeException e) {
                        game.setDialogParameter(null);
                    }
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            // ── BRIBES dialog: always decline ──────────────────────────────────
            // Fires when a SW player was ejected and team has AVOID_BAN inducements.
            // Decline by sending CLIENT_USE_INDUCEMENT with AVOID_BAN inducement type
            // and empty player list. RandomStrategy sends null inducement type which
            // the server can't process, leaving fBribesChoiceAway/Home=null forever.
            case BRIBES: {
                String bribesTeamId = getDialogTeamId(dialog);
                // Find the AVOID_BAN inducement type
                com.fumbbl.ffb.inducement.InducementType avoidBanType = null;
                com.fumbbl.ffb.model.TurnData td = (bribesTeamId != null && bribesTeamId.equals(game.getTeamHome().getId()))
                    ? game.getTurnDataHome() : game.getTurnDataAway();
                for (com.fumbbl.ffb.inducement.InducementType t : td.getInducementSet().getInducementTypes()) {
                    if (t.hasUsage(com.fumbbl.ffb.inducement.Usage.AVOID_BAN)) { avoidBanType = t; break; }
                }
                com.fumbbl.ffb.net.commands.ClientCommandUseInducement declineBribe =
                    new com.fumbbl.ffb.net.commands.ClientCommandUseInducement(avoidBanType, new String[0]);
                try {
                    if (bribesTeamId != null) {
                        MatchRunner.injectForTeam(gameState, declineBribe,
                            bribesTeamId.equals(game.getTeamHome().getId()));
                    } else {
                        MatchRunner.inject(gameState, declineBribe);
                    }
                } catch (RuntimeException e) {
                    game.setDialogParameter(null);
                }
                break;
            }

            // ── Bribery/Corruption reroll: always decline ─────────────────────
            // Fires when a SW player was ejected and the team is offered a bribe
            // reroll. Use RandomStrategy to send a valid decline command; just clearing
            // the dialog causes the server to re-fire it 100,000 times (MAX_ITERATIONS).
            case BRIBERY_AND_CORRUPTION_RE_ROLL: {
                comm.clearCaptured();
                RandomStrategy.respondToDialog(dialog, game, comm);
                com.fumbbl.ffb.net.commands.ClientCommand brCaptured = comm.getCapturedCommand();
                if (brCaptured != null) {
                    String brTeamId = getDialogTeamId(dialog);
                    try {
                        if (brTeamId != null) {
                            MatchRunner.injectForTeam(gameState, brCaptured,
                                brTeamId.equals(game.getTeamHome().getId()));
                        } else {
                            MatchRunner.inject(gameState, brCaptured);
                        }
                    } catch (RuntimeException e) {
                        game.setDialogParameter(null);
                    }
                } else {
                    game.setDialogParameter(null);
                }
                break;
            }

            // ── Default: fall back to RandomStrategy for truly rare dialogs ────
            // DEBUG: print dialog IDs to diagnose any remaining loops
            // Use RandomStrategy (non-seeded RANDOM) rather than simply clearing,
            // to ensure the server receives a valid response and doesn't re-fire
            // the same dialog → MAX_ITERATIONS loop.  RandomStrategy declines most
            // optional dialogs with a safe response.  Only decisionRng-consuming
            // dialogs can affect parity; RandomStrategy uses java.util.Random.
            default:
                // Any dialog landing here is handled by the NON-SEEDED RandomStrategy —
                // silent nondeterminism for parity. Always log it so the discovery pass
                // can promote it to an explicit deterministic case.
                System.err.println("UNHANDLED_DIALOG: " + dialog.getId() + " turnMode=" + game.getTurnMode());
                comm.clearCaptured();
                RandomStrategy.respondToDialog(dialog, game, comm);
                com.fumbbl.ffb.net.commands.ClientCommand captured = comm.getCapturedCommand();
                if (captured != null) {
                    String teamId = getDialogTeamId(dialog);
                    try {
                        if (teamId != null) {
                            MatchRunner.injectForTeam(gameState, captured,
                                teamId.equals(game.getTeamHome().getId()));
                        } else {
                            MatchRunner.inject(gameState, captured);
                        }
                    } catch (RuntimeException e) {
                        game.setDialogParameter(null);
                    }
                } else {
                    game.setDialogParameter(null);
                }
                break;
        }
    }

    // ── Step recording ────────────────────────────────────────────────────────

    private void recordStep(Game game, String chosen, int callCount) {
        boolean homePlaying = game.isHomePlaying();
        int turn = homePlaying
            ? game.getTurnDataHome().getTurnNr()
            : game.getTurnDataAway().getTurnNr();
        int half = game.getHalf();
        String active = homePlaying ? "home" : "away";
        String canonicalStr = stateString(game);
        long hashLong = fnv1a64(canonicalStr.getBytes(java.nio.charset.StandardCharsets.UTF_8));
        String hash = String.format("%016x", hashLong);
        if (DEBUG) {
            System.err.println("JSTEP i=" + stepIndex + " rng_calls=" + callCount + " mode=" + game.getTurnMode()
                + " chosen=" + chosen + " state=" + canonicalStr);
        }
        if (System.getenv("FFB_IDSTATE") != null) {
            StringBuilder sb = new StringBuilder("JIDSTATE i=" + stepIndex + " ");
            for (com.fumbbl.ffb.model.Player<?> p : game.getPlayers()) {
                com.fumbbl.ffb.FieldCoordinate c = game.getFieldModel().getPlayerCoordinate(p);
                PlayerState ps = game.getFieldModel().getPlayerState(p);
                sb.append(p.getId()).append("=")
                  .append(c == null ? "?" : (c.getX() + "," + c.getY())).append("/")
                  .append(ps == null ? "?" : Integer.toHexString(ps.getBase())).append(" ");
            }
            System.err.println(sb.toString());
        }
        pending.add(new PendingStep(stepIndex++, turn, half, active, hash, chosen));
    }

    /** Per-team once-per-turn flags, fixed order: blitz, foul, hand-over, pass.
     *  ttm/ktm are absent because TurnData exposes no accessor and it is engine code, not harness.
     *  Must stay byte-identical with ffb-rust `state_hash.rs`. */
    private static String turnFlags(com.fumbbl.ffb.model.TurnData td) {
        if (td == null) return "0000";
        return (td.isBlitzUsed() ? "1" : "0")
             + (td.isFoulUsed() ? "1" : "0")
             + (td.isHandOverUsed() ? "1" : "0")
             + (td.isPassUsed() ? "1" : "0");
    }

    public static String stateString(Game game) {
        boolean homePlaying = game.isHomePlaying();
        int half = Math.max(1, game.getHalf());
        int turnHome = game.getTurnDataHome().getTurnNr();
        int turnAway = game.getTurnDataAway().getTurnNr();
        String active = homePlaying ? "home" : "away";
        int scoreHome = game.getGameResult().getScoreHome();
        int scoreAway = game.getGameResult().getScoreAway();
        FieldModel fm = game.getFieldModel();
        FieldCoordinate ball = fm.getBallCoordinate();
        int bx = ball != null ? ball.getX() : -1;
        int by = ball != null ? ball.getY() : -1;
        boolean inPlay = fm.isBallInPlay();
        List<String> playerParts = new ArrayList<>();
        addPlayersFromTeam(game.getTeamHome(), fm, playerParts, "h");
        addPlayersFromTeam(game.getTeamAway(), fm, playerParts, "a");
        playerParts.sort(null);
        StringBuilder sb = new StringBuilder();
        sb.append('h').append(half);
        sb.append('t').append(turnHome).append(turnAway);
        sb.append('a').append(active);
        sb.append('s').append(scoreHome).append(',').append(scoreAway);
        sb.append(" b").append(bx).append(',').append(by).append(',').append(inPlay ? "true" : "false");
        sb.append(" f").append(turnFlags(game.getTurnDataHome())).append(',').append(turnFlags(game.getTurnDataAway()));
        sb.append(" r").append(game.getTurnDataHome().getReRolls()).append(',').append(game.getTurnDataAway().getReRolls());
        sb.append(" ap").append(actingPlayerPart(game));
        sb.append(" w").append(game.getFieldModel().getWeather() != null ? game.getFieldModel().getWeather().getName() : "-");
        sb.append(" tm").append(game.getTurnMode() != null ? game.getTurnMode().getName() : "-");
        sb.append(" p");
        for (int i = 0; i < playerParts.size(); i++) {
            if (i > 0) sb.append('|');
            sb.append(playerParts.get(i));
        }
        return sb.toString();
    }

    // ── State hash (FNV-1a 64-bit — must match ffb-rust/crates/ffb-sim/src/parity_log.rs) ──

    public static String stateHash(Game game) {
        boolean homePlaying = game.isHomePlaying();
        int half = Math.max(1, game.getHalf());
        int turnHome = game.getTurnDataHome().getTurnNr();
        int turnAway = game.getTurnDataAway().getTurnNr();
        String active = homePlaying ? "home" : "away";
        int scoreHome = game.getGameResult().getScoreHome();
        int scoreAway = game.getGameResult().getScoreAway();

        FieldModel fm = game.getFieldModel();
        FieldCoordinate ball = fm.getBallCoordinate();
        int bx = ball != null ? ball.getX() : -1;
        int by = ball != null ? ball.getY() : -1;
        int inPlay = fm.isBallInPlay() ? 1 : 0;

        List<String> playerParts = new ArrayList<>();
        addPlayersFromTeam(game.getTeamHome(), fm, playerParts, "h");
        addPlayersFromTeam(game.getTeamAway(), fm, playerParts, "a");
        playerParts.sort(null);

        StringBuilder sb = new StringBuilder();
        sb.append('h').append(half);
        sb.append('t').append(turnHome).append(turnAway);
        sb.append('a').append(active);
        sb.append('s').append(scoreHome).append(',').append(scoreAway);
        sb.append(" b").append(bx).append(',').append(by).append(',').append(inPlay == 1 ? "true" : "false");
        sb.append(" f").append(turnFlags(game.getTurnDataHome())).append(',').append(turnFlags(game.getTurnDataAway()));
        sb.append(" r").append(game.getTurnDataHome().getReRolls()).append(',').append(game.getTurnDataAway().getReRolls());
        sb.append(" ap").append(actingPlayerPart(game));
        sb.append(" w").append(game.getFieldModel().getWeather() != null ? game.getFieldModel().getWeather().getName() : "-");
        sb.append(" tm").append(game.getTurnMode() != null ? game.getTurnMode().getName() : "-");
        sb.append(" p");
        for (int i = 0; i < playerParts.size(); i++) {
            if (i > 0) sb.append('|');
            sb.append(playerParts.get(i));
        }

        String canonical = sb.toString();
        long hash = fnv1a64(canonical.getBytes(StandardCharsets.UTF_8));
        return String.format("%016x", hash);
    }

    /** `h03,2` — the acting player's index in the same ordering addPlayersFromTeam uses (sorted by
     *  squad number, first 11 per team) plus its spent movement; `-` when nobody is activated.
     *  Must stay byte-identical with ffb-rust `state_hash.rs::acting_player_part`. */
    private static String actingPlayerPart(Game game) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer != null) ? actingPlayer.getPlayerId() : null;
        if (pid == null || pid.isEmpty()) return "-";
        int currentMove = actingPlayer.getCurrentMove();
        Team[] teams = { game.getTeamHome(), game.getTeamAway() };
        String[] prefixes = { "h", "a" };
        for (int t = 0; t < 2; t++) {
            if (teams[t] == null) continue;
            List<Player<?>> players = new ArrayList<>(java.util.Arrays.asList(teams[t].getPlayers()));
            players.sort(java.util.Comparator.comparingInt(Player::getNr));
            if (players.size() > 11) players = players.subList(0, 11);
            for (int i = 0; i < players.size(); i++) {
                if (pid.equals(players.get(i).getId())) {
                    return String.format("%s%02d,%d", prefixes[t], i, currentMove);
                }
            }
        }
        return "?," + currentMove;
    }

    private static void addPlayersFromTeam(Team team, FieldModel fm, List<String> out, String prefix) {
        if (team == null) return;
        List<Player<?>> players = new ArrayList<>(java.util.Arrays.asList(team.getPlayers()));
        players.sort(java.util.Comparator.comparingInt(Player::getNr));
        if (players.size() > 11) players = players.subList(0, 11);
        for (int i = 0; i < players.size(); i++) {
            Player<?> p = players.get(i);
            PlayerState ps = fm.getPlayerState(p);
            FieldCoordinate coord = fm.getPlayerCoordinate(p);
            boolean onPitch = coord != null && coord.getX() >= 0 && coord.getX() <= 25
                                             && coord.getY() >= 0 && coord.getY() <= 14;
            int x = onPitch ? coord.getX() : -1;
            int y = onPitch ? coord.getY() : -1;
            String state = playerStateStr(ps);
            // Effective stats (base + temporary modifiers). Must stay byte-identical with
            // ffb-rust `state_hash.rs::collect_player_parts`.
            // Trailing ACTIVE bit: the hash was blind to it, and it decides whether a player can
            // be activated at all — several re-activation/lost-deactivation bugs stayed invisible
            // for whole games (thrown players re-acting, bomb catchers retired for the half).
            int activeBit = (ps != null && ps.isActive()) ? 1 : 0;
            out.add(String.format("%s%02d:%d,%d,%s,%d/%d/%d/%d,%d", prefix, i, x, y, state,
                p.getMovementWithModifiers(), p.getStrengthWithModifiers(),
                p.getAgilityWithModifiers(), p.getArmourWithModifiers(), activeBit));
        }
    }

    private static String playerStateStr(PlayerState ps) {
        if (ps == null) return "Reserve";
        switch (ps.getBase()) {
            case PlayerState.STANDING:       return "Standing";
            case PlayerState.MOVING:         return "Moving";
            case PlayerState.PRONE:          return "Prone";
            case PlayerState.STUNNED:        return "Stunned";
            case PlayerState.KNOCKED_OUT:    return "Ko";
            // The three casualty states used to collapse to one "Injured" label, so the compared
            // hash could not tell a DEAD player from a bruised one. Must stay in lockstep with
            // ffb-rust `crates/ffb-model/src/util/state_hash.rs::player_state_str`.
            case PlayerState.BADLY_HURT:     return "Bh";
            case PlayerState.SERIOUS_INJURY: return "Si";
            case PlayerState.RIP:            return "Rip";
            case PlayerState.RESERVE:        return "Reserve";
            default:                         return "Reserve";
        }
    }

    static long fnv1a64(byte[] data) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : data) {
            hash ^= Byte.toUnsignedLong(b);
            hash *= 1099511628211L;
        }
        return hash;
    }

    // ── Concrete action dispatch ─────────────────────────────────────────────────

    /**
     * Tier 3, phase 2: the acting player was selected with a declared action; send the
     * concrete command for it (AGENT_CONTRACT.md §8). Stage A covers Move/stand-up only;
     * Stage B adds Block, Blitz, Pass, HandOver, Foul.
     */
    private void sendConcreteAction(Game game, GameState gameState) {
        ActingPlayer ap = game.getActingPlayer();
        String pid = ap.getPlayerId();
        PlayerAction pa = ap.getPlayerAction();
        if (DEBUG) System.err.println("JAVA_P2 pid=" + pid + " action=" + pa + " si=" + stepIndex);
        if (pa == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        switch (pa) {
            case MOVE:
            case STAND_UP:
                sendMoveAction(game, gameState, pid);
                break;
            case BLOCK:
                sendBlockAction(game, gameState, pid);
                break;
            case BLITZ:
            case BLITZ_MOVE:
            case BLITZ_SELECT:
            case STAND_UP_BLITZ: {
                // BB2016 has no SELECT_BLITZ_TARGET step. Drive the bb2016 3-command blitz: the
                // folded Rust agent blitzes an ADJACENT target (no pre-move), so send a 0-square
                // CLIENT_BLITZ_MOVE (enter the blitz in place) then CLIENT_BLOCK(target). The target
                // is picked with the SAME adjacent/coord-sorted/actionRng logic as the folded agent
                // (pickBlockTarget), matching the RNG order (player, action, target).
                if (isBb2016(game)) {
                    FieldCoordinate bcoord = playerCoordinate(game, pid);
                    if (!bb2016BlitzMoveSent) {
                        bb2016BlitzMoveSent = true;
                        FieldCoordinate cmdFrom = game.isHomePlaying() ? bcoord : bcoord.transform();
                        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandBlitzMove(
                            pid, cmdFrom, new FieldCoordinate[]{}));
                    } else {
                        Player<?> btarget = pickBlockTarget(game, pid);
                        if (btarget == null) {
                            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                        } else {
                            MatchRunner.inject(gameState, new ClientCommandBlock(
                                pid, btarget.getId(), false, false, false, false, false));
                        }
                    }
                    break;
                }
                // Blitz block: the target was already chosen at SELECT_BLITZ_TARGET (which
                // consumed the actionRng pick); CLIENT_BLOCK with a targetSelectionState
                // dispatches as BLITZ. After the block, end the activation with CONFIRM.
                com.fumbbl.ffb.model.TargetSelectionState tss = game.getFieldModel().getTargetSelectionState();
                if (!blitzBlockSent && tss != null && tss.getSelectedPlayerId() != null) {
                    blitzBlockSent = true;
                    if (DEBUG) System.err.println("JAVA_BLITZ_BLOCK pid=" + pid + " def=" + tss.getSelectedPlayerId());
                    MatchRunner.inject(gameState, new ClientCommandBlock(
                        pid, tss.getSelectedPlayerId(), false, false, false, false, false));
                } else {
                    if (DEBUG) System.err.println("JAVA_BLITZ_END pid=" + pid + " sent=" + blitzBlockSent);
                    MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandConfirm());
                }
                break;
            }
            case FOUL:
            case FOUL_MOVE:
                sendFoulAction(game, gameState, pid);
                break;
            case PASS:
            case PASS_MOVE:
            case THROW_BOMB:
            case HAIL_MARY_PASS:
                sendPassAction(game, gameState, pid);
                break;
            case MULTIPLE_BLOCK:
                sendSynchronousMultiBlock(game, gameState, pid);
                break;
            case HAND_OVER:
            case HAND_OVER_MOVE:
                sendHandOverAction(game, gameState, pid);
                break;
            case THROW_TEAM_MATE:
            case THROW_TEAM_MATE_MOVE:
            // A kick uses the same declaration command and the same candidate rule: every
            // edition's TtmMechanic.canBeKicked is canBeThrown() plus STANDING (plus not-rooted and
            // own-team), which is exactly what sendThrowTeamMateAction already computes.
            case KICK_TEAM_MATE:
            case KICK_TEAM_MATE_MOVE:
                sendThrowTeamMateAction(game, gameState, pid);
                break;
            default:
                System.err.println("UNHANDLED_ACTING_ACTION: " + pa + " pid=" + pid + " — deselecting");
                MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
                break;
        }
    }

    /**
     * Blitz target selection (SELECT_BLITZ_TARGET step): same candidate computation and
     * actionRng pick as a block target — this is where Rust's compute_follow_up(Blitz)
     * BLOCK_PICK call maps to. Injects CLIENT_TARGET_SELECTED, which makes
     * StepSelectBlitzTargetEnd consume the team blitz (setBlitzUsed).
     */
    private void sendBlitzTargetSelection(Game game, GameState gameState) {
        ActingPlayer ap = game.getActingPlayer();
        String pid = (ap != null) ? ap.getPlayerId() : null;
        // The heuristic agent SCORES the blitz target (Rust `AgentPrompt::BlitzTarget`, T = 0.15)
        // when the `blitztarget` class is on. It must NOT fall through to pickBlockTarget, which
        // would spend an actionRng draw the Rust side does not spend in that configuration.
        if (pid != null && heuristic != null
            && heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.BLITZ_TARGET)) {
            java.util.List<String> candidates = new ArrayList<>();
            for (Player<?> op : blockTargetCandidates(game, pid)) {
                candidates.add(op.getId());
            }
            String chosen = heuristic.blitzTarget(game, pid, candidates);
            if (chosen == null) {
                MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            } else {
                MatchRunner.inject(gameState,
                    new com.fumbbl.ffb.net.commands.ClientCommandTargetSelected(chosen));
            }
            return;
        }
        Player<?> blockTarget = (pid != null) ? pickBlockTarget(game, pid) : null;
        if (blockTarget == null) {
            System.err.println("BLITZ_TARGET_NONE pid=" + pid + " — ending turn for acting player");
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        if (DEBUG) System.err.println("JAVA_BLITZ_TARGET pid=" + pid + " target=" + blockTarget.getId());
        MatchRunner.inject(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandTargetSelected(blockTarget.getId()));
    }

    /**
     * Block / blitz-block target: adjacent opponents whose state base is STANDING or
     * MOVING (mirrors Rust PlayerState::can_be_blocked — note: NOT hasTackleZones, no
     * confused check), coordinate-sorted, 1 actionRng pick (AGENT_CONTRACT.md §3/§6).
     */
    /**
     * The candidate list {@link #pickBlockTarget} draws from, WITHOUT drawing: adjacent opponents
     * whose state base is STANDING or MOVING, in coordinate order. Extracted so the heuristic
     * agent can score the same list without spending an actionRng draw on it.
     */
    private List<Player<?>> blockTargetCandidates(Game game, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        List<Player<?>> targets = new ArrayList<>();
        if (coord == null) {
            return targets;
        }
        Team opponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        for (Player<?> op : opponent.getPlayers()) {
            FieldCoordinate oc = fm.getPlayerCoordinate(op);
            PlayerState ops = fm.getPlayerState(op);
            if (oc == null || ops == null) continue;
            int base = ops.getBase();
            if ((base == PlayerState.STANDING || base == PlayerState.MOVING)
                    && isAdjacentCoord(coord, oc)) {
                targets.add(op);
            }
        }
        sortPlayersByCoordinate(targets, fm);
        return targets;
    }

    private Player<?> pickBlockTarget(Game game, String playerId) {
        FieldModel fm = game.getFieldModel();
        List<Player<?>> targets = blockTargetCandidates(game, playerId);
        if (targets.isEmpty()) return null;
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        if (DEBUG) System.err.println("JAVA_BLOCK_PICK pid=" + playerId + " N=" + targets.size() + " idx=" + idx + " def=" + targets.get(idx).getId());
        return targets.get(idx);
    }

    private void sendBlockAction(Game game, GameState gameState, String playerId) {
        Player<?> target = pickBlockTarget(game, playerId);
        if (target == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        MatchRunner.inject(gameState,
            new ClientCommandBlock(playerId, target.getId(), false, false, false, false, false));
    }

    /**
     * Foul target: adjacent opponents whose state base is PRONE or STUNNED (mirrors
     * Rust PlayerState::can_be_fouled), coordinate-sorted, 1 actionRng pick.
     */
    private void sendFoulAction(Game game, GameState gameState, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        if (coord == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        Team opponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        List<Player<?>> targets = new ArrayList<>();
        for (Player<?> op : opponent.getPlayers()) {
            FieldCoordinate oc = fm.getPlayerCoordinate(op);
            PlayerState ops = fm.getPlayerState(op);
            if (oc == null || ops == null) continue;
            int base = ops.getBase();
            if ((base == PlayerState.PRONE || base == PlayerState.STUNNED)
                    && isAdjacentCoord(coord, oc)) {
                targets.add(op);
            }
        }
        sortPlayersByCoordinate(targets, fm);
        if (targets.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        String targetId = targets.get(idx).getId();
        if (DEBUG) System.err.println("JAVA_FOUL pid=" + playerId + " N=" + targets.size() + " idx=" + idx + " target=" + targetId);
        MatchRunner.inject(gameState, new ClientCommandFoul(playerId, targetId, false));
    }

    /**
     * Pass target: teammate coordinates on pitch, coordinate-sorted, 1 actionRng pick.
     * With no teammates on pitch: 2 decisionRng calls for a random coordinate
     * (AGENT_CONTRACT.md §2.6 quirk). Away-side coordinates are mirrored for the command.
     */
    private void sendPassAction(Game game, GameState gameState, String playerId) {
        FieldModel fm = game.getFieldModel();
        boolean isHome = game.isHomePlaying();
        Team team = isHome ? game.getTeamHome() : game.getTeamAway();
        List<FieldCoordinate> teammates = new ArrayList<>();
        for (Player<?> tp : team.getPlayers()) {
            if (tp.getId().equals(playerId)) continue;
            FieldCoordinate tc = fm.getPlayerCoordinate(tp);
            if (tc != null && tc.getX() >= 0 && tc.getX() <= 25 && tc.getY() >= 0 && tc.getY() <= 14) {
                teammates.add(tc);
            }
        }
        teammates.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        FieldCoordinate coord;
        if (!teammates.isEmpty()) {
            int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), teammates.size());
            coord = teammates.get(idx);
        } else {
            decisionRngAdvances += 2;
            int x = (int) Long.remainderUnsigned(decisionRng.nextLong(), 26L);
            int y = (int) Long.remainderUnsigned(decisionRng.nextLong(), 14L) + 1;
            coord = new FieldCoordinate(x, y);
        }
        if (DEBUG) System.err.println("JAVA_PASS pid=" + playerId + " coord=(" + coord.getX() + "," + coord.getY() + ")");
        FieldCoordinate cmdCoord = isHome ? coord : coord.transform();
        MatchRunner.inject(gameState, new ClientCommandPass(playerId, cmdCoord));
    }

    /**
     * Multiple Block targets: TWO distinct picks from the coordinate-sorted adjacent blockable
     * opponents (first idx % N, second idx % (N-1)) — mirrors Rust's MultiBlockTargets arm.
     * With fewer than two targets left (both KO'd since the turn-start snapshot) deselect like
     * any stale declaration.
     */
    private void sendSynchronousMultiBlock(Game game, GameState gameState, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        Team opponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        java.util.List<Player<?>> targets = new java.util.ArrayList<>();
        if (coord != null) {
            targets.addAll(java.util.Arrays.asList(
                com.fumbbl.ffb.util.UtilPlayer.findAdjacentBlockablePlayers(game, opponent, coord)));
        }
        sortPlayersByCoordinate(targets, fm);
        if (targets.size() < 2) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        int i1 = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        Player<?> d1 = targets.remove(i1);
        int i2 = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        Player<?> d2 = targets.remove(i2);
        // SynchronousMultiBlockLogicModule offers the STAB alternative exactly when the acting
        // player has providesMultipleBlockAlternative (registered by Stab in every edition);
        // otherwise it auto-selects BlockKind.BLOCK. Mirror that ENGINE property rather than an
        // invented rule, and take the alternative for the FIRST-drawn target only, so one
        // multiblock exercises BOTH the block group and the stab group. Deterministic: NO extra
        // actionRng draw - two are already spent above and a third would desync the stream.
        // Rust mirror: RandomAgent's MultiBlockTargets arm.
        // The BLOCKER is this method's own playerId parameter, not game.getActingPlayer() -
        // the acting player is not yet committed at this point. Rust mirrors it by reading the
        // MultiBlockTargets prompt's player_id.
        com.fumbbl.ffb.model.Player<?> mbActor = game.getPlayerById(playerId);
        boolean canStab = mbActor != null && mbActor.hasSkillProperty(
            com.fumbbl.ffb.model.property.NamedProperties.providesMultipleBlockAlternative);
        com.fumbbl.ffb.model.BlockKind kind1 = canStab
            ? com.fumbbl.ffb.model.BlockKind.STAB
            : com.fumbbl.ffb.model.BlockKind.BLOCK;
        java.util.List<com.fumbbl.ffb.model.BlockTarget> blockTargets = new java.util.ArrayList<>();
        blockTargets.add(new com.fumbbl.ffb.model.BlockTarget(d1.getId(), kind1, fm.getPlayerState(d1)));
        blockTargets.add(new com.fumbbl.ffb.model.BlockTarget(d2.getId(), com.fumbbl.ffb.model.BlockKind.BLOCK, fm.getPlayerState(d2)));
        if (DEBUG) System.err.println("JAVA_MB pid=" + playerId + " d1=" + d1.getId() + " d2=" + d2.getId());
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandSynchronousMultiBlock(blockTargets));
    }

    /**
     * Hand-over receiver: adjacent teammates (any state — mirrors Rust's HandOver
     * branch which has no state filter), coordinate-sorted, 1 actionRng pick.
     */
    private void sendHandOverAction(Game game, GameState gameState, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        if (coord == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        Team team = game.isHomePlaying() ? game.getTeamHome() : game.getTeamAway();
        List<Player<?>> receivers = new ArrayList<>();
        for (Player<?> tp : team.getPlayers()) {
            if (tp.getId().equals(playerId)) continue;
            FieldCoordinate tc = fm.getPlayerCoordinate(tp);
            if (tc != null && isAdjacentCoord(coord, tc)) {
                receivers.add(tp);
            }
        }
        sortPlayersByCoordinate(receivers, fm);
        if (receivers.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), receivers.size());
        String receiverId = receivers.get(idx).getId();
        if (DEBUG) System.err.println("JAVA_HANDOVER pid=" + playerId + " N=" + receivers.size() + " idx=" + idx + " recv=" + receiverId);
        MatchRunner.inject(gameState, new ClientCommandHandOver(playerId, receiverId));
    }

    /**
     * Throw Team-Mate — pick the thrown player (phase 1). Candidate set mirrors Rust's
     * legal_throw_team_mate_targets: adjacent STANDING teammates for whom Player.canBeThrown()
     * holds (Right Stuff; in BB2020 only at ST<=3), coordinate-sorted, 1 actionRng pick. Empty → deselect (the human-Ogre / no
     * throwable-teammate case). The target square is chosen later, at the INIT_THROW_TEAM_MATE
     * waiting state (sendThrowTeamMateTarget).
     */
    /**
     * Swoop target (BB2016/BB2020 SWOOP step). Java's mixed StepSwoop ends its execute with
     * UtilServerPlayerSwoop.updateSwoopSquares and WAITS for a CLIENT_SWOOP naming one of the (at
     * most four) orthogonally adjacent squares — there is no decline, unlike BB2025's optional skill
     * offer. Without a handler here the step never advanced and the game was abandoned on
     * STUCK_STEP: SWOOP, which is exactly what a thrown BB2020 goblin Doom Diver produced once
     * BB2020 could throw at all.
     *
     * Candidate set + order + the single actionRng pick mirror Rust's SwoopTarget prompt handling in
     * random_agent.rs: the swoop squares the engine itself published, coordinate-sorted.
     */
    /**
     * Punt target (BB2025 INIT_PUNT step). StepInitPunt publishes the legal punt squares as
     * MoveSquares and then WAITS for a CLIENT_FIELD_COORDINATE naming one. There was no handler at
     * all, so the runner fell through to the UNHANDLED_STEP default (inject END_TURN) and the punt
     * was aborted — which, with the Rust agent aborting in lockstep, is why the entire Punt step
     * family (InitPunt, PuntDirection, PuntDistance, EndPunt) never executed while the matrices
     * stayed green.
     *
     * Candidate set + order + the single actionRng pick mirror Rust's PuntTarget prompt handling in
     * random_agent.rs: the punt squares the engine itself published, coordinate-sorted.
     */
    private void sendRaidingPartyTarget(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer == null) ? null : actingPlayer.getPlayerId();
        FieldModel fm = game.getFieldModel();
        List<FieldCoordinate> squares = new ArrayList<>();
        for (com.fumbbl.ffb.MoveSquare ms : fm.getMoveSquares()) {
            squares.add(ms.getCoordinate());
        }
        squares.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        if (squares.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), squares.size());
        FieldCoordinate target = squares.get(idx);
        if (DEBUG) System.err.println("JAVA_RP_SQ pid=" + pid + " N=" + squares.size() + " idx=" + idx
            + " target=" + target);
        boolean isHome = pid != null && game.getTeamHome().getPlayerById(pid) != null;
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandFieldCoordinate(
            isHome ? target : target.transform()));
    }

    /**
     * Furious Outburst square waits (FIRST_MOVE / SECOND_MOVE). Identical contract to
     * sendRaidingPartyTarget: the step publishes its eligible squares as MoveSquares and waits
     * for CLIENT_FIELD_COORDINATE, so the answer is a coordinate-sorted single actionRng pick,
     * mirrored for the away coach (the server un-mirrors).
     */
    private void sendFuriousOutburstSquare(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer == null) ? null : actingPlayer.getPlayerId();
        FieldModel fm = game.getFieldModel();
        List<FieldCoordinate> squares = new ArrayList<>();
        for (com.fumbbl.ffb.MoveSquare ms : fm.getMoveSquares()) {
            squares.add(ms.getCoordinate());
        }
        squares.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        if (squares.isEmpty()) {
            // Java's step accepts a null-action CLIENT_ACTING_PLAYER as "end the player action",
            // which is what the Rust agent's empty-square answer maps to.
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), squares.size());
        FieldCoordinate target = squares.get(idx);
        if (DEBUG) System.err.println("JAVA_FO_SQ pid=" + pid + " N=" + squares.size() + " idx=" + idx
            + " target=" + target);
        boolean isHome = pid != null && game.getTeamHome().getPlayerById(pid) != null;
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandFieldCoordinate(
            isHome ? target : target.transform()));
    }

    private void sendBlastinTarget(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        Player<?> actor = (actingPlayer == null) ? null : actingPlayer.getPlayer();
        if (actor == null) {
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        FieldModel fm = game.getFieldModel();
        boolean playingHasActor = game.playingTeamHasActingPLayer();
        FieldCoordinate source = playingHasActor
            ? fm.getPlayerCoordinate(actor)
            : fm.getPlayerCoordinate(game.getDefender());
        List<Player<?>> candidates = new ArrayList<>();
        for (Player<?> cand : game.getPlayers()) {
            if (cand == actor) continue;
            FieldCoordinate cc = fm.getPlayerCoordinate(cand);
            if (cc == null || source == null || cc.distanceInSteps(source) > 3) continue;
            com.fumbbl.ffb.PlayerState ps = fm.getPlayerState(cand);
            if (ps == null || ps.getBase() != com.fumbbl.ffb.PlayerState.STANDING) continue;
            if (cand.getTeam() == game.getActingTeam() && playingHasActor) continue;
            candidates.add(cand);
        }
        if (candidates.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        candidates.sort(Comparator
            .comparingInt((Player<?> cand) -> fm.getPlayerCoordinate(cand).getX())
            .thenComparingInt(cand -> fm.getPlayerCoordinate(cand).getY()));
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), candidates.size());
        Player<?> target = candidates.get(idx);
        if (DEBUG) System.err.println("JAVA_BLASTIN_TARGET N=" + candidates.size() + " idx=" + idx
            + " pid=" + target.getId() + " phase=" + (playingHasActor ? "initial" : "replacement"));
        // The command must come from the PLAYING coach (initial: acting team; replacement:
        // the opposing coach after the home_playing flip).
        MatchRunner.injectForTeam(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandTargetSelected(target.getId()),
            game.isHomePlaying());
        return;
    }

    private void sendPuntTarget(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer == null) ? null : actingPlayer.getPlayerId();
        FieldModel fm = game.getFieldModel();
        List<FieldCoordinate> squares = new ArrayList<>();
        for (com.fumbbl.ffb.MoveSquare ms : fm.getMoveSquares()) {
            squares.add(ms.getCoordinate());
        }
        squares.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        if (squares.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), squares.size());
        FieldCoordinate target = squares.get(idx);
        if (DEBUG) System.err.println("JAVA_PUNT pid=" + pid + " N=" + squares.size() + " idx=" + idx
            + " target=" + target);
        boolean isHome = pid != null && game.getTeamHome().getPlayerById(pid) != null;
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandFieldCoordinate(
            isHome ? target : target.transform()));
    }

    /**
     * Hit And Run move window (BB2020/BB2025 HIT_AND_RUN step). StepHitAndRun publishes the eligible
     * squares as MoveSquares and WAITS for a CLIENT_FIELD_COORDINATE naming one (CLIENT_END_TURN is
     * its abort). There was no handler, so the runner fell through to the UNHANDLED_STEP default and
     * aborted the move — and with the Rust agent aborting in lockstep, the `HitAndRun` step never
     * executed while the matrices stayed green.
     *
     * Same candidate/order/single-actionRng-pick contract as sendPuntTarget and sendSwoopTarget.
     */
    private void sendHitAndRunTarget(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer == null) ? null : actingPlayer.getPlayerId();
        FieldModel fm = game.getFieldModel();
        List<FieldCoordinate> squares = new ArrayList<>();
        for (com.fumbbl.ffb.MoveSquare ms : fm.getMoveSquares()) {
            squares.add(ms.getCoordinate());
        }
        squares.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        if (squares.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), squares.size());
        FieldCoordinate target = squares.get(idx);
        if (DEBUG) System.err.println("JAVA_HITRUN pid=" + pid + " N=" + squares.size() + " idx=" + idx
            + " target=" + target);
        boolean isHome = pid != null && game.getTeamHome().getPlayerById(pid) != null;
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandFieldCoordinate(
            isHome ? target : target.transform()));
    }

    private void sendSwoopTarget(Game game, GameState gameState) {
        ActingPlayer actingPlayer = game.getActingPlayer();
        String pid = (actingPlayer == null) ? null : actingPlayer.getPlayerId();
        FieldModel fm = game.getFieldModel();
        List<FieldCoordinate> squares = new ArrayList<>();
        for (com.fumbbl.ffb.MoveSquare ms : fm.getMoveSquares()) {
            squares.add(ms.getCoordinate());
        }
        squares.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        if (squares.isEmpty()) {
            // Nothing legal to send; end the turn rather than spin (the Rust agent does the same).
            MatchRunner.inject(gameState, new ClientCommandEndTurn(game.getTurnMode(), null));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), squares.size());
        FieldCoordinate target = squares.get(idx);
        if (DEBUG) System.err.println("JAVA_SWOOP pid=" + pid + " N=" + squares.size() + " idx=" + idx
            + " target=" + target);
        boolean isHome = pid != null && game.getTeamHome().getPlayerById(pid) != null;
        MatchRunner.inject(gameState, new com.fumbbl.ffb.net.commands.ClientCommandSwoop(
            pid, isHome ? target : target.transform()));
    }

    private void sendThrowTeamMateAction(Game game, GameState gameState, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        if (coord == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        Team team = game.isHomePlaying() ? game.getTeamHome() : game.getTeamAway();
        List<Player<?>> targets = new ArrayList<>();
        for (Player<?> tp : team.getPlayers()) {
            if (tp.getId().equals(playerId)) continue;
            // Player.canBeThrown() — the engine's own predicate, the one every edition's TtmMechanic
            // uses. Testing the raw canBeThrown PROPERTY instead (as this did) silently disabled
            // Throw Team-Mate for the whole BB2020 ruleset: bb2020's RightStuff registers
            // canBeThrownIfStrengthIs3orLess, not canBeThrown, so the candidate list was always
            // empty and StepThrowTeamMate never ran in a BB2020 game.
            if (!tp.canBeThrown()) continue;
            FieldCoordinate tc = fm.getPlayerCoordinate(tp);
            PlayerState ts = fm.getPlayerState(tp);
            if (tc == null || ts == null) continue;
            if (ts.getBase() == PlayerState.STANDING && isAdjacentCoord(coord, tc)) {
                targets.add(tp);
            }
        }
        sortPlayersByCoordinate(targets, fm);
        if (targets.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        String thrownId = targets.get(idx).getId();
        // A KICK declares through this same command — StepInitSelecting reads
        // ClientCommandThrowTeamMate.isKicked() and publishes IS_KICKED_PLAYER, which is how
        // StepEndSelecting learns to build the TTM sequence as a kick. Sending the command without
        // the flag made the engine resolve a declared KICK_TEAM_MATE as a plain throw: it spent the
        // team's PASS instead of its ktmUsed slot, so Java and Rust offered different actions for
        // the rest of the turn (ogre bb2020 seed 7 i=111).
        ActingPlayer ap = game.getActingPlayer();
        PlayerAction declared = (ap == null) ? null : ap.getPlayerAction();
        boolean kicked = declared == PlayerAction.KICK_TEAM_MATE
            || declared == PlayerAction.KICK_TEAM_MATE_MOVE;
        if (DEBUG) System.err.println("JAVA_TTM pid=" + playerId + " N=" + targets.size() + " idx=" + idx
            + " thrown=" + thrownId + " kicked=" + kicked);
        MatchRunner.inject(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandThrowTeamMate(playerId, thrownId, kicked));
    }

    /**
     * Throw Team-Mate — pick the target square (phase 2), sent once the thrown player is picked up
     * (StepInitThrowTeamMate is waiting). Deterministic, 0 actionRng, mirroring Rust's
     * ThrowTeamMateTarget handler: 3 squares toward the opponent end zone, clamped to the pitch,
     * sent in the acting client's view (canonical for home, mirrored for away).
     */
    private void sendThrowTeamMateTarget(Game game, GameState gameState, String playerId) {
        FieldCoordinate coord = playerCoordinate(game, playerId);
        if (coord == null) {
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        boolean isHome = game.isHomePlaying(); // the thrower is the acting player, on the playing team
        int dir = isHome ? 1 : -1;
        int tx = Math.max(0, Math.min(25, coord.getX() + dir * 3));
        int ty = Math.max(0, Math.min(14, coord.getY()));
        FieldCoordinate target = new FieldCoordinate(tx, ty);
        FieldCoordinate cmd = isHome ? target : target.transform();
        if (DEBUG) System.err.println("JAVA_TTM_TARGET pid=" + playerId + " target=(" + tx + "," + ty + ")");
        MatchRunner.inject(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandThrowTeamMate(playerId, cmd));
    }

    /**
     * Pushback choice (AGENT_CONTRACT.md §7): the min-(x, y) square in canonical
     * (server/home-view) coordinates among the unlocked pushback squares — matching
     * Rust's AgentPrompt::Pushback min_by_key((x, y)) response. Deterministic, 0 RNG.
     * The pushed player is the occupant of the square the push direction originates
     * from (same derivation as the real client's PushbackLogicModule.findPushback).
     */
    private void sendPushback(Game game, GameState gameState) {
        com.fumbbl.ffb.PushbackSquare[] squares = game.getFieldModel().getPushbackSquares();
        com.fumbbl.ffb.PushbackSquare best = null;
        // The heuristic agent SCORES the pushback (Rust `AgentPrompt::Pushback`, T = 0.15) over the
        // unlocked squares. Without it, AGENT_CONTRACT.md section 7's deterministic min-(x,y) pick
        // stands, so tier 2/3 and the lower rungs are untouched.
        if (heuristic != null
            && heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.PUSHBACK)
            && squares != null) {
            java.util.List<FieldCoordinate> unlocked = new ArrayList<>();
            for (com.fumbbl.ffb.PushbackSquare sq : squares) {
                if (sq == null || sq.isLocked() || sq.getCoordinate() == null) continue;
                unlocked.add(sq.getCoordinate());
            }
            if (!unlocked.isEmpty()) {
                ActingPlayer pbAp = game.getActingPlayer();
                String attackerId = (pbAp != null) ? pbAp.getPlayerId() : null;
                // Rust's prompt carries the step-local defender, i.e. the OCCUPANT being pushed --
                // which for every candidate square is the same player, so deriving it from any of
                // them agrees. Use the first square's push origin.
                FieldCoordinate probeFrom = null;
                for (com.fumbbl.ffb.PushbackSquare sq : squares) {
                    if (sq == null || sq.isLocked() || sq.getCoordinate() == null) continue;
                    probeFrom = pushOrigin(sq.getCoordinate(), sq.getDirection());
                    break;
                }
                Player<?> pushedProbe = (probeFrom != null) ? game.getFieldModel().getPlayer(probeFrom) : null;
                String defenderId = (pushedProbe != null) ? pushedProbe.getId() : null;
                FieldCoordinate chosen =
                    heuristic.pushbackChoice(game, attackerId, defenderId, unlocked);
                for (com.fumbbl.ffb.PushbackSquare sq : squares) {
                    if (sq == null || sq.isLocked() || sq.getCoordinate() == null) continue;
                    if (sq.getCoordinate().equals(chosen)) { best = sq; break; }
                }
            }
        }
        if (best == null && squares != null) {
            for (com.fumbbl.ffb.PushbackSquare sq : squares) {
                if (sq == null || sq.isLocked() || sq.getCoordinate() == null) continue;
                if (best == null
                        || sq.getCoordinate().getX() < best.getCoordinate().getX()
                        || (sq.getCoordinate().getX() == best.getCoordinate().getX()
                            && sq.getCoordinate().getY() < best.getCoordinate().getY())) {
                    best = sq;
                }
            }
        }
        if (best == null) {
            System.err.println("PUSHBACK_NO_SQUARES — deselecting");
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }
        FieldCoordinate to = best.getCoordinate();
        FieldCoordinate from = pushOrigin(to, best.getDirection());
        Player<?> pushed = (from != null) ? game.getFieldModel().getPlayer(from) : null;
        if (pushed == null) {
            System.err.println("PUSHBACK_NO_PUSHED_PLAYER to=" + to + " dir=" + best.getDirection());
            game.setDialogParameter(null);
            return;
        }
        boolean homeChoice = best.isHomeChoice();
        com.fumbbl.ffb.Pushback pushback = new com.fumbbl.ffb.Pushback(pushed.getId(), to);
        // StepPushback transforms commands from the away side back to canonical, so the
        // away choice must be sent in the away team's mirrored view.
        if (!homeChoice) pushback = pushback.transform();
        if (DEBUG) System.err.println("JAVA_PUSHBACK pushed=" + pushed.getId() + " to=(" + to.getX() + "," + to.getY() + ") homeChoice=" + homeChoice);
        MatchRunner.injectForTeam(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandPushback(pushback), homeChoice);
    }

    /** The square a push into `to` along `direction` originates from. */
    private static FieldCoordinate pushOrigin(FieldCoordinate to, com.fumbbl.ffb.Direction direction) {
        if (to == null || direction == null) return null;
        switch (direction) {
            case NORTH:     return to.add(0, 1);
            case NORTHEAST: return to.add(-1, 1);
            case EAST:      return to.add(-1, 0);
            case SOUTHEAST: return to.add(-1, -1);
            case SOUTH:     return to.add(0, -1);
            case SOUTHWEST: return to.add(1, -1);
            case WEST:      return to.add(1, 0);
            case NORTHWEST: return to.add(1, 1);
            default:        return null;
        }
    }

    /**
     * Drop snapshot actions no longer legal this turn (AGENT_CONTRACT.md §5) — the
     * eligible list is captured at turn start, so Blitz/Block/Pass/HandOver/Foul
     * entries may have been consumed by an earlier activation. Mirrors Rust's
     * RandomAgent::filter_stale_actions exactly; both sides must keep the action
     * pick's N identical. Move/StandUp always survive, so the result is never empty.
     */
    private static PlayerAction[] filterStaleActions(Game game, PlayerAction[] actions) {
        TurnData td = game.isHomePlaying() ? game.getTurnDataHome() : game.getTurnDataAway();
        List<PlayerAction> live = new ArrayList<>();
        // Non-REGULAR window modes (PASS_BLOCK): the harness contract shrinks the list to
        // MOVE + the UseSkill specials. A window Block/Blitz/Foul was always a
        // declare-then-deselect no-op, and a window BLITZ against the SUSPENDED THROWER
        // re-fires CONFIRM_END_ACTION forever (dark_elf bb2020 seed 61: this harness hit its
        // 2M-iteration cap). Rust's RandomAgent applies the identical filter so idx % N stays
        // aligned.
        if (game.getTurnMode() == com.fumbbl.ffb.TurnMode.PASS_BLOCK) {
            for (PlayerAction a : actions) {
                if (a == PlayerAction.MOVE || a == PlayerAction.TREACHEROUS || a == PlayerAction.BLACK_INK) {
                    live.add(a);
                }
            }
            return live.toArray(new PlayerAction[0]);
        }
        for (PlayerAction a : actions) {
            boolean keep;
            switch (a) {
                case BLOCK:
                case BLITZ:
                case STAND_UP_BLITZ:
                    keep = !td.isBlitzUsed();
                    break;
                case PASS:
                case HAIL_MARY_PASS:
                    keep = !td.isPassUsed();
                    break;
                case HAND_OVER:
                    keep = !td.isHandOverUsed();
                    break;
                case FOUL:
                    keep = !td.isFoulUsed();
                    break;
                case THROW_TEAM_MATE:
                    // BB2016 spends the team's PASS action on a Throw Team-Mate: bb2016
                    // ThrowTeamMateBehaviour does turnData.setPassUsed(true), and bb2016
                    // StepInitSelecting REJECTS CLIENT_THROW_TEAM_MATE while
                    // `!game.getTurnData().isPassUsed()` is false. Filtering only on isTtmUsed() let
                    // this runner re-declare a second TTM in the same turn forever - the step never
                    // advanced and the game died on STUCK_STEP: INIT_SELECTING (ogre bb2016 seed 1:
                    // TTMs declared at i=2 and i=6, then ~500 spins). Later editions keep the
                    // separate ttmUsed flag.
                    //
                    // BB2020 does the same as BB2016: `bb2020/ThrowTeamMateBehaviour` calls
                    // setPassUsed(true), and `bb2020/TtmMechanic.isTtmAvailable` is literally
                    // `!turnData.isPassUsed()`. Only BB2025 tracks TTM on its own flag. Excluding
                    // BB2020 here reproduced the bb2016 symptom exactly, once BB2020 could throw at
                    // all: the runner re-declared a TTM whose command StepInitSelecting then
                    // refused, and 9 of 10 ogre bb2020 seeds died on
                    // `STUCK_STEP: INIT_SELECTING unadvanced for 501 iters`.
                    keep = isBb2025(game) ? !td.isTtmUsed() : (!td.isTtmUsed() && !td.isPassUsed());
                    break;
                case KICK_TEAM_MATE:
                    // BB2016 spends the team's BLITZ on a Kick Team-Mate
                    // (bb2016/TtmMechanic.isKtmAvailable is !turnData.isBlitzUsed()); BB2020 and
                    // BB2025 track it on their own flag.
                    keep = isBb2016(game) ? !td.isBlitzUsed() : !td.isKtmUsed();
                    break;
                default:
                    keep = true;
                    break;
            }
            if (keep) live.add(a);
        }
        return live.toArray(new PlayerAction[0]);
    }

    /**
     * The block-die index to send. Delegates to the heuristic agent when the {@code blockchoice}
     * class is switched on, and otherwise answers 0 — the value the byte-matched random parity
     * contract picks, so switching the class off leaves the existing gate untouched.
     *
     * <p>{@code nrOfDice < 0} means the dice are "against" the attacker and the DEFENDING coach
     * chooses; Rust carries that as {@code own_choice = nr_of_dice >= 0} and flips every weight.
     */
    private int heuristicBlockChoice(Game game, int nrOfDice, int[] blockRoll) {
        if (heuristic == null
            || !heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.BLOCK_CHOICE)
            || blockRoll == null
            || blockRoll.length == 0) {
            return 0;
        }
        String attackerId = (game.getActingPlayer() != null)
            ? game.getActingPlayer().getPlayerId() : null;
        return heuristic.blockChoice(game, attackerId, game.getDefenderId(), blockRoll,
            nrOfDice >= 0);
    }

    /**
     * The re-roll source to answer a re-roll offer with, or null to DECLINE.
     *
     * <p>The random parity contract always declines (AGENT_CONTRACT.md section 7) and that is still
     * what happens whenever the heuristic driver is absent or does not own the `reroll` class --
     * so tier 2, tier 3 and every lower rung keep their existing byte-matched streams.
     *
     * <p>When it DOES own the class, the decision comes from the mirrored scorer and an acceptance
     * is answered with the team re-roll, which is the source Rust's engine consumes for the
     * "TRR" ReRollSource its ask_for_reroll_if_available publishes.
     */
    private com.fumbbl.ffb.ReRollSource reRollSourceFor(
            Game game, com.fumbbl.ffb.ReRolledAction action, boolean teamReRollOption) {
        if (heuristic == null
            || !heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.RE_ROLL_OFFER)) {
            return null;
        }
        if (!teamReRollOption) {
            return null;
        }
        return heuristic.useReRoll(game, action) ? com.fumbbl.ffb.ReRollSources.TEAM_RE_ROLL : null;
    }

    /** Sort players by their coordinate (x, y) ascending — AGENT_CONTRACT.md §6. */
    private static void sortPlayersByCoordinate(List<Player<?>> players, FieldModel fm) {
        players.sort(Comparator.comparingInt((Player<?> p) -> {
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            return c != null ? c.getX() : Integer.MAX_VALUE;
        }).thenComparingInt((Player<?> p) -> {
            FieldCoordinate c = fm.getPlayerCoordinate(p);
            return c != null ? c.getY() : Integer.MAX_VALUE;
        }));
    }

    /** The acting team's player coordinate, or null when off-pitch/unknown. */
    private static FieldCoordinate playerCoordinate(Game game, String playerId) {
        Player<?> p = game.getPlayerById(playerId);
        return (p != null) ? game.getFieldModel().getPlayerCoordinate(p) : null;
    }

    /**
     * After a player is activated for Move, send a concrete 1-step move command.
     * Uses actionRng to pick an adjacent empty square, matching Rust's
     * RandomAgent::compute_follow_up() which uses action_rng for legal_move_targets().
     *
     * Field: x in [0, 25], y in [0, 14] (same as Rust's FIELD_WIDTH=26, FIELD_HEIGHT=15).
     * Occupancy: any player (home or away) at the target square makes it illegal.
     * Sort: by (x, y) before picking — matches Rust's sorted target list.
     */
    /**
     * The eight neighbours of {@code from} that are on pitch, unoccupied by ANY player, and not
     * already on the planned path — coordinate-sorted.
     *
     * <p>Byte-mirror of Rust's {@code random_agent::free_neighbours}. This was the inline loop
     * inside {@link #sendMoveAction}; it is factored out so the {@code --multimove} spike walks by
     * exactly the same rule at every step rather than a second, subtly different one.
     *
     * <p>Sorted by {@code (x, y)} and never by player id — AGENT_CONTRACT.md section 6.
     */
    private static List<FieldCoordinate> freeNeighbours(
            Game game, FieldCoordinate from, List<FieldCoordinate> exclude) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        int[] dx = {0, 1, 1, 1, 0, -1, -1, -1};  // N, NE, E, SE, S, SW, W, NW
        int[] dy = {-1, -1, 0, 1, 1, 1, 0, -1};
        List<FieldCoordinate> targets = new ArrayList<>();
        for (int i = 0; i < 8; i++) {
            int nx = from.getX() + dx[i];
            int ny = from.getY() + dy[i];
            if (nx < 0 || nx > 25 || ny < 0 || ny > 14) { continue; }
            FieldCoordinate nc = new FieldCoordinate(nx, ny);
            boolean occupied = false;
            for (com.fumbbl.ffb.model.Player<?> p : game.getTeamHome().getPlayers()) {
                if (nc.equals(fm.getPlayerCoordinate(p))) { occupied = true; break; }
            }
            if (!occupied) {
                for (com.fumbbl.ffb.model.Player<?> p : game.getTeamAway().getPlayers()) {
                    if (nc.equals(fm.getPlayerCoordinate(p))) { occupied = true; break; }
                }
            }
            if (occupied || exclude.contains(nc)) { continue; }
            targets.add(nc);
        }
        // Sort by (x, y) — matches Rust's .sort_by_key(|c| (c.x, c.y))
        targets.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));
        return targets;
    }

    private void sendMoveAction(Game game, GameState gameState, String playerId) {
        com.fumbbl.ffb.model.FieldModel fm = game.getFieldModel();
        // Find the player's current coordinate
        FieldCoordinate coord = null;
        Team team = game.isHomePlaying() ? game.getTeamHome() : game.getTeamAway();
        for (com.fumbbl.ffb.model.Player<?> p : team.getPlayers()) {
            if (p.getId().equals(playerId)) {
                coord = fm.getPlayerCoordinate(p);
                break;
            }
        }
        if (coord == null) {
            // No coordinate: deselect instead
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }

        List<FieldCoordinate> targets = freeNeighbours(game, coord, java.util.Collections.emptyList());

        if (DEBUG) System.err.println("JAVA_SMA pid=" + playerId + " coord=" + coord.getX() + "," + coord.getY() + " targets=" + targets.size() + " isHome=" + game.isHomePlaying());

        if (targets.isEmpty()) {
            // No adjacent empty square: deselect (player stays, activation counted but no move)
            MatchRunner.inject(gameState, new ClientCommandActingPlayer(null, null, false));
            return;
        }

        // Pick random target using actionRng (matches Rust's action_rng.pick_action(targets.len()))
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), targets.size());
        FieldCoordinate target = targets.get(idx);
        if (DEBUG) System.err.println("JAVA_PICK pid=" + playerId + " N=" + targets.size() + " idx=" + idx + " t=(" + target.getX() + "," + target.getY() + ")");

        // The planned path. One square unless the --multimove spike is on, in which case keep
        // walking by the SAME candidate rule, one actionRng draw per extra square, capped at the
        // player's MA + 2 (the two rushes) so we never propose a path the engine must refuse.
        // Byte-mirror of RandomAgent::extend_multimove.
        List<FieldCoordinate> path = new ArrayList<>();
        path.add(target);
        if (multimove > 1) {
            com.fumbbl.ffb.model.Player<?> mover = game.getPlayerById(playerId);
            int ma = (mover != null) ? mover.getMovementWithModifiers() : 0;
            ActingPlayer ap = game.getActingPlayer();
            int spent = (ap != null) ? ap.getCurrentMove() : 0;
            int budget = Math.max(0, ma + 2 - spent);
            int want = Math.min(multimove, budget);
            while (path.size() < want) {
                FieldCoordinate from = path.get(path.size() - 1);
                List<FieldCoordinate> cands = freeNeighbours(game, from, path);
                if (cands.isEmpty()) { break; }
                int k = (int) Long.remainderUnsigned(actionRng.nextLong(), cands.size());
                path.add(cands.get(k));
            }
        }
        // StepInitSelecting.fetchMoveStack/fetchFromSquare mirrors coords when homeCommand=false.
        // Coordinates from getPlayerCoordinate are in server-canonical (home-relative) space.
        // Away team commands must be in the away team's view (mirrored), so the server can
        // un-mirror them back to canonical. Home team commands are passed through as-is.
        boolean isHome = game.isHomePlaying();
        FieldCoordinate cmdFrom = isHome ? coord : coord.transform();

        // Send move command: ClientCommandMove(actingPlayerId, fromCoord, path[], null).
        // Every element of the path is mirrored for the away coach, exactly as
        // UtilServerPlayerMove.fetchMoveStack un-mirrors it server-side.
        FieldCoordinate[] cmdPath = new FieldCoordinate[path.size()];
        for (int pi = 0; pi < path.size(); pi++) {
            FieldCoordinate c = path.get(pi);
            cmdPath[pi] = isHome ? c : c.transform();
        }
        if (DEBUG) System.err.println("JAVA_PATH pid=" + playerId + " len=" + path.size()
            + " currentMove=" + (game.getActingPlayer() != null ? game.getActingPlayer().getCurrentMove() : -1)
            + " multimove=" + multimove);
        ClientCommandMove moveCmd = new ClientCommandMove(playerId, cmdFrom, cmdPath, null);
        MatchRunner.inject(gameState, moveCmd);
    }

    // ── Eligible player computation (mirrors Rust's eligible_players_for_activation) ──

    /**
     * Computes the list of players eligible for activation and their available actions,
     * matching Rust's eligible_players_for_activation() exactly.
     *
     * Returns a list of [playerId (String), actions (PlayerAction[])] pairs,
     * in team roster order (no sorting — same as Rust's team.players.iter()).
     */
    private List<Object[]> computeEligiblePlayers(Game game) {
        boolean homePlaying = game.isHomePlaying();
        Team team = homePlaying ? game.getTeamHome() : game.getTeamAway();
        Team opponent = homePlaying ? game.getTeamAway() : game.getTeamHome();
        FieldModel fm = game.getFieldModel();
        TurnData td = homePlaying ? game.getTurnDataHome() : game.getTurnDataAway();
        FieldCoordinate ballCoord = fm.getBallCoordinate();

        List<Object[]> eligible = new ArrayList<>();

        for (Player<?> p : team.getPlayers()) {
            PlayerState ps = fm.getPlayerState(p);
            FieldCoordinate coord = fm.getPlayerCoordinate(p);
            if (ps == null || coord == null) continue;

            boolean onPitch = coord.getX() >= 0 && coord.getX() <= 25
                           && coord.getY() >= 0 && coord.getY() <= 14;
            if (!onPitch) continue;

            // Only STANDING (includes MOVING, BLOCKED) or PRONE players can be activated.
            // EXHAUSTED players are excluded by isStanding() returning false for base=14.
            boolean isStanding = ps.isStanding();
            boolean isProne = (ps.getBase() == PlayerState.PRONE);
            if (!isStanding && !isProne) { continue; }

            List<PlayerAction> actions = new ArrayList<>();

            if (isProne) {
                // Prone players: MOVE (= stand up and move), optionally BLITZ (= stand up and blitz)
                // Mirrors Rust: [StandUp (→PlayerActionChoice::Move), StandUpBlitz (→Blitz)]
                actions.add(PlayerAction.MOVE);
                if (!td.isBlitzUsed()) {
                    if (hasAdjacentPlayerWithTackleZones(coord, opponent.getPlayers(), fm)) {
                        actions.add(PlayerAction.BLITZ);
                    }
                }
            } else {
                // Standing player — actions in same order as Rust's eligible_players_for_activation
                actions.add(PlayerAction.MOVE);

                // Block + Blitz: mirrors Rust eligible_players_for_activation — BOTH are
                // offered together, and only when adjacent to an opponent with tackle
                // zones and no blitz used yet (no standalone move-into-contact Blitz).
                if (!td.isBlitzUsed()) {
                    if (hasAdjacentBlockTarget(coord, opponent.getPlayers(), fm)) {
                        actions.add(PlayerAction.BLOCK);
                        actions.add(PlayerAction.BLITZ);
                    }
                }

                // Pass: player carries the ball. Mirrors Rust eligible_players_for_activation:
                // a player whose skills prevent a regular pass (My Ball / No Ball →
                // preventRegularPassAction) is NOT offered PASS — the engine forbids it, so the
                // eligible list must exclude it or the two agents' turn-start snapshots diverge in
                // size, shifting the shared actionRng modulo (high_elf seed 14 i=46: the My Ball
                // Dragon Prince carrier got a 5-action snapshot here vs Rust's 4, so idx%N picked
                // BLOCK in Rust but MOVE in Java).
                if (!td.isPassUsed() && ballCoord != null && ballCoord.equals(coord)
                        && !p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.preventRegularPassAction)) {
                    actions.add(PlayerAction.PASS);
                }

                // Hail Mary Pass: a canPassToAnySquare carrier declares HAIL_MARY_PASS as its
                // own action. Offered DIRECTLY AFTER PASS — Rust's
                // eligible_players_for_activation inserts it at the same slot, and the two
                // turn-start snapshots must match in length and order.
                if (!td.isPassUsed() && ballCoord != null && ballCoord.equals(coord)
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canPassToAnySquare)) {
                    actions.add(PlayerAction.HAIL_MARY_PASS);
                }

                // Hand-off: player carries ball and adjacent teammate
                if (!td.isHandOverUsed() && ballCoord != null && ballCoord.equals(coord)) {
                    if (hasAdjacentTeammate(p, coord, team.getPlayers(), fm)) {
                        actions.add(PlayerAction.HAND_OVER);
                    }
                }

                // Foul: adjacent to prone/stunned opponent, foul not used
                // (SneakiestOfTheLot exception not handled here for simplicity)
                if (!td.isFoulUsed()) {
                    if (hasAdjacentFoulTarget(coord, opponent.getPlayers(), fm)) {
                        actions.add(PlayerAction.FOUL);
                    }
                }

                // ThrowBomb (Bombardier): shares the pass-action slot
                if (!td.isBombUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.enableThrowBombAction)) {
                    actions.add(PlayerAction.THROW_BOMB);

                    // All You Can Eat (bb2020+ star special): the client's
                    // isAllYouCanEatAvailable rule — Throw Bomb available + REGULAR turn mode
                    // + unused canUseThrowBombActionTwice. Offered DIRECTLY AFTER THROW_BOMB
                    // — Rust matches the slot.
                    if (game.getTurnMode() == TurnMode.REGULAR
                            && com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                                com.fumbbl.ffb.model.property.NamedProperties.canUseThrowBombActionTwice)) {
                        actions.add(PlayerAction.ALL_YOU_CAN_EAT);
                    }
                }

                // ThrowTeamMate: TTM skill + adjacent teammate
                if (!td.isTtmUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canThrowTeamMates)) {
                    if (hasAdjacentTeammate(p, coord, team.getPlayers(), fm)) {
                        actions.add(PlayerAction.THROW_TEAM_MATE);
                    }
                }

                // KickTeamMate: KTM skill + adjacent teammate. Available in EVERY edition —
                // skill/mixed/KickTeamMate is registered for BB2020 and BB2025, skill/bb2016 for
                // BB2016, and all three TtmMechanics implement isKtmAvailable. Restricting this to
                // BB2025 left the mechanic unreachable in BB2020 even though the BB2020 ogre roster
                // carries the skill — the same shape of harness-side blind spot that kept BB2020
                // Throw Team-Mate dead. BB2016 spends the blitz instead of a ktmUsed flag.
                boolean ktmAvailable = isBb2016(game) ? !td.isBlitzUsed() : !td.isKtmUsed();
                if (ktmAvailable
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canKickTeamMates)) {
                    if (hasAdjacentTeammate(p, coord, team.getPlayers(), fm)) {
                        actions.add(PlayerAction.KICK_TEAM_MATE);
                    }
                }

                // Multiple Block (bb2020/bb2025): the client's isMultiBlockActionAvailable rule
                // — canBlockTwoAtOnce (uncancelled) + standing + MORE THAN ONE adjacent
                // blockable opponent. bb2016's multi-block is a different mechanism, so the
                // offer is bb2020/bb2025-only. Rust inserts it at the same slot.
                if (!isBb2016(game)
                        && com.fumbbl.ffb.util.UtilCards.hasSkillWithProperty(p,
                            com.fumbbl.ffb.model.property.NamedProperties.canBlockTwoAtOnce)
                        && !p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.preventRegularBlockAction)
                        && com.fumbbl.ffb.util.UtilPlayer.findAdjacentBlockablePlayers(game, opponent, coord).length > 1) {
                    actions.add(PlayerAction.MULTIPLE_BLOCK);
                }

                // Treacherous (bb2020+ star special): the client's isTreacherousAvailable rule —
                // unused canStabTeamMateForBall skill + an adjacent BLOCKABLE teammate carrying
                // the ball (bb2025 SelectLogicModule). Declared as the client's command pair:
                // ClientCommandActingPlayer(PASS_MOVE) + ClientCommandUseSkill(treacherous).
                // Offered DIRECTLY AFTER KICK_TEAM_MATE — Rust inserts it at the same slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canStabTeamMateForBall)
                        && java.util.Arrays.stream(com.fumbbl.ffb.util.UtilPlayer.findAdjacentBlockablePlayers(
                                game, team, coord))
                            .anyMatch(tp -> com.fumbbl.ffb.util.UtilPlayer.hasBall(game, tp))) {
                    actions.add(PlayerAction.TREACHEROUS);
                }

                // Raiding Party (bb2025 star special): the client's isRaidingPartyAvailable
                // rule (LogicModule) — unused canMoveOpenTeamMate + an acting-team mate that is
                // STANDING, within 5 steps, OPEN (no adjacent opponents with tacklezones) and
                // has an adjacent EMPTY in-field square that itself neighbours an opponent.
                // Offered DIRECTLY AFTER TREACHEROUS — Rust inserts it at the same slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canMoveOpenTeamMate)
                        && java.util.Arrays.stream(team.getPlayers()).anyMatch(tm -> {
                            FieldCoordinate tc = fm.getPlayerCoordinate(tm);
                            if (tc == null) return false;
                            com.fumbbl.ffb.PlayerState ts = fm.getPlayerState(tm);
                            if (ts == null || ts.getBase() != com.fumbbl.ffb.PlayerState.STANDING) return false;
                            if (tc.distanceInSteps(coord) > 5) return false;
                            if (com.fumbbl.ffb.util.ArrayTool.isProvided(
                                    com.fumbbl.ffb.util.UtilPlayer.findAdjacentPlayersWithTacklezones(
                                        game, opponent, tc, false))) return false;
                            return java.util.Arrays.stream(fm.findAdjacentCoordinates(tc,
                                    com.fumbbl.ffb.FieldCoordinateBounds.FIELD, 1, false))
                                .anyMatch(sq -> {
                                    java.util.List<Player<?>> onSq = fm.getPlayers(sq);
                                    if (onSq != null && !onSq.isEmpty()) return false;
                                    return java.util.Arrays.stream(fm.findAdjacentCoordinates(sq,
                                            com.fumbbl.ffb.FieldCoordinateBounds.FIELD, 1, false))
                                        .anyMatch(adj -> {
                                            java.util.List<Player<?>> occ = fm.getPlayers(adj);
                                            return occ != null && !occ.isEmpty()
                                                && !team.hasPlayer(occ.get(0));
                                        });
                                });
                        })) {
                    actions.add(PlayerAction.RAIDING_PARTY);
                }

                // Look Into My Eyes (bb2025 star special): the client's
                // isLookIntoMyEyesAvailable rule — unused canStealBallFromOpponent + an
                // adjacent BLOCKABLE opponent carrying the ball. Offered DIRECTLY AFTER
                // RAIDING_PARTY — Rust inserts it at the same slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canStealBallFromOpponent)
                        && java.util.Arrays.stream(com.fumbbl.ffb.util.UtilPlayer.findAdjacentBlockablePlayers(
                                game, opponent, coord))
                            .anyMatch(op -> com.fumbbl.ffb.util.UtilPlayer.hasBall(game, op))) {
                    actions.add(PlayerAction.LOOK_INTO_MY_EYES);
                }

                // Baleful Hex (bb2025 star special): the client's isBalefulHexAvailable
                // rule — unused canMakeOpponentMissTurn + any opponent within 5 Chebyshev
                // steps. Offered DIRECTLY AFTER LOOK_INTO_MY_EYES — Rust matches the slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canMakeOpponentMissTurn)
                        && java.util.Arrays.stream(opponent.getPlayers()).anyMatch(op -> {
                            FieldCoordinate oc = fm.getPlayerCoordinate(op);
                            return oc != null && oc.distanceInSteps(coord) <= 5;
                        })) {
                    actions.add(PlayerAction.BALEFUL_HEX);
                }

                // Catch of the Day (bb2025 star special): the client's
                // isCatchOfTheDayAvailable rule — unused canGetBallOnGround + loose ball
                // (isBallMoving) within 3 steps. Offered DIRECTLY AFTER BALEFUL_HEX — Rust
                // matches the slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canGetBallOnGround)
                        && fm.isBallMoving() && fm.getBallCoordinate() != null
                        && fm.getBallCoordinate().distanceInSteps(coord) <= 3) {
                    actions.add(PlayerAction.CATCH_OF_THE_DAY);
                }

                // "Blastin' Solves Everything" (bb2025 star special): the client's
                // isThenIStartedBlastinAvailable rule — unused canBlastRemotePlayer + any
                // opponent within 3 steps. Offered DIRECTLY AFTER CATCH_OF_THE_DAY — Rust
                // matches the slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canBlastRemotePlayer)
                        && java.util.Arrays.stream(opponent.getPlayers()).anyMatch(op -> {
                            FieldCoordinate oc = fm.getPlayerCoordinate(op);
                            return oc != null && oc.distanceInSteps(coord) <= 3;
                        })) {
                    actions.add(PlayerAction.THEN_I_STARTED_BLASTIN);
                }

                // Furious Outburst (bb2025 star special): the client's
                // isFuriousOutburstAvailable rule — ACTIVE + STANDING, the team blitz not yet
                // used, unused canTeleportBeforeAndAfterAvRollAttack, and a blockable opponent
                // within 3 steps. Offered DIRECTLY AFTER THEN_I_STARTED_BLASTIN — Rust inserts
                // it at the same slot.
                if (ps.isActive() && ps.getBase() == PlayerState.STANDING
                        && !td.isBlitzUsed()
                        && com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                            com.fumbbl.ffb.model.property.NamedProperties.canTeleportBeforeAndAfterAvRollAttack)
                        && com.fumbbl.ffb.util.ArrayTool.isProvided(
                            com.fumbbl.ffb.util.UtilPlayer.findBlockablePlayers(game, opponent, coord, 3))) {
                    actions.add(PlayerAction.FURIOUS_OUTPBURST);
                }

                // Beer Barrel Bash! (bb2020+ star special): the client's isThrowKegAvailable rule
                // — REGULAR turn mode, base STANDING, unused canThrowKeg. The rule itself has NO
                // target clause (the target is clicked afterwards in the client's THROW_KEG
                // state), so sendConcreteAction below deselects when no valid target exists.
                // Offered DIRECTLY AFTER FURIOUS_OUTPBURST — Rust inserts it at the same slot.
                if (game.getTurnMode() == TurnMode.REGULAR
                        && ps.getBase() == PlayerState.STANDING
                        && com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                            com.fumbbl.ffb.model.property.NamedProperties.canThrowKeg)) {
                    actions.add(PlayerAction.THROW_KEG);
                }

                // Wisdom of the White Dwarf (bb2020+ star special): the client's
                // isWisdomAvailable rule, which delegates to GameMechanic.isWisdomAvailable —
                // unused canGrantSkillsToTeamMates plus a STANDING-or-PRONE, ACTIVE team-mate
                // within 2 squares that is missing at least one grantable skill. Offered
                // DIRECTLY AFTER THROW_KEG — Rust inserts it at the same slot.
                if (coord != null
                        && ((com.fumbbl.ffb.mechanics.GameMechanic) game.getMechanic(
                                com.fumbbl.ffb.mechanics.Mechanic.Type.GAME))
                            .isWisdomAvailable(game, p)) {
                    actions.add(PlayerAction.WISDOM_OF_THE_WHITE_DWARF);
                }

                // "Excuse Me, Are You a Zoat?" (bb2025 star special): the client's
                // isAutoGazeZoatAvailable rule — unused canGazeAutomaticallyThreeSquaresAway
                // plus an opponent WITH TACKLE ZONES within 3 that is not already distracted.
                // Note the property is NOT canGazeAutomatically (that is Black Ink's); both sit
                // on the same CLIENT_USE_SKILL dispatch chain in StepInitSelecting.
                // Offered DIRECTLY AFTER WISDOM_OF_THE_WHITE_DWARF — Rust matches the slot.
                if (coord != null
                        && com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                            com.fumbbl.ffb.model.property.NamedProperties.canGazeAutomaticallyThreeSquaresAway)
                        && java.util.Arrays.stream(com.fumbbl.ffb.util.UtilPlayer.findPlayersWithTackleZones(
                                game, opponent, coord, 3))
                            .anyMatch(op -> !fm.getPlayerState(op).isDistracted())) {
                    actions.add(PlayerAction.AUTO_GAZE_ZOAT);
                }

                // Black Ink (bb2020+ star special): the client's isBlackInkAvailable rule —
                // unused canGazeAutomatically skill + an adjacent standing-or-prone,
                // NOT-distracted opponent. Declared as ActingPlayer(MOVE) +
                // ClientCommandUseSkill; the player continues the move after the gaze.
                // Offered DIRECTLY AFTER TREACHEROUS — Rust inserts it at the same slot.
                if (com.fumbbl.ffb.util.UtilCards.hasUnusedSkillWithProperty(p,
                        com.fumbbl.ffb.model.property.NamedProperties.canGazeAutomatically)
                        && java.util.Arrays.stream(com.fumbbl.ffb.util.UtilPlayer.findAdjacentStandingOrPronePlayers(
                                game, opponent, coord))
                            .anyMatch(op -> !fm.getPlayerState(op).isDistracted())) {
                    actions.add(PlayerAction.BLACK_INK);
                }

                // Punt (BB2025 only): Punt skill, ball carrier, ball in play
                if (isBb2025(game) && !td.isPuntUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canPunt)
                        && fm.isBallInPlay() && ballCoord != null && ballCoord.equals(coord)) {
                    actions.add(PlayerAction.PUNT);
                }

                // SecureTheBall (BB2025 only): ball moving through this player's square.
                // Mirror Rust eligible_players_for_activation: a player with the Unsteady trait
                // (NamedProperties.preventSecureTheBallAction) may NOT Secure the Ball. Without this
                // the harness offered SecureTheBall to an Unsteady Flesh Golem that Rust omits, so
                // the two agents' eligible lists differed in size and the shared actionRng picked a
                // different action (necromantic seed 83 i=142: home_03 SECURE_THE_BALL vs HandOff).
                if (isBb2025(game) && fm.isBallInPlay() && fm.isBallMoving()
                        && ballCoord != null && ballCoord.equals(coord)
                        && !td.isSecureTheBallUsed()
                        && !p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.preventSecureTheBallAction)) {
                    actions.add(PlayerAction.SECURE_THE_BALL);
                }

                // HypnoticGaze: player moves and gazes an adjacent opponent (canGazeDuringMove)
                if (p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canGazeDuringMove)) {
                    actions.add(PlayerAction.GAZE);
                }
            }

            eligible.add(new Object[] { p.getId(), actions.toArray(new PlayerAction[0]) });
        }

        return eligible;
    }

    private static boolean hasAdjacentPlayerWithTackleZones(
            FieldCoordinate coord, Player<?>[] players, FieldModel fm) {
        for (Player<?> op : players) {
            PlayerState ops = fm.getPlayerState(op);
            FieldCoordinate oc = fm.getPlayerCoordinate(op);
            if (oc == null || ops == null) continue;
            if (isAdjacentCoord(coord, oc) && hasTackleZones(ops)) return true;
        }
        return false;
    }

    private static boolean hasAdjacentBlockTarget(
            FieldCoordinate coord, Player<?>[] opponents, FieldModel fm) {
        // Mirrors Rust: adjacent to opponent with tackle zones (range 1; ViciousVines/etc. ignored)
        for (Player<?> op : opponents) {
            PlayerState ops = fm.getPlayerState(op);
            FieldCoordinate oc = fm.getPlayerCoordinate(op);
            if (oc == null || ops == null) continue;
            if (isAdjacentCoord(coord, oc) && hasTackleZones(ops)) return true;
        }
        return false;
    }

    private static boolean hasAdjacentTeammate(
            Player<?> self, FieldCoordinate coord, Player<?>[] teammates, FieldModel fm) {
        for (Player<?> tp : teammates) {
            if (tp.getId().equals(self.getId())) continue;
            FieldCoordinate tc = fm.getPlayerCoordinate(tp);
            if (tc != null && isAdjacentCoord(coord, tc)) return true;
        }
        return false;
    }

    private static boolean hasAdjacentFoulTarget(
            FieldCoordinate coord, Player<?>[] opponents, FieldModel fm) {
        for (Player<?> op : opponents) {
            PlayerState ops = fm.getPlayerState(op);
            FieldCoordinate oc = fm.getPlayerCoordinate(op);
            if (oc == null || ops == null) continue;
            int base = ops.getBase();
            if ((base == PlayerState.PRONE || base == PlayerState.STUNNED)
                    && isAdjacentCoord(coord, oc)) return true;
        }
        return false;
    }

    private static boolean isBb2025(Game game) {
        return game.getOptions().getRulesVersion() == com.fumbbl.ffb.RulesCollection.Rules.BB2025;
    }

    private static boolean isBb2016(Game game) {
        return game.getOptions().getRulesVersion() == com.fumbbl.ffb.RulesCollection.Rules.BB2016;
    }

    /** True if two coordinates are 8-directionally adjacent (distance ≤ 1, not same square). */
    private static boolean isAdjacentCoord(FieldCoordinate a, FieldCoordinate b) {
        int dx = Math.abs(a.getX() - b.getX());
        int dy = Math.abs(a.getY() - b.getY());
        return dx <= 1 && dy <= 1 && (dx + dy > 0);
    }

    /**
     * True if the player projects tackle zones.
     * Mirrors Rust: base == STANDING|MOVING|BLOCKED and not confused/hypnotized/eye-gouged.
     */
    private static boolean hasTackleZones(PlayerState ps) {
        int base = ps.getBase();
        if (base != PlayerState.STANDING && base != PlayerState.MOVING && base != 12 /*BLOCKED*/) {
            return false;
        }
        try {
            // Confused or hypnotized players don't project tackle zones
            if (ps.isConfused() || ps.isHypnotized()) return false;
        } catch (Exception ignored) {
            // If these methods don't exist, treat as not distracted
        }
        return true;
    }

    // ── Captured-command injection helper ─────────────────────────────────────

    /** Inject the command captured by comm into the game state, routing by team if known. */
    private void injectCaptured(IDialogParameter dialog, Game game, GameState gameState) {
        com.fumbbl.ffb.net.commands.ClientCommand captured = comm.getCapturedCommand();
        if (captured != null) {
            String teamId = getDialogTeamId(dialog);
            try {
                if (teamId != null) {
                    MatchRunner.injectForTeam(gameState, captured,
                        teamId.equals(game.getTeamHome().getId()));
                } else {
                    MatchRunner.inject(gameState, captured);
                }
            } catch (RuntimeException e) {
                game.setDialogParameter(null);
            }
        } else {
            game.setDialogParameter(null);
        }
    }

    // ── Setup helpers (mirrors MatchRunner) ───────────────────────────────────

    private static void resetCurrentTeam(Game game) {
        boolean homePlaying = game.isHomePlaying();
        Team team = homePlaying ? game.getTeamHome() : game.getTeamAway();
        FieldModel fm = game.getFieldModel();
        for (Player<?> p : team.getPlayers()) {
            PlayerState ps = fm.getPlayerState(p);
            if (ps.canBeSetUpNextDrive()) {
                fm.setPlayerState(p, ps.changeBase(PlayerState.RESERVE));
                UtilBox.putPlayerIntoBox(game, p);
            }
        }
    }

    private static void placeReserves(Game game, GameState gameState) {
        boolean homePlaying = game.isHomePlaying();
        Team team = homePlaying ? game.getTeamHome() : game.getTeamAway();

        FieldModel fm = game.getFieldModel();

        List<Player<?>> players = new ArrayList<>(java.util.Arrays.asList(team.getPlayers()));
        players.sort(java.util.Comparator.comparingInt(Player::getNr));

        // Only place players in RESERVE state — injured (BADLY_HURT/SI/RIP) and
        // still-KO'd players must not be placed. Filter availability FIRST, then cap at 11:
        // with 12+-man drafted teams, a KO'd jersey 1-11 must be replaced from the reserves
        // (jersey 12+), else fewer than min(11, available) players are fielded and the setup
        // validation loops on SETUP_ERROR forever (half-2 setup, human seed 2). Matches
        // Rust's canonical_setup_action().
        List<Player<?>> available = new ArrayList<>();
        for (Player<?> p : players) {
            PlayerState ps = fm.getPlayerState(p);
            if (ps != null && ps.getBase() == PlayerState.RESERVE) {
                available.add(p);
            }
        }
        if (available.size() > 11) available = available.subList(0, 11);

        int[][] losSquares = {{12,7},{12,6},{12,8},{12,5},{12,9},{12,4},{12,10}};
        int[][] overflowSq = {{5,5},{5,7},{5,9},{6,6},{6,8},{4,6},{4,8},{3,6},{3,8},{2,5},{2,9},{1,7}};
        int li = 0, oi = 0;
        int placed = 0;
        int n = available.size();
        int losNeeded = n >= 3 ? 3 : n;

        for (Player<?> p : available) {
            if (placed >= n) break;

            if (losNeeded > 0) {
                while (li < losSquares.length) {
                    int ox = losSquares[li][0], oy = losSquares[li++][1];
                    FieldCoordinate gc = homePlaying
                        ? new FieldCoordinate(ox, oy)
                        : new FieldCoordinate(ox, oy).transform();
                    if (fm.getPlayer(gc) == null) {
                        UtilServerSetup.setupPlayer(gameState, p.getId(), new FieldCoordinate(ox, oy));
                        losNeeded--;
                        placed++;
                        break;
                    }
                }
            } else {
                while (oi < overflowSq.length) {
                    int ox = overflowSq[oi][0], oy = overflowSq[oi++][1];
                    FieldCoordinate gc = homePlaying
                        ? new FieldCoordinate(ox, oy)
                        : new FieldCoordinate(ox, oy).transform();
                    if (fm.getPlayer(gc) == null) {
                        UtilServerSetup.setupPlayer(gameState, p.getId(), new FieldCoordinate(ox, oy));
                        placed++;
                        break;
                    }
                }
            }
        }
    }

    // ── Dialog team resolution ────────────────────────────────────────────────

    /**
     * The PlayerActions sendConcreteAction actually carries out. Anything else falls through to its
     * `default:` arm, which deselects without changing the game state.
     */
    private static boolean isHandledActingAction(PlayerAction pa) {
        switch (pa) {
            case MOVE:
            case STAND_UP:
            case BLOCK:
            case BLITZ:
            case BLITZ_MOVE:
            case BLITZ_SELECT:
            case STAND_UP_BLITZ:
            case FOUL:
            case FOUL_MOVE:
            case PASS:
            case PASS_MOVE:
            case HAND_OVER:
            case HAND_OVER_MOVE:
            case THROW_TEAM_MATE:
            case THROW_TEAM_MATE_MOVE:
            case KICK_TEAM_MATE:
            case KICK_TEAM_MATE_MOVE:
            case THROW_BOMB:
            case HAIL_MARY_PASS:
            case TREACHEROUS:
            case BLACK_INK:
            case MULTIPLE_BLOCK:
            case RAIDING_PARTY:
            case LOOK_INTO_MY_EYES:
            case BALEFUL_HEX:
            case CATCH_OF_THE_DAY:
            case THEN_I_STARTED_BLASTIN:
            case ALL_YOU_CAN_EAT:
            case FURIOUS_OUTPBURST:
            case THROW_KEG:
            case WISDOM_OF_THE_WHITE_DWARF:
            case AUTO_GAZE_ZOAT:
                return true;
            default:
                return false;
        }
    }

    private static String getDialogTeamId(IDialogParameter dialog) {
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter) {
            return ((com.fumbbl.ffb.dialog.DialogBlockRollPropertiesParameter) dialog).getChoosingTeamId();
        }
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogArgueTheCallParameter) {
            return ((com.fumbbl.ffb.dialog.DialogArgueTheCallParameter) dialog).getTeamId();
        }
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogBriberyAndCorruptionParameter) {
            return ((com.fumbbl.ffb.dialog.DialogBriberyAndCorruptionParameter) dialog).getTeamId();
        }
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter) {
            return ((com.fumbbl.ffb.dialog.DialogPlayerChoiceParameter) dialog).getTeamId();
        }
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogBribesParameter) {
            return ((com.fumbbl.ffb.dialog.DialogBribesParameter) dialog).getTeamId();
        }
        if (dialog instanceof com.fumbbl.ffb.dialog.DialogPettyCashParameter) {
            return ((com.fumbbl.ffb.dialog.DialogPettyCashParameter) dialog).getTeamId();
        }
        return null;
    }

    private static String resolveTeamId(String name) {
        switch (name.toLowerCase()) {
            case "human":    return "teamHumanKalimar";
            case "orc":      return "teamOrcBattleLore";
            case "darkelves":
            case "darkelf":
            case "dark_elf": return "teamDarkElfKalimar";
            default:         return name;
        }
    }

    private static String escJson(String s) {
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    /**
     * Interception target: every player UtilPassing.findInterceptors returns (the ENGINE's own
     * predicate, the same call StepIntercept makes), coordinate-sorted, 1 actionRng pick.
     * Mirrors Rust's AgentPrompt::Interception handling in random_agent.rs.
     */
    private void sendInterceptorChoice(Game game, GameState gameState) {
        Player<?>[] found = UtilPassing.findInterceptors(game,
            game.getPlayerById(game.getThrowerId()), game.getPassCoordinate());
        java.util.List<Player<?>> candidates = new java.util.ArrayList<>();
        if (found != null) {
            for (Player<?> p : found) {
                if (p != null) {
                    candidates.add(p);
                }
            }
        }
        if (candidates.isEmpty()) {
            MatchRunner.inject(gameState, new ClientCommandInterceptorChoice(null, null));
            return;
        }
        // The heuristic agent decides only WHETHER to intercept (Rust `AgentPrompt::Interception`,
        // T = 0.20); accepting takes `find_interceptors().first()` -- the ENGINE's own first
        // candidate, deliberately NOT the coordinate-sorted one the random pick below draws from.
        if (heuristic != null
            && heuristic.handles(com.fumbbl.ffb.ai.parity.heuristic.PromptClass.INTERCEPTION)) {
            String chosen = heuristic.intercept() ? candidates.get(0).getId() : null;
            MatchRunner.inject(gameState, new ClientCommandInterceptorChoice(chosen, null));
            return;
        }
        candidates.sort((a, b) -> {
            FieldCoordinate ca = game.getFieldModel().getPlayerCoordinate(a);
            FieldCoordinate cb = game.getFieldModel().getPlayerCoordinate(b);
            int ax = ca != null ? ca.getX() : Integer.MAX_VALUE;
            int bx = cb != null ? cb.getX() : Integer.MAX_VALUE;
            if (ax != bx) {
                return Integer.compare(ax, bx);
            }
            int ay = ca != null ? ca.getY() : Integer.MAX_VALUE;
            int by = cb != null ? cb.getY() : Integer.MAX_VALUE;
            return Integer.compare(ay, by);
        });
        int idx = (int) Long.remainderUnsigned(actionRng.nextLong(), candidates.size());
        MatchRunner.inject(gameState, new ClientCommandInterceptorChoice(candidates.get(idx).getId(), null));
    }

}
