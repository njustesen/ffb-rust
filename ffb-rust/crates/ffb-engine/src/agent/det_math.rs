//! Bit-reproducible `exp` and `ln` for `f32`, shared by the Rust and Java heuristic agents.
//!
//! # Why this exists
//!
//! `HeuristicAgent` has to make byte-identical decisions in two languages so the parity matrix can
//! run it on both engines (`docs/PARITY_HEURISTIC_CAMPAIGN.md`). Almost all of its arithmetic is
//! already portable for free: IEEE-754 requires `+ - * /` on `binary32` to be **correctly
//! rounded**, Java 17+ is unconditionally strict-FP (JEP 306), and Rust does not contract to FMA.
//! Given identical inputs, both languages must produce the identical bit pattern.
//!
//! The exceptions are the transcendentals. Rust's `f32::exp`/`f32::ln` call into the platform
//! libm; Java's `Math.exp`/`Math.log` are a different implementation, and `StrictMath` is a third.
//! None are guaranteed to agree to the last bit, and a one-ulp disagreement in a softmax weight
//! can flip which option a draw selects, which desynchronises the two games.
//!
//! # The approach
//!
//! **These functions do not need to be as accurate as libm. They need to be identical.** So they
//! are built exclusively from correctly-rounded `f32` primitives — `+ - * /`, comparisons, and
//! exponent surgery via `to_bits`/`from_bits` — with a fixed-degree polynomial in a fixed
//! evaluation order. Every step is bit-determined by the IEEE spec, so a faithful transcription
//! (`DetMath.java`, using `Float.floatToRawIntBits`/`intBitsToFloat`) is identical by construction
//! rather than by luck.
//!
//! Accuracy is nonetheless within a few ulp over the ranges the agent actually uses: `ln` sees
//! `p_step` in `[1e-6, 1]`, and `exp` sees only non-positive arguments (softmax shifts by the max).
//!
//! Any change here is a **policy change** — it moves weights, so it moves decisions. Re-run the A/B
//! arms and record before/after, and mirror the change into `DetMath.java` in the same change set.
//! `testdata/det_math_golden.txt` pins the exact bit patterns; the Java twin asserts on that file.

/// `log2(e)`.
const LOG2E: f32 = 1.442_695_04;
/// `ln(2)`, split so the range reduction keeps its low bits (fdlibm's float split).
const LN2_HI: f32 = 6.931_457_52e-1;
const LN2_LO: f32 = 1.428_606_77e-6;
/// `ln(2)`, for recomposing `ln` from the exponent.
const LN2: f32 = 6.931_471_8e-1;
/// Below this, `e^x` is zero in `f32` to within a subnormal, which no weight cares about. Clamping
/// keeps the `2^k` scaling inside the normal range in both languages.
const EXP_LO: f32 = -88.0;
/// Above this, `e^x` overflows `f32`.
const EXP_HI: f32 = 88.0;

/// `e^x`, bit-identical to `DetMath.expF32`.
///
/// Cody-Waite range reduction to `r` in roughly `[-0.347, 0.347]`, a degree-7 Taylor polynomial in
/// Horner form, then a scale by `2^k`.
pub fn exp_f32(x: f32) -> f32 {
    if x.is_nan() {
        return x;
    }
    if x >= EXP_HI {
        return f32::INFINITY;
    }
    if x <= EXP_LO {
        return 0.0;
    }
    // k = round(x * log2(e)). `round` is ties-away-from-zero and exact at these magnitudes, and is
    // specified identically in both languages.
    let kf = (x * LOG2E).round();
    let k = kf as i32;
    // Two-step subtraction, so the reduced argument keeps its precision.
    let r = x - kf * LN2_HI - kf * LN2_LO;
    // e^r by Horner. The degree and the order of operations are both part of the contract.
    let mut p = 1.0f32 / 5040.0;
    p = p * r + 1.0 / 720.0;
    p = p * r + 1.0 / 120.0;
    p = p * r + 1.0 / 24.0;
    p = p * r + 1.0 / 6.0;
    p = p * r + 1.0 / 2.0;
    p = p * r + 1.0;
    p = p * r + 1.0;
    p * pow2i(k)
}

/// `ln(x)`, bit-identical to `DetMath.lnF32`.
///
/// Splits `x` into `2^e * m` with `m` near 1, then uses the odd series in `s = (m-1)/(m+1)`, which
/// converges quickly over that interval.
pub fn ln_f32(x: f32) -> f32 {
    if x.is_nan() || x < 0.0 {
        return f32::NAN;
    }
    if x == 0.0 {
        return f32::NEG_INFINITY;
    }
    if x == f32::INFINITY {
        return x;
    }
    let mut bits = x.to_bits();
    let mut e: i32 = 0;
    // Scale a subnormal into the normal range before reading its exponent.
    if (bits >> 23) & 0xff == 0 {
        let scaled = x * 16_777_216.0; // 2^24
        bits = scaled.to_bits();
        e -= 24;
    }
    e += (((bits >> 23) & 0xff) as i32) - 127;
    // Mantissa in [1, 2).
    let mut m = f32::from_bits((bits & 0x007f_ffff) | 0x3f80_0000);
    // Centre on 1 so the series argument stays small.
    if m > 1.414_213_6 {
        m = m * 0.5;
        e += 1;
    }
    let s = (m - 1.0) / (m + 1.0);
    let z = s * s;
    let mut p = 1.0f32 / 11.0;
    p = p * z + 1.0 / 9.0;
    p = p * z + 1.0 / 7.0;
    p = p * z + 1.0 / 5.0;
    p = p * z + 1.0 / 3.0;
    p = p * z + 1.0;
    2.0 * s * p + (e as f32) * LN2
}

