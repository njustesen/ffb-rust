package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

/**
 * Rust {@code receiver_of}, {@code handoff_weight} and {@code foul_weight} — what it is worth to
 * put the ball in someone else's hands, and what it is worth to foul.
 *
 * <p>{@link #receiverOf} is the function with the most ways to be quietly wrong, and every one of
 * them is an off-by-one-step rather than a bad constant:
 *
 * <ul>
 *   <li><b>{@code active}</b> — a receiver who still holds his activation can catch AND run this
 *       turn; one who has already gone has {@code reachAfter = 0} and one turn fewer. That single
 *       flag separates a touchdown from a token positional credit.
 *   <li><b>{@code effectiveD}</b> — where the BALL ends up, not where the receiver stands. Using
 *       the receiver's own distance prices every give as though he never moved afterwards.
 *   <li><b>{@code scoresNow}</b> — false when the CARRIER could score by himself, which stops the
 *       agent giving away a run it had already won.
 *   <li><b>{@code pRunIn}</b> — a five-way ladder on the receiver's MA with a GFI factor per rush.
 * </ul>
 */
public final class BallMoves {

    private BallMoves() {
    }

    /** Everything about a receiver that matters to a throw or a hand-off. */
    public static final class Receiver {
        /** Catch chance, at the accurate-pass/hand-off target of AG−1 plus tackle zones. */
        public final float pCatch;
        /** Value of the square once HE is the carrier. */
        public final float v;
        /** He still holds his activation, so he can catch and then run it in on THIS turn. */
        public final boolean scoresNow;
        /** Turns he needs to reach the endzone himself. */
        public final int tts;
        /** Turns he actually has — one fewer once he has already been activated. */
        public final int turns;

        Receiver(float pCatch, float v, boolean scoresNow, int tts, int turns) {
            this.pCatch = pCatch;
            this.v = v;
            this.scoresNow = scoresNow;
            this.tts = tts;
            this.turns = turns;
        }
    }

    /** The receiver as the model reads him. Skills are the three that change the arithmetic. */
    public static final class RcvSpec {
        public final FieldCoordinate at;
        public final int ma;
        public final int ag;
        public final int str;
        public final boolean active;
        public final boolean hasCatch;
        public final boolean sureHands;
        public final boolean sideStep;

        public RcvSpec(FieldCoordinate at, int ma, int ag, int str, boolean active,
                boolean hasCatch, boolean sureHands, boolean sideStep) {
            this.at = at;
            this.ma = ma;
            this.ag = ag;
            this.str = str;
            this.active = active;
            this.hasCatch = hasCatch;
            this.sureHands = sureHands;
            this.sideStep = sideStep;
        }
    }

    /** Board facts the receiver model needs beyond the rasters. */
    public static final class Ctx {
        /** The acting team's turn number; the receiver's own horizon is {@code 8 - turnNr}. */
        public final int turnNr;
        public final boolean blizzard;

        public Ctx(int turnNr, boolean blizzard) {
            this.turnNr = turnNr;
            this.blizzard = blizzard;
        }
    }

