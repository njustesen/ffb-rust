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

}
