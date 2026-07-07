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
    use std::collections::HashSet;

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

    #[test]
    fn clone_equals_original() {
        for v in [SessionMode::Home, SessionMode::Away, SessionMode::Spec, SessionMode::Admin, SessionMode::Dev] {
            assert_eq!(v.clone(), v);
        }
    }

    #[test]
    fn debug_format_contains_variant_name() {
        assert!(format!("{:?}", SessionMode::Home).contains("Home"));
        assert!(format!("{:?}", SessionMode::Dev).contains("Dev"));
    }

    #[test]
    fn hash_works_in_set() {
        let mut set = HashSet::new();
        set.insert(SessionMode::Home);
        set.insert(SessionMode::Away);
        set.insert(SessionMode::Home); // duplicate
        assert_eq!(set.len(), 2);
        assert!(set.contains(&SessionMode::Home));
        assert!(!set.contains(&SessionMode::Admin));
    }

    #[test]
    fn spec_admin_dev_are_all_distinct() {
        assert_ne!(SessionMode::Spec, SessionMode::Admin);
        assert_ne!(SessionMode::Spec, SessionMode::Dev);
        assert_ne!(SessionMode::Admin, SessionMode::Dev);
    }
}
