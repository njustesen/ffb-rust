/// 1:1 translation of `com.fumbbl.ffb.util.pathfinding.PathFindState`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathFindState {
    GLOBAL,
    NORMAL,
    HAS_JUMPED,
}

impl Default for PathFindState {
    fn default() -> Self { PathFindState::NORMAL }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_normal() {
        assert_eq!(PathFindState::default(), PathFindState::NORMAL);
    }

    #[test]
    fn variants_are_distinct() {
        assert_ne!(PathFindState::GLOBAL, PathFindState::HAS_JUMPED);
    }
}
