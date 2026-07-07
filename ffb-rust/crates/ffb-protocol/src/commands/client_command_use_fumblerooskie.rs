/// 1:1 translation of ClientCommandUseFumblerooskie (Java: no fields).
#[derive(Debug, Clone, Default)]
pub struct ClientCommandUseFumblerooskie;

impl ClientCommandUseFumblerooskie {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_construct() {
        let _cmd = ClientCommandUseFumblerooskie::new();
    }

    #[test]
    fn default_works() {
        let _cmd = ClientCommandUseFumblerooskie::default();
    }
    #[test]
    fn debug_format_nonempty() {
        assert!(!format!("{:?}", ClientCommandUseFumblerooskie::default()).is_empty());
    }


    #[test]
    fn clone_does_not_panic() {
        let _ = ClientCommandUseFumblerooskie::default().clone();
    }

    #[test]
    fn debug_format_contains_struct_name() {
        let s = format!("{:?}", ClientCommandUseFumblerooskie::default());
        assert!(s.contains("ClientCommandUseFumblerooskie"));
    }
}
