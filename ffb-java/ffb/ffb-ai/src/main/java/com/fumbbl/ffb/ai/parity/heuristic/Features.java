package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;
import com.fumbbl.ffb.PlayerState;
import com.fumbbl.ffb.model.FieldModel;
import com.fumbbl.ffb.model.Game;
import com.fumbbl.ffb.model.Player;

import java.util.ArrayList;
import java.util.List;

/**
 * The CORE raster tier of the Rust {@code Features} struct.
 *
 * <p>Three whole-pitch arrays, and everything downstream reads them:
 *
 * <ul>
 *   <li>{@code occ} — one byte per square: which side stands there, and whether that player has
 *       tackle zones. A PRONE player occupies his square but marks nothing.
 *   <li>{@code tz[side]} — how many STANDING opponents of {@code side} mark each square. Note the
 *       index: a standing HOME player increments {@code tz[AWAY]}, because it is the away team
 *       that has to dodge out of him.
 *   <li>{@code rowPrefix[side]} — an EXCLUSIVE prefix count, per row, of opponents of
 *       {@code side} strictly to the left of each column, over {@code W + 1} columns so that a
 *       half-open range query needs no bounds check. This is the piece a from-scratch
 *       reimplementation gets wrong.
 * </ul>
 *
 * <p><b>Built from a snapshot, not from a Game.</b> The raster arithmetic and the model plumbing
 * are separate concerns, and only the arithmetic has to agree bit for bit with Rust — so the
 * arithmetic is what the cross-language fixture pins ({@code features_golden.txt}), and
 * {@link #snapshot(Game)} is the thin adapter that is exercised by the parity sweep instead.
 */
public final class Features {

    public static final int XMAX = 25;
    public static final int YMAX = 14;
    public static final int W = 26;
    public static final int H = 15;
    public static final int CELLS = W * H;

    public static final byte OCC_NONE = 0;
    public static final byte OCC_HOME = 1;
    public static final byte OCC_AWAY = 2;
    /** High bit: the occupant has tackle zones. */
    public static final byte OCC_TZ = (byte) 0x80;

    /** One player as the rasters see him. */
    public static final class Snap {
        public final boolean home;
        public final int x;
        public final int y;
        /** Has tackle zones — i.e. standing and not otherwise incapacitated. */
        public final boolean standing;
        /** Java {@code PlayerState.isActive()} — still able to act this turn. */
        public final boolean active;
        /** Movement and strength WITH modifiers; the threat raster reads both. */
        public final int ma;
        public final int st;
        /** Jersey number. With the side it forms the canonical ordering key. */
        public final int nr;

        public Snap(boolean home, int x, int y, boolean standing, boolean active,
                int ma, int st, int nr) {
            this.home = home;
            this.x = x;
            this.y = y;
            this.standing = standing;
            this.active = active;
            this.ma = ma;
            this.st = st;
            this.nr = nr;
        }
    }

    /** The board state the HEAVY tier needs beyond the players themselves. */
    public static final class BoardState {
        public final FieldCoordinate ball;
        public final boolean ballInPlay;
        public final boolean ballMoving;
        public final boolean blitzUsedHome;
        public final boolean blitzUsedAway;

        public BoardState(FieldCoordinate ball, boolean ballInPlay, boolean ballMoving,
                boolean blitzUsedHome, boolean blitzUsedAway) {
            this.ball = ball;
            this.ballInPlay = ballInPlay;
            this.ballMoving = ballMoving;
            this.blitzUsedHome = blitzUsedHome;
            this.blitzUsedAway = blitzUsedAway;
        }
    }

    public final byte[] occ = new byte[CELLS];
    public final byte[][] tz = new byte[2][CELLS];
    public final int[][] rowPrefix = new int[2][H * (W + 1)];
    public final float[] unactivated = new float[2];

    // ── HEAVY tier ──────────────────────────────────
    public final float[][] threatReach = new float[2][CELLS];
    public final byte[][] threatStr = new byte[2][CELLS];
    public final float[][] threatMark = new float[2][CELLS];
    public final float[][] lane = new float[2][CELLS];
    public final float[][] support = new float[2][CELLS];

    /** Where the ball is, or null. */
    public FieldCoordinate ball;
    public boolean ballLoose;
    public boolean ballCarried;
    /** The square the carrier stands on, or null. Rust keeps the id; only the square is read. */
    public FieldCoordinate carrierAt;
    /** Whether the carrier is on the home team; meaningless when {@link #carrierAt} is null. */
    public boolean carrierIsHome;

