use crate::util::rng::entropy_server::EntropyServer;
use crate::util::rng::fortuna::Fortuna;

/// Feeds network-sourced entropy into the Fortuna PRNG — 1:1 translation of Java NetworkEntropySource.
pub struct NetworkEntropySource {
    entropy_server: EntropyServer,
    pool_index: usize,
}

impl NetworkEntropySource {
    pub fn new(port: u16, buffer_size: usize) -> Self {
        Self {
            entropy_server: EntropyServer::new(port, buffer_size),
            pool_index: 0,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.entropy_server.get_port()
    }

    pub fn is_connected(&self) -> bool {
        self.entropy_server.is_connected()
    }

    pub fn run(&mut self, fortuna: &mut Fortuna) {
        // Phase ZU: read entropy bytes from server and feed into fortuna pools
        todo!("Phase ZU: entropy pipeline from socket to Fortuna")
    }
}

impl Default for NetworkEntropySource {
    fn default() -> Self {
        Self::new(0, 4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_connected() {
        let source = NetworkEntropySource::new(8888, 256);
        assert!(!source.is_connected());
        assert_eq!(source.get_port(), 8888);
    }

    #[test]
    fn test_pool_index_starts_at_zero() {
        let source = NetworkEntropySource::new(0, 64);
        assert_eq!(source.pool_index, 0);
    }
}
