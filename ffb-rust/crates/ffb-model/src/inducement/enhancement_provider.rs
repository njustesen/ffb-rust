/// Marker trait for inducement effects that provide temporary enhancements — 1:1 translation of Java EnhancementProvider.
pub trait EnhancementProvider {
    fn enhancements(&self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NoEnhancement;
    impl EnhancementProvider for NoEnhancement {}

    #[test]
    fn test_default_empty_enhancements() {
        let e = NoEnhancement;
        assert!(e.enhancements().is_empty());
    }

    #[test]
    fn test_trait_object_works() {
        let e: &dyn EnhancementProvider = &NoEnhancement;
        assert!(e.enhancements().is_empty());
    }
}