    public static int ix(int x, int y) {
        return y * W + x;
    }

    /** Rust {@code side_idx}: home is 0, away is 1. */
    public static int sideIdx(boolean home) {
        return home ? 0 : 1;
    }

    public static boolean onPitch(int x, int y) {
        return x >= 0 && x <= XMAX && y >= 0 && y <= YMAX;
    }

    /**
     * Rust {@code carrier_at[side]}: where OUR carrier stands, per side.
     *
     * <p>Not "where the carrier stands" — it is null when the ball is held by the OTHER team, which
     * is what makes the receiver and cage intents fire only for the side that actually has it.
     */
    public FieldCoordinate carrierAtFor(boolean home) {
        return (carrierAt != null && carrierIsHome == home) ? carrierAt : null;
    }

    public int tzAgainst(FieldCoordinate c, boolean home) {
        return tz[sideIdx(home)][ix(c.getX(), c.getY())] & 0xff;
    }

    public boolean occupied(int i) {
        return (occ[i] & 0x7f) != OCC_NONE;
    }

    /**
     * Rust {@code Features::build}, CORE tier.
     *
     * <p>Every accumulation here is commutative — {@code occ} writes one distinct square per
     * player, {@code tz} and {@code rowPrefix} are integer increments, and {@code unactivated}
     * sums a constant addend — so the input order does not matter and no canonical sort is needed.
     * That is a property of this method specifically; {@code build_threat} does NOT have it.
     */
    public static Features build(List<Snap> players) {
        return build(players, new BoardState(null, false, false, false, false), false);
    }

    /**
     * @param heavy also build the threat, lane and support tiers. Rust gates these the same way:
     *     only the prompts that actually search the board pay for them.
     */
    public static Features build(List<Snap> players, BoardState board, boolean heavy) {
        Features f = new Features();
        // Rust seeds these three, and the defaults are NOT zero: an unthreatened square reports
        // the baseline ST 3 rather than 0, an unobstructed lane is fully open, and every square
        // carries the Retreat floor. buildLane/buildSupport overwrite every cell, but threatStr
        // does not -- only squares some opponent can actually reach get written.
        for (int s = 0; s < 2; s++) {
            java.util.Arrays.fill(f.threatStr[s], (byte) 3);
            java.util.Arrays.fill(f.lane[s], 1.0f);
            java.util.Arrays.fill(f.support[s], 0.10f);
        }
        float[] unact = new float[2];
        for (Snap p : players) {
            if (!onPitch(p.x, p.y)) {
                continue;
            }
            int s = sideIdx(p.home);
            f.occ[ix(p.x, p.y)] =
                (byte) ((p.home ? OCC_HOME : OCC_AWAY) | (p.standing ? OCC_TZ : 0));
            if (p.standing) {
                for (int dy = -1; dy <= 1; dy++) {
                    for (int dx = -1; dx <= 1; dx++) {
                        if (dx == 0 && dy == 0) {
                            continue;
                        }
                        int nx = p.x + dx;
                        int ny = p.y + dy;
                        if (onPitch(nx, ny)) {
                            f.tz[1 - s][ix(nx, ny)]++;
                        }
                    }
                }
            }
            // Opponents-of-home are the away players, so they fill rowPrefix[0].
            int oppOf = 1 - s;
            for (int x = p.x + 1; x <= W; x++) {
                f.rowPrefix[oppOf][p.y * (W + 1) + x]++;
            }
            if (p.active) {
                unact[s] += 1.0f / 11.0f;
            }
        }
        f.unactivated[0] = Math.min(unact[0], 1.0f);
        f.unactivated[1] = Math.min(unact[1], 1.0f);

        f.ball = board.ball;
        boolean inPlay = board.ballInPlay && board.ball != null
            && onPitch(board.ball.getX(), board.ball.getY());
        // ballMoving means LOOSE ON THE GROUND, not in flight.
        f.ballLoose = inPlay && board.ballMoving;
        f.ballCarried = inPlay && !board.ballMoving;
        if (f.ballCarried) {
            for (Snap p : players) {
                if (p.x == board.ball.getX() && p.y == board.ball.getY()) {
                    f.carrierAt = new FieldCoordinate(p.x, p.y);
                    f.carrierIsHome = p.home;
                    break;
                }
            }
        }

        if (heavy) {
            f.buildThreat(players, board);
            f.buildLane();
            f.buildSupport(players);
        }
        return f;
    }

    /** Chebyshev distance — Rust {@code FieldCoordinate::distance_in_steps}. */
    private static int steps(int ax, int ay, int bx, int by) {
        return Math.max(Math.abs(ax - bx), Math.abs(ay - by));
    }

