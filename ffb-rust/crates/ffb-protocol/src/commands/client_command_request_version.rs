/// 1:1 translation of ClientCommandRequestVersion (Java: no fields).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandRequestVersion;

impl ClientCommandRequestVersion {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandRequestVersion::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandRequestVersion::default();
    }

    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandRequestVersion::new()).is_empty());
    }

    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandRequestVersion::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandRequestVersion::default());
        assert!(s.contains("ClientCommandRequestVersion"));
    }
}
