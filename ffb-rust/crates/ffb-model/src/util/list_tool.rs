/// 1:1 translation of `com.fumbbl.ffb.util.ListTool`.
pub struct ListTool;

impl ListTool {
    /// Java: `firstElement(List<String>)` — `null` / empty → `None`.
    pub fn first_element(list: &[String]) -> Option<&str> {
        if list.is_empty() { None } else { Some(&list[0]) }
    }

    /// Java: `addAll(List<String>, String[])` — append all array items to list.
    /// Returns `false` if list or array is empty/null.
    pub fn add_all(list: &mut Vec<String>, array: &[&str]) -> bool {
        if array.is_empty() { return false; }
        for s in array { list.push(s.to_string()); }
        true
    }

    /// Java: `replaceAll(List<String>, String[])` — clear then add_all.
    pub fn replace_all(list: &mut Vec<String>, array: &[&str]) -> bool {
        list.clear();
        Self::add_all(list, array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_element_empty_returns_none() {
        assert_eq!(ListTool::first_element(&[]), None);
    }

    #[test]
    fn add_all_appends() {
        let mut v = vec!["a".to_string()];
        let added = ListTool::add_all(&mut v, &["b", "c"]);
        assert!(added);
        assert_eq!(v, vec!["a", "b", "c"]);
    }

    #[test]
    fn replace_all_clears_first() {
        let mut v = vec!["old".to_string()];
        ListTool::replace_all(&mut v, &["new"]);
        assert_eq!(v, vec!["new"]);
    }
}