    private static int endzoneX(boolean home) {
        return home ? XMAX : 0;
    }

    /**
     * Rust {@code build_threat}. Only ONE opponent can blitz per turn, so the block term is a max
     * over opponents and everyone else contributes a smaller marking term.
     *
     * <p><b>The player order is load-bearing here and nowhere else in this class.</b>
     * {@code threatStr} is written under a strict {@code >} against {@code threatReach}, so two
     * opponents that reach a square equally TIE, and whichever is visited first records ITS
     * strength. Rust iterates canonical (side, jersey nr) order; so does this. Iterating a map
     * would make the agent non-deterministic run to run, which is the bug ITER1 found.
     */
    private void buildThreat(List<Snap> players, BoardState board) {
        List<Snap> ordered = new ArrayList<>(players);
        ordered.sort((a, b) -> {
            int sa = sideIdx(a.home);
            int sb = sideIdx(b.home);
            return sa != sb ? Integer.compare(sa, sb) : Integer.compare(a.nr, b.nr);
        });

        for (int s = 0; s < 2; s++) {
            boolean victimHome = s == 0;
            boolean oppBlitzSpent = victimHome ? board.blitzUsedAway : board.blitzUsedHome;
            float[] second = new float[CELLS];
            float[] third = new float[CELLS];

            for (Snap opp : ordered) {
                if (!onPitch(opp.x, opp.y) || opp.home == victimHome || !opp.standing) {
                    continue;
                }
                boolean markedNow = (tz[1 - s][ix(opp.x, opp.y)] & 0xff) > 0;
                int r = opp.ma + 3;
                for (int y = Math.max(opp.y - r, 0); y <= Math.min(opp.y + r, YMAX); y++) {
                    for (int x = Math.max(opp.x - r, 0); x <= Math.min(opp.x + r, XMAX); x++) {
                        int d = steps(opp.x, opp.y, x, y);
                        int st = Math.max(d - 1, 0);
                        float reach;
                        if (d == 1) {
                            reach = 1.0f;
                        } else if (st <= opp.ma) {
                            // a player who is himself marked is unlikely to leave freely
                            reach = markedNow ? 0.55f : 1.0f;
                        } else if (st <= opp.ma + 2) {
                            reach = 0.25f;
                        } else {
                            continue;
                        }
                        int i = ix(x, y);
                        // The block term needs a blitz unless the opponent already stands adjacent.
                        if ((d == 1 || !oppBlitzSpent) && reach > threatReach[s][i]) {
                            threatReach[s][i] = reach;
                            threatStr[s][i] = (byte) opp.st;
                        }
                        if (reach > second[i]) {
                            third[i] = second[i];
                            second[i] = reach;
                        } else if (reach > third[i]) {
                            third[i] = reach;
                        }
                    }
                }
            }
            for (int i = 0; i < CELLS; i++) {
                threatMark[s][i] = 0.18f * (second[i] + third[i]);
            }
        }
    }

    /** Rust {@code opponents_between}: an exclusive-prefix range query over one row. */
    private int opponentsBetween(boolean home, int y, int x0, int x1) {
        if (y < 0 || y > YMAX) {
            return 0;
        }
        int lo = Math.min(x0, x1);
        int hi = Math.max(x0, x1);
        int[] p = rowPrefix[sideIdx(home)];
        int row = y * (W + 1);
        int a = p[row + Math.min(Math.max(lo, 0), W)];
        int b = p[row + Math.min(Math.max(hi, 0), W)];
        return Math.max(b - a, 0);
    }

    /** Rust {@code build_lane}: corridor openness, opponents within two rows toward the endzone. */
    private void buildLane() {
        for (int s = 0; s < 2; s++) {
            boolean home = s == 0;
            int ez = endzoneX(home);
            for (int y = 0; y < H; y++) {
                for (int x = 0; x < W; x++) {
                    int corridor = 0;
                    for (int dy = -2; dy <= 2; dy++) {
                        corridor += opponentsBetween(home, y + dy, x, ez);
                    }
                    lane[s][ix(x, y)] = 1.0f / (1.0f + 0.35f * (float) corridor);
                }
            }
        }
    }