/// `2^k` by writing the exponent field directly. Outside the normal exponent range the value is
/// built in two exact steps rather than being lost to a malformed exponent.
fn pow2i(k: i32) -> f32 {
    if (-126..=127).contains(&k) {
        return f32::from_bits(((k + 127) as u32) << 23);
    }
    if k > 127 {
        return f32::INFINITY;
    }
    let half = f32::from_bits(1u32 << 23); // 2^-126
    let rest = k + 126;
    if rest < -126 {
        return 0.0;
    }
    half * f32::from_bits(((rest + 127) as u32) << 23)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regenerates `testdata/det_math_golden.txt`. Run deliberately, never in CI:
    /// `cargo test -p ffb-engine --lib det_math::tests::emit_golden_table -- --ignored`.
    /// Regenerating is a POLICY CHANGE -- it means the functions moved, so the Java twin must be
    /// updated in the same change set and the A/B arms re-measured.
    #[test]
    #[ignore]
    fn emit_golden_table() {
        use std::fmt::Write as _;
        let mut out = String::new();
        writeln!(out, "# det_math golden bit patterns -- see agent/det_math.rs.").unwrap();
        writeln!(out, "# Format: <fn> <input bits, hex> <output bits, hex>").unwrap();
        writeln!(out, "# Regenerate with the ignored test `emit_golden_table`.").unwrap();
        writeln!(out, "# Both engines assert on it: any change lands on both sides together.").unwrap();
        // exp: the softmax range (non-positive), plus the guarded edges.
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..=140 {
            xs.push(-(i as f32) * 0.7);
        }
        for i in 0..40 {
            xs.push(-(i as f32) * 0.013_7);
        }
        for v in [0.0f32, -0.5, -1.0, -1e-7, -87.9, -88.0, -88.1, -1000.0, 87.9, 88.0, 1000.0] {
            xs.push(v);
        }
        for x in xs {
            writeln!(out, "exp {:08x} {:08x}", x.to_bits(), exp_f32(x).to_bits()).unwrap();
        }
        // ln: p_step in (0, 1], which is all the agent ever passes, plus edges.
        let mut ys: Vec<f32> = Vec::new();
        for i in 1..=120 {
            ys.push((i as f32) / 120.0);
        }
        for i in 0..30 {
            ys.push(1e-6 * (1.7f32).powi(i));
        }
        for v in [1.0f32, 0.5, 2.0, 1e-6, 1e-30, f32::MIN_POSITIVE] {
            ys.push(v);
        }
        for y in ys {
            writeln!(out, "ln {:08x} {:08x}", y.to_bits(), ln_f32(y).to_bits()).unwrap();
        }
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/agent/testdata/det_math_golden.txt");
        std::fs::write(path, out).expect("write golden");
        eprintln!("wrote {path}");
    }

    /// The pinned bit patterns. Both engines assert against this same file, so a drift on either
    /// side fails a unit test instead of a 100-seed sweep.
    #[test]
    fn matches_the_golden_table() {
        let golden = include_str!("testdata/det_math_golden.txt");
        let mut checked = 0;
        for line in golden.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut it = line.split_whitespace();
            let f = it.next().expect("fn");
            let inp = u32::from_str_radix(it.next().expect("in"), 16).expect("hex in");
            let out = u32::from_str_radix(it.next().expect("out"), 16).expect("hex out");
            let got = match f {
                "exp" => exp_f32(f32::from_bits(inp)).to_bits(),
                "ln" => ln_f32(f32::from_bits(inp)).to_bits(),
                other => panic!("unknown fn {other}"),
            };
            assert_eq!(
                got,
                out,
                "{f}({}) gave 0x{got:08x}, golden 0x{out:08x}",
                f32::from_bits(inp)
            );
            checked += 1;
        }
        assert!(checked >= 200, "golden table too small: {checked}");
    }

    /// Accuracy is not the contract, but a gross error would quietly change play strength, so hold
    /// both functions to a few ulp against the platform libm over the ranges the agent uses.
    #[test]
    fn close_enough_to_libm_over_the_ranges_the_agent_uses() {
        for i in 0..2000 {
            let x = -(i as f32) * 0.02;
            let a = exp_f32(x);
            let b = x.exp();
            assert!((a - b).abs() <= 1e-6 * b.max(1e-30), "exp({x}): {a} vs {b}");
        }
        for i in 1..2000 {
            let x = (i as f32) / 2000.0;
            let a = ln_f32(x);
            let b = x.ln();
            assert!((a - b).abs() <= 4e-6 * b.abs().max(1.0), "ln({x}): {a} vs {b}");
        }
    }

    #[test]
    fn round_trips() {
        for i in 1..500 {
            let x = (i as f32) / 500.0;
            let back = exp_f32(ln_f32(x));
            assert!((back - x).abs() < 1e-5, "exp(ln({x})) = {back}");
        }
    }

    #[test]
    fn edges_are_defined() {
        assert_eq!(exp_f32(0.0).to_bits(), 1.0f32.to_bits());
        assert_eq!(exp_f32(-1000.0), 0.0);
        assert_eq!(exp_f32(1000.0), f32::INFINITY);
        assert!(exp_f32(f32::NAN).is_nan());
        assert_eq!(ln_f32(1.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(ln_f32(0.0), f32::NEG_INFINITY);
        assert!(ln_f32(-1.0).is_nan());
        assert!(ln_f32(f32::NAN).is_nan());
    }

    /// Monotonic, because the sampler walks a cumulative sum built from it.
    #[test]
    fn exp_is_monotonic() {
        let mut prev = exp_f32(-40.0);
        for i in 1..4000 {
            let x = -40.0 + (i as f32) * 0.02;
            let v = exp_f32(x);
            assert!(v >= prev, "exp not monotonic at {x}");
            prev = v;
        }
    }
}
