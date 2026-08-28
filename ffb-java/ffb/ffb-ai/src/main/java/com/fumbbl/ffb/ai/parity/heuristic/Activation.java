package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * The tier-1 activation ranking from Rust's {@code handle_activate}.
 *
 * <p>Before any pathfinding runs, every eligible player is scored search-free, and only the top
 * {@link #TIER2} get a Dijkstra at all. So this decides not merely who is likeliest to be picked
 * but <b>who is even considered properly</b> — and a disagreement here does not look like a wrong
 * move. It looks like the right move made by the wrong player, or by a player the other engine
 * never scored.
 *
 * <p>{@link #playerWeight} is a chain of else-ifs, so the ORDER of the rungs can matter as much as
 * the constants — but only where two rungs can hold at once, and most of these cannot. "Can fetch a
 * loose ball" and "is the carrier" are mutually EXCLUSIVE by construction (a loose ball is nobody's
 * possession), so their relative order is unobservable and no board can pin it. The one adjacent
 * pair that CAN both hold is {@code prone && marked} against {@code proxy > 0.25}, and the
 * {@code prone_marked_with_support} board exists to pin exactly that.
 *
 * <p>The negatrait multiplier applies AFTER the ladder, and the {@code awaitingRun} override after
 * that — it OVERWRITES rather than scales, so a player who was just thrown the ball outranks
 * everyone however bad his own rung was.
 */
public final class Activation {

    /**
     * How many players get a real search. Rust's comment is worth carrying: a player without a
     * search contributes ONE placeholder option standing in for his whole destination space, so he
     * collects probability mass that should have been spread over a dozen squares and is
     * over-sampled against players whose destinations are enumerated. 16 covers every player who
     * can ever be eligible.
     */
    public static final int TIER2 = 16;

    private Activation() {
    }

    /** One eligible player, scored before any search. */
    public static final class Cand {
        public final String id;
        /** Canonical ordering key: home before away, then jersey number. Never the id. */
        public final int side;
        public final int nr;
        public final float wPlayer;
        public final float proxy;

        public Cand(String id, int side, int nr, float wPlayer, float proxy) {
            this.id = id;
            this.side = side;
            this.nr = nr;
            this.wPlayer = wPlayer;
            this.proxy = proxy;
        }
    }

    /**
     * Rust's {@code w_player} ladder.
     *
     * @param awaitingRun this player was just thrown the ball; running it on is the reason the
     *     throw was made, so he is forced to 1.0.
     */
    public static float playerWeight(boolean isCarrier, boolean marked, boolean canFetch,
            boolean prone, float proxy, boolean negatrait, boolean awaitingRun) {
        float w;
        if (isCarrier && marked) {
            w = 0.95f;
        } else if (canFetch) {
            w = 0.92f;
        } else if (isCarrier) {
            w = 0.88f;
        } else if (prone && marked) {
            w = 0.70f;
        } else if (proxy > 0.25f) {
            w = 0.45f;
        } else {
            w = 0.30f;
        }
        if (negatrait) {
            w *= 0.55f;
        }
        if (awaitingRun) {
            w = 1.0f;
        }
        return w;
    }

    /**
     * Can this player reach a LOOSE ball at all this activation? Rust measures in Chebyshev steps
     * against {@code MA + 2}, i.e. including both rushes, and only while the ball is loose — a
     * carried ball is somebody's possession, not something to fetch.
     */
    public static boolean canFetch(Features f, FieldCoordinate at, int ma) {
        if (!f.ballLoose || f.ball == null) {
            return false;
        }
        int d = Math.max(Math.abs(at.getX() - f.ball.getX()), Math.abs(at.getY() - f.ball.getY()));
        return d <= ma + 2;
    }

    /**
     * Rank the candidates: {@code wPlayer * max(proxy, 0.05)} descending, canonical key ascending
     * on ties.
     *
     * <p>The floor on {@code proxy} is what stops a player whose proxy is exactly 0 from being
     * unrankable — without it every such player collapses to the same 0 and the tie-break decides
     * everything.
     *
     * <p>Rust sorts canonically FIRST and then ranks with a stable sort, which is the same result
     * as an explicit two-level comparator; this does the latter so it does not depend on the input
     * arriving in canonical order.
     */
    public static List<Cand> rank(List<Cand> cands) {
        List<Cand> out = new ArrayList<>(cands);
        out.sort(Comparator
            .<Cand>comparingDouble(c -> -(c.wPlayer * Math.max(c.proxy, 0.05f)))
            .thenComparingInt(c -> c.side)
            .thenComparingInt(c -> c.nr));
        return out;
    }

    /** The indices into {@link #rank}'s output that get a real search. */
    public static int tier2Count(int nCands) {
        return Math.min(TIER2, nCands);
    }
}