    /** Rust {@code build_support}: the mover-independent Cage / Mark / Screen / Retreat intents. */
    private void buildSupport(List<Snap> players) {
        int[][] screenHits = new int[2][CELLS];
        int[] screenTot = new int[2];

        for (int s = 0; s < 2; s++) {
            boolean myHome = s == 0;
            FieldCoordinate target = null;
            if (carrierAt != null && carrierIsHome == myHome) {
                target = carrierAt;
            } else if (ballLoose && ball != null) {
                target = ball;
            }
            if (target == null) {
                continue;
            }
            for (Snap p : players) {
                if (!onPitch(p.x, p.y) || p.home == myHome || !p.standing) {
                    continue;
                }
                int dOt = steps(p.x, p.y, target.getX(), target.getY());
                if (dOt == 0 || dOt > 12) {
                    continue;
                }
                screenTot[s]++;
                // Squares on a shortest-ish approach: going via them costs at most one extra step.
                for (int y = 0; y < H; y++) {
                    for (int x = 0; x < W; x++) {
                        int dOs = steps(p.x, p.y, x, y);
                        int dSt = steps(x, y, target.getX(), target.getY());
                        if (dSt >= 1 && dOs + dSt <= dOt + 1) {
                            screenHits[s][ix(x, y)]++;
                        }
                    }
                }
            }
        }

        for (int s = 0; s < 2; s++) {
            boolean myHome = s == 0;
            FieldCoordinate ownCarrier =
                (carrierAt != null && carrierIsHome == myHome) ? carrierAt : null;
            int oppOcc = myHome ? OCC_AWAY : OCC_HOME;

            for (int y = 0; y < H; y++) {
                for (int x = 0; x < W; x++) {
                    int i = ix(x, y);
                    float best = 0.10f; // Retreat floor

                    // Cage, weighted by which side the threat is actually on.
                    if (ownCarrier != null) {
                        int dx = Math.abs(x - ownCarrier.getX());
                        int dy = Math.abs(y - ownCarrier.getY());
                        if (dx <= 1 && dy <= 1 && dx + dy > 0) {
                            if (dx == 1 && dy == 1) {
                                float t = Math.min(threatReach[s][i], 2.0f) / 2.0f;
                                best = Math.max(best, 0.35f + 0.40f * t);
                            } else {
                                best = Math.max(best, 0.35f);
                            }
                        }
                    }

                    // Mark, the best adjacent opposing player worth standing next to.
                    float markBest = 0.0f;
                    for (int dy = -1; dy <= 1; dy++) {
                        for (int dx = -1; dx <= 1; dx++) {
                            if (dx == 0 && dy == 0) {
                                continue;
                            }
                            int nx = x + dx;
                            int ny = y + dy;
                            if (!onPitch(nx, ny)) {
                                continue;
                            }
                            byte o = occ[ix(nx, ny)];
                            if ((o & 0x7f) != oppOcc || (o & OCC_TZ) == 0) {
                                continue;
                            }
                            boolean isCarrier = ballCarried && ball != null
                                && ball.getX() == nx && ball.getY() == ny;
                            float mv = isCarrier ? 1.0f : 0.30f;
                            for (Snap p : players) {
                                if (p.x == nx && p.y == ny) {
                                    if (!p.active) {
                                        mv = Math.max(mv, 0.45f);
                                    }
                                    break;
                                }
                            }
                            markBest = Math.max(markBest, mv);
                        }
                    }
                    if (markBest > 0.0f) {
                        best = Math.max(best, 0.50f * markBest);
                    }

                    // Screen, a line between the ball and the threat rather than a huddle.
                    if (screenTot[s] > 0) {
                        float share = (float) screenHits[s][i] / (float) screenTot[s];
                        if (share > 0.0f) {
                            best = Math.max(best, 0.45f * share);
                        }
                    }

                    support[s][i] = best;
                }
            }
        }
    }

    /** The adapter: every on-pitch player, as the rasters see him. */
    public static List<Snap> snapshot(Game game) {
        FieldModel fm = game.getFieldModel();
        List<Snap> out = new ArrayList<>();
        for (boolean home : new boolean[] {true, false}) {
            for (Player<?> p : (home ? game.getTeamHome() : game.getTeamAway()).getPlayers()) {
                FieldCoordinate c = fm.getPlayerCoordinate(p);
                PlayerState ps = fm.getPlayerState(p);
                if (c == null || ps == null || !onPitch(c.getX(), c.getY())) {
                    continue;
                }
                out.add(new Snap(home, c.getX(), c.getY(), ps.hasTacklezones(), ps.isActive(),
                    p.getMovementWithModifiers(), p.getStrengthWithModifiers(), p.getNr()));
            }
        }
        return out;
    }

    public static Features build(Game game) {
        return build(snapshot(game));
    }
}