    /** Rust {@code receiver_of}. */
    public static Receiver receiverOf(Features f, Ctx ctx, RcvSpec rcv, FieldCoordinate from,
            ValueModel.Mover m) {
        int i = Features.ix(rcv.at.getX(), rcv.at.getY());
        int s = Features.sideIdx(m.home);
        int tz = f.tz[s][i] & 0xff;
        float raw = Reach.pRoll(Math.max(rcv.ag - 1 + tz, 2));
        float pCatch = rcv.hasCatch ? Reach.pWithReRoll(raw, 1.0f) : raw;

        // The receiver, valued as though HE were the carrier — measured from where the ball
        // STARTS, so the advance term reads as ground the ball gained.
        ValueModel.Mover rm = new ValueModel.Mover(m.home, true, rcv.ma, rcv.ag, rcv.str,
            rcv.sureHands, rcv.sideStep, rcv.hasCatch,
            ValueModel.endzoneDistance(from.getX(), m.home),
            Math.max(8 - ctx.turnNr, 0), m.unactivated);

        int dRcv = ValueModel.endzoneDistance(rcv.at.getX(), m.home);
        int ma = Math.max(rm.ma, 1);
        int reachAfter = rcv.active ? rm.ma + 2 : 0;
        // P(he actually gets it in | he catches it). ~0.85 covers the dodges a run through traffic
        // needs; each rush beyond MA costs its own roll on top.
        float pGfi = Reach.pRoll(Reach.gfiTarget(ctx.blizzard));
        float pRunIn;
        if (!rcv.active) {
            pRunIn = 0.0f;
        } else if (dRcv <= Math.max(rm.ma - 2, 0)) {
            pRunIn = 0.95f;
        } else if (dRcv <= rm.ma) {
            pRunIn = 0.85f;
        } else if (dRcv <= rm.ma + 1) {
            pRunIn = 0.85f * pGfi;
        } else if (dRcv <= rm.ma + 2) {
            pRunIn = 0.85f * pGfi * pGfi;
        } else {
            // Out of scoring reach this turn, but he can still run: the delivery discount floors it.
            pRunIn = 0.0f;
        }

        // Where the ball ENDS UP, not where the receiver is standing. This is the tempo bought.
        int effectiveD = Math.max(dRcv - reachAfter, 0);
        boolean carrierScoresNow = m.dNow <= m.ma + 2;
        boolean scoresNow = effectiveD == 0 && rcv.active && !carrierScoresNow;

        // ABSOLUTE, on the same scale `value_at` uses for a carrier's own move: how far up the
        // pitch is the ball when the turn ends? That is the only way a ball-move and a run can be
        // compared at all.
        float maxGain = (float) Math.max(Math.min(m.dNow, m.ma + 2), 1);
        float advance = Math.min(Math.max((float) (m.dNow - effectiveD) / maxGain, 0.0f), 1.0f);
        float exposure = ValueModel.exposure(f, i, m.home, rm.str);
        float lane = f.lane[s][i];
        // Getting the ball out of trouble is the other half of why teams hand off, and it is a
        // MARGIN: zero when the receiver is in as much danger as the carrier.
        float relief = Math.max(exposure
            - ValueModel.exposure(f, Features.ix(from.getX(), from.getY()), m.home, m.str), 0.0f);
        float v;
        if (scoresNow) {
            // The one case worth paying a catch roll for, and it needs no lookahead to value.
            v = pRunIn;
        } else {
            // A token positional credit only. Crediting the ground a throw buys, or the safety it
            // buys, measured worse over 3200 games: this agent cannot collect on either.
            v = Math.min(0.12f * advance * exposure * lane + 0.10f * relief, 0.20f);
        }
        return new Receiver(pCatch, v, scoresNow,
            (effectiveD + ma - 1) / ma,
            rcv.active ? m.turnsLeft : Math.max(m.turnsLeft - 1, 0));
    }

    /**
     * Rust {@code handoff_weight}.
     *
     * <p>A hand-off is a pass with no throw to fumble — only the catch can fail — so it is strictly
     * the safer way to move the ball one square, and it is priced the same way. Which mostly means:
     * a hand-off to an unactivated receiver who can then run it in is a TOUCHDOWN, not "a slightly
     * better square".
     */
    public static float handoffWeight(Features f, Ctx ctx, RcvSpec rcv, FieldCoordinate from,
            ValueModel.Mover m) {
        Receiver r = receiverOf(f, ctx, rcv, from, m);

        int maT = Math.max(m.ma, 1);
        int ownTts = (m.dNow + maT - 1) / maT;
        boolean hopeless = ownTts > m.turnsLeft;
        boolean rescues = hopeless && r.tts <= r.turns;

        float v = r.v;
        if (r.scoresNow) {
            v = 1.0f;
        } else if (rescues) {
            v = Math.max(v, 0.85f);
        }
        // A drive that was going to score nothing has little left to lose, which the generic
        // turnover cost -- priced off how many players are still unactivated -- cannot see.
        float risk = Arrival.cTurnover(m.unactivated, 0, false) * (hopeless ? 0.30f : 1.0f);
        // No payoff multiplier: `scoresNow` already carries P(touchdown).
        return r.pCatch * v * 1.0f - (1.0f - r.pCatch) * risk;
    }

