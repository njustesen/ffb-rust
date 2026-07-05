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
}
