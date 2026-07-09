/// 1:1 translation of `com.fumbbl.ffb.util.RawScanner<T2>`.
///
/// In Java this performs runtime classgraph scanning to find subclasses/implementors.
/// Rust uses compile-time type registration instead — this struct is retained for
/// structural completeness with no-op implementations.
#[derive(Debug, Default)]
pub struct RawScanner {
    /// Java: `persistentClass` (the Class<T2> token — unused at runtime in Rust).
    pub type_name: String,
}

impl RawScanner {
    pub fn new() -> Self { Self::default() }

    pub fn for_type(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into() }
    }

    /// Java: `getSubclasses()` — in Rust always empty; subclasses are registered statically.
    pub fn get_subclasses(&self) -> Vec<String> { vec![] }

    /// Java: `getClassesImplementing()` — in Rust always empty.
    pub fn get_classes_implementing(&self) -> Vec<String> { vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subclasses_is_empty() {
        let s = RawScanner::for_type("SomeInterface");
        assert!(s.get_subclasses().is_empty());
    }

    #[test]
    fn type_name_stored() {
        let s = RawScanner::for_type("Foo");
        assert_eq!(s.type_name, "Foo");
    }
}
