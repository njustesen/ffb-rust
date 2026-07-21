/// Marker trait for inducement effects that provide temporary enhancements — 1:1 translation of Java EnhancementProvider.
pub trait EnhancementProvider {
    fn enhancements(&self) -> Vec<String> {
        Vec::new()
    }
}
