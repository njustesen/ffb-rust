package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

/**
 * Rust {@code value_at} — what a square is WORTH to a particular mover.
 *
 * <p>Three branches, and they are genuinely different formulas rather than variations on one:
 *
 * <ul>
 *   <li>a <b>carrier</b> is priced by how much ground the square gains, scaled by urgency and by
 *       whether the endzone is reachable at all in the turns the half has left;
 *   <li>a mover standing on a <b>loose ball</b> is a pickup, priced by the pickup roll;
 *   <li>everyone else takes the support raster — unless he is a plausible <b>receiver</b>, which
 *       needs three conditions to hold at once.
 * </ul>
 *
 * <p>The branch taken is part of the contract, not an implementation detail: two implementations can
 * agree on a number while disagreeing about which rule produced it, and that disagreement surfaces
 * the moment the board changes. {@link Scored#rule} carries it, and the fixture checks it.
 */
public final class ValueModel {

    /**
     * Residual worth of advancing a carrier who can no longer reach the endzone in time.
     *
     * <p>Rust froze this constant deliberately: it used to be read from an environment variable
     * while it was being fitted, and env-dependent POLICY cannot be mirrored on this side at all.
     */
    public static final float HOPELESS_DAMP = 0.25f;

    private ValueModel() {
    }

    /** Which rule produced a value. Mirrors the Rust {@code Rule} variants this model can return. */
    public enum Rule {
        SCORE_TOUCHDOWN,
        SCORE_ADVANCE,
        PICKUP,
        SUPPORT
    }

    /** A value and the rule behind it. */
    public static final class Scored {
        public final float v;
        public final Rule rule;

        public Scored(float v, Rule rule) {
            this.v = v;
            this.rule = rule;
        }
    }

    /** The mover-dependent inputs; everything else is an array read off {@link Features}. */
    public static final class Mover {
        public final boolean home;
        public final boolean isCarrier;
        public final int ma;
        public final int ag;
        public final int str;
        public final boolean sureHands;
        public final boolean sideStep;
        public final boolean hasCatch;
        /** How far the mover is from the endzone he attacks, right now. */
        public final int dNow;
        public final int turnsLeft;
        public final float unactivated;

        public Mover(boolean home, boolean isCarrier, int ma, int ag, int str, boolean sureHands,
                boolean sideStep, boolean hasCatch, int dNow, int turnsLeft, float unactivated) {
            this.home = home;
            this.isCarrier = isCarrier;
            this.ma = ma;
            this.ag = ag;
            this.str = str;
            this.sureHands = sureHands;
            this.sideStep = sideStep;
            this.hasCatch = hasCatch;
            this.dNow = dNow;
            this.turnsLeft = turnsLeft;
            this.unactivated = unactivated;
        }
    }

    /** Rust {@code strength_factor}. */
    public static float strengthFactor(int att, int def) {
        if (att > 2 * def) {
            return 1.4f;
        }
        if (att > def) {
            return 1.2f;
        }
        if (2 * att < def) {
            return 0.5f;
        }
        if (att < def) {
            return 0.7f;
        }
        return 1.0f;
    }

    public static int endzoneX(boolean home) {
        return home ? Features.XMAX : 0;
    }

    public static int endzoneDistance(int x, boolean home) {
        return Math.abs(x - endzoneX(home));
    }

    /**
     * Rust {@code exposure}: how safe a square is for a mover of the given strength.
     *
     * <p>Exact despite being rasterised — the reach factor and the blitzer's strength are stored
     * apart, so {@code strengthFactor} can be applied here rather than baked into the raster.
     */
    public static float exposure(Features f, int i, boolean home, int moverStr) {
        int s = Features.sideIdx(home);
        float block = f.threatReach[s][i] * strengthFactor(f.threatStr[s][i], moverStr);
        return 1.0f / (1.0f + block + f.threatMark[s][i]);
    }

    /** Rust {@code urgency}. */
    public static float urgency(int dSq, int ma, int turnsLeft) {
        int tts = (int) Math.ceil((double) dSq / (double) Math.max(ma, 1));
        float u = 1.0f - (float) (turnsLeft - tts) / 3.0f;
        return Math.min(Math.max(u, 0.0f), 1.0f);
    }

