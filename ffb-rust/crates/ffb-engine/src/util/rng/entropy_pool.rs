/// SHA-256-based entropy accumulator — 1:1 translation of Java EntropyPool.
pub struct EntropyPool {
    byte_count: usize,
    digest_bytes: Vec<u8>,
}

impl EntropyPool {
    const MIN_ENTROPY_BYTES: usize = 32;

    pub fn new() -> Self {
        Self { byte_count: 0, digest_bytes: Vec::new() }
    }

    pub fn add_entropy(&mut self, data: u8) {
        self.digest_bytes.push(data);
        self.byte_count += 1;
    }

    pub fn get_entropy(&mut self) -> Vec<u8> {
        self.byte_count = 0;
        // Phase ZU: return SHA-256 digest of accumulated bytes
        todo!("Phase ZU: SHA-256 digest of accumulated entropy bytes")
    }

    pub fn has_enough_entropy(&self) -> bool {
        self.byte_count >= Self::MIN_ENTROPY_BYTES
    }

    pub fn byte_count(&self) -> usize {
        self.byte_count
    }
}

impl Default for EntropyPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_enough_entropy_initially() {
        let pool = EntropyPool::new();
        assert!(!pool.has_enough_entropy());
        assert_eq!(pool.byte_count(), 0);
    }

    #[test]
    fn test_add_entropy_increments_count() {
        let mut pool = EntropyPool::new();
        for b in 0u8..32 {
            pool.add_entropy(b);
        }
        assert!(pool.has_enough_entropy());
        assert_eq!(pool.byte_count(), 32);
    }
}
