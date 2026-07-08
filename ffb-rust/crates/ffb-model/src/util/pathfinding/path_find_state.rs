#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathFindState {
    GLOBAL,
    NORMAL,
    HAS_JUMPED,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_variant_exists() {
        let s = PathFindState::GLOBAL;
        assert_eq!(s, PathFindState::GLOBAL);
    }

    #[test]
    fn normal_variant_exists() {
        let s = PathFindState::NORMAL;
        assert_eq!(s, PathFindState::NORMAL);
    }

    #[test]
    fn has_jumped_variant_exists() {
        let s = PathFindState::HAS_JUMPED;
        assert_eq!(s, PathFindState::HAS_JUMPED);
    }

    #[test]
    fn variants_are_not_equal() {
        assert_ne!(PathFindState::NORMAL, PathFindState::HAS_JUMPED);
        assert_ne!(PathFindState::NORMAL, PathFindState::GLOBAL);
    }

    #[test]
    fn copy_and_clone_work() {
        let s = PathFindState::NORMAL;
        let s2 = s;
        assert_eq!(s, s2);
        let s3 = s.clone();
        assert_eq!(s, s3);
    }

    #[test]
    fn hash_works() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PathFindState::NORMAL);
        set.insert(PathFindState::HAS_JUMPED);
        set.insert(PathFindState::GLOBAL);
        assert_eq!(set.len(), 3);
    }
}