    /** Rust {@code value_at}. */
    public static Scored valueAt(Features f, int i, Mover m) {
        int sqx = i % Features.W;
        int sqy = i / Features.W;
        int dSq = endzoneDistance(sqx, m.home);
        int s = Features.sideIdx(m.home);

        float sideline;
        if (sqy == 0 || sqy == Features.YMAX) {
            sideline = m.sideStep ? 1.0f : 0.25f;
        } else if (m.isCarrier && (sqy == 1 || sqy == Features.YMAX - 1)) {
            sideline = 0.6f;
        } else {
            sideline = 1.0f;
        }
        float exposure = exposure(f, i, m.home, m.str);

        if (m.isCarrier) {
            float base;
            if (dSq == 0) {
                base = 1.0f;
            } else {
                // Measure the gain against what THIS activation could reach, not the whole pitch.
                int maxGain = Math.max(Math.min(m.dNow, m.ma + 2), 1);
                float advance = (float) (m.dNow - dSq) / (float) maxGain;
                advance = Math.min(Math.max(advance, 0.0f), 1.0f);
                // If the endzone is out of reach in the turns the half has left, running there
                // cannot score and must not be priced as though it could. `urgency` alone gets this
                // backwards: it saturates at 1.0 exactly when the score becomes impossible.
                int tts = (int) Math.ceil((double) dSq / (double) Math.max(m.ma, 1));
                float reachableInTime = tts <= m.turnsLeft ? 1.0f : HOPELESS_DAMP;
                base = (0.15f + 0.85f * advance)
                    * (0.75f + 0.5f * urgency(dSq, m.ma, m.turnsLeft))
                    * reachableInTime;
            }
            float v = base * sideline * exposure * f.lane[s][i];
            return new Scored(v, dSq == 0 ? Rule.SCORE_TOUCHDOWN : Rule.SCORE_ADVANCE);
        }

        // Pickup — the single highest-value thing on the board while the ball is loose.
        if (f.ballLoose && f.ball != null && f.ball.getX() == sqx && f.ball.getY() == sqy) {
            int tgt = Math.max(m.ag + (f.tz[s][i] & 0xff), 2);
            float raw = Reach.pRoll(tgt);
            float p = m.sureHands ? Reach.pWithReRoll(raw, 1.0f) : raw;
            float v = (0.55f + 0.45f * p) * sideline * exposure * f.lane[s][i];
            return new Scored(v, Rule.PICKUP);
        }

        // RECEIVER: could he catch here, and run it in next turn? Worth far more than standing in a
        // screen, and the only thing that ever gives a pass somewhere to go.
        float support = f.support[s][i];
        FieldCoordinate cc = f.carrierAtFor(m.home);
        if (cc != null) {
            int dx = Math.abs(sqx - cc.getX());
            int dy = Math.abs(sqy - cc.getY());
            // The range table tops out below 14 in each axis; beyond that it cannot be thrown.
            boolean throwable = dx < 14 && dy < 14 && Math.max(dx, dy) > 0;
            boolean canRunItIn = dSq <= m.ma + 2;
            // And is this a RECEIVER position — ahead of the ball, not beside it? Without this the
            // intent fires on half the pitch and pulls the whole team off the cage.
            boolean aheadOfBall = dSq < endzoneDistance(cc.getX(), m.home);
            if (throwable && canRunItIn && aheadOfBall) {
                float raw = Reach.pRoll(Math.max(m.ag - 1 + (f.tz[s][i] & 0xff), 2));
                float catchQ = m.hasCatch ? Reach.pWithReRoll(raw, 1.0f) : raw;
                float closeness = 1.0f - Math.min(Math.max(
                    (float) dSq / (float) Math.max(m.ma + 2, 1), 0.0f), 1.0f) * 0.35f;
                // Deliberately below a threatened cage (0.75): escorting the ball beats running a
                // route when the carrier is under pressure.
                support = Math.max(support, 0.30f * catchQ + 0.20f * closeness);
            }
        }
        return new Scored(support * sideline * exposure, Rule.SUPPORT);
    }
}
