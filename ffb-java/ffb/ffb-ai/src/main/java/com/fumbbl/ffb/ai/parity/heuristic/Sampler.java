package com.fumbbl.ffb.ai.parity.heuristic;

import com.fumbbl.ffb.ai.parity.Xoshiro256StarStar;

/**
 * The heuristic agent's scored-option buffer and sampler.
 *
 * <p>Mirror of the sampling half of the Rust {@code HeuristicAgent} — {@code unit}, {@code argmax},
 * {@code pick} and {@code softmax_pick}. Specified by {@code AGENT_CONTRACT_HEURISTIC.md} sections
 * 1 and 2.
 *
 * <p><b>The draw count is the contract, and it is not constant.</b> If this side spends two draws
 * where Rust spends one, the stream desynchronises and every decision afterwards is unrelated
 * noise — which surfaces as a state-hash mismatch far downstream that looks nothing like its cause.
 * The Rust side pins the same table with {@code pick_draw_counts_match_the_contract}.
 *
 * <p>The other half of the contract is that <b>options are enumerated in the same order on both
 * sides</b>: the agent samples an <i>index</i>, and the caller maps that index back to a command.
 * Ordering rules are in {@code AGENT_CONTRACT_HEURISTIC.md} section 5 — never by player id.
 */
public final class Sampler {

    /** Mirrors Rust's {@code EPS}. */
    private static final float EPS = 0.02f;

    private final Xoshiro256StarStar rng;
    /** Multiplies every temperature: 0 = argmax (no RNG at all), 1 = the table, 1e6 = uniform. */
    private final float tempScale;

    private float[] weights = new float[64];
    private int n = 0;
    /** Draw counter, so tests can assert the contract in section 2 rather than just the answer. */
    private long draws = 0;

    public Sampler(long seed, float tempScale) {
        // AGENT_CONTRACT_HEURISTIC.md section 1: "HEURISTI".
        this.rng = new Xoshiro256StarStar(seed ^ 0x4845555249535449L);
        this.tempScale = tempScale;
    }

    public float tempScale() {
        return tempScale;
    }

    /** Drop every option; call before enumerating a new decision. */
    public void clear() {
        n = 0;
    }

    /** Add one option's weight. The INDEX is what the caller gets back, so order matters. */
    public void push(float w) {
        if (n == weights.length) {
            float[] bigger = new float[n * 2];
            System.arraycopy(weights, 0, bigger, 0, n);
            weights = bigger;
        }
        weights[n++] = w;
    }

    public int size() {
        return n;
    }

    /**
     * Rust's {@code unit()}: {@code ((next_u64() >> 11) as f32) / (1u64 << 53) as f32}.
     *
     * <p>The shift is UNSIGNED, and {@code long -> float} is round-to-nearest-even in both
     * languages, so this agrees bit for bit.
     */
    float unit() {
        draws++;
        return (float) (rng.nextLong() >>> 11) / (float) (1L << 53);
    }

    /** How many {@code nextLong()} calls this sampler has consumed. Test instrumentation only. */
    long drawCount() {
        return draws;
    }

    /** First strict maximum wins — matches Rust's {@code argmax}, which uses {@code >}. */
    private int argmax() {
        int bi = 0;
        float bw = -Float.MAX_VALUE;
        for (int i = 0; i < n; i++) {
            if (weights[i] > bw) {
                bw = weights[i];
                bi = i;
            }
        }
        return bi;
    }

    /**
     * Sample an option index over the pushed weights. Mirrors Rust's {@code pick}.
     *
     * <p>Draws: 0 when {@code n <= 1} or {@code tempScale <= 0}; 1 when
     * {@code 0 < tempScale < 0.1} (eps is 0 and the probe is short-circuited away); exactly 2
     * otherwise, on both branches.
     */
    public int pick(float tBase) {
        if (n <= 1) {
            return 0;
        }
        if (tempScale <= 0.0f) {
            return argmax();
        }
        float t = Math.max(tBase * tempScale, 1e-6f);
        float max = maxWeight();
        float eps = tempScale < 0.1f ? 0.0f : EPS;
        // `eps > 0 &&` short-circuits in Rust too, so the probe draw is NOT taken when eps is 0.
        if (eps > 0.0f && unit() < eps) {
            // Rust: `(next_u64() as usize) % n` -- unsigned remainder.
            draws++;
            return (int) Math.min(Long.remainderUnsigned(rng.nextLong(), n), n - 1L);
        }
        // The cumulative array is UNNORMALISED; the draw is scaled by the accumulator.
        float acc = 0.0f;
        float[] cum = new float[n];
        for (int i = 0; i < n; i++) {
            acc += DetMath.expF32((weights[i] - max) / t);
            cum[i] = acc;
        }
        float r = unit() * acc;
        // Rust: `cum.partition_point(|&c| c < r).min(n - 1)` -- first index with cum >= r.
        int idx = n - 1;
        for (int i = 0; i < n; i++) {
            if (!(cum[i] < r)) {
                idx = i;
                break;
            }
        }
        return Math.min(idx, n - 1);
    }

    /**
     * Softmax over an explicit weight array, used by WIDE {@code ActivatePlayer}'s two-level draw.
     * Mirrors Rust's {@code softmax_pick}: no eps escape, a NORMALISED distribution, and exactly
     * one draw when it decides.
     */
    public int softmaxPick(float[] w, int len, float tBase) {
        if (len <= 1) {
            return 0;
        }
        if (tempScale <= 0.0f) {
            int bi = 0;
            for (int i = 1; i < len; i++) {
                if (w[i] > w[bi]) {
                    bi = i;
                }
            }
            return bi;
        }
        float t = Math.max(tBase * tempScale, 1e-6f);
        float max = -Float.MAX_VALUE;
        for (int i = 0; i < len; i++) {
            if (w[i] > max) {
                max = w[i];
            }
        }
        float[] ps = new float[len];
        float acc = 0.0f;
        for (int i = 0; i < len; i++) {
            ps[i] = DetMath.expF32((w[i] - max) / t);
            acc += ps[i];
        }
        if (acc > 0.0f) {
            for (int i = 0; i < len; i++) {
                ps[i] /= acc;
            }
        }
        float r = unit();
        float c = 0.0f;
        // Rust's fall-through default when the walk does not trigger is n - 1.
        int pick = len - 1;
        for (int i = 0; i < len; i++) {
            c += ps[i];
            if (r < c) {
                pick = i;
                break;
            }
        }
        return pick;
    }

    /**
     * Rust folds with {@code f32::max}, which returns the non-NaN operand; {@code Math.max}
     * propagates NaN instead. Weights are never NaN, but write the loop out rather than rely on
     * that.
     */
    private float maxWeight() {
        float max = -Float.MAX_VALUE;
        for (int i = 0; i < n; i++) {
            if (weights[i] > max) {
                max = weights[i];
            }
        }
        return max;
    }
}
