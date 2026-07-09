pub struct IServerLogLevel;

impl IServerLogLevel {
    pub const NO_LOGGING: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const WARN: i32 = 2;
    pub const INFO: i32 = 3;
    pub const DEBUG: i32 = 4;
    pub const TRACE: i32 = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels_ordered() {
        assert!(IServerLogLevel::ERROR < IServerLogLevel::WARN);
        assert!(IServerLogLevel::WARN < IServerLogLevel::INFO);
        assert!(IServerLogLevel::INFO < IServerLogLevel::DEBUG);
        assert!(IServerLogLevel::DEBUG < IServerLogLevel::TRACE);
    }

    #[test]
    fn test_no_logging_is_zero() {
        assert_eq!(IServerLogLevel::NO_LOGGING, 0);
    }
}
