/// 1:1 translation of `com.fumbbl.ffb.server.SessionMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionMode {
    Home,
    Away,
    Spec,
    Admin,
    Dev,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_are_distinct() {
        let variants = [
            SessionMode::Home,
            SessionMode::Away,
            SessionMode::Spec,
            SessionMode::Admin,
            SessionMode::Dev,
        ];
        for i in 0..variants.len() {
            for j in (i + 1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }

    #[test]
    fn home_is_not_away() {
        assert_ne!(SessionMode::Home, SessionMode::Away);
    }

    #[test]
    fn copy_semantics() {
        let a = SessionMode::Admin;
        let b = a;
        assert_eq!(a, b);
    }
}
