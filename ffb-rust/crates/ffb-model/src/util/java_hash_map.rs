//! Iteration order of a `java.util.HashMap<String, V>` filled by `put` in a given order.
//!
//! Java walks the table bucket by bucket; a key's bucket is `(n - 1) & (h ^ (h >>> 16))` for the
//! final table size `n` and `h = key.hashCode()`, and keys sharing a bucket keep their insertion
//! order (resizes split chains without reordering them). The table starts at 16 and doubles
//! whenever the size passes 3/4 of it. Where Java hands out such a map's `values()` -- e.g.
//! `Roster.getPositions()` over `fRosterPositionById` -- this is the order a Rust port must use.

/// Java `String.hashCode()` (UTF-16 code units, `31 * h + c` with int overflow).
pub fn java_string_hash(s: &str) -> i32 {
    s.encode_utf16().fold(0i32, |h, c| h.wrapping_mul(31).wrapping_add(c as i32))
}

/// The table size of a default `new HashMap<>()` after `size` puts of distinct keys.
fn table_size(size: usize) -> usize {
    let mut n = 16usize;
    while size > n * 3 / 4 {
        n *= 2;
    }
    n
}

/// `items` (in insertion order) re-ordered the way Java iterates a `HashMap` keyed by `key`.
pub fn java_hash_map_order<T>(items: Vec<T>, key: impl Fn(&T) -> &str) -> Vec<T> {
    let n = table_size(items.len());
    let mut keyed: Vec<(usize, usize, T)> = items.into_iter().enumerate().map(|(i, item)| {
        let h = java_string_hash(key(&item)) as u32;
        (((h ^ (h >> 16)) as usize) & (n - 1), i, item)
    }).collect();
    keyed.sort_by_key(|(bucket, i, _)| (*bucket, *i));
    keyed.into_iter().map(|(_, _, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_hash_matches_java() {
        // Java: "".hashCode() == 0, "a".hashCode() == 97, "Aa" and "BB" collide (2112).
        assert_eq!(java_string_hash(""), 0);
        assert_eq!(java_string_hash("a"), 97);
        assert_eq!(java_string_hash("Aa"), 2112);
        assert_eq!(java_string_hash("BB"), 2112);
        // "polygenelubricants".hashCode() == Integer.MIN_VALUE (a known Java overflow case).
        assert_eq!(java_string_hash("polygenelubricants"), i32::MIN);
    }

    #[test]
    fn colliding_keys_keep_insertion_order_and_buckets_ascend() {
        // "BB" and "Aa" share bucket 0 (2112 & 15) in insertion order; "a" (97) is bucket 1.
        let order = java_hash_map_order(vec!["a", "BB", "Aa"], |s| s);
        assert_eq!(order, vec!["BB", "Aa", "a"]);
    }

    #[test]
    fn the_table_grows_past_three_quarters() {
        assert_eq!(table_size(12), 16);
        assert_eq!(table_size(13), 32);
        assert_eq!(table_size(25), 64);
    }
}
