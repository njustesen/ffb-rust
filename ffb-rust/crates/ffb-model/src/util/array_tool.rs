/// 1:1 translation of `com.fumbbl.ffb.util.ArrayTool`.
pub struct ArrayTool;

impl ArrayTool {
    /// Java: `isProvided(boolean[])` — non-null and non-empty.
    pub fn is_provided_bool(array: &[bool]) -> bool { !array.is_empty() }

    /// Java: `isProvided(int[])`.
    pub fn is_provided_int(array: &[i32]) -> bool { !array.is_empty() }

    /// Java: `isProvided(Object[])`.
    pub fn is_provided<T>(array: &[T]) -> bool { !array.is_empty() }

    /// Java: `total(int[])`.
    pub fn total(array: &[i32]) -> i32 {
        if !Self::is_provided_int(array) { return 0; }
        array.iter().sum()
    }

    /// Java: `join(String[], String)`.
    pub fn join_str(array: &[&str], join: &str) -> Option<String> {
        if array.is_empty() { return None; }
        Some(array.join(join))
    }

    /// Java: `join(int[], String)`.
    pub fn join_int(array: &[i32], join: &str) -> Option<String> {
        if array.is_empty() { return None; }
        let parts: Vec<String> = array.iter().map(|v| v.to_string()).collect();
        Some(parts.join(join))
    }

    /// Java: `join(boolean[], String)`.
    pub fn join_bool(array: &[bool], join: &str) -> Option<String> {
        if array.is_empty() { return None; }
        let parts: Vec<String> = array.iter().map(|v| v.to_string()).collect();
        Some(parts.join(join))
    }

    /// Java: `isEqual(Object[], Object[])`.
    pub fn is_equal<T: PartialEq>(a: &[T], b: &[T]) -> bool {
        if !Self::is_provided(a) && !Self::is_provided(b) { return true; }
        if a.len() != b.len() { return false; }
        a.iter().zip(b.iter()).all(|(x, y)| x == y)
    }

    /// Java: `isEqual(int[], int[])`.
    pub fn is_equal_int(a: &[i32], b: &[i32]) -> bool {
        Self::is_equal(a, b)
    }

    /// Java: `toIntArray(List<Integer>)`.
    pub fn to_int_array(list: &[i32]) -> Vec<i32> { list.to_vec() }

    /// Java: `toBooleanArray(List<Boolean>)`.
    pub fn to_boolean_array(list: &[bool]) -> Vec<bool> { list.to_vec() }

    /// Java: `firstElement(String[])`.
    pub fn first_element<'a>(array: &'a [&str]) -> Option<&'a str> {
        if !Self::is_provided(array) { None } else { Some(array[0]) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_sums_array() {
        assert_eq!(ArrayTool::total(&[1, 2, 3]), 6);
        assert_eq!(ArrayTool::total(&[]), 0);
    }

    #[test]
    fn join_int_with_comma() {
        assert_eq!(ArrayTool::join_int(&[1, 2, 3], ","), Some("1,2,3".to_string()));
        assert_eq!(ArrayTool::join_int(&[], ","), None);
    }

    #[test]
    fn is_equal_arrays() {
        assert!(ArrayTool::is_equal_int(&[1, 2], &[1, 2]));
        assert!(!ArrayTool::is_equal_int(&[1, 2], &[1, 3]));
        assert!(ArrayTool::is_equal_int(&[], &[]));
    }
}
