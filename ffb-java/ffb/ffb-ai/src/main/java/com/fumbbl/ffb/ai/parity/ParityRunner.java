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

    /**
     * Parity tier (see ffb-rust AGENT_CONTRACT.md):
     *   2 — T2 regression behavior: one decisionRng pick per turn, then immediate
     *       deselect + EndTurn (no concrete actions). This is the 26-race suite.
     *   3 — T3 Phase 2: real activations; after each pick the acting player performs
     *       a concrete action (move, later block/blitz/pass/hand-over/foul).
     */
    private int tier = 2;

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
        List<String> positional = new ArrayList<>();
        for (int i = 0; i < args.length; i++) {
            if ("--tier".equals(args[i]) && i + 1 < args.length) {
                tierArg = Integer.parseInt(args[++i]);
            } else if ("--seed-end".equals(args[i]) && i + 1 < args.length) {
                seedEndArg = Long.parseUnsignedLong(args[++i]);
            } else if ("--ruleset".equals(args[i]) && i + 1 < args.length) {
                rulesetArg = args[++i];
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
            runner.run(gameState, homeTeamId, awayTeamId, s);

            out.flush();
            if (path != null) out.close();
        }
    }

    // ── Game loop ─────────────────────────────────────────────────────────────

    public void run(GameState gameState, String homeTeamId, String awayTeamId, long seed) {
        Game game = gameState.getGame();

        this.currentSeed = seed;
        this.decisionRng = new Xoshiro256StarStar(seed ^ 0xDEADBEEFCAFE0001L);
        this.actionRng = new Xoshiro256StarStar(seed ^ 0xC0FFEE_ACE0_0001L);
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
                        } else {
                            action = actions[0];
                        }
                        String chosen = "Activate(" + playerId + "," + action.toString() + ")";
                        recordStep(game, chosen, gameState.getDiceRoller().getCallCount());
                        blitzBlockSent = false;
                        // BLITZ is declared as BLITZ_MOVE: StepInitSelecting dispatches it to
                        // BLITZ_SELECT (target selection sets blitzUsed), then the block is
                        // sent from phase 2 once the target selection state exists.
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
        switch (dialog.getId()) {

            // ── Informational / clear-only dialogs ──────────────────────────
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

            // ── Touchback: nearest player to kick-from (13,8) ───────────────
            case TOUCHBACK: {
                boolean homeReceives = !game.isHomePlaying();
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

            case BLOCK_ROLL:
            case BLOCK_ROLL_PARTIAL_RE_ROLL: {
                // Always pick die index 0 — deterministic, matches Rust's index=0 choice.
                comm.clearCaptured();
                comm.sendBlockChoice(0);
                injectCaptured(dialog, game, gameState);
                break;
            }

            case BLOCK_ROLL_PROPERTIES: {
                // BB2025 block roll: the step waits for CLIENT_BLOCK_CHOICE. Pick die
                // index 0 and never use a reroll (AGENT_CONTRACT.md §7) — sending a
                // reroll-decline here would just re-show the dialog forever.
                comm.clearCaptured();
                comm.sendBlockChoice(0);
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

            case RE_ROLL:
            case RE_ROLL_PROPERTIES: {
                // Always decline — deterministic. No game RNG consumed for the declined roll.
                comm.clearCaptured();
                if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollParameter) {
                    comm.sendUseReRoll(
                        ((com.fumbbl.ffb.dialog.DialogReRollParameter) dialog).getReRolledAction(), null);
                } else if (dialog instanceof com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter) {
                    comm.sendUseReRoll(
                        ((com.fumbbl.ffb.dialog.DialogReRollPropertiesParameter) dialog).getReRolledAction(), null);
                } else {
                    game.setDialogParameter(null);
                    break;
                }
                injectCaptured(dialog, game, gameState);
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
                    String skillName = (su.getSkill() == null) ? null : su.getSkill().getClass().getSimpleName();
                    boolean useSkill = (skillName == null)
                        || (!"DumpOff".equals(skillName) && !"PrimalSavagery".equals(skillName)
                            && !"SafePairOfHands".equals(skillName));
                    comm.clearCaptured();
                    comm.sendUseSkill(su.getSkill(), useSkill, su.getPlayerId());
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
                    } else if (mode == PlayerChoiceMode.ANIMAL_SAVAGERY && pids != null && pids.length > 0) {
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
            System.err.println("JSTEP i=" + stepIndex + " rng_calls=" + callCount + " chosen=" + chosen + " state=" + canonicalStr);
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
        sb.append(" p");
        for (int i = 0; i < playerParts.size(); i++) {
            if (i > 0) sb.append('|');
            sb.append(playerParts.get(i));
        }

        String canonical = sb.toString();
        long hash = fnv1a64(canonical.getBytes(StandardCharsets.UTF_8));
        return String.format("%016x", hash);
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
            out.add(String.format("%s%02d:%d,%d,%s", prefix, i, x, y, state));
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
            case PlayerState.BADLY_HURT:     return "Injured";
            case PlayerState.SERIOUS_INJURY: return "Injured";
            case PlayerState.RIP:            return "Injured";
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
                sendPassAction(game, gameState, pid);
                break;
            case HAND_OVER:
            case HAND_OVER_MOVE:
                sendHandOverAction(game, gameState, pid);
                break;
            case THROW_TEAM_MATE:
            case THROW_TEAM_MATE_MOVE:
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
    private Player<?> pickBlockTarget(Game game, String playerId) {
        FieldModel fm = game.getFieldModel();
        FieldCoordinate coord = playerCoordinate(game, playerId);
        if (coord == null) return null;
        Team opponent = game.isHomePlaying() ? game.getTeamAway() : game.getTeamHome();
        List<Player<?>> targets = new ArrayList<>();
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
     * legal_throw_team_mate_targets: adjacent STANDING teammates with the canBeThrown property
     * (Right Stuff), coordinate-sorted, 1 actionRng pick. Empty → deselect (the human-Ogre / no
     * throwable-teammate case). The target square is chosen later, at the INIT_THROW_TEAM_MATE
     * waiting state (sendThrowTeamMateTarget).
     */
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
            if (!tp.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canBeThrown)) continue;
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
        if (DEBUG) System.err.println("JAVA_TTM pid=" + playerId + " N=" + targets.size() + " idx=" + idx + " thrown=" + thrownId);
        MatchRunner.inject(gameState,
            new com.fumbbl.ffb.net.commands.ClientCommandThrowTeamMate(playerId, thrownId));
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
        if (squares != null) {
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
        for (PlayerAction a : actions) {
            boolean keep;
            switch (a) {
                case BLOCK:
                case BLITZ:
                case STAND_UP_BLITZ:
                    keep = !td.isBlitzUsed();
                    break;
                case PASS:
                    keep = !td.isPassUsed();
                    break;
                case HAND_OVER:
                    keep = !td.isHandOverUsed();
                    break;
                case FOUL:
                    keep = !td.isFoulUsed();
                    break;
                case THROW_TEAM_MATE:
                    keep = !td.isTtmUsed();
                    break;
                case KICK_TEAM_MATE:
                    keep = !td.isKtmUsed();
                    break;
                default:
                    keep = true;
                    break;
            }
            if (keep) live.add(a);
        }
        return live.toArray(new PlayerAction[0]);
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

        // Compute adjacent squares on pitch, unoccupied by any player.
        // Mirrors Rust's legal_move_targets(): coord.neighbours() filtered by is_on_pitch() and no player.
        int[] dx = {0, 1, 1, 1, 0, -1, -1, -1};  // N, NE, E, SE, S, SW, W, NW
        int[] dy = {-1, -1, 0, 1, 1, 1, 0, -1};
        List<FieldCoordinate> targets = new ArrayList<>();
        for (int i = 0; i < 8; i++) {
            int nx = coord.getX() + dx[i];
            int ny = coord.getY() + dy[i];
            if (nx < 0 || nx > 25 || ny < 0 || ny > 14) { continue; }
            FieldCoordinate nc = new FieldCoordinate(nx, ny);
            // Check if occupied by any player (home or away)
            boolean occupied = false;
            for (com.fumbbl.ffb.model.Player<?> p : game.getTeamHome().getPlayers()) {
                if (nc.equals(fm.getPlayerCoordinate(p))) { occupied = true; break; }
            }
            if (!occupied) {
                for (com.fumbbl.ffb.model.Player<?> p : game.getTeamAway().getPlayers()) {
                    if (nc.equals(fm.getPlayerCoordinate(p))) { occupied = true; break; }
                }
            }
            if (!occupied) targets.add(nc);
        }
        // Sort by (x, y) — matches Rust's .sort_by_key(|c| (c.x, c.y))
        targets.sort(Comparator.comparingInt(FieldCoordinate::getX).thenComparingInt(FieldCoordinate::getY));

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
        // StepInitSelecting.fetchMoveStack/fetchFromSquare mirrors coords when homeCommand=false.
        // Coordinates from getPlayerCoordinate are in server-canonical (home-relative) space.
        // Away team commands must be in the away team's view (mirrored), so the server can
        // un-mirror them back to canonical. Home team commands are passed through as-is.
        boolean isHome = game.isHomePlaying();
        FieldCoordinate cmdFrom = isHome ? coord : coord.transform();
        FieldCoordinate cmdTarget = isHome ? target : target.transform();

        // Send move command: ClientCommandMove(actingPlayerId, fromCoord, [toCoord], null)
        ClientCommandMove moveCmd = new ClientCommandMove(playerId, cmdFrom, new FieldCoordinate[]{cmdTarget}, null);
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

                // Pass: player carries the ball
                if (!td.isPassUsed() && ballCoord != null && ballCoord.equals(coord)) {
                    actions.add(PlayerAction.PASS);
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
                }

                // ThrowTeamMate: TTM skill + adjacent teammate
                if (!td.isTtmUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canThrowTeamMates)) {
                    if (hasAdjacentTeammate(p, coord, team.getPlayers(), fm)) {
                        actions.add(PlayerAction.THROW_TEAM_MATE);
                    }
                }

                // KickTeamMate (BB2025 only): KTM skill + adjacent teammate
                if (isBb2025(game) && !td.isKtmUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canKickTeamMates)) {
                    if (hasAdjacentTeammate(p, coord, team.getPlayers(), fm)) {
                        actions.add(PlayerAction.KICK_TEAM_MATE);
                    }
                }

                // Punt (BB2025 only): Punt skill, ball carrier, ball in play
                if (isBb2025(game) && !td.isPuntUsed()
                        && p.hasSkillProperty(com.fumbbl.ffb.model.property.NamedProperties.canPunt)
                        && fm.isBallInPlay() && ballCoord != null && ballCoord.equals(coord)) {
                    actions.add(PlayerAction.PUNT);
                }

                // SecureTheBall (BB2025 only): ball moving through this player's square
                if (isBb2025(game) && fm.isBallInPlay() && fm.isBallMoving()
                        && ballCoord != null && ballCoord.equals(coord)
                        && !td.isSecureTheBallUsed()) {
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
}
