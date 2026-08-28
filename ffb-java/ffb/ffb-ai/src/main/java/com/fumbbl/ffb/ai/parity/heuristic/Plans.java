package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.FieldCoordinate;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * The destination orderings {@code build_plans} enumerates from.
 *
 * <p>The arrival weights are pinned elsewhere; what lives here is the ORDER, and the order is the
 * part nothing downstream can survive getting wrong — the agent samples an INDEX into these lists,
 * so two implementations that agree on every weight and disagree on one comparison pick different
 * squares.
 */
public final class Plans {

    /** Run-up squares a PassMove considers throwing from, besides standing still. */
    public static final int THROW_SPOTS = 6;

    private Plans() {
    }

    /** One scored destination: a cell index and the weight of arriving there. */
    public static final class Dest {
        public final float w;
        public final int i;

        public Dest(float w, int i) {
            this.w = w;
            this.i = i;
        }
    }

    /**
     * Rust {@code top_moves}: every REACHED square, ordered by arrival weight descending, with the
     * cell index ascending as the tie-break.
     *
     * <p>The tie-break is not decoration. On an open pitch most reachable squares score identically
     * — a plain move improves nothing — so ties do nearly all the ordering work, and a comparator
     * that leaves them to the sort's own stability agrees only by luck.
     *
     * <p><b>Measured caveat.</b> Dropping the index fallback here does NOT change the result on
     * any fixture board, and cannot: the list is built from {@code Reach::order}, which is sorted
     * ascending, and both languages' sorts are stable, so ties keep their input order either way.
     * The explicit fallback is kept because it is what Rust writes and because it stops the
     * ordering depending on a property of {@code order} that lives in a different method — but the
     * fixture does not test it, and claiming otherwise would be false.
     */
    public static List<Dest> topMoves(Features f, Reach r, ValueModel.Mover m, int k) {
        List<Dest> v = new ArrayList<>(r.order.length);
        for (int idx : r.order) {
            v.add(new Dest(Arrival.weight(f, r, idx, m), idx));
        }
        v.sort(Comparator.<Dest>comparingDouble(d -> -d.w).thenComparingInt(d -> d.i));
        if (k < v.size()) {
            return new ArrayList<>(v.subList(0, k));
        }
        return v;
    }

    /**
     * Rust {@code risked}: fold the chance of never arriving into a plan's weight.
     *
     * <p><b>Not {@code w * p}.</b> That is right only for a positive weight, and it turns a bad
     * plan reached by a risky route into a better-looking one — the risk would improve it. The
     * turnover term is charged at {@code gfi = 0} and as a CARRIER regardless of the mover, because
     * these plans are all ones that end with the ball changing hands.
     */
    public static float risked(float w, float pArrive, ValueModel.Mover m) {
        return pArrive * w - (1.0f - pArrive) * Arrival.cTurnover(m.unactivated, 0, true);
    }

    /**
     * Rust {@code run_up_squares}: where a carrier might throw from — his own square, plus the best
     * a run-up could reach.
     *
     * <p>Two orderings live in this one function, and collapsing them into one is the obvious
     * mistake. The mover's CURRENT square goes first unconditionally, so "use none of my move" can
     * never be dropped; the rest are ranked by ARRIVAL PROBABILITY weighted by forward progress —
     * deliberately not by arrival weight, because a throwing platform is judged by whether he gets
     * there and how far up the pitch it is, not by what standing there is worth.
     */
    public static List<Integer> runUpSquares(Reach r, ValueModel.Mover m, FieldCoordinate here) {
        List<Integer> out = new ArrayList<>();
        out.add(Features.ix(here.getX(), here.getY()));
        if (r == null) {
            return out;
        }
        List<Dest> v = new ArrayList<>(r.order.length);
        for (int i : r.order) {
            float fwd = (float) (m.dNow - ValueModel.endzoneDistance(i % Features.W, m.home));
            v.add(new Dest(r.pArrive(i) * (1.0f + 0.25f * Math.max(fwd, 0.0f)), i));
        }
        v.sort(Comparator.<Dest>comparingDouble(d -> -d.w).thenComparingInt(d -> d.i));
        int taken = 0;
        for (Dest d : v) {
            if (taken >= THROW_SPOTS) {
                break;
            }
            taken++;
            if (!out.contains(d.i)) {
                out.add(d.i);
            }
        }
        return out;
    }
}