    /**
     * Rust {@code pass_weight}, given the ENGINE's grading of the six die faces.
     *
     * <p>A fumble is a turnover on the spot, so a pass has to be an EXPECTATION and the three
     * outcomes are priced apart:
     *
     * <pre>
     * pComplete = pAccurate * pCatch
     * pLost     = pFumble + pScatter * 0.45 + pAccurate * (1 - pCatch)
     * w         = pComplete * v - pLost * risk
     * </pre>
     *
     * <p><b>The 0.45 is the whole argument.</b> A scattered ball is NOT a turnover — it lands three
     * squares away and either side may reach it — so pricing a scatter like a fumble makes the
     * agent refuse every pass it should be making.
     *
     * <p>{@code nAccurate} and {@code nFumble} are counted by rolling all six faces through the
     * engine's own grader, NOT derived from the target number: a 1 fumbles whatever the target is,
     * and the accurate band differs by edition. Tackle zones on the thrower shift the effective
     * ROLL rather than the target, which is why the caller passes the shifted counts rather than a
     * modifier.
     *
     * @param nAccurate faces the engine grades ACCURATE, out of six.
     * @param nFumble faces the engine grades FUMBLE, out of six.
     */
    public static float passWeight(Features f, Ctx ctx, RcvSpec rcv, FieldCoordinate from,
            ValueModel.Mover m, int nAccurate, int nFumble) {
        float pAccurate = (float) nAccurate / 6.0f;
        float pFumble = (float) nFumble / 6.0f;
        float pScatter = Math.max(1.0f - pAccurate - pFumble, 0.0f);

        Receiver r = receiverOf(f, ctx, rcv, from, m);
        float pComplete = pAccurate * r.pCatch;
        // A scattered ball is not a turnover; only a fumble, or a dropped catch, hands the turn
        // over outright.
        float pLost = pFumble + pScatter * 0.45f + pAccurate * (1.0f - r.pCatch);

        int maT = Math.max(m.ma, 1);
        int ownTts = (m.dNow + maT - 1) / maT;
        boolean hopeless = ownTts > m.turnsLeft;
        // A completion that leaves somebody who CAN still make it turns a dead drive into a live one.
        boolean rescues = hopeless && r.tts <= r.turns;

        float v = r.v;
        if (r.scoresNow) {
            v = 1.0f;
        } else if (rescues) {
            v = Math.max(v, 0.85f);
        }
        float risk = Arrival.cTurnover(m.unactivated, 0, false) * (hopeless ? 0.30f : 1.0f);
        return pComplete * v - pLost * risk;
    }

    /**
     * Count the ACCURATE and FUMBLE faces the way {@code pass_weight} does — through the engine's
     * own grader, with tackle zones shifting the roll.
     *
     * <p>This is the production path; the fixture feeds the counts in directly so that what it
     * pins is the arithmetic rather than a second copy of the pass tables, which both engines
     * already share and the parity matrix already covers.
     *
     * @return {@code {nAccurate, nFumble}}, or null when the throw is not legal at all.
     */
    public static int[] gradeFaces(com.fumbbl.ffb.mechanics.PassMechanic mech,
            com.fumbbl.ffb.model.Game game, com.fumbbl.ffb.model.Player<?> thrower,
            FieldCoordinate from, FieldCoordinate to, int tzOnThrower) {
        com.fumbbl.ffb.PassingDistance dist = basePassingDistance(mech, game, from, to);
        if (dist == null) {
            return null;
        }
        int nAccurate = 0;
        int nFumble = 0;
        for (int die = 1; die <= 6; die++) {
            com.fumbbl.ffb.mechanics.PassResult res =
                mech.evaluatePass(thrower, die - tzOnThrower, dist, java.util.Collections.emptyList(), false);
            if (res == com.fumbbl.ffb.mechanics.PassResult.ACCURATE) {
                nAccurate++;
            } else if (res == com.fumbbl.ffb.mechanics.PassResult.FUMBLE) {
                nFumble++;
            }
        }
        return new int[] {nAccurate, nFumble};
    }

