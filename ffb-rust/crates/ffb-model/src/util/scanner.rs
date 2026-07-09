/// 1:1 translation of `com.fumbbl.ffb.util.Scanner<T extends IKeyedItem>`.
///
/// In Java this performs runtime classgraph scanning, filters by `@RulesCollection`
/// annotation and rules version, then instantiates matching classes.  In Rust this
/// functionality is handled at compile time through trait objects and explicit
/// registration.  The struct is retained for structural completeness.
#[derive(Debug, Default)]
pub struct Scanner {
    /// Java: `rawScanner` — type name token (unused at runtime in Rust).
    pub type_name: String,
}

impl Scanner {
    pub fn new() -> Self { Self::default() }

    pub fn for_type(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into() }
    }

    /// Java: `getSubclassInstances()` — in Rust always empty Vec; callers use static dispatch.
    pub fn get_subclass_instances(&self) -> Vec<String> { vec![] }

    /// Java: `getInstancesImplementing()` — in Rust always empty Vec.
    pub fn get_instances_implementing(&self) -> Vec<String> { vec![] }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_name_stored() {
        let s = Scanner::for_type("ISkill");
        assert_eq!(s.type_name, "ISkill");
    }

    #[test]
    fn instances_empty() {
        let s = Scanner::new();
        assert!(s.get_subclass_instances().is_empty());
    }
}
