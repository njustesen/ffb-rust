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

        public Snap(boolean home, int x, int y, boolean standing, boolean active) {
            this.home = home;
            this.x = x;
            this.y = y;
            this.standing = standing;
            this.active = active;
        }
    }

    public final byte[] occ = new byte[CELLS];
    public final byte[][] tz = new byte[2][CELLS];
    public final int[][] rowPrefix = new int[2][H * (W + 1)];
    public final float[] unactivated = new float[2];

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
        Features f = new Features();
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
        return f;
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
                out.add(new Snap(home, c.getX(), c.getY(), ps.hasTacklezones(), ps.isActive()));
            }
        }
        return out;
    }

    public static Features build(Game game) {
        return build(snapshot(game));
    }
}
