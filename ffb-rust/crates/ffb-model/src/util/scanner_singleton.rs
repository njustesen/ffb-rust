/// 1:1 translation of `com.fumbbl.ffb.util.ScannerSingleton`.
///
/// In Java this wraps `io.github.classgraph.ClassGraph` to scan the classpath.
/// Rust has no equivalent runtime classpath scanning — this struct is retained
/// for structural completeness but its functionality is replaced by compile-time
/// trait objects and enum variants throughout the codebase.
#[derive(Debug, Default)]
pub struct ScannerSingleton;

impl ScannerSingleton {
    pub fn new() -> Self { Self }

    /// Java: `getInstance()` — returns singleton instance.
    pub fn get_instance() -> Self { Self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_instance_returns_default() {
        let _s = ScannerSingleton::get_instance();
    }

    #[test]
    fn new_is_default() {
        let _s = ScannerSingleton::new();
    }
}
