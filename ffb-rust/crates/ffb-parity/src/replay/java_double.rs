//! Java's `double` -> text, as the replay harness writes it.
//!
//! The harness writes its action records with minimal-json 0.9.5, whose `Json.value(double)` is
//!
//! ```java
//! if (Double.isInfinite(value) || Double.isNaN(value))
//!     throw new IllegalArgumentException("Infinite and NaN values not permitted in JSON");
//! return new JsonNumber(cutOffPointZero(Double.toString(value)));
//! // cutOffPointZero: string.endsWith(".0") ? string.substring(0, string.length() - 2) : string
//! ```
//!
//! (checked with `javap -c com.eclipsesource.json.Json` on the frozen harness jar). The JVM is
//! Temurin 21, so `Double.toString` is the JDK 19+ algorithm (Giulietti, `DoubleToDecimal`), whose
//! javadoc fixes the digits: let R be the decimals that round to the value, m the minimal length
//! over R; T = the decimals of R with length m, or with length 1 or 2 when m == 1; the result is the
//! decimal of T closest to the value (ties: even last digit). Layout: plain for
//! 1e-3 <= |v| < 1e7 (at least one fraction digit), otherwise `d.ddd` + `E` + exponent (at least
//! one fraction digit).
//!
//! The shortest-closest digits come from Rust's `{:e}` (shortest round-trip, closest among the
//! shortest); the length-2 candidate of the m == 1 case from `{:.1e}` (correctly rounded).

/// The significant digits and the decimal exponent: value = `d0.d1d2...` x 10^exp.
///
/// Rust's `{:e}` gives a shortest decimal that rounds to the value, but on an exact tie between
/// two shortest candidates it does not take the even one (bits 0xc307b40843963382 =
/// -833984006375024.25: Java `-8.339840063750242E14`, `{:e}` ...243). So whenever the choice is
/// not unique -- a same-length neighbour also rounds to the value, or m == 1 (where length-2
/// decimals compete, e.g. Double.MIN_VALUE -> 4.9E-324, not 5E-324) -- the candidate is the exact
/// value rounded half-even to that length, kept if it rounds back to the value.
fn shortest_digits(abs: f64) -> (Vec<u8>, i32) {
    let (digits, exp) = split_sci(&format!("{abs:e}"));
    let n = digits.len();
    let mantissa: u64 = std::str::from_utf8(&digits).unwrap().parse().unwrap();
    let neighbour_round_trips = |m: u64| m > 0 && to_f64_int(m, exp - (n as i32 - 1)) == abs;
    let ambiguous = n == 1 || neighbour_round_trips(mantissa + 1) || neighbour_round_trips(mantissa - 1);
    if !ambiguous {
        return (digits, exp);
    }
    let exact = split_sci(&format!("{abs:.1100e}"));
    let mut best = (digits, exp);
    let c = round_sig(&exact, n);
    if to_f64(&c) == abs {
        best = c;
    }
    if n == 1 {
        // The half-even 2-digit decimal is at least as close as the 1-digit one (itself the
        // 2-digit decimal d.0); it wins when it is a different decimal that rounds to the value.
        let c2 = round_sig(&exact, 2);
        if c2.0[1] != b'0' && to_f64(&c2) == abs {
            best = c2;
        }
    }
    best
}

/// The exact digits rounded half-even to `n` significant digits.
fn round_sig(exact: &(Vec<u8>, i32), n: usize) -> (Vec<u8>, i32) {
    let (digits, mut exp) = (exact.0.as_slice(), exact.1);
    let mut kept: Vec<u8> = digits[..n].to_vec();
    let rest = &digits[n..];
    let up = match rest.first() {
        None => false,
        Some(&d) if d > b'5' => true,
        Some(&d) if d < b'5' => false,
        Some(_) => rest[1..].iter().any(|&d| d != b'0') || (kept[n - 1] - b'0') % 2 == 1,
    };
    if up {
        let mut i = n;
        loop {
            if i == 0 {
                kept.insert(0, b'1');
                kept.truncate(n);
                exp += 1;
                break;
            }
            i -= 1;
            if kept[i] == b'9' {
                kept[i] = b'0';
            } else {
                kept[i] += 1;
                break;
            }
        }
    }
    (kept, exp)
}

