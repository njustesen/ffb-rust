use std::cmp::Ordering;

/// 1:1 translation of `com.fumbbl.ffb.util.NaturalOrderComparator`.
///
/// Performs natural-order string comparison (digits sorted numerically).
/// Original Java source was ported from a C implementation by Martin Pool.
pub struct NaturalOrderComparator;

impl NaturalOrderComparator {
    pub fn new() -> Self { Self }

    /// Java: `compare(String a, String b)`.
    pub fn compare(&self, a: &str, b: &str) -> Ordering {
        let a: Vec<char> = a.chars().collect();
        let b: Vec<char> = b.chars().collect();
        let mut ia = 0usize;
        let mut ib = 0usize;

        loop {
            let mut nza = 0i32;
            let mut nzb = 0i32;

            let mut ca = Self::char_at(&a, ia);
            let mut cb = Self::char_at(&b, ib);

            // skip leading spaces and zeros, count zeros
            while ca.is_ascii_whitespace() || ca == '0' {
                if ca == '0' { nza += 1; } else { nza = 0; }
                ia += 1;
                ca = Self::char_at(&a, ia);
            }
            while cb.is_ascii_whitespace() || cb == '0' {
                if cb == '0' { nzb += 1; } else { nzb = 0; }
                ib += 1;
                cb = Self::char_at(&b, ib);
            }

            // run of digits
            if ca.is_ascii_digit() && cb.is_ascii_digit() {
                let result = self.compare_right(&a[ia..], &b[ib..]);
                if result != 0 { return if result < 0 { Ordering::Less } else { Ordering::Greater }; }
            }

            if ca == '\0' && cb == '\0' {
                return (nza - nzb).cmp(&0);
            }

            if ca < cb { return Ordering::Less; }
            if ca > cb { return Ordering::Greater; }

            ia += 1;
            ib += 1;
        }
    }

    fn compare_right(&self, a: &[char], b: &[char]) -> i32 {
        let mut bias = 0i32;
        let mut ia = 0usize;
        let mut ib = 0usize;
        loop {
            let ca = Self::char_at(a, ia);
            let cb = Self::char_at(b, ib);
            if !ca.is_ascii_digit() && !cb.is_ascii_digit() { return bias; }
            if !ca.is_ascii_digit() { return -1; }
            if !cb.is_ascii_digit() { return 1; }
            if ca < cb { if bias == 0 { bias = -1; } }
            else if ca > cb { if bias == 0 { bias = 1; } }
            else if ca == '\0' && cb == '\0' { return bias; }
            ia += 1; ib += 1;
        }
    }

    fn char_at(s: &[char], i: usize) -> char {
        if i >= s.len() { '\0' } else { s[i] }
    }
}

impl Default for NaturalOrderComparator {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_sorted_numerically() {
        let cmp = NaturalOrderComparator::new();
        assert_eq!(cmp.compare("pic2", "pic10"), Ordering::Less);
    }

    #[test]
    fn equal_strings_return_equal() {
        let cmp = NaturalOrderComparator::new();
        assert_eq!(cmp.compare("abc", "abc"), Ordering::Equal);
    }

    #[test]
    fn alpha_before_numeric_prefix() {
        let cmp = NaturalOrderComparator::new();
        assert_eq!(cmp.compare("a1", "b1"), Ordering::Less);
    }
}
