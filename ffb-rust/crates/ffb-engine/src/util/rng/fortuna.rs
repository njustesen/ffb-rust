use crate::util::rng::entropy_pool::EntropyPool;

const NUMBER_OF_POOLS: usize = 32;

/// Fortuna PRNG using AES-CTR with entropy pool reseeding — 1:1 translation of Java Fortuna.
pub struct Fortuna {
    pools: Vec<EntropyPool>,
    current_pool: usize,
    pool_selector: u64,
    nonce: [u8; 16],
    key: [u8; 32],
    random_data: Vec<u8>,
    byte_offset: usize,
    last_rekeying: i64,
    number_of_rekeyings: u64,
    number_of_bytes: u64,
}

impl Fortuna {
    pub fn new() -> Self {
        let nonce: [u8; 16] = [
            0x4E, 0xC1, 0x37, 0xA4, 0x26, 0xDA, 0xBF, 0x8A,
            0xA0, 0xBE, 0xB8, 0xBC, 0x0C, 0x2B, 0x89, 0xD6,
        ];
        let key: [u8; 32] = [
            0x95, 0xA8, 0xEE, 0x8E, 0x89, 0x97, 0x9B, 0x9E,
            0xFD, 0xCB, 0xC6, 0xEB, 0x97, 0x97, 0x52, 0x8D,
            0x43, 0x2D, 0xC2, 0x60, 0x61, 0x55, 0x38, 0x18,
            0xEA, 0x63, 0x5E, 0xC5, 0xD5, 0xA7, 0x72, 0x7E,
        ];
        Self {
            pools: (0..NUMBER_OF_POOLS).map(|_| EntropyPool::new()).collect(),
            current_pool: NUMBER_OF_POOLS - 1,
            pool_selector: 1,
            nonce,
            key,
            random_data: Vec::new(),
            byte_offset: 0,
            last_rekeying: 0,
            number_of_rekeyings: 0,
            number_of_bytes: 0,
        }
    }

    pub fn get_die_roll(&mut self, die_type: i32) -> i32 {
        // Phase ZU: AES-CTR random byte generation
        todo!("Phase ZU: Fortuna AES-CTR die roll")
    }

    pub fn add_entropy(&mut self, pool_index: usize, data: u8) {
        if pool_index < self.pools.len() {
            self.pools[pool_index].add_entropy(data);
        }
    }

    pub fn get_number_of_rekeyings(&self) -> u64 {
        self.number_of_rekeyings
    }

    pub fn get_number_of_bytes(&self) -> u64 {
        self.number_of_bytes
    }
}

impl Default for Fortuna {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_fortuna_has_pools() {
        let f = Fortuna::new();
        assert_eq!(f.pools.len(), NUMBER_OF_POOLS);
    }

    #[test]
    fn test_initial_rekey_count_zero() {
        let f = Fortuna::new();
        assert_eq!(f.get_number_of_rekeyings(), 0);
        assert_eq!(f.get_number_of_bytes(), 0);
    }
}
