package com.fumbbl.ffb.ai.parity.heuristic;

/**
 * Rust {@code arrival_parts} — the signed weight of arriving at a square.
 *
 * <p>This is where the reach search and the value model are finally multiplied together:
 *
 * <pre>w = p·V − (1−p)·c_turnover − rush_penalty</pre>
 *
 * <p>with one short-circuit: a carrier arriving IN the endzone scores, and a touchdown ends the
 * drive, so there is no "after" to lose and only the rush is priced.
 *
 * <p>The parts are kept alongside the total because "why that square?" is the first question anyone
 * asks of a movement decision, and the total alone cannot answer it — and because three terms
 * summing to the same number by different routes is exactly the disagreement a single value hides.
 */
public final class Arrival {

    public final float w;
    public final float pArrive;
    public final float v;
    public final int gfi;

    private Arrival(float w, float pArrive, float v, int gfi) {
        this.w = w;
        this.pArrive = pArrive;
        this.v = v;
        this.gfi = gfi;
    }

    /**
     * Rust {@code c_turnover}: what losing the ball here costs in expectation.
     *
     * <p>Scaled by how many team-mates have NOT yet acted — a turnover early in a turn forfeits
     * more than one at the end of it.
     */
    public static float cTurnover(float unactivated, int gfi, boolean carriesBall) {
        return (0.20f + 0.55f * unactivated)
            * (carriesBall ? 1.4f : 1.0f)
            * (1.0f + 0.15f * (float) gfi);
    }

    /**
     * Rust {@code rush_penalty}: risk aversion on rushes, OVER AND ABOVE the expectation already in
     * {@link #cTurnover}.
     *
     * <p>A turnover forfeits the rest of the drive rather than one square, and that compounding is
     * invisible to a single-step mean. Four times harsher for a player who is not carrying: he has
     * nothing to gain that justifies the risk.
     */
    public static float rushPenalty(int gfi, boolean carriesBall) {
        if (gfi <= 0) {
            return 0.0f;
        }
        return (carriesBall ? 0.10f : 0.40f) * (float) gfi;
    }

    /** Rust {@code arrival_parts}. */
    public static Arrival parts(Features f, Reach r, int i, ValueModel.Mover m) {
        float pa = r.pArrive(i);
        int gfi = r.gfi[i];
        int x = i % Features.W;
        if (m.isCarrier && ValueModel.endzoneDistance(x, m.home) == 0) {
            // A touchdown ends the drive: there is no "after" to lose, so only the rush is priced.
            return new Arrival(pa - rushPenalty(gfi, true), pa, 1.0f, gfi);
        }
        float v = ValueModel.valueAt(f, i, m).v;
        float w = pa * v
            - (1.0f - pa) * cTurnover(m.unactivated, gfi, m.isCarrier)
            - rushPenalty(gfi, m.isCarrier);
        return new Arrival(w, pa, v, gfi);
    }

    /** Rust {@code arrival_weight}. */
    public static float weight(Features f, Reach r, int i, ValueModel.Mover m) {
        return parts(f, r, i, m).w;
    }
}
