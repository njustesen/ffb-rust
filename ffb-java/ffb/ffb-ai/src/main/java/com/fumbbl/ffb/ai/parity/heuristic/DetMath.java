package com.fumbbl.ffb.ai.parity.heuristic;

/**
 * Bit-reproducible {@code exp} and {@code ln} for {@code float}, shared with the Rust heuristic
 * agent.
 *
 * <p><b>Line-for-line transcription of {@code ffb-rust/crates/ffb-engine/src/agent/det_math.rs}.
 * Do not "improve" anything here.</b> Any change is a change to both engines and must land on both
 * sides in the same change set, with the golden table regenerated.
 *
 * <p><b>Why this exists.</b> {@code HeuristicAgent} has to make byte-identical decisions in two
 * languages so the parity matrix can run it on both engines. Almost all of its arithmetic is
 * portable for free: IEEE-754 requires {@code + - * /} on {@code binary32} to be correctly rounded,
 * Java 17+ is unconditionally strict-FP (JEP 306), and Rust does not contract to FMA. The
 * exceptions are the transcendentals — Rust's libm, {@code Math}, and {@code StrictMath} are three
 * different implementations with no bit-agreement guarantee, and a one-ulp disagreement in a
 * softmax weight can flip which option a draw selects, desynchronising the two games.
 *
 * <p><b>The approach.</b> These functions do not need to be as accurate as libm; they need to be
 * identical to each other. So they are built exclusively from correctly-rounded {@code float}
 * primitives, exponent surgery via {@link Float#floatToRawIntBits}/{@link Float#intBitsToFloat},
 * and a fixed-degree polynomial in a fixed evaluation order. Every step is bit-determined by the
 * IEEE spec, so the two implementations agree by construction rather than by luck.
 *
 * @see com.fumbbl.ffb.ai.parity.heuristic.DetMathTest
 */
public final class DetMath {

    private DetMath() {}

    /** {@code log2(e)}. */
    private static final float LOG2E = 1.44269504f;
    /** {@code ln(2)}, split so the range reduction keeps its low bits (fdlibm's float split). */
    private static final float LN2_HI = 6.93145752e-1f;
    private static final float LN2_LO = 1.42860677e-6f;
    /** {@code ln(2)}, for recomposing {@code ln} from the exponent. */
    private static final float LN2 = 6.9314718e-1f;
    /** Below this, {@code e^x} is zero in {@code float} to within a subnormal. */
    private static final float EXP_LO = -88.0f;
    /** Above this, {@code e^x} overflows {@code float}. */
    private static final float EXP_HI = 88.0f;

    /**
     * {@code e^x}, bit-identical to Rust's {@code det_math::exp_f32}.
     *
     * <p>Cody-Waite range reduction to {@code r} in roughly {@code [-0.347, 0.347]}, a degree-7
     * Taylor polynomial in Horner form, then a scale by {@code 2^k}.
     */
    public static float expF32(float x) {
        if (Float.isNaN(x)) {
            return x;
        }
        if (x >= EXP_HI) {
            return Float.POSITIVE_INFINITY;
        }
        if (x <= EXP_LO) {
            return 0.0f;
        }
        // Rust's f32::round is ties-AWAY-FROM-ZERO. Math.round is ties-toward-positive-infinity and
        // Math.rint is ties-to-even, so neither matches; do it explicitly.
        float kf = roundTiesAway(x * LOG2E);
        int k = (int) kf;
        // Two-step subtraction, so the reduced argument keeps its precision.
        float r = x - kf * LN2_HI - kf * LN2_LO;
        // e^r by Horner. The degree and the order of operations are both part of the contract.
        float p = 1.0f / 5040.0f;
        p = p * r + 1.0f / 720.0f;
        p = p * r + 1.0f / 120.0f;
        p = p * r + 1.0f / 24.0f;
        p = p * r + 1.0f / 6.0f;
        p = p * r + 1.0f / 2.0f;
        p = p * r + 1.0f;
        p = p * r + 1.0f;
        return p * pow2i(k);
    }

    /**
     * {@code ln(x)}, bit-identical to Rust's {@code det_math::ln_f32}.
     *
     * <p>Splits {@code x} into {@code 2^e * m} with {@code m} near 1, then uses the odd series in
     * {@code s = (m-1)/(m+1)}.
     */
    public static float lnF32(float x) {
        if (Float.isNaN(x) || x < 0.0f) {
            return Float.NaN;
        }
        if (x == 0.0f) {
            return Float.NEGATIVE_INFINITY;
        }
        if (x == Float.POSITIVE_INFINITY) {
            return x;
        }
        int bits = Float.floatToRawIntBits(x);
        int e = 0;
        // Scale a subnormal into the normal range before reading its exponent.
        if (((bits >>> 23) & 0xff) == 0) {
            float scaled = x * 16777216.0f; // 2^24
            bits = Float.floatToRawIntBits(scaled);
            e -= 24;
        }
        e += ((bits >>> 23) & 0xff) - 127;
        // Mantissa in [1, 2).
        float m = Float.intBitsToFloat((bits & 0x007fffff) | 0x3f800000);
        // Centre on 1 so the series argument stays small.
        if (m > 1.4142136f) {
            m = m * 0.5f;
            e += 1;
        }
        float s = (m - 1.0f) / (m + 1.0f);
        float z = s * s;
        float p = 1.0f / 11.0f;
        p = p * z + 1.0f / 9.0f;
        p = p * z + 1.0f / 7.0f;
        p = p * z + 1.0f / 5.0f;
        p = p * z + 1.0f / 3.0f;
        p = p * z + 1.0f;
        return 2.0f * s * p + ((float) e) * LN2;
    }

    /**
     * Round half away from zero, matching Rust's {@code f32::round}.
     *
     * <p>{@code Math.round} rounds halves toward positive infinity and {@code Math.rint} rounds
     * them to even, so both differ from Rust at exactly the .5 cases.
     */
    static float roundTiesAway(float v) {
        float f = (float) Math.floor(Math.abs(v) + 0.5f);
        return v < 0.0f ? -f : f;
    }

    /**
     * {@code 2^k} by writing the exponent field directly. Outside the normal exponent range the
     * value is built in two exact steps rather than being lost to a malformed exponent.
     */
    static float pow2i(int k) {
        if (k >= -126 && k <= 127) {
            return Float.intBitsToFloat((k + 127) << 23);
        }
        if (k > 127) {
            return Float.POSITIVE_INFINITY;
        }
        float half = Float.intBitsToFloat(1 << 23); // 2^-126
        int rest = k + 126;
        if (rest < -126) {
            return 0.0f;
        }
        return half * Float.intBitsToFloat((rest + 127) << 23);
    }
}