    /**
     * The BASE passing distance — the abstract class's own implementation, not the edition
     * override.
     *
     * <p>Two reasons, and the second is the load-bearing one:
     *
     * <ul>
     *   <li>The bb2025 override dereferences {@code game.getActingPlayer().getPlayerAction()} to
     *       ask whether the throw is a bomb. The agent scores passes BEFORE declaring an action, so
     *       at that moment there is no acting player and the override throws.
     *   <li><b>Rust does not implement that override at all.</b> Its {@code find_passing_distance}
     *       is the base method and nothing else, so the base is what the two agents have to agree
     *       on. Calling the override here would make the Java agent disagree with the Rust one on
     *       any board where the refinement fires.
     * </ul>
     *
     * <p>The refinement it skips is PASS_TO_PARTNER, which needs a partnered pair (Two For One) —
     * absent from every roster in this tier, so nothing observable is lost today. That Rust lacks
     * it is a pre-existing port gap, recorded here rather than papered over.
     *
     * <p>Read reflectively from the engine's OWN table rather than transcribed, so the edition's
     * ranges stay the single source of truth.
     */
    static com.fumbbl.ffb.PassingDistance basePassingDistance(
            com.fumbbl.ffb.mechanics.PassMechanic mech, com.fumbbl.ffb.model.Game game,
            FieldCoordinate from, FieldCoordinate to) {
        if (from == null || to == null) {
            return null;
        }
        int deltaY = Math.abs(to.getY() - from.getY());
        int deltaX = Math.abs(to.getX() - from.getX());
        if (deltaY >= 14 || deltaX >= 14) {
            return null;
        }
        com.fumbbl.ffb.PassingDistance d;
        try {
            java.lang.reflect.Field f =
                com.fumbbl.ffb.mechanics.PassMechanic.class.getDeclaredField(
                    "PASSING_DISTANCES_TABLE");
            f.setAccessible(true);
            com.fumbbl.ffb.PassingDistance[][] table =
                (com.fumbbl.ffb.PassingDistance[][]) f.get(mech);
            d = table[deltaY][deltaX];
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException("PassMechanic.PASSING_DISTANCES_TABLE moved", e);
        }
        boolean blizzard = game.getFieldModel().getWeather() == com.fumbbl.ffb.Weather.BLIZZARD;
        if (blizzard && (d == com.fumbbl.ffb.PassingDistance.LONG_BOMB
                || d == com.fumbbl.ffb.PassingDistance.LONG_PASS)) {
            return null;
        }
        return d;
    }

    /** Rust {@code p_2d6_at_least}. */
    public static float p2d6AtLeast(int need) {
        if (need <= 2) {
            return 1.0f;
        }
        if (need > 12) {
            return 0.0f;
        }
        int ways = 0;
        for (int a = 1; a <= 6; a++) {
            for (int b = 1; b <= 6; b++) {
                if (a + b >= need) {
                    ways++;
                }
            }
        }
        return (float) ways / 36.0f;
    }

    /**
     * Rust {@code foul_weight}.
     *
     * <p>An expectation, not a stack of multipliers: what the foul buys minus what it risks. The
     * assist counts come from the engine's own {@code UtilPlayer} — they modify the ARMOUR roll,
     * which is the whole reason to prefer one victim over another, and the reason to foul at all.
     *
     * @param offAssists {@code UtilPlayer.findOffensiveFoulAssists}
     * @param defAssists {@code UtilPlayer.findDefensiveFoulAssists}
     */
    public static float foulWeight(Features f, int av, int offAssists, int defAssists,
            FieldCoordinate defCoord, int bribes, ValueModel.Mover m) {
        float pBreak = Math.min(Math.max(
            p2d6AtLeast(av - (offAssists - defAssists) + 1), 0.03f), 0.97f);
        float victim;
        if (f.ballCarried && f.ball != null && f.ball.equals(defCoord)) {
            victim = 1.0f;
        } else if (f.ball != null && Math.max(Math.abs(f.ball.getX() - defCoord.getX()),
                Math.abs(f.ball.getY() - defCoord.getY())) <= 1) {
            victim = 0.7f;
        } else {
            victim = 0.35f;
        }
        float ejectCost = bribes > 0 ? 0.07f : 0.45f;
        // The referee spots a foul on DOUBLES -- armour, then injury if the armour broke. Fixed at
        // about 1/6, rising with the chance of hurting the victim, and nothing the agent chooses
        // lowers it.
        float pEject = 0.167f + pBreak * (5.0f / 6.0f) * 0.167f;
        float timing = m.unactivated <= 3.0f / 11.0f ? 1.0f : 0.85f;
        return (pBreak * victim - pEject * ejectCost) * timing;
    }
}