fn to_f64(d: &(Vec<u8>, i32)) -> f64 {
    let mantissa: u64 = std::str::from_utf8(&d.0).unwrap().parse().unwrap();
    to_f64_int(mantissa, d.1 - (d.0.len() as i32 - 1))
}

/// `mantissa x 10^scale`, correctly rounded.
fn to_f64_int(mantissa: u64, scale: i32) -> f64 {
    format!("{mantissa}e{scale}").parse().unwrap()
}

/// Split Rust's `{:e}` text (`"1.0499999999999998e0"`, `"5e-324"`) into digits and exponent.
fn split_sci(text: &str) -> (Vec<u8>, i32) {
    let (mantissa, exp) = text.split_once('e').expect("{:e} has an exponent");
    let digits: Vec<u8> = mantissa.bytes().filter(|b| *b != b'.').collect();
    (digits, exp.parse().expect("{:e} exponent is an integer"))
}

/// Java `Double.toString(double)` (JDK 19+).
pub fn java_double_to_string(v: f64) -> String {
    if v.is_nan() {
        return "NaN".to_string();
    }
    if v.is_infinite() {
        return if v > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    let mut out = String::new();
    if v.is_sign_negative() {
        out.push('-');
    }
    if v == 0.0 {
        out.push_str("0.0");
        return out;
    }
    let (digits, exp) = shortest_digits(v.abs());
    let digit = |i: usize| digits[i] as char;
    if (-3..7).contains(&exp) {
        if exp >= 0 {
            let int_len = exp as usize + 1;
            for i in 0..int_len {
                out.push(if i < digits.len() { digit(i) } else { '0' });
            }
            out.push('.');
            if digits.len() > int_len {
                (int_len..digits.len()).for_each(|i| out.push(digit(i)));
            } else {
                out.push('0');
            }
        } else {
            out.push_str("0.");
            for _ in 0..(-exp - 1) {
                out.push('0');
            }
            (0..digits.len()).for_each(|i| out.push(digit(i)));
        }
    } else {
        out.push(digit(0));
        out.push('.');
        if digits.len() > 1 {
            (1..digits.len()).for_each(|i| out.push(digit(i)));
        } else {
            out.push('0');
        }
        out.push('E');
        out.push_str(&exp.to_string());
    }
    out
}

/// minimal-json `Json.value(double).toString()`: `Double.toString` with a trailing `.0` cut off;
/// NaN and the infinities are Java's `IllegalArgumentException`.
pub fn json_double(v: f64) -> Result<String, String> {
    if v.is_infinite() || v.is_nan() {
        return Err("Infinite and NaN values not permitted in JSON".to_string());
    }
    let s = java_double_to_string(v);
    Ok(match s.strip_suffix(".0") {
        Some(cut) => cut.to_string(),
        None => s,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `(bits, Double.toString)` pairs printed by Temurin 21.0.12 from `scripts/DoubleProbe.java`
    /// (`java scripts/DoubleProbe.java <this file>`: edge cases, random bit patterns, tie-prone
    /// binary fractions). Every distinct score text of a Java run is checked by
    /// `ffb-replay double-check <java_dir>`.
    const JAVA: &[(u64, &str)] = &include!("java_double_probe.in");

    #[test]
    fn matches_java_double_to_string() {
        for (bits, want) in JAVA {
            let v = f64::from_bits(*bits);
            assert_eq!(java_double_to_string(v), *want, "bits {bits:#x}");
        }
    }

    #[test]
    fn minimal_json_cuts_point_zero() {
        assert_eq!(json_double(1.0).unwrap(), "1");
        assert_eq!(json_double(-0.0).unwrap(), "-0");
        assert_eq!(json_double(1.0e10).unwrap(), "1.0E10");
        assert_eq!(json_double(1.0499999999999998).unwrap(), "1.0499999999999998");
        assert_eq!(json_double(0.7272727272727273).unwrap(), "0.7272727272727273");
        assert!(json_double(f64::NAN).is_err());
        assert!(json_double(f64::INFINITY).is_err());
    }
}
