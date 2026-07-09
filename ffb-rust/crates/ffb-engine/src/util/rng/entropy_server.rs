/// Network-socket entropy server that buffers incoming entropy bytes — 1:1 translation of Java EntropyServer.
pub struct EntropyServer {
    port: u16,
    buffer: Vec<u8>,
    b_start: usize,
    b_end: usize,
    run_control: bool,
    connected: bool,
}

impl EntropyServer {
    pub fn new(port: u16, buffer_size: usize) -> Self {
        Self {
            port,
            buffer: vec![0u8; buffer_size],
            b_start: 0,
            b_end: 0,
            run_control: false,
            connected: false,
        }
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn stop(&mut self) {
        self.run_control = false;
    }

    pub fn run(&mut self) {
        // Phase ZU: accept TCP socket connections and buffer entropy bytes
        todo!("Phase ZU: TCP entropy socket server")
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        // Phase ZU: read from ring buffer (blocking wait until data available)
        todo!("Phase ZU: ring-buffer entropy read")
    }
}

impl Default for EntropyServer {
    fn default() -> Self {
        Self::new(0, 4096)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_not_connected() {
        let server = EntropyServer::new(9999, 1024);
        assert!(!server.is_connected());
        assert_eq!(server.get_port(), 9999);
    }

    #[test]
    fn test_buffer_size_respected() {
        let server = EntropyServer::new(0, 512);
        assert_eq!(server.buffer.len(), 512);
    }
}
